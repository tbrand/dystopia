use dytp_component::audit::Audit;
use dytp_component::node::Node;
use failure::Error;
use futures::prelude::*;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

lazy_static! {
    pub static ref NODES: Arc<RwLock<Vec<Node>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Debug)]
pub struct GetRoute {
    hops: usize,
}

impl GetRoute {
    pub fn new(hops: usize) -> GetRoute {
        GetRoute { hops }
    }
}

impl Future for GetRoute {
    type Item = Option<Vec<SocketAddr>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- GetRoute");

        if let Ok(nodes) = NODES.try_read() {
            if nodes.len() < self.hops {
                log::warn!("gateway doesn't know enough nodes for hops={}", self.hops);
                log::warn!("wait for a while...");

                return Ok(Async::Ready(None));
            }

            let mut rng = &mut rand::thread_rng();
            let route: Vec<SocketAddr> = nodes
                .choose_multiple(&mut rng, self.hops as usize)
                .cloned()
                .map(|node| node.addr)
                .collect();

            Ok(Async::Ready(Some(route)))
        } else {
            task::current().notify();

            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct GetAllNodes;

impl GetAllNodes {
    pub fn new() -> GetAllNodes {
        GetAllNodes {}
    }
}

impl Future for GetAllNodes {
    type Item = Vec<Node>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- GetAllNodes");

        if let Ok(nodes) = NODES.try_read() {
            return Ok(Async::Ready(nodes.clone()));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}

#[derive(Debug)]
pub struct RegisterNodes {
    nodes: Vec<Node>,
}

impl RegisterNodes {
    pub fn new(nodes: Vec<Node>) -> RegisterNodes {
        RegisterNodes { nodes }
    }
}

impl Future for RegisterNodes {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- RegisterNodes");

        if let Ok(mut ns) = NODES.try_write() {
            ns.clear();

            self.nodes.iter().for_each(|node| {
                log::info!("ADD: {} ({})", node.addr, node.version);
                ns.push(node.clone());
            });

            Ok(Async::Ready(()))
        } else {
            task::current().notify();

            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct RegisterNode {
    audit: Audit,
}

impl RegisterNode {
    pub fn new(audit: Audit) -> RegisterNode {
        RegisterNode { audit }
    }
}

impl Future for RegisterNode {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- RegisterNode");

        if let Ok(mut ns) = NODES.try_write() {
            if None == ns.iter().find(|n| n.addr == self.audit.addr) {
                ns.push(self.audit.clone().into());
            }

            return Ok(Async::Ready(()));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}

#[derive(Debug)]
pub struct RemoveNode {
    addr: SocketAddr,
}

impl RemoveNode {
    pub fn new(addr: SocketAddr) -> RemoveNode {
        RemoveNode { addr }
    }
}

impl Future for RemoveNode {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- RemoveNode");

        if let Ok(mut ns) = NODES.try_write() {
            if let Some(idx) = ns
                .iter()
                .enumerate()
                .find(|(_, n)| n.addr == self.addr)
                .map(|(idx, _)| idx)
            {
                ns.remove(idx);

                return Ok(Async::Ready(()));
            }
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
