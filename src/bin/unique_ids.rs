use std::time::{SystemTime, UNIX_EPOCH};

use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    Server.run(&mut GenerateNode::new())?;
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Generate {
    Generate,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename = "generate_ok")]
struct GenerateOk {
    pub id: u64,
}

fn generate_ok(common_body: CommonBody, meta: Meta, gen_ok: GenerateOk) -> Message<GenerateOk> {
    Message {
        meta,
        body: Body {
            common: common_body,
            custom: gen_ok,
        },
    }
}

impl Node<Generate, GenerateOk> for GenerateNode {
    fn handle_init(&mut self, message: &Message<Init>) -> Result<Message<InitOk>, Message<Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);

        match message.body.custom.node_id[1..].parse() {
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
        }
    }

    fn handle(
        &mut self,
        message: &Message<Generate>,
    ) -> Result<Message<GenerateOk>, Message<Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);

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
            .and_then(|timestamp| {
                // snowflake id := 40 bit timestamp | 8 bit node id | 16 bit sequence number
                let timestamp_part = (timestamp.as_millis() >> (128 - 40)) as u64;
                let id_part = (self.id.unwrap() as u64) << 16;
                let seq_part = self.seq as u64;
                let snowflake = timestamp_part | id_part | seq_part;
                self.seq += 1;
                Ok(generate_ok(common_body, meta, GenerateOk { id: snowflake }))
            })
    }
}
