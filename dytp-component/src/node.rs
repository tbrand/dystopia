use crate::node_state::NodeState;
use crate::schema::nodes;
use diesel::deserialize::Queryable;
use diesel::prelude::*;
use semver::Version;
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub addr: SocketAddr,
    pub state: NodeState,
    pub version: Version,
}

impl Node {
    pub fn new(addr: &SocketAddr, version: &Version) -> Node {
        Node {
            addr: addr.clone(),
            state: NodeState::ACTIVE,
            version: version.clone(),
        }
    }
}

impl Queryable<nodes::SqlType, diesel::pg::Pg> for Node {
    type Row = (String, String, String);

    fn build(row: Self::Row) -> Self {
        Node {
            addr: row.0.parse().unwrap(),
            state: row.1.parse().unwrap(),
            version: row.2.parse().unwrap(),
        }
    }
}

impl Insertable<nodes::table> for Node {
    type Values = (String, String, String);

    fn values(self) -> Self::Values {
        let addr = format!("{}", self.addr);
        let state = format!("{}", self.state);
        let version = format!("{}", self.version);

        (addr, state, version)
    }
}
