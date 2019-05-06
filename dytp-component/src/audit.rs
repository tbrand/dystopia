use crate::node::Node;
use crate::node_state::NodeState;
use crate::schema::audits;
use diesel::deserialize::Queryable;
use semver::Version;
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq)]
pub struct Audit {
    pub addr: SocketAddr,
    pub state: NodeState,
    pub version: Version,
    pub ts: i64,
}

impl Audit {
    pub fn new(addr: &SocketAddr, state: NodeState, version: &Version, ts: i64) -> Audit {
        Audit {
            addr: addr.clone(),
            state,
            version: version.clone(),
            ts,
        }
    }
}

impl Into<Node> for Audit {
    fn into(self) -> Node {
        Node::new(&self.addr, &self.version)
    }
}

impl Queryable<audits::SqlType, diesel::pg::Pg> for Audit {
    type Row = (String, String, String, i64);

    fn build(row: Self::Row) -> Self {
        Audit {
            addr: row.0.parse().unwrap(),
            state: row.1.parse().unwrap(),
            version: row.2.parse().unwrap(),
            ts: row.3,
        }
    }
}

#[derive(Insertable)]
#[table_name = "audits"]
pub struct AuditInsert {
    pub addr: String,
    pub state: String,
    pub version: String,
    pub ts: i64,
}

impl AuditInsert {
    pub fn new(addr: &SocketAddr, state: &NodeState, version: &Version, ts: i64) -> AuditInsert {
        let addr = format!("{}", addr);
        let state = format!("{}", state);
        let version = format!("{}", version);

        AuditInsert {
            addr,
            state,
            version,
            ts,
        }
    }
}
