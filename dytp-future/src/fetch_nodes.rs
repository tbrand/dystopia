use crate::error::Result;
use dytp_component::node::Node;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct FetchNodes {
    upstream: Upstream,
}

impl FetchNodes {
    pub fn new(cloud_addr: SocketAddr) -> Result<FetchNodes> {
        let mut upstream = Upstream::new(cloud_addr)?;
        let buf: Vec<u8> = plain::ToCloud::FETCH.into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(FetchNodes { upstream })
    }
}

impl Future for FetchNodes {
    type Item = Option<(i64, Vec<Node>)>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- FetchNodes");

        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                let ts_nodes: Vec<String> = std::str::from_utf8(&payload)
                    .unwrap()
                    .split(" ")
                    .map(|s| s.to_owned())
                    .collect();

                let ts: i64 = ts_nodes[0].parse().unwrap();

                if ts_nodes.len() == 1 || ts_nodes.len() % 2 != 1 {
                    return Ok(Async::Ready(Some((ts, Vec::new()))));
                }

                let addr_versions: Vec<String> =
                    ts_nodes[1..].iter().map(|n| n.to_owned()).collect();
                let mut nodes = Vec::new();

                for idx in 0..addr_versions.len() / 2 {
                    let addr = addr_versions[idx * 2].parse();
                    let version = addr_versions[idx * 2 + 1].parse();

                    if addr.is_err() || version.is_err() {
                        return Ok(Async::Ready(Some((ts, nodes))));
                    }

                    nodes.push(Node::new(&addr.unwrap(), &version.unwrap()));
                }

                return Ok(Async::Ready(Some((ts, nodes))));
            }
            Ok(Async::Ready(None)) => {
                log::warn!("failed to get node list.");
                log::warn!("this may require you to change cloud endpoint if this happens again.");

                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                task::current().notify();

                return Ok(Async::NotReady);
            }
            Err(e) => {
                log::warn!("failed to get node list due to error={:?}", e);

                return Ok(Async::Ready(None));
            }
        }
    }
}
