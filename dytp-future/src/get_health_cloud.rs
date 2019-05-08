use crate::error::Result;
use dytp_component::health_resp_cloud::HealthRespCloud;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct GetHealthCloud {
    pub addr: SocketAddr,
    upstream: Upstream,
}

impl GetHealthCloud {
    pub fn new(cloud_addr: SocketAddr) -> Result<GetHealthCloud> {
        let mut upstream = Upstream::new(cloud_addr.clone())?;
        let buf: Vec<u8> = plain::Common::HEALTH.into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(GetHealthCloud {
            addr: cloud_addr,
            upstream,
        })
    }
}

impl Future for GetHealthCloud {
    type Item = Option<HealthRespCloud>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                let health = HealthRespCloud::from(&payload as &[u8]);

                return Ok(Async::Ready(Some(health)));
            }
            Ok(Async::Ready(None)) => {
                log::warn!("failed to get cloud health on {}", self.addr);

                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!(
                    "failed to get cloud health on {} due to error={:?}",
                    self.addr,
                    e
                );

                return Ok(Async::Ready(None));
            }
        }
    }
}
