use crate::error::Result;
use dytp_component::audit::Audit;
use dytp_component::error::AuditError;
use dytp_connection::prelude::*;
use dytp_protocol::method::plain;
use failure::Error;
use futures::prelude::*;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct SyncAudit {
    ts: i64,
    upstream: Upstream,
}

impl SyncAudit {
    pub fn new(cloud_addr: SocketAddr, ts: i64) -> Result<SyncAudit> {
        let mut upstream = Upstream::new(cloud_addr)?;
        let buf: Vec<u8> = plain::ToCloud::SYNC { ts }.into();

        upstream.write(&buf)?;
        upstream.flush()?;

        Ok(SyncAudit { ts, upstream })
    }
}

impl Future for SyncAudit {
    type Item = Option<Vec<Audit>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- SyncAudit");

        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                if payload.len() == 0 {
                    return Ok(Async::Ready(None));
                }

                let payload: Vec<&str> =
                    std::str::from_utf8(&payload).unwrap().split(" ").collect();

                if payload.len() % 4 != 0 {
                    return Err(AuditError::InvalidAudit.into());
                }

                let mut audit = Vec::new();

                for idx in 0..payload.len() / 4 {
                    let addr = payload[idx * 4].parse();
                    let state = payload[idx * 4 + 1].parse();
                    let version = payload[idx * 4 + 2].parse();
                    let ts = payload[idx * 4 + 3].parse();

                    if addr.is_err() || state.is_err() || version.is_err() || ts.is_err() {
                        return Err(AuditError::InvalidAudit.into());
                    }

                    audit.push(Audit::new(
                        &addr.unwrap(),
                        state.unwrap(),
                        &version.unwrap(),
                        ts.unwrap(),
                    ));
                }

                return Ok(Async::Ready(Some(audit)));
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
