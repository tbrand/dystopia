use crate::manager::mem::ON_MEM_AUDIT;
use dytp_component::node_state::NodeState;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

pub struct Check {
    addr: SocketAddr,
}

impl Check {
    pub fn new(addr: SocketAddr) -> Check {
        Check { addr }
    }
}

impl Future for Check {
    type Item = Option<NodeState>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(audit) = ON_MEM_AUDIT.try_read() {
            for a in audit.iter().rev() {
                if a.addr == self.addr {
                    return Ok(Async::Ready(Some(a.state.clone())));
                }
            }

            return Ok(Async::Ready(None));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
