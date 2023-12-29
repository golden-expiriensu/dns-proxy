use packed_struct::prelude::*;

use self::header::*;

mod header;

pub struct DnsMessage {
    header: DnsHeader,
}

impl DnsMessage {
    pub fn parse(query: &[u8]) -> Result<DnsMessage, PackingError> {
        if query.len() < DNS_HEADER_SIZE {
            return Err(PackingError::BufferTooSmall);
        }
        let header = DnsHeader::unpack_from_slice(&query[..DNS_HEADER_SIZE])?;
        Ok(DnsMessage { header })
    }

    pub fn build(&self) -> Result<Vec<u8>, PackingError> {
        self.header.pack_to_vec()
    }

    pub fn response(&self) -> DnsMessage {
        DnsMessage {
            header: self.header.reply(),
        }
    }
}
