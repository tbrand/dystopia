use crate::error::Result;
use dytp_component::node_state::NodeState;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct Check {
    upstream: Upstream,
}

impl Check {
    pub fn new(global_addr: SocketAddr, cloud_addr: SocketAddr) -> Result<Check> {
        let mut upstream = Upstream::new(cloud_addr)?;
        let buf: Vec<u8> = plain::ToCloud::CHECK { addr: global_addr }.into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(Check { upstream })
    }
}

impl Future for Check {
    type Item = Option<NodeState>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.upstream.poll()? {
            Async::Ready(state) => {
                let state = state
                    .map(|s| std::str::from_utf8(&s).unwrap().to_owned())
                    .as_ref()
                    .and_then(|s| {
                        if s == "E" {
                            None
                        } else {
                            Some(NodeState::from_str(s).unwrap())
                        }
                    });

                return Ok(Async::Ready(state));
            }
            Async::NotReady => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
        }
    }
}
