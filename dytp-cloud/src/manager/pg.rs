use crate::error::Result;
use crate::manager::ts;
use crate::manager::{Manager, ManagerClone};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dytp_component::audit::{Audit, AuditInsert};
use dytp_component::error::AuditError;
use dytp_component::node::{Node, NodeInsert, NodeUpdate};
use dytp_component::node_state::NodeState;
use dytp_component::schema::audits;
use dytp_component::schema::nodes;
use failure::Error;
use futures::prelude::*;
use semver::Version;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Pg {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Pg {
    pub fn new(database_url: &str) -> Pg {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder().build(manager).unwrap();

        Pg { pool }
    }
}

fn node_create(conn: &PgConnection, a: &SocketAddr, v: &Version) -> Result<Node> {
    diesel::insert_into(nodes::table)
        .values(NodeInsert::new(a, v))
        .get_result::<Node>(conn)
        .map_err(|e| e.into())
}

fn node_update(conn: &PgConnection, a: &SocketAddr, update: NodeUpdate) -> Result<Node> {
    use dytp_component::schema::nodes::dsl::*;

    diesel::update(nodes.find(format!("{}", a)))
        .set(update)
        .get_result::<Node>(conn)
        .map_err(|e| e.into())
}

fn audit_create(conn: &PgConnection, a: &SocketAddr, s: &NodeState, v: &Version) -> Result<Audit> {
    diesel::insert_into(audits::table)
        .values(AuditInsert::new(a, s, v, ts()))
        .get_result::<Audit>(conn)
        .map_err(|e| e.into())
}

impl Manager for Pg {
    fn join(&self, a: SocketAddr, v: Version) -> Box<Future<Item = (), Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let node = {
            use dytp_component::schema::nodes::dsl::*;

            nodes
                .filter(addr.eq(format!("{}", a)))
                .limit(1)
                .load::<Node>(&conn)
                .unwrap()
        };

        if node.len() == 0 {
            node_create(&conn, &a, &v).unwrap();
        } else {
            if node[0].state == NodeState::ACTIVE {
                log::warn!("node {} is already active", node[0].addr);

                node_update(&conn, &a, NodeUpdate::new(None, Some(&v))).unwrap();
            } else {
                log::info!("node {} has been recovered", node[0].addr);

                node_update(
                    &conn,
                    &a,
                    NodeUpdate::new(Some(&NodeState::ACTIVE), Some(&v)),
                )
                .unwrap();
            }
        }

        audit_create(&conn, &a, &NodeState::ACTIVE, &v).unwrap();

        Box::new(futures::future::ok(()))
    }

    fn delete(&self, a: SocketAddr) -> Box<Future<Item = (), Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let _n: Result<usize> = {
            use dytp_component::schema::nodes::dsl::*;

            diesel::delete(nodes.find(format!("{}", a)))
                .execute(&conn)
                .map_err(|e| e.into())
        };

        Box::new(futures::future::ok(()))
    }

    fn pending_delete(
        &self,
        addr: SocketAddr,
        version: Version,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();

        node_update(
            &conn,
            &addr,
            NodeUpdate::new(Some(&NodeState::PENDING_DELETE), None),
        )
        .unwrap();

        audit_create(&conn, &addr, &NodeState::PENDING_DELETE, &version).unwrap();

        Box::new(futures::future::ok(()))
    }

    fn list(&self, active_only: bool) -> Box<Future<Item = Vec<Node>, Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let nodes = if active_only {
            use dytp_component::schema::nodes::dsl::*;

            nodes
                .filter(state.eq(format!("{}", NodeState::ACTIVE)))
                .load::<Node>(&conn)
                .unwrap()
        } else {
            use dytp_component::schema::nodes::dsl::*;

            nodes.load::<Node>(&conn).unwrap()
        };

        Box::new(futures::future::ok(nodes))
    }

    fn sync(&self, t: i64) -> Box<Future<Item = Vec<Audit>, Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let audits = {
            use dytp_component::schema::audits::dsl::*;

            audits
                .select((addr, state, version, ts))
                .filter(ts.gt(t))
                .order_by(ts.desc())
                .load::<Audit>(&conn)
                .unwrap()
        };

        Box::new(futures::future::ok(audits))
    }

    fn deleted_ts(&self, a: SocketAddr) -> Box<Future<Item = i64, Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let audits = {
            use dytp_component::schema::audits::dsl::*;

            audits
                .filter(addr.eq(format!("{}", a)))
                .order_by(ts.desc())
                .limit(1)
                .load::<Audit>(&conn)
                .unwrap()
        };

        if audits.len() > 0 {
            let audit = &audits[0];

            match audit.state {
                NodeState::ACTIVE => {
                    log::warn!("try to delete but found active audit");

                    return Box::new(futures::future::err(AuditError::InvalidAudit.into()));
                }
                NodeState::PENDING_DELETE => {
                    return Box::new(futures::future::ok(audit.ts));
                }
            }
        }

        return Box::new(futures::future::err(AuditError::InvalidAudit.into()));
    }

    fn latest_ts(&self) -> Box<Future<Item = i64, Error = Error> + Send> {
        let conn = self.pool.clone().get().unwrap();
        let audits = {
            use dytp_component::schema::audits::dsl::*;

            audits
                .select((addr, state, version, ts))
                .order_by(ts.desc())
                .limit(1)
                .load::<Audit>(&conn)
                .unwrap()
        };

        let ts = if audits.len() > 0 { *&audits[0].ts } else { 0 };

        Box::new(futures::future::ok(ts))
    }
}

impl ManagerClone for Pg {
    fn box_clone(&self) -> Box<Manager + Send> {
        Box::new(self.clone())
    }
}
