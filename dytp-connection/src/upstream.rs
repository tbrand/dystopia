use crate::Connection;
use bytes::BytesMut;
use dytp_protocol::delim::Delim;
use failure::Error;
use futures::try_ready;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::prelude::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Upstream {
    stream: TcpStream,
    rb: BytesMut,
    wb: BytesMut,
    read_delim: Delim,
    write_delim: Delim,
    read_timeout: Duration,
    read_since: Option<Instant>,
    pub parse_http: bool,
}

impl Upstream {
    pub fn peer(&self) -> Result<std::net::SocketAddr> {
        self.stream.peer_addr().map_err(|e| e.into())
    }

    fn parse_http(&mut self) -> Poll<(), Error> {
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = httparse::Response::new(&mut headers);

        match req.parse(&self.rb)? {
            httparse::Status::Complete(body_start) => {
                for header in headers.iter() {
                    if header.name == "Content-Length" {
                        let len: usize = std::str::from_utf8(header.value)?.parse()?;

                        if self.rb.len() - body_start == len {
                            if !self.rb.ends_with(b"\r\n") {
                                // It doesn't end with http delimiter.
                                // Change the mode from Delim::Http to Delim::None
                                self.read_delim = Delim::None;
                            }

                            return Ok(Async::Ready(()));
                        } else {
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
            httparse::Status::Partial => {
                return Ok(Async::NotReady);
            }
        }

        log::debug!("content-length header not found");

        Ok(Async::Ready(()))
    }
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

    fn read_timeout(&self) -> &Duration {
        &self.read_timeout
    }

    fn read_timeout_mut(&mut self) -> &mut Duration {
        &mut self.read_timeout
    }

    fn read_since(&self) -> &Option<Instant> {
        &self.read_since
    }

    fn read_since_mut(&mut self) -> &mut Option<Instant> {
        &mut self.read_since
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
            read_timeout: Duration::from_secs(5),
            read_since: None,
            parse_http: false,
        })
    }

    pub fn new_with_timeout<A: ToSocketAddrs>(addr: A, read_timeout: u64) -> Result<Upstream> {
        let stream = TcpStream::connect(addr)?;
        let _ = stream.set_nodelay(true);
        let _ = stream.set_nonblocking(true);
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_read_timeout(Some(Duration::from_secs(read_timeout)));

        Ok(Upstream {
            stream,
            rb: BytesMut::new(),
            wb: BytesMut::new(),
            read_delim: Delim::Dytp,
            write_delim: Delim::Dytp,
            read_timeout: Duration::from_secs(read_timeout),
            read_since: None,
            parse_http: false,
        })
    }
}

impl Future for Upstream {
    type Item = Option<BytesMut>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.read_since().is_none() {
            *self.read_since_mut() = Some(Instant::now());
        }

        let disconnected = self.fill()?.is_ready();

        if !self.rb.is_empty() && self.parse_http {
            try_ready!(self.parse_http());

            self.parse_http = false;
        }

        if !self.rb.is_empty() {
            if let Some(payload) = self.try_read_delim() {
                *self.read_since_mut() = None;

                return Ok(Async::Ready(Some(payload)));
            }
        }

        if disconnected {
            Ok(Async::Ready(None))
        } else {
            if Instant::now().duration_since(*self.read_since().as_ref().unwrap())
                > *self.read_timeout()
            {
                log::debug!("read timeout");

                return Ok(Async::Ready(None));
            }

            task::current().notify();

            Ok(Async::NotReady)
        }
    }
}
