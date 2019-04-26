use crate::node_state::NodeState;
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
