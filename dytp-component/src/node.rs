use crate::node_state::NodeState;
use crate::schema::nodes;
use diesel::deserialize::Queryable;
use semver::Version;
use serde_derive::Serialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Serialize)]
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.addr, self.state, self.version)
    }
}

impl Into<Vec<u8>> for Node {
    fn into(self) -> Vec<u8> {
        format!("{} {} {}", self.addr, self.state, self.version).into_bytes()
    }
}

impl From<&[u8]> for Node {
    fn from(n: &[u8]) -> Node {
        let re = regex::Regex::new(r"^(.+?)\s(.+?)\s(.+?)$").unwrap();

        for cap in re.captures_iter(std::str::from_utf8(n).unwrap()) {
            let addr = cap[1].parse().unwrap();
            let state = cap[2].parse().unwrap();
            let version = cap[3].parse().unwrap();

            return Node {
                addr,
                state,
                version,
            };
        }

        unreachable!();
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
