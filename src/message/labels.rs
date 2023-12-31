use anyhow::{ensure, Result};
use std::io::Write;

use crate::message::question::BUF_LEN_INVALID_ERR;

#[derive(Debug, PartialEq)]
pub struct Labels(Vec<String>);

const TERMINATOR_BYTE: usize = 0x00;
const TERMINATOR_BYTE_SIZE: usize = 1;
const DOMAIN_NAME_LEN_BYTE_SIZE: usize = 1;

impl Labels {
    pub fn unpack(buf: &[u8]) -> Result<Self> {
        let mut len = 0;
        let mut words = Vec::new();
        loop {
            let word_len = buf[len] as usize;
            if word_len == TERMINATOR_BYTE {
                return Ok(Labels(words));
            }

            len += DOMAIN_NAME_LEN_BYTE_SIZE;
            words.push(String::from_utf8_lossy(&buf[len..len + word_len]).into());
            len += word_len;

            ensure!(
                len < buf.len(),
                "Invalid domain encoding discovered at byte {}",
                len
            )
        }
    }

    pub fn pack(&self, buf: &mut [u8]) -> Result<()> {
        ensure!(buf.len() == self.len(), BUF_LEN_INVALID_ERR);

        let mut wrote = 0;
        for word in &self.0 {
            buf[wrote] = word.len() as u8;
            wrote += DOMAIN_NAME_LEN_BYTE_SIZE;
            write!(&mut buf[wrote..wrote + word.len()], "{word}")?;
            wrote += word.len();
        }

        buf[wrote] = TERMINATOR_BYTE as u8;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0
            .iter()
            .map(|s| s.len() + DOMAIN_NAME_LEN_BYTE_SIZE)
            .sum::<usize>()
            + TERMINATOR_BYTE_SIZE
    }

    pub fn to_string(&self) -> String {
        self.0.join(".")
    }
}

impl Clone for Labels {
    fn clone(&self) -> Self {
        Labels(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw).unwrap();
        assert_eq!(raw.len(), labels.len());
        assert_eq!(labels.0, vec!["google", "com"]);
    }

    #[test]
    fn unpack_invalid_domain_encoding() {
        let raw = b"\x06google\x03com";
        let labels = Labels::unpack(raw);
        assert!(labels.is_err());
    }

    #[test]
    fn pack() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw).unwrap();
        let mut buf = vec![0u8; raw.len()];
        labels.pack(&mut buf).unwrap();
        assert_eq!(raw, &buf[..]);
    }

    #[test]
    fn pack_invalid_input() {
        let raw = b"\x06google\x03com\x00";
        let labels = Labels::unpack(raw).unwrap();
        let mut buf = vec![0u8; raw.len() - 1];
        assert_eq!(
            labels.pack(&mut buf).unwrap_err().to_string(),
            BUF_LEN_INVALID_ERR,
        );
    }
}
