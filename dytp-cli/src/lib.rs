pub mod error;

use crate::error::Result;
use std::net::SocketAddr;

pub fn main_inner(addr: SocketAddr, component: &str, method: &str) -> Result<()> {
    log::debug!("addr={}, component={}, method={}", addr, component, method);

    Ok(())
}
