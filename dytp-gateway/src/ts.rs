use failure::Error;
use futures::prelude::*;
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

lazy_static! {
    pub static ref LATEST_TS: Arc<RwLock<i64>> = Arc::new(RwLock::new(0));
}

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
        if let Ok(latest_ts) = LATEST_TS.read() {
            return Ok(Async::Ready(*latest_ts));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}

pub struct RecordTs {
    ts: i64,
}

impl RecordTs {
    pub fn new(ts: i64) -> RecordTs {
        RecordTs { ts }
    }
}

impl Future for RecordTs {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(mut latest_ts) = LATEST_TS.write() {
            *latest_ts = self.ts;

            return Ok(Async::Ready(()));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
