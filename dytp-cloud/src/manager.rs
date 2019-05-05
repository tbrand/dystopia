pub mod mem;
pub mod pg;

use dytp_component::audit::Audit;
use dytp_component::node::Node;

use chrono::prelude::*;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::net::SocketAddr;

pub fn ts() -> i64 {
    Utc::now().timestamp_nanos()
}

pub enum ManagerType {
    MEM,
    PG { database_url: String },
}

pub trait ManagerClone {
    fn box_clone(&self) -> Box<Manager + Send>;
}

pub trait Manager: ManagerClone {
    fn join(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send>;
    fn delete(&self, addr: SocketAddr) -> Box<Future<Item = (), Error = Error> + Send>;
    fn pending_delete(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send>;
    fn list(&self, active_only: bool) -> Box<Future<Item = Vec<Node>, Error = Error> + Send>;
    fn sync(&self, ts: i64) -> Box<Future<Item = Vec<Audit>, Error = Error> + Send>;
    fn deleted_ts(&self, addr: SocketAddr) -> Box<Future<Item = i64, Error = Error> + Send>;
    fn latest_ts(&self) -> Box<Future<Item = i64, Error = Error> + Send>;
}

impl Clone for Box<Manager + Send> {
    fn clone(&self) -> Box<Manager + Send> {
        self.box_clone()
    }
}

pub fn create(t: ManagerType) -> Box<Manager + Send> {
    match t {
        ManagerType::MEM => Box::new(mem::Mem::new()),
        ManagerType::PG { database_url } => Box::new(pg::Pg::new(&database_url)),
    }
}
