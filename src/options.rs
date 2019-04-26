pub fn address<'a, 'b>(default: &'a str) -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("address")
        .long("address")
        .short("a")
        .default_value(default)
        .help("Binded address (specified by `host:port`).")
        .takes_value(true)
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
}

pub fn node_deletion_timeout<'a, 'b>() -> clap::Arg<'a, 'b> {
    clap::Arg::with_name("node-deletion-timeout")
        .long("node-deletion-timeout")
        .default_value("20")
        .help("A timeout seconds of node deletion.")
}
