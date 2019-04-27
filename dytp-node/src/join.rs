use crate::error::{NodeError, Result};
use crate::state::State;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use openssl::rsa::Padding;
use semver::Version;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

#[derive(Debug)]
pub struct Join {
    state: Arc<RwLock<State>>,
    upstream: Upstream,
}

impl Join {
    pub fn new(
        state: Arc<RwLock<State>>,
        global_addr: SocketAddr,
        cloud_addr: SocketAddr,
        version: Version,
    ) -> Result<Join> {
        let mut upstream = Upstream::new(cloud_addr)?;
        let buf: Vec<u8> = plain::ToCloud::JOIN {
            addr: global_addr,
            version,
        }
        .into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(Join { state, upstream })
    }
}

impl Future for Join {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- Join");

        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                let state = self.state.read().unwrap();
                let rsa = &state.rsa;
                let mut buf = vec![0; rsa.size() as usize];

                rsa.private_decrypt(&payload, &mut buf, Padding::PKCS1)
                    .unwrap();

                self.upstream.write(&buf)?;
                self.upstream.flush()?;

                return Ok(Async::Ready(()));
            }
            Ok(Async::Ready(None)) => {
                log::warn!("failed to join to the cloud.");
                log::warn!("this may require you to change cloud endpoint if this happens again.");
                log::warn!("this process will be exited.");

                return Err(NodeError::JoiningFailure.into());
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!("failed to join to the cloud due to error={:?}", e);

                return Err(NodeError::JoiningFailure.into());
            }
        }
    }
}
