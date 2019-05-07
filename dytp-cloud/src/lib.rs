pub mod error;
pub mod manager;

use crate::error::Result;
use crate::manager::Manager;
use chrono::prelude::*;
use clap::crate_version;
use dytp_component::health_resp_cloud::HealthRespCloud;
use dytp_component::node_state::NodeState;
use dytp_connection::prelude::*;
use dytp_future::get_node_health::GetNodeHealth;
use dytp_future::get_pub_key::GetPubKey;
use dytp_protocol::method::plain;
use failure::Error;
use futures::future::Either;
use openssl::rsa::Padding;
use semver::Version;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

fn audit(
    manager: Box<Manager + Send>,
    mut origin: Origin,
    ts: i64,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let f = manager.sync(ts).map(move |audit| {
        let buf = audit.iter().fold(Vec::<u8>::new(), |mut audit, a| {
            if audit.len() > 0 {
                audit.append(&mut b" ".to_vec());
            }

            audit.append(
                &mut format!("{} {} {} {}", a.addr, a.state, a.version, a.ts)
                    .as_bytes()
                    .to_vec(),
            );
            audit
        });

        origin.write(&buf).unwrap();
        origin.flush().unwrap();
    });

    Box::new(f)
}

fn list(
    manager: Box<Manager + Send>,
    mut origin: Origin,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let f = manager
        .latest_ts()
        .join(manager.list(true))
        .map(move |(ts, nodes)| {
            let buf = nodes
                .iter()
                .map(|node| format!("{} {}", node.addr, node.version))
                .fold(format!("{}", ts).as_bytes().to_vec(), |mut nodes, node| {
                    nodes.append(&mut b" ".to_vec());
                    nodes.append(&mut node.as_bytes().to_vec());
                    nodes
                });

            origin.write(&buf).unwrap();
            origin.flush().unwrap();
        });

    Box::new(f)
}

fn join(
    manager: Box<Manager + Send>,
    mut origin: Origin,
    addr: SocketAddr,
    version: Version,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let get_pub_key = GetPubKey::new(addr.clone());

    if let Err(_) = get_pub_key {
        return Box::new(future::ok(()));
    }

    let f = get_pub_key
        .unwrap()
        .map_err(|e| e.into())
        .and_then(move |rsa| {
            let f = if let Some(rsa) = rsa {
                let data: &[u8] = &[0, 1, 2, 3];
                let mut buf = vec![0; rsa.size() as usize];

                rsa.public_encrypt(data, &mut buf, Padding::PKCS1).unwrap();

                origin.write(&buf).unwrap();
                origin.flush().unwrap();

                let f = origin
                    .into_future()
                    .map_err(|(e, _)| e)
                    .and_then(move |(buf, _)| {
                        let f = if let Some(buf) = buf {
                            let f = if &buf[0..data.len()] == data {
                                log::info!("new node has joined! <- {}", addr);

                                Either::A(manager.join(addr, version))
                            } else {
                                Either::B(future::ok(()))
                            };

                            Either::A(f)
                        } else {
                            Either::B(future::ok(()))
                        };

                        f
                    });

                Either::A(f)
            } else {
                Either::B(future::ok(()))
            };

            f
        });

    Box::new(f)
}

fn health(
    manager: Box<Manager + Send>,
    mut origin: Origin,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let f = manager.list(false).map_err(|e| e.into()).map(move |nodes| {
        let res: Vec<u8> = HealthRespCloud::new(crate_version!(), &nodes).into();

        origin.write(&res).unwrap();
        origin.flush().unwrap();

        ()
    });

    Box::new(f)
}

