pub mod error;
pub mod rely;
pub mod route;
pub mod route_node;
pub mod ts;

use crate::error::Result;
use crate::rely::Rely;
use crate::route::{GetAllNodes, GetRoute, RegisterNode, RegisterNodes, RemoveNode};
use crate::route_node::RouteNode;
use crate::ts::{LatestTs, RecordTs};
use dytp_component::node_state::NodeState;
use dytp_connection::prelude::*;
use dytp_future::fetch_nodes::FetchNodes;
use dytp_future::get_pub_key::GetPubKey;
use dytp_future::sync_audit::SyncAudit;
use failure::Error;
use futures::future::{join_all, Either};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

type ProcessFuture = Box<Future<Item = (), Error = Error> + Send>;

fn ignore() -> ProcessFuture {
    Box::new(future::ok(()))
}

fn process(socket: TcpStream, hops: usize, read_timeout: u64) {
    log::debug!("received new request");

    let request = Request::new_with_timeout(socket, read_timeout);
    let process = request
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(move |(ctx, req)| {
            if let Some(ctx) = ctx {
                match ctx {
                    RequestContext::Common(common) => {
                        return Box::new(future::ok(())) as ProcessFuture;
                    }
                    RequestContext::Http { tls, buf, ip } => {
                        let route = GetRoute::new(hops);

                        log::debug!("decided the route.");

                        let f = route.and_then(move |nodes| {
                            if let Some(nodes) = nodes {
                                let get_pub_keys = nodes
                                    .iter()
                                    .map(|n| GetPubKey::new(*n))
                                    .collect::<Vec<Result<GetPubKey>>>();

                                if get_pub_keys.iter().any(|g| g.is_err()) {
                                    return ignore();
                                }

                                let rsa_keys = get_pub_keys
                                    .into_iter()
                                    .map(|g| g.unwrap())
                                    .collect::<Vec<GetPubKey>>();

                                let f = join_all(rsa_keys).and_then(move |rsa_keys| {
                                    if rsa_keys.iter().any(|r| r.is_none()) {
                                        return ignore();
                                    }

                                    log::debug!("received public keys.");

                                    let route_nodes: Vec<RouteNode> = rsa_keys
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, r)| {
                                            if idx < nodes.len() - 1 {
                                                RouteNode::new(
                                                    nodes[idx],
                                                    nodes[idx + 1],
                                                    r.unwrap(),
                                                )
                                            } else {
                                                RouteNode::new(nodes[idx], ip, r.unwrap())
                                            }
                                        })
                                        .collect();

                                    let origin =
                                        Origin::new_with_timeout(req.stream(), read_timeout);

                                    if let Ok(upstream) =
                                        Upstream::new_with_timeout(nodes[0], read_timeout)
                                    {
                                        if let Ok(rely) =
                                            Rely::new(origin, upstream, route_nodes, &buf, tls)
                                        {
                                            return Box::new(rely) as ProcessFuture;
                                        }
                                    }

                                    ignore()
                                });

                                return Box::new(f) as ProcessFuture;
                            }

                            ignore()
                        });

                        return Box::new(f) as ProcessFuture;
                    }
                }
            }

            ignore()
        })
        .map_err(|e| {
            log::error!("gateway error={:?}", e);
        });

    tokio::spawn(process);
}

fn sync(cloud_addr: SocketAddr) {
    let f = GetAllNodes::new()
        .and_then(move |nodes| {
            let f = if nodes.len() == 0 {
                log::debug!("Fetch nodes from cloud...");

                let f = match FetchNodes::new(cloud_addr) {
                    Ok(f) => {
                        let f = f.and_then(|res| {
                            if let Some((ts, nodes)) = res {
                                Either::A(
                                    RegisterNodes::new(nodes)
                                        .join(RecordTs::new(ts))
                                        .map(|_| ()),
                                )
                            } else {
                                Either::B(future::ok(()))
                            }
                        });

                        Either::A(f)
                    }
                    Err(e) => {
                        log::error!("failed to connect to the cloud due to error={:?}", e);

                        Either::B(future::ok(()))
                    }
                };

                Either::A(f)
            } else {
                let f = LatestTs::new().and_then(move |ts| match SyncAudit::new(cloud_addr, ts) {
                    Ok(f) => {
                        let f = f.and_then(|res| {
                            if let Some(audit) = res {
                                let f = if audit.len() > 0 {
                                    let ts = audit[0].ts;
                                    let mut f: Box<Future<Item = (), Error = Error> + Send> =
                                        Box::new(future::ok(()));

                                    for a in audit.into_iter().rev() {
                                        f = Box::new(f.and_then(move |_| match a.state {
                                            NodeState::ACTIVE => {
                                                log::info!("ACTIVE: {} ({})", a.addr, a.version);
                                                Either::A(RegisterNode::new(a))
                                            }
                                            NodeState::PENDING_DELETE => {
                                                log::info!("DELETE: {} ({})", a.addr, a.version);
                                                Either::B(RemoveNode::new(a.addr))
                                            }
                                        }));
                                    }

                                    let f = f.and_then(move |_| RecordTs::new(ts));

                                    Either::A(f)
                                } else {
                                    Either::B(future::ok(()))
                                };

                                Either::A(f)
                            } else {
                                Either::B(future::ok(()))
                            }
                        });

                        Either::A(f)
                    }
                    Err(e) => {
                        log::error!("failed to connect to the cloud due to error={:?}", e);

                        Either::B(future::ok(()))
                    }
                });

                Either::B(f)
            };

            f
        })
        .map_err(|e| {
            log::error!(
                "error occurred during syncing with cloud due to error={:?}",
                e
            )
        });

    tokio::spawn(f);
}

pub fn main_inner(
    addr: SocketAddr,
    cloud_addr: SocketAddr,
    hops: usize,
    read_timeout: u64,
) -> Result<()> {
    let listener = TcpListener::bind(&addr).unwrap();
    let tasks = listener
        .incoming()
        .for_each(move |socket| {
            process(socket, hops, read_timeout);
            Ok(())
        })
        .map_err(|e| {
            log::error!("gateway error={:?}", e);
        });

    let sync = Interval::new(Instant::now(), Duration::from_secs(5))
        .for_each(move |_| {
            sync(cloud_addr);
            Ok(())
        })
        .map_err(|e| log::error!("failed to get node list due to error={:?}", e));

    let mut runtime = Runtime::new()?;

    log::info!("gateway running on {}", addr);
    log::info!("start syncing nodes via cloud on {}", cloud_addr);

    runtime.spawn(tasks);
    runtime.spawn(sync);

    if let Err(e) = runtime.shutdown_on_idle().wait() {
        log::error!("shutdown server process due to error={:?}", e);
    }

    Ok(())
}
