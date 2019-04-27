use crate::manager::mem::ts;
use crate::manager::mem::ON_MEM_AUDIT;
use crate::manager::mem::ON_MEM_NODES;
use dytp_component::audit::Audit;
use dytp_component::node::Node;
use dytp_component::node_state::NodeState;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::net::SocketAddr;
use tokio::prelude::*;

pub struct Join {
    addr: SocketAddr,
    version: Version,
}

impl Join {
    pub fn new(addr: SocketAddr, version: Version) -> Join {
        Join { addr, version }
    }
}

impl Future for Join {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- Join");

        if let Ok(mut nodes) = ON_MEM_NODES.try_write() {
            if let Ok(mut audit) = ON_MEM_AUDIT.try_write() {
                if let Some((idx, node)) =
                    nodes.iter().enumerate().find(|(_, a)| a.addr == self.addr)
                {
                    match node.state {
                        NodeState::ACTIVE => {
                            log::warn!("node {} is already active", self.addr);
                            nodes[idx].version = self.version.clone();
                            return Ok(Async::Ready(()));
                        }
                        NodeState::PENDING_DELETE => {
                            log::info!("node {} has been recovered", self.addr);
                            nodes[idx].version = self.version.clone();
                            nodes[idx].state = NodeState::ACTIVE;
                        }
                    }
                } else {
                    nodes.push(Node::new(&self.addr, &self.version));
                }

                audit.push(Audit::new(
                    &self.addr,
                    NodeState::ACTIVE,
                    &self.version,
                    ts(),
                ));

                return Ok(Async::Ready(()));
            }
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