fn process(socket: TcpStream, manager: Box<Manager + Send>, read_timeout: u64) {
    let origin = Origin::new_with_timeout(socket, read_timeout);
    let f = origin
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(move |(buf, origin)| {
            if let Some(buf) = buf {
                use std::ops::Deref;

                // TODO:
                // Couldn't execute below since it's DYTP protocol
                //
                // match plain::Common::from(buf.deref()) {
                //     plain::Common::HEALTH => {
                //         return health(manager, origin);
                //     }
                //     _ => {}
                // }

                match plain::ToCloud::from(buf.deref()) {
                    plain::ToCloud::SYNC { ts } => {
                        return audit(manager, origin, ts);
                    }
                    plain::ToCloud::FETCH => {
                        return list(manager, origin);
                    }
                    plain::ToCloud::JOIN { addr, version } => {
                        return join(manager, origin, addr, version);
                    }
                    _ => {}
                }
            }

            Box::new(future::ok::<(), Error>(()))
        })
        .map_err(|e| {
            log::error!("cloud error={:?}", e);
        });

    tokio::spawn(f);
}

fn healthcheck(manager: Box<Manager + Send>, node_deletion_timeout: u64) {
    let f = manager.list(false)
        .map_err(|e| log::error!("error={:?}", e))
        .map(move |nodes| {
            for node in nodes.into_iter() {
                let manager = manager.clone();

                match node.state {
                    NodeState::ACTIVE => {
                        if let Ok(g) = GetNodeHealth::new(node.addr.clone()) {
                            let f = g
                                .map_err(move |e| {
                                    log::error!("health check error={:?}", e);

                                    let f = manager.pending_delete(node.addr.clone(), node.version.clone()).map_err(|e| {
                                        log::error!(
                                            "couldn't change the state of the node due to errror={:?}",
                                            e
                                        );
                                    });

                                    tokio::spawn(f);
                                })
                                .map(|_| ());

                            tokio::spawn(f);
                        } else {
                            log::warn!(
                                "couldn't connect to {}. change state to PENDING_DELETE",
                                node.addr
                            );

                            let f = manager.pending_delete(node.addr.clone(), node.version.clone()).map_err(|e| {
                                log::error!("couldn't change the state of the node due to error={:?}", e);
                            });

                            tokio::spawn(f);
                        }
                    }
                    NodeState::PENDING_DELETE => {
                        log::warn!("node {} is pending to be deleted", node.addr);

                        let f = manager.deleted_ts(node.addr).and_then(move |ts| {
                            let duration_secs = (Utc::now().timestamp_nanos() - ts) as u64 / 1000_000_000;

                            if duration_secs > node_deletion_timeout {
                                log::warn!("node {} stay 'pending delete' for {:?} secs", node.addr, duration_secs);
                                log::warn!("node {} will be deleted completly", node.addr);

                                Either::A(manager.delete(node.addr))
                            } else {
                                Either::B(future::ok(()))
                            }
                        }).map_err(|e| {
                            log::error!("failed to found deletion timestamp due to error={:?}", e);
                        });

                        tokio::spawn(f);
                    }
                }
            }
        });

    tokio::spawn(f);
}

pub fn main_inner(
    addr: SocketAddr,
    healthcheck_timeout: u64,
    node_deletion_timeout: u64,
    read_timeout: u64,
) -> Result<()> {
    let manager = manager::create()?;
    let manager_healthcheck = manager.clone();

    let listener = TcpListener::bind(&addr).unwrap();
    let tasks = listener
        .incoming()
        .for_each(move |socket| {
            process(socket, manager.clone(), read_timeout);
            Ok(())
        })
        .map_err(|e| {
            log::error!("cloud error={:?}", e);
        });

    let healthcheck = Interval::new(Instant::now(), Duration::from_secs(healthcheck_timeout))
        .for_each(move |_| {
            healthcheck(manager_healthcheck.clone(), node_deletion_timeout);
            Ok(())
        })
        .map_err(|e| log::error!("during healthcheck error={:?}", e));

    let mut runtime = Runtime::new()?;

    log::info!("cloud running on {}", addr);
    log::info!("start healthchecking...");

    runtime.spawn(tasks);
    runtime.spawn(healthcheck);

    if let Err(e) = runtime.shutdown_on_idle().wait() {
        log::error!("shutdown server process due to error={:?}", e);
    }

    Ok(())
}
