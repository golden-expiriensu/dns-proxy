use anyhow::{anyhow, Result};
use packed_struct::prelude::*;

use self::{
    answer::Answer,
    header::{Header, Indicator, ResponseCode, DNS_HEADER_SIZE},
    question::Question,
};

mod answer;
mod header;
mod labels;
mod question;
mod rr;

pub struct Message {
    header: Header,
    question: Option<Question>,
    answer: Option<answer::Answer>,
}

impl Message {
    pub fn unpack(query: &[u8]) -> Result<Message> {
        let question = Question::unpack(&query[DNS_HEADER_SIZE..])?;
        Ok(Message {
            header: Header::unpack(query)?,
            question: Some(question),
            answer: None,
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let len = DNS_HEADER_SIZE
            + self.question.as_ref().map_or(0, |q| q.len())
            + self.answer.as_ref().map_or(0, |q| q.len());

        let mut buf = vec![0; len];
        let mut wrote = 0;

        let next = DNS_HEADER_SIZE;
        self.header.pack_to_slice(&mut buf[..next])?;
        wrote += next;

        let (question, next) = match self.question.as_ref() {
            Some(q) => (q, q.len()),
            None => return Ok(buf),
        };
        question.pack(&mut buf[wrote..wrote + next])?;
        wrote += next;

        let (answer, next) = match self.answer.as_ref() {
            Some(a) => (a, a.len()),
            None => return Ok(buf),
        };
        answer.pack(&mut buf[wrote..wrote + next])?;

        Ok(buf)
    }

    pub fn resolve(&self) -> Result<Message> {
        let answer = Answer::resolve(
            self.question
                .as_ref()
                .ok_or(anyhow!("Cannot resolve question because it is `None`"))?,
        )?;

        let mut header = self.header.clone();
        header.qr = Indicator::Response;
        header.ancount = header.qdcount;
        header.rcode = match header.opcode.into() {
            0 => ResponseCode::NoError,
            _ => ResponseCode::NotImplemented,
        };

        Ok(Message {
            header,
            question: self.question.clone(),
            answer: Some(answer),
        })
    }

    pub fn format_error() -> Message {
        let mut header = Header::default();
        header.rcode = ResponseCode::FormatError;
        Message {
            header,
            question: None,
            answer: None,
        }
    }

    pub fn server_failure() -> Message {
        let mut header = Header::default();
        header.rcode = ResponseCode::ServerFailure;
        Message {
            header,
            question: None,
            answer: None,
        }
    }
}
