use std::time::{SystemTime, UNIX_EPOCH};

use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

impl Node<Generate> for GenerateNode {
    fn handle_init(
        &mut self,
        message: &Message<Init>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);

        match message.body.custom.node_id[1..].parse() {
            Ok(id) => {
                self.id = Some(id);
                self.seq = 0;
                let message = Message::init_ok(common_body, meta);
                sender.send(message)?
            }
            Err(_) => {
                let message = Message::error(
                    common_body,
                    meta,
                    Error {
                        code: ErrorCode::MalformedRequest,
                    },
                );
                sender.send(message)?
            }
        };
        Ok(())
    }

    fn handle(
        &mut self,
        message: &Message<Generate>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);
        let result = SystemTime::now()
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
                let id = self.id.ok_or(Message::error(
                    common_body.clone(),
                    meta.clone(),
                    Error {
                        code: ErrorCode::Abort,
                    },
                ))?;
                let id_part = (id as u64) << 16;
                let seq_part = self.seq as u64;
                let snowflake = timestamp_part | id_part | seq_part;
                self.seq += 1;
                Ok(generate_ok(common_body, meta, GenerateOk { id: snowflake }))
            });
        match result {
            Ok(message) => sender.send(message)?,
            Err(error_message) => sender.send(error_message)?,
        };

        Ok(())
    }
}
