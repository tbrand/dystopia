use crate::state::State;
use dytp_connection::prelude::*;
use failure::Error;
use futures::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct PubKey {
    state: Arc<RwLock<State>>,
    origin: Origin,
}

impl PubKey {
    pub fn new(state: Arc<RwLock<State>>, origin: Origin) -> PubKey {
        PubKey { state, origin }
    }
}

impl Future for PubKey {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Ok(state) = self.state.try_read() {
            let pub_key = state.rsa.public_key_to_der().unwrap();

            self.origin.write(&pub_key)?;
            self.origin.flush()?;

            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}
