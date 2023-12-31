use std::{
    io::{self, Cursor, Write},
    mem::size_of,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::rr;

#[derive(Debug, PartialEq)]
pub enum ParsingError {
    InvalidDomainEncoding { at_byte: usize },
    InvalidMetadataEncoding(String),
    ResourceRecordError(rr::Error),
}

impl std::error::Error for ParsingError {}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::InvalidDomainEncoding { at_byte } => {
                write!(f, "Invalid domain encoding discovered at byte {}", at_byte)
            }
            ParsingError::InvalidMetadataEncoding(value) => {
                write!(f, "Invalid metadata encoding: {}", value)
            }
            ParsingError::ResourceRecordError(value) => {
                write!(f, "{}", value)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Question {
    domain: Vec<String>,
    qtype: rr::Type,
    qclass: rr::Class,
}

const NULL_BYTE: usize = 0x00;
const NULL_BYTE_SIZE: usize = 1;

const DOMAIN_NAME_LEN_BYTE_SIZE: usize = 1;
const METADATA_SIZE: usize = size_of::<rr::Type>() + size_of::<rr::Class>();

const BUFFER_TOO_SMALL_ERR: &'static str = "Buffer too small";

impl Question {
    pub fn parse(buf: &[u8]) -> Result<Self, ParsingError> {
        let mut len = 0;
        let mut domain = Vec::new();
        loop {
            let next_len = buf[len] as usize;
            if next_len == NULL_BYTE {
                len += NULL_BYTE_SIZE;
                let mut metadata = Cursor::new(vec![0u8; METADATA_SIZE]);
                metadata
                    .get_mut()
                    .clone_from_slice(&buf[len..len + METADATA_SIZE]);

                return Ok(Question {
                    domain,
                    qtype: metadata
                        .read_u16::<BigEndian>()
                        .map_err(|e| ParsingError::InvalidMetadataEncoding(e.to_string()))?
                        .try_into()
                        .map_err(|e| ParsingError::ResourceRecordError(e))?,
                    qclass: metadata
                        .read_u16::<BigEndian>()
                        .map_err(|e| ParsingError::InvalidMetadataEncoding(e.to_string()))?
                        .try_into()
                        .map_err(|e| ParsingError::ResourceRecordError(e))?,
                });
            }

            len += DOMAIN_NAME_LEN_BYTE_SIZE;
            domain.push(String::from_utf8_lossy(&buf[len..len + next_len]).into());
            len += next_len;

            if len + METADATA_SIZE >= buf.len() {
                return Err(ParsingError::InvalidDomainEncoding { at_byte: len });
            }
        }
    }

    pub fn build(&self, buf: &mut [u8]) -> Result<(), io::Error> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                BUFFER_TOO_SMALL_ERR,
            ));
        }

        let mut wrote = 0;
        for label in &self.domain {
            buf[wrote] = label.len() as u8;
            wrote += DOMAIN_NAME_LEN_BYTE_SIZE;
            write!(&mut buf[wrote..wrote + label.len()], "{label}")?;
            wrote += label.len();
        }

        buf[wrote] = NULL_BYTE as u8;
        wrote += NULL_BYTE_SIZE;

        let mut cursor = Cursor::new(vec![0u8; METADATA_SIZE]);
        cursor.write_u16::<BigEndian>(self.qtype.clone().into())?;
        cursor.write_u16::<BigEndian>(self.qclass.clone().into())?;
        (&mut buf[wrote..wrote + METADATA_SIZE]).copy_from_slice(cursor.get_ref());
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.domain
            .iter()
            .map(|s| s.len() + DOMAIN_NAME_LEN_BYTE_SIZE)
            .sum::<usize>()
            + NULL_BYTE_SIZE
            + METADATA_SIZE
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
    fn parse() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::parse(question_raw).unwrap();
        assert_eq!(question_raw.len(), question.len());
        assert_eq!(question.domain, vec!["google", "com"]);
        assert_eq!(question.qtype, rr::Type::A);
        assert_eq!(question.qclass, rr::Class::In);
    }

    #[test]
    fn parse_invalid_domain_encoding() {
        let question_raw = b"\x06google\x03com\x00\x01\x00\x01";
        let question = Question::parse(question_raw);
        assert_eq!(
            Err(ParsingError::InvalidDomainEncoding { at_byte: 11 }),
            question,
        );
    }

    #[test]
    fn parse_invalid_unsupported_class() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x01\x01";
        let question = Question::parse(question_raw);
        assert_eq!(
            Err(ParsingError::ResourceRecordError(
                rr::Error::UnsupportedClass(u16::from_be_bytes([0x01, 0x01]))
            )),
            question,
        );
    }

    #[test]
    fn build() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::parse(question_raw).unwrap();
        let mut buf = vec![0u8; question_raw.len()];
        question.build(&mut buf).unwrap();
        assert_eq!(question_raw, &buf[..]);
    }

    #[test]
    fn build_invalid_input() {
        let question_raw = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let question = Question::parse(question_raw).unwrap();
        let mut buf = vec![0u8; question_raw.len() - 1];
        assert_eq!(
            question.build(&mut buf).unwrap_err().to_string(),
            BUFFER_TOO_SMALL_ERR,
        );
    }
}
