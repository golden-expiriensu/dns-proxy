use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

fn main() {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 2053);
    let udp_socket = UdpSocket::bind(addr).expect("Failed to bind to address");
    println!("Successfully bound to address: {:?}", addr);
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let response = [];
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
