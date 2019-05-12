use crate::error::Result;
use crate::state::State;
use bytes::BytesMut;
use dytp_connection::prelude::*;
use dytp_protocol::delim::Delim;
use failure::Error;
use futures::prelude::*;
use openssl::rsa::Padding;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

#[derive(Debug, PartialEq)]
enum Handshake {
    RecvAesKey { hop: u8 },
    RecvRely { hop: u8 },
    Done,
}

#[derive(Debug)]
pub struct Rely {
    state: Arc<RwLock<State>>,
    origin: Origin,
    upstream: Upstream,
    hop: u8,
    tls: bool,
    aes_key_iv: Option<Vec<u8>>,
    handshake: Handshake,
    pending_buf: BytesMut,
    origin_closed: bool,
    upstream_closed: bool,
}

impl Rely {
    pub fn new(
        state: Arc<RwLock<State>>,
        mut origin: Origin,
        mut upstream: Upstream,
        hop: u8,
        tls: bool,
    ) -> Rely {
        log::debug!(
            "rely: hop={}, peer={}, tls={}",
            hop,
            upstream.peer().unwrap(),
            tls
        );

        origin.set_read_delim(Delim::Dytp);
        origin.set_write_delim(Delim::Dytp);

        if hop == 0 {
            if tls {
                upstream.set_read_delim(Delim::None);
                upstream.set_write_delim(Delim::None);
            } else {
                upstream.set_read_delim(Delim::Http);
                upstream.set_write_delim(Delim::Http);
                upstream.parse_http = true;
            }
        } else {
            upstream.set_read_delim(Delim::Dytp);
            upstream.set_write_delim(Delim::Dytp);
        }

        Rely {
            state,
            origin,
            upstream,
            hop,
            tls,
            aes_key_iv: None,
            handshake: Handshake::RecvAesKey { hop },
            pending_buf: BytesMut::new(),
            origin_closed: false,
            upstream_closed: false,
        }
    }

    fn rsa_decrypt(&self, payload: &[u8]) -> Vec<u8> {
        let state = self.state.read().unwrap();
        let mut buf = vec![0; state.rsa.size() as usize];

        let d = state
            .rsa
            .private_decrypt(payload, &mut buf, Padding::PKCS1)
            .unwrap();

        buf[0..d].to_vec()
    }

    fn aes_decrypt(&mut self, payload: &[u8]) -> Vec<u8> {
        self.aes_key_iv
            .as_ref()
            .map(|key_iv| {
                decrypt(
                    Cipher::aes_256_cbc(),
                    &key_iv[0..32],
                    Some(&key_iv[32..48]),
                    payload,
                )
                .unwrap()
            })
            .unwrap()
    }

    fn aes_encrypt(&mut self, payload: &[u8]) -> Vec<u8> {
        self.aes_key_iv
            .as_ref()
            .map(|key_iv| {
                encrypt(
                    Cipher::aes_256_cbc(),
                    &key_iv[0..32],
                    Some(&key_iv[32..48]),
                    payload,
                )
                .unwrap()
            })
            .unwrap()
    }

    fn proxy(&mut self, payload: &[u8]) -> Result<()> {
        self.upstream.write(payload)?;
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

                match self.handshake {
                    Handshake::RecvAesKey { hop } => {
                        if hop == self.hop {
                            self.aes_key_iv = Some(self.rsa_decrypt(&payload));
                        } else {
                            self.proxy(&payload)?;
                        }

                        if hop == 0 {
                            self.handshake = Handshake::Done;
                        } else {
                            self.handshake = Handshake::RecvRely { hop: hop - 1 };
                        }
                    }
                    Handshake::RecvRely { hop } => {
                        self.proxy(&payload)?;
                        self.handshake = Handshake::RecvAesKey { hop };
                    }
                    Handshake::Done => {
                        let decrypted = self.aes_decrypt(&payload);

                        self.proxy(&decrypted)?;
                    }
                }
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

                match self.handshake {
                    Handshake::Done => {
                        let encrypted = self.aes_encrypt(&payload);

                        self.origin.write(&encrypted)?;
                        self.origin.flush()?;
                    }
                    _ => {
                        log::warn!("drop a payload from upstream coming during handshake.");
                    }
                }
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
            // TODO
            log::debug!("origin closed but write buffer is remaining");
        }

        if self.origin_closed && self.origin.rb_remaining() {
            // TODO
            log::debug!("origin closed but read buffer is remaining");
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
