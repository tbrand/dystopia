use crate::Connection;
use bytes::BytesMut;
use dytp_protocol::delim::Delim;
use failure::Error;
use futures::prelude::*;
use futures::try_ready;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::prelude::*;

#[derive(Debug)]
pub struct Origin {
    stream: TcpStream,
    rb: BytesMut,
    wb: BytesMut,
    read_delim: Delim,
    write_delim: Delim,
    read_timeout: Duration,
    read_since: Option<Instant>,
}

impl Connection for Origin {
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
            self.rb.reserve(1024);

            let n = try_ready!(self.stream.read_buf(&mut self.rb));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}

impl Origin {
    pub fn new(stream: TcpStream) -> Self {
        Origin {
            stream,
            rb: BytesMut::new(),
            wb: BytesMut::new(),
            read_delim: Delim::Dytp,
            write_delim: Delim::Dytp,
            read_timeout: Duration::from_secs(1),
            read_since: None,
        }
    }

    pub fn new_with_timeout(stream: TcpStream, read_timeout: u64) -> Self {
        Origin {
            stream,
            rb: BytesMut::new(),
            wb: BytesMut::new(),
            read_delim: Delim::Dytp,
            write_delim: Delim::Dytp,
            read_timeout: Duration::from_secs(read_timeout),
            read_since: None,
        }
    }
}

impl Write for Origin {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
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

impl Stream for Origin {
    type Item = BytesMut;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.try_read()
    }
}

pub mod prelude {
    pub use super::Origin;
    pub use std::io::prelude::*;
    pub use tokio::prelude::*;
}
