// https://github.com/emk/rust-musl-builder/issues/64
extern crate openssl;

#[path = "../options.rs"]
mod options;

use dytp::error::Result;

#[cfg(any(feature = "gateway", feature = "all"))]
use dytp::gateway;

#[cfg(any(feature = "node", feature = "all"))]
use dytp::node;

#[cfg(any(feature = "cloud", feature = "all"))]
use dytp::cloud;

#[cfg(any(feature = "cli", feature = "all"))]
use dytp::cli;

fn subcommand_gateway<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("gateway")
        .about("A gateway into Dystopia")
        .arg(options::address(Some("127.0.0.1:2888")))
        .arg(options::cloud())
        .arg(options::hops())
        .arg(options::read_timeout())
}

fn subcommand_node<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("node")
        .about("A node of Dystopia")
        .arg(options::address(Some("127.0.0.1:3000")))
        .arg(options::global_address())
        .arg(options::cloud())
        .arg(options::read_timeout())
}

fn subcommand_cloud<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("cloud")
        .about("A cloud of Dystopia")
        .arg(options::address(Some("127.0.0.1:2777")))
        .arg(options::healthcheck_interval())
        .arg(options::node_deletion_timeout())
        .arg(options::read_timeout())
}

fn subcommand_cli<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("cli")
        .about("Command line execution")
        .arg(options::address(None))
        .arg(options::component())
        .arg(options::method())
}

#[cfg(any(feature = "gateway", feature = "all"))]
fn exec_gateway(matches: &clap::ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("gateway").unwrap();
    let addr = matches.value_of("address").unwrap().parse()?;
    let cloud_addr = matches.value_of("cloud").unwrap().parse()?;
    let hops = matches.value_of("hops").unwrap().parse()?;
    let read_timeout = matches.value_of("read-timeout").unwrap().parse()?;

    if hops <= 2 {
        log::error!("The hop must be greater than 2.");
        return Ok(());
    }

    if hops >= 10 {
        log::error!("The hop must be less than 10.");
        return Ok(());
    }

    gateway::main_inner(addr, cloud_addr, hops, read_timeout)?;

    Ok(())
}

#[cfg(not(any(feature = "gateway", feature = "all")))]
fn exec_gateway(_maches: &clap::ArgMatches) -> Result<()> {
    log::error!("subcommand `gateway` is not installed in this binary.");

    Ok(())
}

#[cfg(any(feature = "node", feature = "all"))]
fn exec_node(matches: &clap::ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("node").unwrap();
    let addr = matches.value_of("address").unwrap().parse()?;
    let global_addr = matches.value_of("global-address").unwrap().parse()?;
    let cloud_addr = matches.value_of("cloud").unwrap().parse()?;
    let read_timeout = matches.value_of("read-timeout").unwrap().parse()?;

    node::main_inner(addr, global_addr, cloud_addr, read_timeout)?;

    Ok(())
}

#[cfg(not(any(feature = "node", feature = "all")))]
fn exec_node(_maches: &clap::ArgMatches) -> Result<()> {
    log::error!("subcommand `node` is not installed in this binary.");

    Ok(())
}

#[cfg(any(feature = "cloud", feature = "all"))]
fn exec_cloud(matches: &clap::ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("cloud").unwrap();
    let addr = matches.value_of("address").unwrap().parse()?;
    let healthcheck_interval = matches.value_of("healthcheck-interval").unwrap().parse()?;
    let node_deletion_timeout = matches.value_of("node-deletion-timeout").unwrap().parse()?;
    let read_timeout = matches.value_of("read-timeout").unwrap().parse()?;

    cloud::main_inner(
        addr,
        healthcheck_interval,
        node_deletion_timeout,
        read_timeout,
    )?;

    Ok(())
}

#[cfg(not(any(feature = "cloud", feature = "all")))]
fn exec_cloud(_maches: &clap::ArgMatches) -> Result<()> {
    log::error!("subcommand `cloud` is not installed in this binary.");

    Ok(())
}

#[cfg(any(feature = "cli", feature = "all"))]
fn exec_cli(matches: &clap::ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("cli").unwrap();
    let addr = matches.value_of("address").unwrap().parse()?;
    let component = matches.value_of("component").unwrap();
    let method = matches.value_of("method").unwrap();

    cli::main_inner(addr, component, method)?;

    Ok(())
}

#[cfg(not(any(feature = "cli", feature = "all")))]
fn exec_cli(_matches: &clap::ArgMatches) -> Result<()> {
    log::error!("subcommand `cli` is not installed in this binary.");

    Ok(())
}

fn main() -> Result<()> {
    openssl_probe::init_ssl_cert_env_vars();

    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "dytp=info");
    }

    env_logger::init();

    let matches = clap::App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .subcommand(subcommand_gateway())
        .subcommand(subcommand_node())
        .subcommand(subcommand_cloud())
        .subcommand(subcommand_cli())
        .get_matches();

    match matches.subcommand_name() {
        Some("gateway") => exec_gateway(&matches),
        Some("node") => exec_node(&matches),
        Some("cloud") => exec_cloud(&matches),
        Some("cli") => exec_cli(&matches),
        Some(e) => {
            log::error!("unknown subcommand {:?}.", e);

            Ok(())
        }
        None => {
            log::warn!("subcommand is not specified.");

            Ok(())
        }
    }
}
