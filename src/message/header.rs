use packed_struct::prelude::*;

pub const DNS_HEADER_SIZE: usize = 12;

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
#[derive(PackedStruct)]
#[packed_struct(endian = "lsb", bit_numbering = "msb0")]
pub struct DnsHeader {
    #[packed_field(bits = "0..=15")]
    id: u16,
    #[packed_field(bits = "16", ty = "enum")]
    qr: Indicator,
    #[packed_field(bits = "17..=20")]
    opcode: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "21", ty = "enum")]
    aa: AuthoritativeAnswer,
    #[packed_field(bits = "22", ty = "enum")]
    tc: Truncation,
    #[packed_field(bits = "23", ty = "enum")]
    rd: RecursionDesired,
    #[packed_field(bits = "24", ty = "enum")]
    ra: RecursionAvailable,
    #[packed_field(bits = "25..=27")]
    z: Integer<u8, packed_bits::Bits<3>>,
    #[packed_field(bits = "28..=31")]
    rcode: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "32..=47")]
    qdcount: u16,
    #[packed_field(bits = "48..=63")]
    ancount: u16,
    #[packed_field(bits = "64..=79")]
    nscount: u16,
    #[packed_field(bits = "80..=95")]
    arcount: u16,
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

impl DnsHeader {
    pub fn reply(&self) -> DnsHeader {
        DnsHeader {
            id: self.id,
            qr: Indicator::Response,
            opcode: 0.into(),
            aa: AuthoritativeAnswer::No,
            tc: Truncation::No,
            rd: RecursionDesired::No,
            ra: RecursionAvailable::No,
            z: 0.into(),
            rcode: 0.into(),
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }
}
