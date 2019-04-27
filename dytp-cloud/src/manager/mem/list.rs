use crate::manager::mem::ON_MEM_NODES;
use dytp_component::node::Node;
use dytp_component::node_state::NodeState;
use failure::Error;
use futures::prelude::*;
use tokio::prelude::*;

pub struct List {
    active_only: bool,
}

impl List {
    pub fn new(active_only: bool) -> List {
        List { active_only }
    }
}

impl Future for List {
    type Item = Vec<Node>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- List");

        if let Ok(nodes) = ON_MEM_NODES.try_read() {
            let nodes = if self.active_only {
                nodes
                    .iter()
                    .filter(|n| n.state == NodeState::ACTIVE)
                    .map(|n| n.clone())
                    .collect::<Vec<Node>>()
            } else {
                nodes.clone()
            };

            return Ok(Async::Ready(nodes));
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}
