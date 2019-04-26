#[derive(Debug)]
pub struct Size(pub u32);

impl Size {
    pub fn new(size: u32) -> Size {
        Size(size)
    }

    pub fn parse(buf: &[u8]) -> Size {
        if buf.len() < 4 {
            panic!();
        }

        let mut size_buf: [u8; 4] = [0, 0, 0, 0];
        size_buf[0] = buf[0];
        size_buf[1] = buf[1];
        size_buf[2] = buf[2];
        size_buf[3] = buf[3];

        Size(unsafe { std::mem::transmute::<[u8; 4], u32>(size_buf) }.to_be())
    }
}

impl Into<[u8; 4]> for Size {
    fn into(self) -> [u8; 4] {
        unsafe { std::mem::transmute::<u32, [u8; 4]>((self.0 as u32).to_be()) }
    }
}
