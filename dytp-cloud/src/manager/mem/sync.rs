use crate::manager::mem::ON_MEM_AUDIT;
use dytp_component::audit::Audit;
use failure::Error;
use futures::prelude::*;
use tokio::prelude::*;

pub struct Sync {
    ts: i64,
}

impl Sync {
    pub fn new(ts: i64) -> Sync {
        Sync { ts }
    }
}

impl Future for Sync {
    type Item = Vec<Audit>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(audit) = ON_MEM_AUDIT.try_read() {
            let mut audit_res = Vec::new();

            for a in audit.iter().rev() {
                if a.ts > self.ts {
                    audit_res.push(a.clone())
                } else {
                    break;
                }
            }

            return Ok(Async::Ready(audit_res));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
