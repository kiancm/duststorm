use std::io::{stderr, stdin, stdout, Write};

use serde::{Deserialize, Serialize};

pub struct Server {
    pub node: Box<dyn Node>,
}

impl Server {
    pub fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout().lock();
        let mut stderr = stderr().lock();
        let mut input = stdin().lines();
        let line = input.next().expect("first message must be init")?;
        let init_request: Message = serde_json::from_str(&line)?;
        let init_response = self.node.handle_init(&init_request);
        let init_response = match init_response {
            Ok(m) => m,
            Err(m) => m,
        };
        serde_json::to_writer(&mut stdout, &init_response)?;
        stdout.write_all(b"\n")?;

        stderr.write_all(b"Request: ")?;
        serde_json::to_writer_pretty(&mut stderr, &init_request)?;
        stderr.write_all(b"\n")?;
        stderr.write_all(b"Response: ")?;
        serde_json::to_writer_pretty(&mut stderr, &init_response)?;

        for line in input {
            let line = line?;
            serde_json::to_writer_pretty(&mut stderr, &line)?;
            let req: Message = serde_json::from_str(&line)?;
            let res = self.node.handle(&req);
            let res = match res {
                Ok(m) => m,
                Err(m) => m,
            };
            serde_json::to_writer(&mut stdout, &res)?;
            stdout.write_all(b"\n")?;

            stderr.write_all(b"\n")?;
            stderr.write_all(b"Request: ")?;
            serde_json::to_writer_pretty(&mut stderr, &req)?;
            stderr.write_all(b"\n")?;
            stderr.write_all(b"Response: ")?;
            serde_json::to_writer_pretty(&mut stderr, &res)?;
            stderr.write_all(b"\n")?;
        }
        Ok(())
    }
}

pub trait Node {
    fn handle_init(&mut self, message: &Message) -> Result<Message, Message>;
    fn handle(&mut self, message: &Message) -> Result<Message, Message>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(flatten)]
    pub meta: Meta,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    pub src: String,
    pub dest: String,
}

impl Meta {
    pub fn flip(meta: &Meta) -> Meta {
        Meta {
            src: meta.dest.to_owned(),
            dest: meta.src.to_owned()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body {
    #[serde(flatten)]
    pub common: CommonBody,
    #[serde(flatten)]
    pub custom: CustomBody,
}

impl Message {
    pub fn init_ok(common_body: CommonBody, meta: Meta) -> Self {
        Self {
            meta,
            body: Body {
                common: common_body,
                custom: CustomBody::InitOk,
            },
        }
    }
    pub fn error(common_body: CommonBody, meta: Meta, error: Error) -> Self {
        Self {
            meta,
            body: Body {
                common: common_body,
                custom: CustomBody::Error(error),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommonBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomBody {
    Init(Init),
    InitOk,
    Echo(Echo),
    EchoOk(EchoOk),
    Generate,
    GenerateOk(GenerateOk),
    Error(Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Echo {
    pub echo: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoOk {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOk {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub code: ErrorCode,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum ErrorCode {
    Timeout = 0,
    NodeNotFound = 1,
    NotSupported = 10,
    TemporarilyUnavailable = 11,
    MalformedRequest = 12,
    Crash = 13,
    Abort = 14,
    KeyDoesNotExist = 20,
    KeyAlreadyExists = 21,
    PreconditionFailed = 22,
    TxnConflict = 30,
}
