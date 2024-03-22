use std::io::{stderr, stdin, stdout, StderrLock, StdoutLock, Write};

use serde::{Deserialize, Serialize};

pub struct Server;
impl Server {
    pub fn run<I>(&self, node: &mut impl Node<I>) -> Result<(), Box<dyn std::error::Error>>
    where
        for<'a> I: Deserialize<'a> + Serialize,
    {
        let mut sender = Sender::new();
        let mut input = stdin().lines();
        let line = input.next().expect("first message must be init")?;
        let init_request: Message<Init> = serde_json::from_str(&line)?;
        node.handle_init(&init_request, &mut sender)?;

        for line in input {
            let line = line?;
            sender.err.write_all(line.as_bytes())?;
            let req: Message<I> = serde_json::from_str(&line)?;
            node.handle(&req, &mut sender)?;
        }
        Ok(())
    }
}

pub struct Sender<'a> {
    out: StdoutLock<'a>,
    err: StderrLock<'a>,
}

impl<'a> Sender<'a> {
    pub fn new() -> Self {
        Self {
            out: stdout().lock(),
            err: stderr().lock(),
        }
    }

    pub fn send<T: Serialize>(
        &mut self,
        message: Message<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        serde_json::to_writer(&mut self.out, &message)?;
        self.out.write_all(b"\n")?;
        Ok(())
    }

    pub fn debug<T: Serialize>(&mut self, message: T) -> Result<(), Box<dyn std::error::Error>> {
        serde_json::to_writer(&mut self.err, &message)?;
        self.err.write_all(b"\n")?;
        Ok(())
    }
}

pub trait Node<Req> {
    fn handle_init(
        &mut self,
        message: &Message<Init>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn handle(
        &mut self,
        message: &Message<Req>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<T> {
    #[serde(flatten)]
    pub meta: Meta,
    pub body: Body<T>,
}

impl<I> Message<I> {
    pub fn reply<O>(&self, custom_body: O) -> Message<O> {
        let meta = self.meta.reply();
        let body = Body {
            common: CommonBody {
                msg_id: None,
                in_reply_to: self.body.common.msg_id,
            },
            custom: custom_body,
        };

        Message { meta, body }
    }

    pub fn error(&self, error: Error) -> Message<Error> {
        self.reply(error)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum BodyOrError<B> {
    Body(B),
    Error(Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    pub src: String,
    pub dest: String,
}

impl Meta {
    pub fn reply(&self) -> Self {
        Self {
            src: self.dest.to_owned(),
            dest: self.src.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body<T> {
    #[serde(flatten)]
    pub common: CommonBody,
    #[serde(flatten)]
    pub custom: T,
}

impl Message<InitOk> {
    pub fn init_ok(common_body: CommonBody, meta: Meta) -> Self {
        Self {
            meta,
            body: Body {
                common: common_body,
                custom: InitOk::InitOk,
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
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitOk {
    InitOk,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
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

impl Into<Error> for ErrorCode {
    fn into(self) -> Error {
        Error { code: self }
    }
}
