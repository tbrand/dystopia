use crate::error::Result;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use openssl::pkey::Public;
use openssl::rsa::Rsa;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct GetPubKey {
    upstream: Upstream,
}

impl GetPubKey {
    pub fn new(addr: SocketAddr) -> Result<GetPubKey> {
        let mut upstream = Upstream::new(addr)?;
        let buf: Vec<u8> = plain::ToNode::PUB_KEY.into();
        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(GetPubKey { upstream })
    }
}

impl Future for GetPubKey {
    type Item = Option<Rsa<Public>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                return Ok(Async::Ready(
                    Rsa::<Public>::public_key_from_der(&payload).ok(),
                ));
            }
            Ok(Async::Ready(None)) => {
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!("failed to get public key due to {:?}", e);

                return Ok(Async::Ready(None));
            }
        }
    }
}
