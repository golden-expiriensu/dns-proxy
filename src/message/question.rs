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
    pub fn unpack(buf: &[u8], ptr: &mut usize) -> Result<Self> {
        let labels = Labels::unpack(buf, ptr)?;
        let mut metadata = Cursor::new(vec![0u8; METADATA_SIZE]);
        metadata
            .get_mut()
            .clone_from_slice(&buf[*ptr..*ptr + METADATA_SIZE]);
        *ptr += METADATA_SIZE;

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
            qtype: self.qtype,
            qclass: self.qclass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(raw, &mut 0).unwrap();
        assert_eq!(raw.len(), question.len());
        assert_eq!(rr::Type::A, question.qtype);
        assert_eq!(rr::Class::In, question.qclass);
    }

    #[test]
    fn unpack_with_pointer() {
        let default_metadata = b"\x00\x01\x00\x01";
        // F.ISI.ARPA, FOO.F.ISI.ARPA, ARPA
        // 1. a sequence of labels ending in a zero octet
        let raw_0 = b"\x01F\x03ISI\x04ARPA\x00";
        // 0xC0 == 0b1100_0000
        // 2. a sequence of labels ending with a pointer
        let raw_1 = b"\x03FOO\xC0\x00";
        // 0x06 bytes is offset to ARPA in raw_0
        // 3. a pointer
        let raw_2 = b"\xC0\x06";
        let raw = [
            &raw_0[..],
            &default_metadata[..],
            &raw_1[..],
            &default_metadata[..],
            &raw_2[..],
            &default_metadata[..],
        ]
        .concat();

        let mut ptr = 0;

        let question_0 = Question::unpack(&raw, &mut ptr).unwrap();
        assert_eq!("F.ISI.ARPA", question_0.domain.to_string());
        assert_eq!(rr::Type::A, question_0.qtype);
        assert_eq!(rr::Class::In, question_0.qclass);

        let question_1 = Question::unpack(&raw, &mut ptr).unwrap();
        assert_eq!("FOO.F.ISI.ARPA", question_1.domain.to_string());
        assert_eq!(rr::Type::A, question_1.qtype);
        assert_eq!(rr::Class::In, question_1.qclass);

        let question_2 = Question::unpack(&raw, &mut ptr).unwrap();
        assert_eq!("ARPA", question_2.domain.to_string());
        assert_eq!(rr::Type::A, question_2.qtype);
        assert_eq!(rr::Class::In, question_2.qclass);
    }

    #[test]
    fn unpack_unsupported_class() {
        let raw = b"\x06google\x03com\x00\x00\x01\x01\x01";
        let question = Question::unpack(raw, &mut 0);
        assert!(question.is_err());
    }

    #[test]
    fn pack() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(raw, &mut 0).unwrap();
        let mut buf = vec![0u8; raw.len()];
        question.pack(&mut buf).unwrap();
        assert_eq!(raw, &buf[..]);
    }

    #[test]
    fn pack_invalid_input() {
        let raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::unpack(raw, &mut 0).unwrap();
        let mut buf = vec![0u8; raw.len() - 1];
        assert_eq!(
            question.pack(&mut buf).unwrap_err().to_string(),
            BUF_LEN_INVALID_ERR,
        );
    }
}
