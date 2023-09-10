use std::time::{SystemTime, UNIX_EPOCH};

use duststorm::*;

fn main() -> std::io::Result<()> {
    let mut echo_server = Server {
        node: Box::new(GenerateNode::new()),
    };
    echo_server.run()?;
    Ok(())
}
struct GenerateNode {
    id: Option<u8>,
    seq: u16,
}
impl GenerateNode {
    fn new() -> Self {
        Self { id: None, seq: 0 }
    }
}

pub fn generate_ok(common_body: CommonBody, meta: Meta, gen_ok: GenerateOk) -> Message {
    Message {
        meta,
        body: Body {
            common: common_body,
            custom: CustomBody::GenerateOk(gen_ok),
        },
    }
}

impl Node for GenerateNode {
    fn handle_init(&mut self, message: &Message) -> Result<Message, Message> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::flip(&message.meta);

        match &message.body.custom {
            CustomBody::Init(Init {
                node_id,
                node_ids: _,
            }) => match node_id[1..].parse() {
                Ok(id) => {
                    self.id = Some(id);
                    self.seq = 0;
                    Ok(Message::init_ok(common_body, meta))
                }
                Err(_) => Err(Message::error(
                    common_body,
                    meta,
                    Error {
                        code: ErrorCode::MalformedRequest,
                    },
                )),
            },
            _ => Err(Message::error(
                common_body,
                meta,
                Error {
                    code: ErrorCode::MalformedRequest,
                },
            )),
        }
    }

    fn handle(&mut self, message: &Message) -> Result<Message, Message> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::flip(&message.meta);

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_err| {
                Message::error(
                    common_body.clone(),
                    meta.clone(),
                    Error {
                        code: ErrorCode::Abort,
                    },
                )
            })
            .and_then(|timestamp| match message.body.custom {
                CustomBody::Generate => {
                    // snowflake id := 40 bit timestamp | 8 bit node id | 16 bit sequence number
                    let timestamp_part = (timestamp.as_millis() >> (128 - 40)) as u64;
                    let id_part = (self.id.unwrap() as u64) << 16;
                    let seq_part = self.seq as u64;
                    let snowflake = timestamp_part | id_part | seq_part;
                    self.seq += 1;
                    Ok(Message::generate_ok(
                        common_body,
                        meta,
                        GenerateOk { id: snowflake },
                    ))
                }
                _ => Err(Message::error(
                    common_body,
                    meta,
                    Error {
                        code: ErrorCode::Abort,
                    },
                )),
            })
    }
}
