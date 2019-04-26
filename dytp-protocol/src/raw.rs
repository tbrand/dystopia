use bytes::BytesMut;

#[derive(Debug)]
pub struct Raw(pub BytesMut);

impl Raw {
    pub fn new(buf: &[u8]) -> Raw {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(buf);
        Raw(raw)
    }

    pub fn wrap(buf: BytesMut) -> Raw {
        Raw(buf)
    }
}
