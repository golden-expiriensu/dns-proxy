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
            Ok((size, source)) => {
                // TODO: don't panic if error
                let query = Message::unpack(&buf[0..size]).unwrap();
                udp_socket
                    .send_to(&query.resolve().unwrap().pack().unwrap(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
