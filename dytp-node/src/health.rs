use clap::crate_version;
use dytp_component::health_resp::HealthResp;
use dytp_connection::prelude::*;
use failure::Error;
use futures::prelude::*;

#[derive(Debug)]
pub struct Health {
    origin: Origin,
}

impl Health {
    pub fn new(origin: Origin) -> Health {
        Health { origin }
    }
}

impl Future for Health {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let res: Vec<u8> = HealthResp::new(crate_version!()).into();

        self.origin.write(&res)?;
        self.origin.flush()?;

        Ok(Async::Ready(()))
    }
}
