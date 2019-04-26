<h1 align="center">
  <img src="https://user-images.githubusercontent.com/3483230/56796643-f002da00-684d-11e9-824f-41d0420c8d49.png" width="200px"/> Dystopia
</h1>

<p align="center">
  <i>Real Anonymity on the Internet</i>
</p>

Dystopia aims to realize real anonymity on the internet world.
It implements onion routing with original protocols inspired by [Tor](https://www.torproject.org/).
You can quickly enter the anonymous internet by the below.

```bash
curl -v https://google.com -x <TODO: public gateway> -L
```

<i>Dystopia is on super super super early stage. Any feedbacks or contributions are very welcome!</i>

## Performance evaluation

Dystopia is optimized for onion routing powered by **Rust** and its **Future**.

TODO: Add results

## Components

### Gateway

For those who want to **use** dystopia cloud.
It decides the route and encrypt/decrypt the raw payloads.
Only this component knows the routes.
So gateway should be run by the users.
The top example `curl` command uses **public** gateway. So it isn't **real** anonymity.

### Node

For those who want to join an existing cloud.
Node constructs a cloud.
It just passes through the encrypted payloads according to the protocol.
We need more nodes! If you could join our public cloud, check [TODO].

### Cloud

For those who want to **build** a owned dystopia cloud.
Cloud is a single point which manages nodes and serves the list to the gateway.

## Build and Install

```bash
# Build `dytp` binary which contains all subcommands: `gateway`, `node` and `cloud`.
cargo build --features all

# Build `dytp` binary which contains subcommands: `gateway`.
cargo build --features gateway

# Build `dytp` binary which contains subcommands: `node`.
cargo build --features node

# Build `dytp` binary which contains subcommands: `cloud`.
cargo build --features cloud
```

## Fault Tolerant

### When gateway is down

Users who use the gateway can't execute any requests.
No effects for other users.

### When node is down

The node will be removed from cloud right after checking its health.
The routes contains the die nodes will be failed.

### When cloud is down

Gateway would be failed to sync node list on cloud.
But if the nodes on the route alive, requests would be succeeded.
There is no effects on nodes except it doesn't receive healthcheck requests from cloud.

## Contribution

- Join a public cloud
- Send pull requests or patches.
- Open issues.

## TODOs
- [ ] Use Redis or MySQL to manage nodes
- [ ] Speicfying a RSA key path to acquire HA.
- [ ] Inbound outbound report for nodes.
- [ ] Create an icon and logo.
