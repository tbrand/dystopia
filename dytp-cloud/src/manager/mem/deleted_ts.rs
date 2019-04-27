use crate::manager::mem::ON_MEM_AUDIT;
use dytp_component::error::AuditError;
use dytp_component::node_state::NodeState;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

pub struct DeletedTs {
    addr: SocketAddr,
}

impl DeletedTs {
    pub fn new(addr: SocketAddr) -> DeletedTs {
        DeletedTs { addr }
    }
}

impl Future for DeletedTs {
    type Item = i64;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- DeletedTs");

        if let Ok(audit) = ON_MEM_AUDIT.try_read() {
            for a in audit.iter().rev() {
                if a.addr == self.addr {
                    match a.state {
                        NodeState::ACTIVE => {
                            log::warn!("try to delete but found active audit");
                            return Err(AuditError::InvalidAudit.into());
                        }
                        NodeState::PENDING_DELETE => {
                            return Ok(Async::Ready(a.ts));
                        }
                    }
                }
            }

            log::warn!("deletion audit not found");

            return Err(AuditError::InvalidAudit.into());
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
