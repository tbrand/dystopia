pub mod delete;
pub mod deleted_ts;
pub mod join;
pub mod latest_ts;
pub mod list;
pub mod pending_delete;
pub mod sync;

use crate::manager::{Manager, ManagerClone};
use dytp_component::audit::Audit;
use dytp_component::node::Node;
use failure::Error;
use futures::prelude::*;
use lazy_static::lazy_static;
use semver::Version;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref ON_MEM_NODES: Arc<RwLock<Vec<Node>>> = Arc::new(RwLock::new(Vec::new()));
    pub static ref ON_MEM_AUDIT: Arc<RwLock<Vec<Audit>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Clone)]
pub struct Mem;

impl Mem {
    pub fn new() -> Mem {
        Mem {}
    }
}

impl Manager for Mem {
    fn join(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(join::Join::new(addr, version))
    }

    fn delete(&self, addr: SocketAddr) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(delete::Delete::new(addr))
    }

    fn pending_delete(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(pending_delete::PendingDelete::new(addr, version))
    }

    fn list(&self, active_only: bool) -> Box<Future<Item = Vec<Node>, Error = Error> + Send> {
        Box::new(list::List::new(active_only))
    }

    fn sync(&self, ts: i64) -> Box<Future<Item = Vec<Audit>, Error = Error> + Send> {
        Box::new(sync::Sync::new(ts))
    }

    fn deleted_ts(&self, addr: SocketAddr) -> Box<Future<Item = i64, Error = Error> + Send> {
        Box::new(deleted_ts::DeletedTs::new(addr))
    }

    fn latest_ts(&self) -> Box<Future<Item = i64, Error = Error> + Send> {
        Box::new(latest_ts::LatestTs::new())
    }
}

impl ManagerClone for Mem {
    fn box_clone(&self) -> Box<Manager + Send> {
        Box::new(self.clone())
    }
}
