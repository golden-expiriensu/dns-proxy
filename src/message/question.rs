use std::{io::Cursor, mem::size_of};

use anyhow::{ensure, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::{labels::Labels, rr};

#[derive(Debug, PartialEq)]
pub struct Question {
    pub domain: Labels,
    pub qtype: rr::Type,
    pub qclass: rr::Class,
}

const METADATA_SIZE: usize = size_of::<rr::Type>() + size_of::<rr::Class>();
pub const BUF_LEN_INVALID_ERR: &'static str = "Buffer length is invalid, expected exact match";

impl Question {
    pub fn unpack(buf: &[u8]) -> Result<Self> {
        let labels = Labels::unpack(buf)?;
        let mut metadata = Cursor::new(vec![0u8; METADATA_SIZE]);
        let ll = labels.len();
        metadata
            .get_mut()
            .clone_from_slice(&buf[ll..ll + METADATA_SIZE]);

        Ok(Question {
            domain: labels,
            qtype: metadata.read_u16::<BigEndian>()?.try_into()?,
            qclass: metadata.read_u16::<BigEndian>()?.try_into()?,
        })
    }

    pub fn pack(&self, buf: &mut [u8]) -> Result<()> {
        ensure!(buf.len() == self.len(), BUF_LEN_INVALID_ERR);
        self.domain.pack(&mut buf[..self.domain.len()])?;

        let mut cursor = Cursor::new(vec![0u8; METADATA_SIZE]);
        cursor.write_u16::<BigEndian>(self.qtype.into())?;
        cursor.write_u16::<BigEndian>(self.qclass.into())?;

        let len = self.domain.len();
        (&mut buf[len..len + METADATA_SIZE]).copy_from_slice(cursor.get_ref());
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.domain.len() + METADATA_SIZE
    }
}

impl Clone for Question {
    fn clone(&self) -> Self {
        Question {
            domain: self.domain.clone(),
            qtype: self.qtype.clone(),
            qclass: self.qclass.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(question_raw).unwrap();
        assert_eq!(question_raw.len(), question.len());
        assert_eq!(question.qtype, rr::Type::A);
        assert_eq!(question.qclass, rr::Class::In);
    }

    #[test]
    fn unpack_invalid_unsupported_class() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x01\x01";
        let question = Question::unpack(question_raw);
        assert!(question.is_err());
    }

    #[test]
    fn pack() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(question_raw).unwrap();
        let mut buf = vec![0u8; question_raw.len()];
        question.pack(&mut buf).unwrap();
        assert_eq!(question_raw, &buf[..]);
    }

    #[test]
    fn pack_invalid_input() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(question_raw).unwrap();
        let mut buf = vec![0u8; question_raw.len() - 1];
        assert_eq!(
            question.pack(&mut buf).unwrap_err().to_string(),
            BUF_LEN_INVALID_ERR,
        );
    }
}
