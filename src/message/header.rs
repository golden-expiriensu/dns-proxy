use packed_struct::prelude::*;

/// Packet Identifier (ID)	          | A random ID assigned to query packets. Response packets must reply with the same ID.
/// Query/Response Indicator (QR)	  | `Response` for a reply packet, `Query` for a question packet.
/// Operation Code (OPCODE)	          | Specifies the kind of query in a message.
/// Authoritative Answer (AA)	      | `Yes` if the responding server "owns" the domain queried, i.e., it's authoritative.
/// Truncation (TC)	                  | `Yes` if the message is larger than 512 bytes. Always `No` in UDP responses.
/// Recursion Desired (RD)	          | Sender sets this to `Yes` if the server should recursively resolve this query, `No` otherwise.
/// Recursion Available (RA)	      | Sender sets this to `Yes` if the server supports recursive queries, `No` otherwise.
/// Reserved (Z)	                  | Used by DNSSEC queries. At inception, it was reserved for future use.
/// Response Code (RCODE)	          | Response code indicating the status of the response.
/// Question Count (QDCOUNT)	      | Number of questions in the Question section.
/// Answer Record Count (ANCOUNT)	  | Number of records in the Answer section.
/// Authority Record Count (NSCOUNT)  | Number of records in the Authority section.
/// Additional Record Count (ARCOUNT) | Number of records in the Additional section.
#[derive(Clone, Copy, Debug, PackedStruct)]
#[packed_struct(size_bytes = 12, bit_numbering = "msb0", endian = "lsb")]
pub struct Header {
    #[packed_field(bits = "0..=15")]
    id: u16,
    #[packed_field(bits = "16", ty = "enum")]
    pub qr: Indicator,
    #[packed_field(bits = "17..=20")]
    pub opcode: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "21", ty = "enum")]
    pub aa: AuthoritativeAnswer,
    #[packed_field(bits = "22", ty = "enum")]
    pub tc: Truncation,
    #[packed_field(bits = "23", ty = "enum")]
    pub rd: RecursionDesired,
    #[packed_field(bits = "24", ty = "enum")]
    pub ra: RecursionAvailable,
    #[packed_field(bits = "25..=27")]
    pub z: Integer<u8, packed_bits::Bits<3>>,
    #[packed_field(bits = "28..=31")]
    pub rcode: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "32..=47")]
    pub qdcount: u16,
    #[packed_field(bits = "48..=63")]
    pub ancount: u16,
    #[packed_field(bits = "64..=79")]
    pub nscount: u16,
    #[packed_field(bits = "80..=95")]
    pub arcount: u16,
}

/// `Response` for a reply packet, `Query` for a question packet.
#[derive(PrimitiveEnum, Clone, Copy, Debug)]
pub enum Indicator {
    Query = 0,
    Response = 1,
}

/// `Yes` if the responding server "owns" the domain queried, i.e., it's authoritative.
#[derive(PrimitiveEnum, Clone, Copy, Debug)]
pub enum AuthoritativeAnswer {
    No = 0,
    Yes = 1,
}

/// `Yes` if the message is larger than 512 bytes. Always `No` in UDP responses.
#[derive(PrimitiveEnum, Clone, Copy, Debug)]
pub enum Truncation {
    No = 0,
    Yes = 1,
}

/// Sender sets this to `Yes` if the server should recursively resolve this query, `No` otherwise.
#[derive(PrimitiveEnum, Clone, Copy, Debug)]
pub enum RecursionDesired {
    No = 0,
    Yes = 1,
}

/// Sender sets this to `Yes` if the server supports recursive queries, `No` otherwise.
#[derive(PrimitiveEnum, Clone, Copy, Debug)]
pub enum RecursionAvailable {
    No = 0,
    Yes = 1,
}

pub const DNS_HEADER_SIZE: usize = 12;

impl Header {
    pub fn unpack(query: &[u8]) -> Result<Self, PackingError> {
        if query.len() < DNS_HEADER_SIZE {
            return Err(PackingError::BufferTooSmall);
        }
        Ok(Header::unpack_from_slice(&query[..DNS_HEADER_SIZE])?)
    }
}
