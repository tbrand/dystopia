#![recursion_limit = "128"]

pub mod error;
pub mod origin;
pub mod request;
pub mod upstream;
pub mod prelude {
    pub use super::origin::Origin;
    pub use super::request::{Request, RequestContext};
    pub use super::upstream::Upstream;
    pub use super::Connection;
    pub use std::io::Write;
}

use bytes::{BufMut, BytesMut};
use dytp_protocol as protocol;
use dytp_protocol::delim::Delim;
use failure::Error;
use futures::prelude::*;
use tokio::prelude::*;

pub trait Connection {
    fn wb(&self) -> &BytesMut;
    fn wb_mut(&mut self) -> &mut BytesMut;
    fn rb(&self) -> &BytesMut;
    fn rb_mut(&mut self) -> &mut BytesMut;
    fn read_delim(&self) -> &Delim;
    fn read_delim_mut(&mut self) -> &mut Delim;
    fn write_delim(&self) -> &Delim;
    fn write_delim_mut(&mut self) -> &mut Delim;
    fn fill(&mut self) -> Poll<(), Error>;

    fn set_read_delim(&mut self, delim: Delim) {
        *self.read_delim_mut() = delim;
    }

    fn set_write_delim(&mut self, delim: Delim) {
        *self.write_delim_mut() = delim;
    }

    fn remaining(&self) -> bool {
        !self.wb().is_empty() || !self.rb().is_empty()
    }

    fn try_write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self.write_delim() {
            Delim::Dytp => {
                let protocol = protocol::Protocol::from(buf);
                self.wb_mut().reserve(protocol.size());

                let protocol_buf: BytesMut = protocol.into();
                self.wb_mut().put(&protocol_buf);
            }
            Delim::Http => {
                self.wb_mut().reserve(buf.len() + 2);
                self.wb_mut().put(buf);
                self.wb_mut().put(b"\r\n" as &[u8]);
            }
            Delim::None => {
                self.wb_mut().reserve(buf.len());
                self.wb_mut().put(buf);
            }
        }

        Ok(buf.len())
    }

    fn try_read(&mut self) -> Poll<Option<BytesMut>, Error> {
        let disconnected = self.fill()?.is_ready();

        if !self.rb().is_empty() {
            if let Some(payload) = match self.read_delim() {
                Delim::Dytp => {
                    if let Some(p) = protocol::parse(self.rb_mut()) {
                        return Ok(Async::Ready(Some((p.1).0)));
                    } else {
                        return Ok(Async::NotReady);
                    }
                }
                Delim::Http => self
                    .rb()
                    .windows(2)
                    .enumerate()
                    .find(|&(_, bytes)| bytes == b"\r\n")
                    .map(|(i, _)| i)
                    .map(|i| {
                        let mut p = self.rb_mut().split_to(i + 2);
                        p.split_off(i);
                        p
                    }),
                Delim::None => {
                    let len = self.rb().len();
                    Some(self.rb_mut().split_to(len))
                }
            } {
                return Ok(Async::Ready(Some(payload)));
            }
        }

        if disconnected {
            Ok(Async::Ready(None))
        } else {
            task::current().notify();

            Ok(Async::NotReady)
        }
    }
}
