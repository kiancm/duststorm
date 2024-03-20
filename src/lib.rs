use std::io::{stderr, stdin, stdout, Write};

use serde::{Deserialize, Serialize};

pub struct Server;
impl Server {
    pub fn run<I, O: Serialize>(&self, node: &mut impl Node<I, O>) -> std::io::Result<()>
    where
        for<'a> I: Deserialize<'a> + Serialize,
    {
        let mut stdout = stdout().lock();
        let mut stderr = stderr().lock();
        let mut input = stdin().lines();
        let line = input.next().expect("first message must be init")?;
        let init_request: Message<Init> = serde_json::from_str(&line)?;
        let init_response = node.handle_init(&init_request);
        let init_response = self.flatten_result(init_response);

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
            let req: Message<I> = serde_json::from_str(&line)?;
            let res = node.handle(&req);
            let res = self.flatten_result(res);

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

    fn flatten_result<T>(
        &self,
        result: Result<Message<T>, Message<Error>>,
    ) -> Message<BodyOrError<T>> {
        match result {
            Ok(ok) => Message {
                meta: ok.meta,
                body: Body {
                    common: ok.body.common,
                    custom: BodyOrError::Body(ok.body.custom),
                },
            },
            Err(err) => Message {
                meta: err.meta,
                body: Body {
                    common: err.body.common,
                    custom: BodyOrError::Error(err.body.custom),
                },
            },
        }
    }
}

pub trait Node<I, O> {
    fn handle_init(&mut self, message: &Message<Init>) -> Result<Message<InitOk>, Message<Error>>;
    fn handle(&mut self, message: &Message<I>) -> Result<Message<O>, Message<Error>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<T> {
    #[serde(flatten)]
    pub meta: Meta,
    pub body: Body<T>,
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
    pub fn reply(meta: &Meta) -> Meta {
        Meta {
            src: meta.dest.to_owned(),
            dest: meta.src.to_owned(),
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

impl Message<Error> {
    pub fn error(common_body: CommonBody, meta: Meta, error: Error) -> Self {
        Self {
            meta,
            body: Body {
                common: common_body,
                custom: error,
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
