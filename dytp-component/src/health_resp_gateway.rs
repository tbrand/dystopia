use crate::node::Node;
use semver::Version;

// TODO:
// May be it's too heavy to return every nodes every times.
// We need pagination or other logics to limit number of return nodes.
#[derive(Debug)]
pub enum HealthRespGateway {
    Ok((Version, Vec<Node>)),
    E,
}

impl HealthRespGateway {
    pub fn new(version: &str, nodes: &[Node]) -> HealthRespGateway {
        match Version::parse(version) {
            Ok(v) => HealthRespGateway::Ok((v, nodes.to_owned())),
            Err(_) => HealthRespGateway::E,
        }
    }
}

impl Into<Vec<u8>> for HealthRespGateway {
    fn into(self) -> Vec<u8> {
        match self {
            HealthRespGateway::Ok((version, nodes)) => format!(
                "{} {}",
                version,
                nodes
                    .iter()
                    .map(|n| format!("{}", n))
                    .collect::<Vec<String>>()
                    .join(" ")
            )
            .into_bytes(),
            HealthRespGateway::E => "E".as_bytes().to_owned(),
        }
    }
}

impl From<&[u8]> for HealthRespGateway {
    fn from(n: &[u8]) -> HealthRespGateway {
        let version_nodes = std::str::from_utf8(n)
            .unwrap()
            .split(" ")
            .collect::<Vec<&str>>();

        if version_nodes.len() % 3 != 1 {
            return HealthRespGateway::E;
        }

        let version = version_nodes[0].parse().unwrap();
        let nodes_len = (version_nodes.len() - 1) / 3;
        let mut nodes = Vec::new();

        for idx in 0..nodes_len {
            let addr = version_nodes[idx * 3 + 1].parse().unwrap();
            let state = version_nodes[idx * 3 + 2].parse().unwrap();
            let version = version_nodes[idx * 3 + 3].parse().unwrap();

            nodes.push(Node {
                addr,
                state,
                version,
            });
        }

        HealthRespGateway::Ok((version, nodes))
    }
}
