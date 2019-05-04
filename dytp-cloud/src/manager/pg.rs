use crate::manager::{Manager, ManagerClone};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dytp_component::audit::Audit;
use dytp_component::node::Node;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::env;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Pg {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Pg {
    pub fn new() -> Pg {
        let database_uri = env::var("DATABASE_URL").unwrap();
        let manager = ConnectionManager::<PgConnection>::new(database_uri);
        let pool = r2d2::Pool::builder().build(manager).unwrap();

        Pg { pool }
    }
}

impl Manager for Pg {
    fn join(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(()))
    }

    fn delete(&self, addr: SocketAddr) -> Box<Future<Item = (), Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(()))
    }

    fn pending_delete(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(()))
    }

    fn list(&self, active_only: bool) -> Box<Future<Item = Vec<Node>, Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(Vec::new()))
    }

    fn sync(&self, ts: i64) -> Box<Future<Item = Vec<Audit>, Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(Vec::new()))
    }

    fn deleted_ts(&self, addr: SocketAddr) -> Box<Future<Item = i64, Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(0))
    }

    fn latest_ts(&self) -> Box<Future<Item = i64, Error = Error> + Send> {
        // TODO
        Box::new(futures::future::ok(0))
    }
}

impl ManagerClone for Pg {
    fn box_clone(&self) -> Box<Manager + Send> {
        Box::new(self.clone())
    }
}
