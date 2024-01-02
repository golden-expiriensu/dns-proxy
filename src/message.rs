use anyhow::{ensure, Result};
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
mod rr;

#[derive(Debug, Default)]
pub struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Message {
    pub fn unpack(query: &[u8]) -> Result<Message> {
        let header = Header::unpack(query)?;

        let mut ptr = DNS_HEADER_SIZE;
        let mut questions = Vec::with_capacity(header.qdcount as usize);
        for _ in 0..header.qdcount {
            ensure!(query.len() > ptr, "Query is too short");
            let question = Question::unpack(&query[ptr..])?;
            ptr += question.len();
            questions.push(question);
        }

        let answers = Vec::with_capacity(questions.len());

        Ok(Message {
            header,
            questions,
            answers,
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let len = DNS_HEADER_SIZE
            + self.questions.iter().map(|q| q.len()).sum::<usize>()
            + self.answers.iter().map(|a| a.len()).sum::<usize>();

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

    pub fn resolve(mut self) -> Result<Self> {
        self.answers = self
            .questions
            .iter()
            .map(Answer::resolve)
            .collect::<Result<_>>()?;

        self.header.qr = Indicator::Response;
        self.header.ancount = self.header.qdcount;
        self.header.rcode = match self.header.opcode.into() {
            0 => ResponseCode::NoError,
            _ => ResponseCode::NotImplemented,
        };

        Ok(self)
    }

    pub fn format_error() -> Message {
        let mut msg = Message::default();
        msg.header.rcode = ResponseCode::FormatError;
        msg
    }

    pub fn server_failure() -> Message {
        let mut msg = Message::default();
        msg.header.rcode = ResponseCode::ServerFailure;
        msg
    }

    pub fn with_id(mut self, id: u16) -> Message {
        self.header.id = id;
        self
    }

    pub fn get_id(&self) -> u16 {
        self.header.id
    }
}
