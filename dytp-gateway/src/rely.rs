use crate::error::Result;
use crate::route_node::RouteNode;
use dytp_connection::prelude::*;
use dytp_protocol::delim::Delim;
use dytp_protocol::method::encrypted;
use failure::Error;
use futures::prelude::*;
use tokio::prelude::*;

#[derive(Debug)]
pub struct Rely {
    origin: Origin,
    upstream: Upstream,
    nodes: Vec<RouteNode>,
    tls: bool,
    origin_closed: bool,
    upstream_closed: bool,
}

impl Rely {
    pub fn new(
        mut origin: Origin,
        upstream: Upstream,
        nodes: Vec<RouteNode>,
        http_buf: &[u8],
        tls: bool,
    ) -> Result<Rely> {
        if tls {
            origin.set_read_delim(Delim::None);
        } else {
            origin.set_read_delim(Delim::Http);
        }

        origin.set_write_delim(Delim::Http);

        let mut rely = Rely {
            origin,
            upstream,
            nodes,
            tls,
            origin_closed: false,
            upstream_closed: false,
        };

        rely.handshake(http_buf)?;

        log::debug!("handshake done!");

        Ok(rely)
    }

    fn handshake(&mut self, http_buf: &[u8]) -> Result<()> {
        for (idx, node) in self.nodes.iter().enumerate() {
            let hop = self.nodes.len() - idx - 1;
            let method = encrypted::Method::RELY {
                hop: hop as u8,
                addr: node.next,
                tls: self.tls,
            };

            let h0: Vec<u8> = method.into();
            let h1 = node.aes_key_iv();

            self.upstream.write(&node.rsa_encrypt(&h0))?;
            self.upstream.write(&node.rsa_encrypt(&h1))?;
            self.upstream.flush()?;
        }

        if self.tls {
            self.origin.write(b"HTTP/1.1 200 OK")?;
            self.origin.write(b"")?;
            self.origin.flush()?;
            self.origin.set_write_delim(Delim::None);
        } else {
            // Remove last http delimiter(\r\n)
            let http_buf = &http_buf[0..http_buf.len() - 2];

            self.rely(http_buf)?;
        }

        Ok(())
    }

    fn decrypt(&self, payload: &[u8]) -> Vec<u8> {
        let mut payload = payload.to_owned();

        for node in self.nodes.iter() {
            payload = node.aes_decrypt(&payload);
        }

        payload.to_owned()
    }

    fn rely(&mut self, buf: &[u8]) -> Result<()> {
        let mut buf = buf.to_vec();

        for node in self.nodes.iter().rev() {
            buf = node.aes_encrypt(&buf);
        }

        self.upstream.write(&buf)?;
        self.upstream.flush()?;

        Ok(())
    }
}

impl Future for Rely {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut notify: bool = false;

        match self.origin.poll() {
            Ok(Async::Ready(Some(payload))) => {
                notify = true;

                self.rely(&payload)?;
            }
            Ok(Async::Ready(None)) => {
                self.origin_closed = true;
            }
            Ok(Async::NotReady) => {
                notify = true;
            }
            Err(_) => {
                notify = true;

                self.origin_closed = true;
            }
        }

        match self.upstream.poll() {
            Ok(Async::Ready(Some(payload))) => {
                notify = true;

                let decrypted = self.decrypt(&payload);

                self.origin.write(&decrypted)?;
                self.origin.flush()?;
            }
            Ok(Async::Ready(None)) => {
                self.upstream_closed = true;
            }
            Ok(Async::NotReady) => {
                notify = true;
            }
            Err(_) => {
                notify = true;

                self.upstream_closed = true;
            }
        }

        if notify {
            task::current().notify();
        }

        if self.origin_closed && self.origin.wb_remaining() {
            log::debug!("origin closed but write buffer is remaining");
            // TODO
        }

        if self.origin_closed && self.origin.rb_remaining() {
            log::debug!("origin closed but read buffer is remaining");
            // TODO
        }

        if self.upstream_closed && self.upstream.wb_remaining() {
            // TODO
            log::debug!("upstream closed but write buffer is remaining");
        }

        if self.upstream_closed && self.upstream.rb_remaining() {
            // TODO
            log::debug!("upstream closed but read buffer is remaining");
        }

        if self.origin_closed || self.upstream_closed {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}
