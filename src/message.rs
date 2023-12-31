use anyhow::{bail, Result};
use packed_struct::prelude::*;

use self::{
    answer::Answer,
    header::{Header, Indicator, DNS_HEADER_SIZE},
    question::Question,
};

mod answer;
mod header;
mod labels;
mod question;
mod rr;

pub struct Message {
    header: Header,
    question: Question,
    answer: Option<answer::Answer>,
}

impl Message {
    pub fn unpack(query: &[u8]) -> Result<Message> {
        let question = Question::unpack(&query[DNS_HEADER_SIZE..])?;
        Ok(Message {
            header: Header::unpack(query)?,
            question,
            answer: None,
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let answer = match self.answer.as_ref() {
            Some(value) => value,
            None => bail!("Message was not resolved yet"),
        };

        let mut buf = vec![0; DNS_HEADER_SIZE + self.question.len() + answer.len()];
        let mut wrote = 0;

        let mut next = DNS_HEADER_SIZE;
        self.header.pack_to_slice(&mut buf[..next])?;
        wrote += next;

        next = self.question.len();
        self.question.pack(&mut buf[wrote..wrote + next])?;
        wrote += next;

        next = answer.len();
        answer.pack(&mut buf[wrote..wrote + next])?;

        Ok(buf)
    }

    pub fn resolve(&self) -> Result<Message> {
        let answer = Answer::resolve(&self.question)?;
        let mut header = self.header.clone();
        header.qr = Indicator::Response;
        header.ancount = header.qdcount;
        Ok(Message {
            header,
            question: self.question.clone(),
            answer: Some(answer),
        })
    }
}
