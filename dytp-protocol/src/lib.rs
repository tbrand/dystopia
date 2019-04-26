pub mod delim;
pub mod method;
pub mod raw;
pub mod size;

use crate::raw::Raw;
use crate::size::Size;
use bytes::BytesMut;
//
// +------------------------------------------------------+
// |[4 bytes: raw payload size] | [any bytes: raw payload]|
// +------------------------------------------------------+
//
#[derive(Debug)]
pub struct Protocol(pub Size, pub Raw);

impl Protocol {
    pub fn size(&self) -> usize {
        4 + (self.1).0.len()
    }
}

pub fn parse(buf: &mut BytesMut) -> Option<Protocol> {
    let size = if buf.len() >= 4 {
        Size::parse(&buf as &[u8])
    } else {
        return None;
    };

    let raw = if buf.len() - 4 >= size.0 as usize {
        let mut raw = buf.split_to(size.0 as usize + 4);
        raw.split_to(4);
        Raw::wrap(raw)
    } else {
        return None;
    };

    Some(Protocol(size, raw))
}

impl From<&[u8]> for Protocol {
    fn from(buf: &[u8]) -> Protocol {
        let size = Size::new(buf.len() as u32);
        let raw = Raw::new(buf);

        Protocol(size, raw)
    }
}

impl Into<BytesMut> for Protocol {
    fn into(self) -> BytesMut {
        let mut buf = BytesMut::new();
        let size_arr: [u8; 4] = self.0.into();
        let raw: BytesMut = (self.1).0;
        buf.extend_from_slice(&size_arr);
        buf.extend_from_slice(&raw);
        buf
    }
}
