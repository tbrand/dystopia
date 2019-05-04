use crate::node_state::NodeState;
use crate::schema::nodes;
use diesel::deserialize::Queryable;
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

#[derive(Insertable)]
#[table_name = "nodes"]
pub struct NodeInsert {
    pub addr: String,
    pub state: String,
    pub version: String,
}

impl NodeInsert {
    pub fn new(addr: &SocketAddr, version: &Version) -> NodeInsert {
        let addr = format!("{}", addr);
        let state = format!("{}", NodeState::ACTIVE);
        let version = format!("{}", version);

        NodeInsert {
            addr,
            state,
            version,
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "nodes"]
pub struct NodeUpdate {
    pub state: Option<String>,
    pub version: Option<String>,
}

impl NodeUpdate {
    pub fn new(state: Option<&NodeState>, version: Option<&Version>) -> NodeUpdate {
        let state = state.map(|s| format!("{}", s));
        let version = version.map(|v| format!("{}", v));

        NodeUpdate { state, version }
    }
}
