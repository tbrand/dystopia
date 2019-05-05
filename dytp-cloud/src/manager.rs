pub mod mem;
pub mod pg;

use crate::error::{DatabaseError, Result};
use chrono::prelude::*;
use dytp_component::audit::Audit;
use dytp_component::node::Node;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::env;
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

pub fn create() -> Result<Box<Manager + Send>> {
    match env::var("DATABASE_URL") {
        Ok(url) => {
            use url::Url;

            let url_parsed = Url::parse(&url)?;

            match url_parsed.scheme() {
                "postgres" => return Ok(Box::new(pg::Pg::new(&url))),
                "mysql" => {
                    log::error!("MySQL is not supported now. But will be soon!");
                    return Err(DatabaseError::InvalidDatabaseUrl { url }.into());
                }
                "sqlite" => {
                    log::error!("SQLite is not supported now. But will be soon!");
                    return Err(DatabaseError::InvalidDatabaseUrl { url }.into());
                }
                _ => return Err(DatabaseError::InvalidDatabaseUrl { url }.into()),
            }
        }
        Err(_) => return Ok(Box::new(mem::Mem::new())),
    }
}
