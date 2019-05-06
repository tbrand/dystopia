use crate::manager::mem::ON_MEM_AUDIT;
use crate::manager::mem::ON_MEM_NODES;
use crate::manager::ts;
use dytp_component::audit::Audit;
use dytp_component::node_state::NodeState;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::net::SocketAddr;
use tokio::prelude::*;

pub struct PendingDelete {
    addr: SocketAddr,
    version: Version,
}

impl PendingDelete {
    pub fn new(addr: SocketAddr, version: Version) -> PendingDelete {
        PendingDelete { addr, version }
    }
}

impl Future for PendingDelete {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(mut nodes) = ON_MEM_NODES.try_write() {
            if let Ok(mut audit) = ON_MEM_AUDIT.try_write() {
                if let Some(idx) = nodes
                    .iter()
                    .enumerate()
                    .find(|(_, n)| n.addr == self.addr)
                    .map(|(idx, _)| idx)
                {
                    nodes[idx].state = NodeState::PENDING_DELETE;
                }

                audit.push(Audit::new(
                    &self.addr,
                    NodeState::PENDING_DELETE,
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
