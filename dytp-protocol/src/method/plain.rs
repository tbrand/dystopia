use semver::Version;
use std::net::SocketAddr;

#[derive(PartialEq, Debug)]
pub enum Common {
    HEALTH, // Healcheck method
    E,      // Invalid method
}

impl Into<Vec<u8>> for Common {
    fn into(self) -> Vec<u8> {
        match self {
            Common::HEALTH => b"HT".to_vec(),
            Common::E => b"E".to_vec(),
        }
    }
}

impl From<&[u8]> for Common {
    fn from(m: &[u8]) -> Common {
        match m {
            b"HT" => Common::HEALTH,
            _ => Common::E,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
pub enum ToNode {
    PUB_KEY, // Send a public key
    E,       // Invalid metod
}

impl Into<Vec<u8>> for ToNode {
    fn into(self) -> Vec<u8> {
        match self {
            ToNode::PUB_KEY => b"PK".to_vec(),
            _ => b"E".to_vec(),
        }
    }
}

impl From<&[u8]> for ToNode {
    fn from(m: &[u8]) -> ToNode {
        match m {
            b"PK" => ToNode::PUB_KEY,
            _ => ToNode::E,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ToCloud {
    FETCH,                                       // Fetch a list of nodes
    SYNC { ts: i64 },                            // Sync audit logs with latest timestamp
    JOIN { addr: SocketAddr, version: Version }, // Joining request
    E,                                           // Invalid method
}

impl Into<Vec<u8>> for ToCloud {
    fn into(self) -> Vec<u8> {
        match self {
            ToCloud::FETCH => b"FC".to_vec(),
            ToCloud::SYNC { ts } => format!("SY {}", ts).into_bytes(),
            ToCloud::JOIN { addr, version } => format!("JN {} {}", addr, version).into_bytes(),
            _ => b"E".to_vec(),
        }
    }
}

impl From<&[u8]> for ToCloud {
    fn from(m: &[u8]) -> ToCloud {
        match m {
            b"FC" => ToCloud::FETCH,
            _ => {
                let re_sync = regex::Regex::new(r"^SY\s(.+?)$").unwrap();

                for cap in re_sync.captures_iter(std::str::from_utf8(m).unwrap()) {
                    if let Ok(ts) = cap[1].parse() {
                        return ToCloud::SYNC { ts };
                    }
                }

                let re_join = regex::Regex::new(r"^JN\s(.+?)\s(.+?)$").unwrap();

                for cap in re_join.captures_iter(std::str::from_utf8(m).unwrap()) {
                    let addr = cap[1].parse();
                    let version = cap[2].parse();

                    if addr.is_err() || version.is_err() {
                        return ToCloud::E;
                    }

                    let addr = addr.unwrap();
                    let version = version.unwrap();

                    return ToCloud::JOIN { addr, version };
                }

                ToCloud::E
            }
        }
    }
}
