pub mod error;

use crate::error::{CliError, Result};
use dytp_future::get_health_cloud::GetHealthCloud;
use dytp_future::get_health_gateway::GetHealthGateway;
use dytp_future::get_health_node::GetHealthNode;
use futures::prelude::*;
use serde_json::json;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

fn print_json<S>(j: Option<S>, pretty: bool)
where
    S: serde::Serialize,
{
    if let Some(j) = j {
        if pretty {
            println!("{}", serde_json::to_string_pretty(&j).unwrap());
        } else {
            println!("{}", serde_json::to_string(&j).unwrap());
        }
    } else {
        println!("{}", json!({"error": "empty response"}));
    }
}

pub fn main_inner(addr: SocketAddr, component: &str, method: &str, pretty: bool) -> Result<()> {
    log::debug!("addr={}, component={}, method={}", addr, component, method);

    let f: Box<Future<Item = (), Error = ()> + Send> = match (component, method) {
        ("cloud", "health") => Box::new(
            GetHealthCloud::new(addr)?
                .map(move |h| {
                    print_json(h, pretty);
                    ()
                })
                .map_err(|e| {
                    log::error!("cli error={:?}", e);
                }),
        ),
        ("gateway", "health") => Box::new(
            GetHealthGateway::new(addr)?
                .map(move |h| {
                    print_json(h, pretty);
                    ()
                })
                .map_err(|e| {
                    log::error!("cli error={:?}", e);
                }),
        ),
        ("node", "health") => Box::new(
            GetHealthNode::new(addr)?
                .map(move |h| {
                    print_json(h, pretty);
                    ()
                })
                .map_err(|e| {
                    log::error!("cli error={:?}", e);
                }),
        ),
        (component, method) => {
            return Err(CliError::InvalidCombination {
                component: component.to_owned(),
                method: method.to_owned(),
            }
            .into());
        }
    };

    let mut runtime = Runtime::new()?;

    runtime.spawn(f);

    if let Err(e) = runtime.shutdown_on_idle().wait() {
        log::error!("cli error={:?}", e);
    }

    Ok(())
}
