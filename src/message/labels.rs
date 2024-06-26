use anyhow::{ensure, Result};
use std::{fmt::Display, io::Write, ops::DerefMut};

use crate::{errors::DnsError, message::labels::pointer::DomainPointer};

mod pointer;

const TERMINATOR_BYTE: u8 = 0x00;
const TERMINATOR_BYTE_SIZE: usize = 1;
const DOMAIN_NAME_LEN_BYTE_SIZE: usize = 1;

#[derive(Debug, Default, PartialEq)]
pub struct Labels(Vec<String>);

impl Labels {
    pub fn unpack(buf: &[u8], ptr: &mut usize) -> Result<Self> {
        let mut words = Vec::new();
        Labels::scan(&mut words, buf, ptr)?;
        Ok(Labels(words))
    }

    fn scan<T>(words: &mut Vec<String>, buf: &[u8], mut ptr: T) -> Result<()>
    where
        T: DerefMut<Target = usize>,
    {
        loop {
            let mut at: usize = *ptr;
            let letter: u8 = buf[at];

            if letter == TERMINATOR_BYTE {
                *ptr += TERMINATOR_BYTE_SIZE;
                return Ok(());
            }
            if DomainPointer::test(letter) {
                let ptr = DomainPointer::try_from((buf, ptr))?;
                return Labels::scan(words, buf, ptr);
            }

            at += DOMAIN_NAME_LEN_BYTE_SIZE;
            let word_len = letter as usize;
            words.push(String::from_utf8_lossy(&buf[at..at + word_len]).into());
            at += word_len;

            ensure!(at < buf.len(), DnsError::InvalidEncoding { at });
            *ptr = at;
        }
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
        for word in &self.0 {
            buf[wrote] = word.len() as u8;
            wrote += DOMAIN_NAME_LEN_BYTE_SIZE;
            write!(&mut buf[wrote..wrote + word.len()], "{word}")?;
            wrote += word.len();
        }

        buf[wrote] = TERMINATOR_BYTE;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0
            .iter()
            .map(|s| s.len() + DOMAIN_NAME_LEN_BYTE_SIZE)
            .sum::<usize>()
            + TERMINATOR_BYTE_SIZE
    }
}

impl Display for Labels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl Clone for Labels {
    fn clone(&self) -> Self {
        Labels(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::DnsError;

    use super::*;

    #[test]
    fn unpack_0() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw, &mut 0).unwrap();
        assert_eq!(raw.len(), labels.len());
        assert_eq!(labels.0, vec!["google", "com"]);
    }

    #[test]
    fn unpack() {
        let raw = b"\x00\x01\x02\x03\x06google\x03com\x00";
        let ptr = 4;
        let labels = Labels::unpack(raw, &mut ptr.clone()).unwrap();
        assert_eq!(raw.len() - ptr, labels.len());
        assert_eq!(labels.0, vec!["google", "com"]);
    }

    #[test]
    fn unpack_invalid_domain_encoding() {
        let raw = b"\x06google\x03com";
        let labels = Labels::unpack(raw, &mut 0);
        assert_eq!(
            DnsError::InvalidEncoding { at: 11 }.to_string(),
            labels.unwrap_err().to_string()
        );
    }

    #[test]
    fn pack() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw, &mut 0).unwrap();
        let mut buf = vec![0u8; raw.len()];
        labels.pack(&mut buf).unwrap();
        assert_eq!(raw, &buf[..]);
    }

    #[test]
    fn pack_invalid_input() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw, &mut 0).unwrap();
        let mut buf = vec![0u8; raw.len() - 1];
        assert_eq!(
            labels.pack(&mut buf).unwrap_err().to_string(),
            DnsError::BufLenNotEq {
                exp: raw.len(),
                act: raw.len() - 1
            }
            .to_string(),
        );
    }
}
