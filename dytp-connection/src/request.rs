use crate::error::{RequestError, Result};
use crate::Connection;
use bytes::BytesMut;
use dytp_protocol::delim::Delim;
use failure::Error;
use futures::prelude::*;
use futures::try_ready;
use http::uri::Uri;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpStream;
use tokio::prelude::*;

#[derive(Debug)]
pub struct RequestContext {
    pub tls: bool,
    pub buf: Vec<u8>,
    pub ip: SocketAddr,
}

fn ip(req: &httparse::Request, port: u16) -> Result<IpAddr> {
    if req.path.is_none() {
        return Err(RequestError::PathNotFound.into());
    }

    let path = req.path.unwrap();
    let uri: Uri = path.parse()?;

    if uri.host().is_none() {
        return Err(RequestError::HostNotFound.into());
    }

    let host = uri.host().unwrap();

    match (host, port).to_socket_addrs().map(|iter| {
        iter.map(|socket_address| {
            // TODO: remove
            log::debug!("socket_address={:?}", socket_address);

            socket_address.ip()
        })
        .collect::<Vec<IpAddr>>()
    }) {
        Ok(mut ip) => {
            if ip.len() == 0 {
                return Err(RequestError::LookupFailure {
                    host: host.to_owned(),
                }
                .into());
            } else {
                return Ok(ip.pop().unwrap());
            }
        }
        Err(e) => {
            log::error!("ip lookup failure due to error={:?}", e);
            Err(e.into())
        }
    }
}

fn port(req: &httparse::Request) -> Result<u16> {
    let port: u16;

    if req.path.is_none() {
        return Err(RequestError::PathNotFound.into());
    }

    let path = req.path.unwrap();
    let uri: Uri = path.parse()?;

    if let Some(p) = uri.port_u16() {
        port = p;
    } else {
        if let Some(scheme) = uri.scheme_str() {
            match scheme {
                "https" | "wss" => {
                    port = 443;
                }
                _ => {
                    port = 80;
                }
            }
        } else {
            if req.method == Some("CONNECT") {
                port = 443;
            } else {
                port = 80;
            }
        }
    }

    Ok(port)
}

fn tls(req: &httparse::Request) -> bool {
    req.method == Some("CONNECT")
}

pub fn parse(buf: &[u8]) -> Result<Option<RequestContext>> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    let res = req.parse(buf)?;

    if res.is_partial() {
        return Ok(None);
    }

    let port = port(&req)?;
    let ip = ip(&req, port)?;
    let tls = tls(&req);

    let request = RequestContext {
        tls,
        buf: buf.to_owned(),
        ip: format!("{}:{}", ip, port).parse().unwrap(),
    };

    Ok(Some(request))
}

#[derive(Debug)]
pub struct Request {
    stream: TcpStream,
    http_buf: BytesMut,
    rb: BytesMut,
    wb: BytesMut,
    read_delim: Delim,
    write_delim: Delim,
}

impl Connection for Request {
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
            self.rb.reserve(1024);

            let n = try_ready!(self.stream.read_buf(&mut self.rb));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}

impl Request {
    pub fn new(stream: TcpStream) -> Self {
        Request {
            stream,
            http_buf: BytesMut::new(),
            rb: BytesMut::new(),
            wb: BytesMut::new(),
            read_delim: Delim::Http,
            write_delim: Delim::Http,
        }
    }

    pub fn stream(self) -> TcpStream {
        self.stream
    }
}

impl Write for Request {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        while !self.wb.is_empty() {
            match self.stream.poll_write(&self.wb) {
                Ok(Async::Ready(n)) => {
                    let _ = self.wb.split_to(n);
                }
                Ok(Async::NotReady) => {
                    return Err(std::io::ErrorKind::BrokenPipe.into());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl Stream for Request {
    type Item = RequestContext;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match try_ready!(self.try_read()) {
            Some(payload) => {
                log::debug!("read {} bytes", payload.len());

                self.http_buf.extend_from_slice(&payload);
                self.http_buf.extend_from_slice(b"\r\n");

                if let Some(context) = parse(&self.http_buf)? {
                    log::debug!("context found");
                    return Ok(Async::Ready(Some(context)));
                } else {
                    log::debug!("context not found");
                }
            }
            None => return Ok(Async::Ready(None)),
        }

        task::current().notify();

        Ok(Async::NotReady)
    }
}

pub mod prelude {
    pub use super::Request;
    pub use std::io::prelude::*;
    pub use tokio::prelude::*;
}
