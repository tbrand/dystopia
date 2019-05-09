pub mod error;
pub mod health;
pub mod join;
pub mod pub_key;
pub mod rely;
pub mod state;

use crate::error::Result;
use crate::health::Health;
use crate::join::Join;
use crate::pub_key::PubKey;
use crate::rely::Rely;
use crate::state::State;
use clap::crate_version;
use dytp_connection::prelude::*;
use dytp_protocol::method::{encrypted, plain};
use failure::Error;
use futures::future;
use futures::prelude::*;
use openssl::rsa::Padding;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;

type ProcessFuture = Box<Future<Item = (), Error = Error> + Send>;

fn process(socket: TcpStream, state: Arc<RwLock<State>>, read_timeout: u64) {
    let origin = Origin::new_with_timeout(socket, read_timeout);
    let process = origin
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(move |(buf, origin)| {
            if let Some(buf) = buf {
                use std::ops::Deref;

                match plain::Common::from(buf.deref()) {
                    plain::Common::HEALTH => {
                        return Box::new(Health::new(origin)) as ProcessFuture;
                    }
                    _ => {}
                }

                match plain::ToNode::from(buf.deref()) {
                    plain::ToNode::PUB_KEY => {
                        return Box::new(PubKey::new(state.clone(), origin)) as ProcessFuture;
                    }
                    _ => {}
                }

                let method = {
                    let state = state.write().unwrap();
                    let mut method = vec![0; 2048];

                    state
                        .rsa
                        .private_decrypt(&buf, &mut method, Padding::PKCS1)
                        .map(move |b| method[0..b].to_owned())
                };

                if let Ok(m) = method {
                    match encrypted::Method::from(m.as_slice()) {
                        encrypted::Method::RELY { hop, addr, tls } => {
                            if let Ok(upstream) = Upstream::new_with_timeout(addr, read_timeout) {
                                return Box::new(Rely::new(
                                    state.clone(),
                                    origin,
                                    upstream,
                                    hop,
                                    tls,
                                )) as ProcessFuture;
                            }
                        }
                        _ => {}
                    }
                }
            }

            Box::new(future::ok::<(), Error>(()))
        })
        .map_err(|e| {
            log::error!("node error={:?}", e);
        });

    tokio::spawn(process);
}

pub fn main_inner(
    addr: SocketAddr,
    global_addr: SocketAddr,
    cloud_addr: SocketAddr,
    read_timeout: u64,
) -> Result<()> {
    let state = Arc::new(RwLock::new(State::new()));
    let listener = TcpListener::bind(&addr).unwrap();
    let version = crate_version!().parse()?;

    let join = Join::new(state.clone(), global_addr, cloud_addr, version)?
        .map(|_| {
            log::info!("successfully join to the cloud!");
        })
        .map_err(|e| {
            log::error!("failed to join to the cloud due to error={:?}", e);
        });

    let tasks = listener
        .incoming()
        .for_each(move |socket| {
            process(socket, state.clone(), read_timeout);
            Ok(())
        })
        .map_err(|e| {
            log::error!("node error={:?}", e);
        });

    let mut runtime = Runtime::new()?;

    log::info!("try to join to the cloud");
    log::info!("node start running on {}", addr);

    runtime.spawn(tasks);
    runtime.spawn(join);

    if let Err(e) = runtime.shutdown_on_idle().wait() {
        log::error!("shutdown server process due to {:?}", e);
    }

    Ok(())
}
