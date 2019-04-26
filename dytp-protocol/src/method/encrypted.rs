use std::net::SocketAddr;

#[derive(PartialEq, Debug)]
pub enum Method {
    RELY {
        hop: u8,
        addr: SocketAddr,
        tls: bool,
    }, // Rely to another node
    E, // Invalid method
}

impl Into<Vec<u8>> for Method {
    fn into(self) -> Vec<u8> {
        match self {
            Method::RELY { hop, addr, tls } => {
                format!("RELY {} {} {}", hop, addr, tls as u8).into_bytes()
            }
            _ => b"E".to_vec(),
        }
    }
}

impl From<&[u8]> for Method {
    fn from(m: &[u8]) -> Method {
        use std::net::AddrParseError;
        use std::num::ParseIntError;

        let re = regex::Regex::new(r"^RELY\s(\d{1})\s(.+?)\s(\d{1})$").unwrap();

        for cap in re.captures_iter(std::str::from_utf8(m).unwrap()) {
            let hop: Result<u8, ParseIntError> = cap[1].parse();
            let addr: Result<SocketAddr, AddrParseError> = cap[2].parse();
            let tls: Result<u8, ParseIntError> = cap[3].parse();

            if hop.is_ok() && addr.is_ok() && tls.is_ok() {
                return Method::RELY {
                    hop: hop.unwrap(),
                    addr: addr.unwrap(),
                    tls: tls.unwrap() != 0,
                };
            }
        }

        Method::E
    }
}
