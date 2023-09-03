use std::io::{stderr, stdin, stdout, Write};

use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    let echo_server = Server { node: Box::new(EchoNode) };
    echo_server.run()?;
    Ok(())
}

struct Server {
    node: Box<dyn Node>,
}

impl Server {
    fn run(&self) -> std::io::Result<()> {
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

trait Node {
    fn handle_init(&self, message: &Message) -> Result<Message, Message>;
    fn handle(&self, message: &Message) -> Result<Message, Message>;
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    #[serde(flatten)]
    meta: Meta,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct Meta {
    src: String,
    dest: String,
}

impl Meta {
    fn flip(meta: &Meta) -> Meta {
        Meta {
            src: meta.dest.to_owned(),
            dest: meta.src.to_owned()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(flatten)]
    common: CommonBody,
    #[serde(flatten)]
    custom: CustomBody,
}

impl Message {
    fn init(common_body: CommonBody, meta: Meta, init: Init) -> Message {
        Self {
            meta,
            body: Body {
                message_type: "init".to_string(),
                common: common_body,
                custom: CustomBody::Init(init),
            },
        }
    }
}
impl Message {
    fn init_ok(common_body: CommonBody, meta: Meta) -> Self {
        Self {
            meta,
            body: Body {
                message_type: "init_ok".to_string(),
                common: common_body,
                custom: CustomBody::InitOk,
            },
        }
    }
}
impl Message {
    fn echo_ok(common_body: CommonBody, meta: Meta, echo_ok: EchoOk) -> Self {
        Self {
            meta,
            body: Body {
                message_type: "echo_ok".to_string(),
                common: common_body,
                custom: CustomBody::EchoOk(echo_ok),
            },
        }
    }
}
impl Message {
    fn error(common_body: CommonBody, meta: Meta, error: Error) -> Self {
        Self {
            meta,
            body: Body {
                message_type: "error".to_string(),
                common: common_body,
                custom: CustomBody::Error(error),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CommonBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum CustomBody {
    Init(Init),
    Echo(Echo),
    EchoOk(EchoOk),
    InitOk,
    Error(Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Echo {
    echo: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct EchoOk {
    echo: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    code: ErrorCode,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
enum ErrorCode {
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

struct EchoNode;

impl Node for EchoNode {
    fn handle_init(&self, message: &Message) -> Result<Message, Message> {
        Ok(Message::init_ok(
            CommonBody {
                msg_id: None,
                in_reply_to: message.body.common.msg_id,
            },
            Meta::flip(&message.meta)
        ))
    }

    fn handle(&self, message: &Message) -> Result<Message, Message> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::flip(&message.meta);
        match &message.body.custom {
            CustomBody::Echo(Echo { echo }) => Ok(Message::echo_ok(common_body, meta, EchoOk { echo: echo.to_string() })),
            _ => Err(Message::error(common_body, meta, Error { code: ErrorCode::MalformedRequest }))
        }
    }
}
