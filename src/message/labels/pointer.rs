use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
};

use byteorder::{BigEndian, ByteOrder};
use packed_struct::PackingError;

const MASK_U8: u8 = 0b1100_0000;
const MASK_U16: u16 = 0b1100_0000_0000_0000;

pub struct DomainPointer(usize);

impl DomainPointer {
    pub fn test(word: u8) -> bool {
        word & MASK_U8 == MASK_U8
    }
}

impl<At> TryFrom<(&[u8], At)> for DomainPointer
where
    At: DerefMut<Target = usize>,
{
    type Error = PackingError;

    fn try_from(value: (&[u8], At)) -> Result<Self, Self::Error> {
        let size = size_of::<u16>();
        let (buf, mut at) = value;

        if buf.len() < *at + size {
            return Err(PackingError::BufferTooSmall);
        }
        let mut ptr = BigEndian::read_u16(&buf[*at..*at + size]);
        ptr &= !MASK_U16;

        *at += size;
        Ok(DomainPointer(ptr as usize))
    }
}

impl Deref for DomainPointer {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DomainPointer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
