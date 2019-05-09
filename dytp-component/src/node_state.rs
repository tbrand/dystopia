use crate::error::{NodeStateError, Result};
use failure::Error;
use serde_derive::Serialize;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NodeState {
    ACTIVE,
    PENDING_DELETE,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeState::ACTIVE => write!(f, "A"),
            NodeState::PENDING_DELETE => write!(f, "D"),
        }
    }
}

impl std::str::FromStr for NodeState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "A" => Ok(NodeState::ACTIVE),
            "D" => Ok(NodeState::PENDING_DELETE),
            _ => Err(NodeStateError::InvalidState { s: s.to_owned() }.into()),
        }
    }
}
