use std::fmt::Display;

use crate::message::header::Header;

#[derive(Debug)]
pub enum DnsError {
    BufLenNotEq { exp: usize, act: usize },
    BufLenSmall { min: usize, act: usize },
    InvalidEncoding { at: usize },
    ResolverNotSpecified,
    ResolverFailed(Header),
    ResolverNoAnsw,
    ResolverNoRecv,
}

impl std::error::Error for DnsError {}

impl Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsError::BufLenNotEq { exp, act } => write!(
                f,
                "Buffer length is not equal to expected size: expected {} bytes, got {}",
                exp, act,
            ),
            DnsError::BufLenSmall { min, act } => write!(
                f,
                "Buffer length is not big enough: expected at least {} bytes, got {}",
                min, act,
            ),
            DnsError::InvalidEncoding { at } => {
                write!(f, "Invalid domain encoding discovered at byte {}", at)
            }
            DnsError::ResolverNotSpecified => write!(
                f,
                "Resolver address is not specified or specifed incorrectly. Usage: `run_server -r|--resolver <address>`"
            ),
            DnsError::ResolverFailed(header) => write!(
                f,
                "Invalid message received from DNS resolver, expected 1 answer, got: {:?}",
                header,
            ),
            DnsError::ResolverNoAnsw => write!(
                f,
                "No answer received from DNS resolver, but it promised exactly 1 answer",
            ),
            DnsError::ResolverNoRecv => write!(
                f,
                "Failed to forward message to the DNS resolver, 0 bytes was sent",
            ),
        }
    }
}
