use std::io::{Error, ErrorKind, Write};

pub struct TcpWriteBuff {
    buff: FixedBuf,
    pos: usize,
}
impl Default for TcpWriteBuff {
    fn default() -> Self {
        TcpWriteBuff::new(1 << 21 - 1)
    }
}

impl TcpWriteBuff {
    pub fn new(len: u32) -> Self {
        //let mut buff = Vec::with_capacity(std::mem::size_of::<u32>() + len as usize);
        //buff.resize(buff.capacity(), 0);
        Self {
            buff: FixedBuf::new(len),
            pos: std::mem::size_of::<u32>(),
        }
    }

    // #[inline]
    // pub fn send(&mut self, out: &mut TcpStream) -> std::io::Result<()> {
    //     assert!(self.pos > std::mem::size_of::<u32>());
    //     let buf_len = self.pos;
    //     self.buff[0..4].copy_from_slice(&(self.pos - std::mem::size_of::<u32>()).to_le_bytes());
    //     self.pos = std::mem::size_of::<u32>();
    //     out.write_all(&self.buff[0..self.pos])
    // }

    pub fn send(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        //info!("Writing on the socket {} bytes", self.pos);
        assert!(self.pos > std::mem::size_of::<u32>());
        self.buff.writable()[0..4]
            .copy_from_slice(&(self.pos - std::mem::size_of::<u32>()).to_le_bytes()[0..4]);
        let res = out.write_all(&self.buff.writable()[..self.pos]);
        self.pos = std::mem::size_of::<u32>();
        res
    }
}

impl Write for TcpWriteBuff {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // info!(
        //     "Writing {} {} {} ",
        //     self.pos,
        //     buf.len(),
        //     self.buff.capacity()
        // );
        if self.buff.available() < buf.len() {
            return Err(ErrorKind::OutOfMemory.into());
        }
        let from = self.pos;
        let to = from + buf.len();
        self.buff.writable()[from..to].copy_from_slice(buf);
        self.pos = to;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.pos = 0;
        Ok(())
    }
}

#[derive(Clone)]
pub struct FixedBuf {
    data: Box<[u8]>,
    read_pos: usize,
    write_pos: usize,
}

impl FixedBuf {
    pub fn new(len: u32) -> Self {
        let mut v_data = Vec::with_capacity(len as usize);
        unsafe {
            v_data.set_len(len as usize);
        }
        Self {
            data: v_data.into_boxed_slice(),
            write_pos: 0,
            read_pos: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.write_pos - self.read_pos
    }

    pub fn available(&self) -> usize {
        self.data.len() - self.write_pos
    }

    pub fn writable(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[self.write_pos..]
    }

    /// Returns true if there are unread bytes in the buffer.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.write_pos == self.read_pos
    }

    /// Discards all data in the buffer.
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }
}

impl Default for FixedBuf {
    fn default() -> Self {
        Self::new(1 << 21 - 1)
    }
}

impl Write for FixedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.write_pos + buf.len() >= self.data.len() {
            return Err(ErrorKind::OutOfMemory.into());
        }
        let from = self.write_pos;
        let to = from + buf.len();
        self.data[from..to].copy_from_slice(buf);
        self.write_pos = to;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.clear();
        Ok(())
    }
}
