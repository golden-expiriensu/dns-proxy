use std::net::UdpSocket;

use anyhow::{bail, ensure, Result};

use crate::{
    errors::DnsError,
    message::header::{Indicator, ResponseCode},
};

use super::{question::Question, Message};

const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

pub struct Resolver(UdpSocket);

impl Resolver {
    pub fn connect(address: String) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(READ_TIMEOUT))?;
        socket.connect(address)?;
        Ok(Resolver(socket))
    }

    pub fn resolve(&self, mut msg: Message) -> Result<Message> {
        msg.header.qr = Indicator::Response;
        let mut buf = [0; 512];
        let mut template = Message {
            header: msg.header.clone(),
            questions: vec![Question::default()],
            answers: vec![],
        };
        template.header.qdcount = 1;

        for question in &msg.questions {
            template.questions[0] = question.clone();
            let sent = self.0.send(&template.pack()?)?;
            ensure!(sent > 0, DnsError::ResolverNoRecv);

            let size = self.0.recv(&mut buf)?;
            let Message {
                header,
                questions: _,
                mut answers,
            } = Message::unpack(&buf[0..size])?;

            if header.rcode != ResponseCode::NoError {
                msg.header.rcode = header.rcode;
                return Ok(msg);
            }
            ensure!(header.ancount > 0, DnsError::ResolverFailed(header));

            let answer = match answers.pop() {
                Some(answer) => answer,
                None => bail!(DnsError::ResolverNoAnsw),
            };
            msg.answers.push(answer);
            msg.header.ancount += 1;
        }
        Ok(msg)
    }
}
