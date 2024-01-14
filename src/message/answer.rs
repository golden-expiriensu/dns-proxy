use std::{io::Cursor, mem::size_of};

use crate::errors::DnsError;

use super::{labels::Labels, rr};

use anyhow::{ensure, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    name: Labels,
    atype: rr::Type,
    aclass: rr::Class,
    ttl: u32,
    length: u16,
    data: Vec<u8>,
}

const METADATA_SIZE: usize =
    size_of::<rr::Type>() + size_of::<rr::Class>() + size_of::<u32>() + size_of::<u16>();

impl Answer {
    pub fn unpack(buf: &[u8], ptr: &mut usize) -> Result<Self> {
        let name = Labels::unpack(buf, ptr)?;

        let mut metadata = Cursor::new(vec![0u8; METADATA_SIZE]);
        let mut at = *ptr;
        ensure!(
            buf.len() > at + METADATA_SIZE,
            DnsError::BufLenSmall {
                min: at + METADATA_SIZE,
                act: buf.len()
            }
        );
        metadata
            .get_mut()
            .clone_from_slice(&buf[at..at + METADATA_SIZE]);
        let atype = metadata.read_u16::<BigEndian>()?.try_into()?;
        let aclass = metadata.read_u16::<BigEndian>()?.try_into()?;
        let ttl = metadata.read_u32::<BigEndian>()?;
        let length = metadata.read_u16::<BigEndian>()?;
        at += METADATA_SIZE;

        let mut data = Vec::with_capacity(length as usize);
        Vec::extend_from_slice(&mut data, &buf[at..at + length as usize]);
        *ptr = at + length as usize;

        Ok(Answer {
            name,
            atype,
            aclass,
            ttl,
            length,
            data,
        })
    }

    pub fn pack(&self, buf: &mut [u8]) -> Result<()> {
        ensure!(
            buf.len() == self.len(),
            DnsError::BufLenNotEq {
                exp: self.len(),
                act: buf.len()
            }
        );
        let mut wrote = 0;

        let mut next = self.name.len();
        self.name.pack(&mut buf[..next])?;
        wrote += next;

        next = METADATA_SIZE;
        let mut cursor = Cursor::new(vec![0u8; next]);
        cursor.write_u16::<BigEndian>(self.atype.into())?;
        cursor.write_u16::<BigEndian>(self.aclass.into())?;
        cursor.write_u32::<BigEndian>(self.ttl)?;
        cursor.write_u16::<BigEndian>(self.length)?;
        (&mut buf[wrote..wrote + next]).copy_from_slice(cursor.get_ref());
        wrote += next;

        next = self.data.len();
        (&mut buf[wrote..wrote + next]).copy_from_slice(&self.data);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.name.len() + METADATA_SIZE + self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::message::question::Question;

    use super::*;

    #[test]
    fn pack() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let ttl = 0xFFDDBBAA;
        let data = vec![127, 0, 0, 1];
        let question = Question::unpack(raw, &mut 0).unwrap();
        let answer = Answer {
            name: question.domain,
            atype: question.qtype,
            aclass: question.qclass,
            ttl,
            length: data.len() as u16,
            data: data.clone(),
        };

        let mut expect = Vec::from_iter(raw.iter().cloned());
        expect.extend_from_slice(&vec![0xFF, 0xDD, 0xBB, 0xAA, 0x00, 0x04]);
        expect.extend_from_slice(&data);

        let mut buf = vec![0u8; answer.len()];
        answer.pack(&mut buf).unwrap();

        assert_eq!(buf, expect);
    }
}
