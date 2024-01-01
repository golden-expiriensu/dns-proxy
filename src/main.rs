use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

use crate::message::Message;

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
                    let bytes = query
                        .resolve()
                        .and_then(|res| res.pack())
                        .or_else(|e| {
                            eprintln!("Cannot resolve query: {}", e);
                            Message::server_failure().pack()
                        })
                        .unwrap_or_else(|e| panic!("Cannot pack message: {}", e));

                    udp_socket.send_to(&bytes, source).unwrap();
                }
                Err(e) => {
                    eprintln!("Cannot unpack message: {}", e);
                    udp_socket
                        .send_to(&Message::format_error().pack().unwrap(), source)
                        .unwrap();
                }
            },
            Err(e) => {
                panic!("Error receiving data: {}", e);
            }
        }
    }
}
