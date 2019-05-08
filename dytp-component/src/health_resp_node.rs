use semver::Version;
use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthRespNode {
    version: Version, // Response including version
}

impl HealthRespNode {
    pub fn new(version: &str) -> HealthRespNode {
        HealthRespNode {
            version: Version::parse(version).unwrap(),
        }
    }
}

impl Into<Vec<u8>> for HealthRespNode {
    fn into(self) -> Vec<u8> {
        format!("OK {}", self.version).into_bytes()
    }
}

impl From<&[u8]> for HealthRespNode {
    fn from(m: &[u8]) -> HealthRespNode {
        let re = regex::Regex::new(r"^OK\s(.+?)$").unwrap();

        for cap in re.captures_iter(std::str::from_utf8(m).unwrap()) {
            if let Ok(v) = cap[1].parse() {
                return HealthRespNode { version: v };
            }
        }

        unreachable!();
    }
}
