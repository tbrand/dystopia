use crate::error::Result;
use dytp_component::health_resp_gateway::HealthRespGateway;
use dytp_connection::prelude::*;
use dytp_protocol::delim::Delim;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct GetGatewayHealth {
    pub addr: SocketAddr,
    upstream: Upstream,
}

impl GetGatewayHealth {
    pub fn new(gateway_addr: SocketAddr) -> Result<GetGatewayHealth> {
        let mut upstream = Upstream::new(gateway_addr.clone())?;
        let buf: Vec<u8> = plain::Common::HEALTH.into();

        upstream.set_write_delim(Delim::Http);
        upstream.set_read_delim(Delim::Http);
        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(GetGatewayHealth {
            addr: gateway_addr,
            upstream,
        })
    }
}

impl Future for GetGatewayHealth {
    type Item = Option<HealthRespGateway>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                let health = HealthRespGateway::from(&payload as &[u8]);

                return Ok(Async::Ready(Some(health)));
            }
            Ok(Async::Ready(None)) => {
                log::warn!("failed to get gateway health on {}", self.addr);

                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!(
                    "failed to get gateway health on {} due to error={:?}",
                    self.addr,
                    e
                );

                return Ok(Async::Ready(None));
            }
        }
    }
}
