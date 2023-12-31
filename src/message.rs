use anyhow::Result;
use packed_struct::prelude::*;

use self::{
    header::{Header, DNS_HEADER_SIZE},
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
}

impl Message {
    pub fn unpack(query: &[u8]) -> Result<Message> {
        let question = Question::unpack(&query[DNS_HEADER_SIZE..])?;
        Ok(Message {
            header: Header::unpack(query)?,
            question,
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        let mut result = vec![0; DNS_HEADER_SIZE + self.question.len()];
        self.header.pack_to_slice(&mut result[..DNS_HEADER_SIZE])?;
        self.question.pack(&mut result[DNS_HEADER_SIZE..])?;
        Ok(result)
    }

    pub fn response(&self) -> Message {
        Message {
            header: self.header.clone_as_response(),
            question: self.question.clone(),
        }
    }
}
