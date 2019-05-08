pub fn address<'a, 'b>(default: Option<&'a str>) -> clap::Arg<'a, 'b> {
    let mut arg = clap::Arg::with_name("address")
        .long("address")
        .short("a")
        .help("Binded address for each component. Target address for cli. (specified by `host:port`).")
        .takes_value(true)
        .required(true);

    if let Some(default) = default {
        arg = arg.default_value(default)
    }

    arg
}

pub fn cloud<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("cloud")
        .long("cloud")
        .short("c")
        .default_value("127.0.0.1:2777")
        .help("Cloud address (specified by `host:port`).")
        .takes_value(true)
}

pub fn hops<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("hops")
        .long("hops")
        .default_value("3")
        .help("Change number of hops of routing. (The value must be between '3' to '9')")
        .takes_value(true)
}

pub fn global_address<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("global-address")
        .long("global-address")
        .short("g")
        .help("Global ip with a port (specified by `host:port`).")
        .takes_value(true)
        .required(true)
}

pub fn healthcheck_interval<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("healthcheck-interval")
        .long("healthcheck-interval")
        .default_value("10")
        .help("Interval seconds of healthchecking.")
        .takes_value(true)
}

pub fn node_deletion_timeout<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("node-deletion-timeout")
        .long("node-deletion-timeout")
        .default_value("20")
        .help("A timeout seconds of node deletion.")
        .takes_value(true)
}

pub fn read_timeout<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("read-timeout")
        .long("read-timeout")
        .default_value("10")
        .help("Read timeout as secs.")
        .takes_value(true)
}

pub fn method<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("method")
        .long("method")
        .short("m")
        .default_value("health")
        .help("Method name to be executed in command line. \"method\" is only supported for now.")
        .takes_value(true)
        .required(true)
}

pub fn component<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("component")
        .long("component")
        .default_value("cloud")
        .help("A target component. One of \"cloud\", \"gateway\" or \"node\".")
        .takes_value(true)
        .required(true)
}

pub fn pretty<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("pretty")
        .long("pretty")
        .help("Pretty print json output.")
        .takes_value(false)
}
