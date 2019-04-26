use semver::Version;

#[derive(Debug)]
pub enum HealthResp {
    Ok(Version), // Response including version
    E,           // Invalid response
}

impl HealthResp {
    pub fn new(version: &str) -> HealthResp {
        match Version::parse(version) {
            Ok(v) => HealthResp::Ok(v),
            Err(_) => HealthResp::E,
        }
    }
}

impl Into<Vec<u8>> for HealthResp {
    fn into(self) -> Vec<u8> {
        match self {
            HealthResp::Ok(v) => format!("OK {}", v).into_bytes(),
            HealthResp::E => "E".as_bytes().to_owned(),
        }
    }
}

impl From<&[u8]> for HealthResp {
    fn from(m: &[u8]) -> HealthResp {
        let re = regex::Regex::new(r"^OK\s(.+?)$").unwrap();

        for cap in re.captures_iter(std::str::from_utf8(m).unwrap()) {
            if let Ok(v) = cap[1].parse() {
                return HealthResp::Ok(v);
            }
        }

        HealthResp::E
    }
}
