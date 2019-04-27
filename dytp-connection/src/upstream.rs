use crate::Connection;
use bytes::BytesMut;
use dytp_protocol::delim::Delim;
use failure::Error;
use std::io::Write;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::prelude::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Upstream {
    stream: TcpStream,
    rb: BytesMut,
    wb: BytesMut,
    read_delim: Delim,
    write_delim: Delim,
}

impl Connection for Upstream {
    fn wb(&self) -> &BytesMut {
        &self.wb
    }

    fn wb_mut(&mut self) -> &mut BytesMut {
        &mut self.wb
    }

    fn rb(&self) -> &BytesMut {
        &self.rb
    }

    fn rb_mut(&mut self) -> &mut BytesMut {
        &mut self.rb
    }

    fn read_delim(&self) -> &Delim {
        &self.read_delim
    }

    fn read_delim_mut(&mut self) -> &mut Delim {
        &mut self.read_delim
    }

    fn write_delim(&self) -> &Delim {
        &self.write_delim
    }

    fn write_delim_mut(&mut self) -> &mut Delim {
        &mut self.write_delim
    }

    fn fill(&mut self) -> Poll<(), Error> {
        loop {
            let mut b = [0; 1024];

            match self.stream.read(&mut b) {
                Ok(n) => {
                    if n > 0 {
                        self.rb.extend_from_slice(&b[0..n]);
                    } else {
                        return Ok(Async::Ready(()));
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Async::NotReady)
    }
}

impl Write for Upstream {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        while !self.wb.is_empty() {
            let n = self.stream.write(&self.wb)?;

            self.wb.split_to(n);
        }

        Ok(())
    }
}

impl Upstream {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Upstream> {
        let stream = TcpStream::connect(addr)?;
        let _ = stream.set_nodelay(true);
        let _ = stream.set_nonblocking(true);
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

        Ok(Upstream {
            stream,
            rb: BytesMut::new(),
            wb: BytesMut::new(),
            read_delim: Delim::Dytp,
            write_delim: Delim::Dytp,
        })
    }
}

impl Future for Upstream {
    type Item = Option<BytesMut>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log::debug!("poll() --- Upstream");

        self.try_read()
    }
}
