use std::mem::size_of;

use byteorder::{BigEndian, ByteOrder};
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
#[derive(Clone, Copy, Debug, Default, PackedStruct)]
#[packed_struct(size_bytes = 12, bit_numbering = "msb0", endian = "msb")]
pub struct Header {
    #[packed_field(bits = "0..=15")]
    pub id: u16,
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
    #[packed_field(bits = "28..=31", ty = "enum")]
    pub rcode: ResponseCode,
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
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum Indicator {
    Query = 0,
    #[default]
    Response = 1,
}

/// `Yes` if the responding server "owns" the domain queried, i.e., it's authoritative.
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum AuthoritativeAnswer {
    #[default]
    No = 0,
    Yes = 1,
}

/// `Yes` if the message is larger than 512 bytes. Always `No` in UDP responses.
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum Truncation {
    #[default]
    No = 0,
    Yes = 1,
}

/// Sender sets this to `Yes` if the server should recursively resolve this query, `No` otherwise.
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum RecursionDesired {
    #[default]
    No = 0,
    Yes = 1,
}

/// Sender sets this to `Yes` if the server supports recursive queries, `No` otherwise.
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum RecursionAvailable {
    #[default]
    No = 0,
    Yes = 1,
}

/// Response code - this 4 bit field is set as part of responses.  The values have the following
/// interpretation:
///
/// • 0 No error condition
///
/// • 1 Format error - The name server was unable to interpret the query.
///
/// • 2 Server failure - The name server was unable to process this query due to a problem with the
///   name server.
///
/// • 3 Name Error - Meaningful only for responses from an authoritative name server, this code
///   signifies that the domain name referenced in the query does not exist.
///
/// • 4 Not Implemented - The name server does not support the requested kind of query.
///
/// • 5 Refused - The name server refuses to perform the specified operation for policy reasons.  For
///   example, a name server may not wish to provide the information to the particular requester,
///   or a name server may not wish to perform a particular operation (e.g., zone transfer) for
///   particular data.
///
#[derive(PrimitiveEnum, Clone, Copy, Debug, Default)]
pub enum ResponseCode {
    #[default]
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

pub const DNS_HEADER_SIZE: usize = 12;

impl Header {
    pub fn unpack(query: &[u8]) -> Result<Self, PackingError> {
        if query.len() < DNS_HEADER_SIZE {
            return Err(PackingError::BufferTooSmall);
        }
        Ok(Header::unpack_from_slice(&query[..DNS_HEADER_SIZE])?)
    }

    pub fn unpack_id(query: &[u8]) -> Result<u16, PackingError> {
        let id_size = size_of::<u16>();
        if query.len() < id_size {
            return Err(PackingError::BufferTooSmall);
        }
        let id = BigEndian::read_u16(&query[0..id_size]);
        Ok(id)
    }
}
