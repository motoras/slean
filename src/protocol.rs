use crate::error::SleanError;
use std::fmt::{Debug, Display};

//2^21 -1 about 2.1 MB
const MAX_MSG_SIZE_BYTES: u32 = (1 << 21) - 1;
const BITS_MSG_SIZE: u32 = 21;

const MASK_22_24: u64 = 0b111 << 22;
const REQ: u64 = 0b0001;
const REPL: u64 = 0b1000;
const ERR: u64 = 0b1111;

#[repr(transparent)]
pub struct FrameDescriptor {
    desc: u64,
}

impl FrameDescriptor {
    pub fn max_size() -> u32 {
        MAX_MSG_SIZE_BYTES
    }
    #[inline]
    pub fn is_req(&self) -> bool {
        self.desc >> 60 == REQ
    }
    #[inline]
    pub fn is_repl(&self) -> bool {
        self.desc >> 60 == REPL
    }
    #[inline]
    pub fn is_err(&self) -> bool {
        self.desc >> 60 == ERR
    }
    #[inline]
    pub fn len(&self) -> u32 {
        (self.desc & 0x000000001FFFFF) as u32
    }
}

impl TryFrom<u64> for FrameDescriptor {
    type Error = SleanError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value & MASK_22_24 != 0 {
            //bit 22 to 24 must be zero
            return Err(SleanError::InvalidFrameHeader(value));
        }
        let fd = FrameDescriptor { desc: value };

        Ok(fd)
    }
}

impl Into<u64> for FrameDescriptor {
    fn into(self) -> u64 {
        self.desc
    }
}

impl Display for FrameDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.desc)
    }
}

impl Debug for FrameDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plain {:#016X}, len: {}", self.desc, self.len())
    }
}

#[cfg(test)]
mod test {
    use super::FrameDescriptor;
    use crate::error::SleanError;
    use crate::protocol::{ERR, MASK_22_24, MAX_MSG_SIZE_BYTES, REPL, REQ};

    #[test]
    fn create_and_check() {
        let zero = 0u64;
        let fd: FrameDescriptor = zero.try_into().unwrap();
        assert!(fd.desc == zero);
        assert!(fd.len() == 0);

        let _2mb = MAX_MSG_SIZE_BYTES as u64;
        let fd: FrameDescriptor = _2mb.try_into().unwrap();
        dbg!(&fd);
        assert!(fd.len() == MAX_MSG_SIZE_BYTES);

        let _1kb = 1000;
        let fd: FrameDescriptor = _1kb.try_into().unwrap();
        dbg!(&fd);
        assert!(fd.len() == _1kb as u32);
        //a broken one
        let msg = MASK_22_24 + 1000;
        let fdr: Result<FrameDescriptor, SleanError> = msg.try_into();
        assert!(fdr.is_err());
    }
    #[test]
    fn check_req() {
        let zero = 1u64 | (REQ << 60);
        let fd: FrameDescriptor = zero.try_into().unwrap();
        assert!(fd.len() == 1);
        assert!(fd.is_req());
        assert!(!fd.is_repl());
        assert!(!fd.is_err());
    }
    #[test]
    fn check_repl() {
        let zero = 1u64 | (REPL << 60);
        let fd: FrameDescriptor = zero.try_into().unwrap();
        assert!(fd.len() == 1);
        assert!(!fd.is_req());
        assert!(fd.is_repl());
        assert!(!fd.is_err());
    }

    #[test]
    fn check_err() {
        let zero = 1u64 | (ERR << 60);
        let fd: FrameDescriptor = zero.try_into().unwrap();
        assert!(fd.len() == 1);
        assert!(!fd.is_req());
        assert!(!fd.is_repl());
        assert!(fd.is_err());
    }
}
