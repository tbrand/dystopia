use crate::error::Result;
use dytp_component::health_resp_node::HealthRespNode;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct GetHealthNode {
    pub addr: SocketAddr,
    upstream: Upstream,
}

impl GetHealthNode {
    pub fn new(node_addr: SocketAddr) -> Result<GetHealthNode> {
        let mut upstream = Upstream::new(node_addr.clone())?;
        let buf: Vec<u8> = plain::Common::HEALTH.into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(GetHealthNode {
            addr: node_addr,
            upstream,
        })
    }
}

impl Future for GetHealthNode {
    type Item = Option<HealthRespNode>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                let health = HealthRespNode::from(&payload as &[u8]);

                return Ok(Async::Ready(Some(health)));
            }
            Ok(Async::Ready(None)) => {
                log::warn!("failed to get node health on {}", self.addr);

                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!(
                    "failed to get node health on {} due to error={:?}",
                    self.addr,
                    e
                );

                return Ok(Async::Ready(None));
            }
        }
    }
}
