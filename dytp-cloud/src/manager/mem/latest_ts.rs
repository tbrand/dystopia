use crate::manager::mem::ON_MEM_AUDIT;
use failure::Error;
use futures::prelude::*;
use tokio::prelude::*;

pub struct LatestTs;

impl LatestTs {
    pub fn new() -> LatestTs {
        LatestTs {}
    }
}

impl Future for LatestTs {
    type Item = i64;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(audit) = ON_MEM_AUDIT.try_read() {
            let ts = if audit.len() > 0 {
                audit[audit.len() - 1].ts
            } else {
                0
            };

            return Ok(Async::Ready(ts));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
