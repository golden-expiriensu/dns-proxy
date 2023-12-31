use std::{io::Cursor, mem::size_of, net::Ipv4Addr};

use crate::message::question::BUF_LEN_INVALID_ERR;

use super::{labels::Labels, question::Question, rr};

use anyhow::{ensure, Result};
use byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug, PartialEq)]
pub struct Answer {
    name: Labels,
    atype: rr::Type,
    aclass: rr::Class,
    ttl: u32,
    length: u16,
    data: Vec<u8>,
}

const DEFAULT_TTL: u32 = 3600;
const MIN_DATA_LEN: usize = 4;
const METADATA_SIZE: usize =
    size_of::<rr::Type>() + size_of::<rr::Class>() + size_of::<u32>() + size_of::<u16>();

impl Answer {
    pub fn resolve(question: &Question) -> Result<Self> {
        let mut answer = Answer {
            name: question.domain.clone(),
            atype: question.qtype,
            aclass: question.qclass,
            ttl: DEFAULT_TTL,
            length: 0,
            data: Vec::with_capacity(MIN_DATA_LEN),
        };

        match question.qtype {
            rr::Type::A => answer.resolve_ipv4()?,
        }

        Ok(answer)
    }

    fn resolve_ipv4(&mut self) -> Result<()> {
        self.length = 4;
        let addr: Ipv4Addr;
        match self.name.to_string().as_str() {
            "google.com" => addr = Ipv4Addr::new(142, 250, 188, 14),
            "codecrafters.io" => addr = Ipv4Addr::new(76, 76, 21, 21),
            _ => addr = Ipv4Addr::new(8, 8, 8, 8),
        }
        self.data = addr.octets().to_vec();
        Ok(())
    }

    pub fn pack(&self, buf: &mut [u8]) -> Result<()> {
        ensure!(buf.len() == self.len(), BUF_LEN_INVALID_ERR);
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
    use super::*;

    #[test]
    fn resolve() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(raw).unwrap();
        let answer = Answer::resolve(&question).unwrap();
        assert_eq!(answer.name.to_string(), "google.com");
        assert_eq!(answer.atype, rr::Type::A);
        assert_eq!(answer.aclass, rr::Class::In);
        assert_eq!(answer.ttl, DEFAULT_TTL);
        assert_eq!(answer.length, 4);
        assert_eq!(answer.data, vec![142, 250, 188, 14])
    }

    #[test]
    fn pack() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(raw).unwrap();
        let answer = Answer::resolve(&question).unwrap();
        let mut buf = vec![0u8; answer.len()];
        answer.pack(&mut buf).unwrap();
        let mut expect = Vec::from_iter(raw.iter().cloned());
        expect.extend_from_slice(&vec![0, 0, 14, 16, 0, 4, 142, 250, 188, 14]);
        assert_eq!(buf, expect);
    }
}
