use std::{
    fmt::Display,
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
};

use crate::{
    errors::DnsError,
    message::{header::Header, resolver::Resolver, Message},
};
use anyhow::{anyhow, bail, Result};

mod errors;
mod message;

fn main() -> Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2053);
    let udp_socket = UdpSocket::bind(addr)?;
    println!("Successfully bound to address: {:?}", addr);

    let mut args = std::env::args();
    let dns_server = if args.nth(1).is_some_and(is_resolver_flag) {
        args.next()
            .ok_or(anyhow!(DnsError::ResolverNotSpecified))
            .and_then(Resolver::connect)?
    } else {
        bail!(DnsError::ResolverNotSpecified)
    };

    let mut buf = [0u8; 512];
    loop {
        let (size, source) = udp_socket.recv_from(&mut buf)?;
        match Message::unpack(&buf[0..size]) {
            Ok(query) => {
                let id = query.get_id();
                let bytes = dns_server
                    .resolve(query)
                    .and_then(|res| res.pack())
                    .or_else(pack_server_failure(id))?;

                udp_socket.send_to(&bytes, source)?;
            }
            Err(e) => {
                println!("Cannot unpack message: {}", e);
                let id = Header::unpack_id(&buf[0..size]).unwrap_or_default();
                let msg = Message::new_client_err().with_id(id);
                udp_socket.send_to(&msg.pack()?, source)?;
            }
        };
    }
}

fn is_resolver_flag(flag: String) -> bool {
    flag == "-r" || flag == "--resolver"
}

fn pack_server_failure<E>(id: u16) -> impl FnOnce(E) -> Result<Vec<u8>>
where
    E: Display,
{
    move |err: E| {
        eprintln!("Cannot resolve query: {}", err);
        Message::new_server_err().with_id(id).pack()
    }
}
