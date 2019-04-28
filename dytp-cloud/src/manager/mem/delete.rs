use crate::manager::mem::ON_MEM_NODES;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

pub struct Delete {
    addr: SocketAddr,
}

impl Delete {
    pub fn new(addr: SocketAddr) -> Delete {
        Delete { addr }
    }
}

impl Future for Delete {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(mut nodes) = ON_MEM_NODES.try_write() {
            if let Some(idx) = nodes
                .iter()
                .enumerate()
                .find(|(_, n)| n.addr == self.addr)
                .map(|(idx, _)| idx)
            {
                nodes.remove(idx);
            }

            return Ok(Async::Ready(()));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
