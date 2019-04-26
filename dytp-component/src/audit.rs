use crate::node::Node;
use crate::node_state::NodeState;
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
