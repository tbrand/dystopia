use semver::Version;

#[derive(Debug)]
pub enum HealthRespNode {
    Ok(Version), // Response including version
    E,           // Invalid response
}

impl HealthRespNode {
    pub fn new(version: &str) -> HealthRespNode {
        match Version::parse(version) {
            Ok(v) => HealthRespNode::Ok(v),
            Err(_) => HealthRespNode::E,
        }
    }
}

impl Into<Vec<u8>> for HealthRespNode {
    fn into(self) -> Vec<u8> {
        match self {
            HealthRespNode::Ok(v) => format!("OK {}", v).into_bytes(),
            HealthRespNode::E => "E".as_bytes().to_owned(),
        }
    }
}

impl From<&[u8]> for HealthRespNode {
    fn from(m: &[u8]) -> HealthRespNode {
        let re = regex::Regex::new(r"^OK\s(.+?)$").unwrap();

        for cap in re.captures_iter(std::str::from_utf8(m).unwrap()) {
            if let Ok(v) = cap[1].parse() {
                return HealthRespNode::Ok(v);
            }
        }

        HealthRespNode::E
    }
}
