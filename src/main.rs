use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

use crate::message::{header::Header, Message};

mod message;

fn main() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2053);
    let udp_socket = UdpSocket::bind(addr).expect("Failed to bind to address");
    println!("Successfully bound to address: {:?}", addr);
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => match Message::unpack(&buf[0..size]) {
                Ok(query) => {
                    let id = query.get_id();
                    let bytes = query
                        .resolve()
                        .and_then(|res| res.pack())
                        .or_else(|e| {
                            eprintln!("Cannot resolve query: {}", e);
                            Message::server_failure().with_id(id).pack()
                        })
                        .unwrap_or_else(|e| panic!("Cannot pack message: {}", e));

                    udp_socket.send_to(&bytes, source).unwrap();
                }
                Err(e) => {
                    println!("Cannot unpack message: {}", e);
                    let id = Header::unpack_id(&buf[0..size]).unwrap_or_default();
                    let msg = Message::format_error().with_id(id);
                    udp_socket.send_to(&msg.pack().unwrap(), source).unwrap();
                }
            },
            Err(e) => {
                panic!("Error receiving data: {}", e);
            }
        }
    }
}
