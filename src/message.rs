use anyhow::Result;
use packed_struct::prelude::*;

use self::{
    answer::Answer,
    header::{Header, Indicator, ResponseCode, DNS_HEADER_SIZE},
    question::Question,
};

mod answer;
pub mod header;
mod labels;
mod question;
pub mod resolver;
mod rr;

#[derive(Debug, Clone, Default)]
pub struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Message {
    pub fn unpack(query: &[u8]) -> Result<Message> {
        let header = Header::unpack(query)?;

        let mut ptr = DNS_HEADER_SIZE;
        let questions = (0..header.qdcount)
            .map(|_| Question::unpack(query, &mut ptr))
            .collect::<Result<Vec<_>>>()?;

        let answers = match header.qr {
            Indicator::Query => Vec::with_capacity(questions.len()),
            Indicator::Response => (0..header.ancount)
                .map(|_| Answer::unpack(query, &mut ptr))
                .collect::<Result<Vec<_>>>()?,
        };

        Ok(Message {
            header,
            questions,
            answers,
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let len = DNS_HEADER_SIZE
            + self.questions.iter().map(Question::len).sum::<usize>()
            + self.answers.iter().map(Answer::len).sum::<usize>();

        let mut buf = vec![0; len];
        let mut next = DNS_HEADER_SIZE;
        self.header.pack_to_slice(&mut buf[..next])?;
        let mut wrote = next;

        for question in &self.questions {
            next = question.len();
            question.pack(&mut buf[wrote..wrote + next])?;
            wrote += next;
        }
        for answer in &self.answers {
            next = answer.len();
            answer.pack(&mut buf[wrote..wrote + next])?;
            wrote += next;
        }
        Ok(buf)
    }

    pub fn new_client_err() -> Self {
        let mut msg = Self::default();
        msg.header.rcode = ResponseCode::FormatError;
        msg
    }

    pub fn new_server_err() -> Self {
        let mut msg = Self::default();
        msg.header.rcode = ResponseCode::ServerFailure;
        msg
    }

    pub fn with_id(mut self, id: u16) -> Self {
        self.header.id = id;
        self
    }

    pub fn get_id(&self) -> u16 {
        self.header.id
    }
}
