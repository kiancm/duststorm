use std::{sync::mpsc::Sender, time::{SystemTime, UNIX_EPOCH}};

use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
    Server.run<Generate, GenerateNode>()?;
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
    GenerateOk { id: u64 }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename = "generate_ok")]
struct GenerateOk {
    pub id: u64,
}

fn generate_ok(common_body: CommonBody, meta: Meta, id: u64) -> Message<Generate> {
    Message {
        meta,
        body: Body {
            common: common_body,
            custom: Generate::GenerateOk { id },
        },
    }
}

impl Node<Generate> for GenerateNode {
    fn handle(
        &mut self,
        message: &Message<Generate>,
        sender: &Sender<Generate>,
    ) -> anyhow::Result<()> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);
        let result = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_err| message.error(ErrorCode::Abort.into()))
            .and_then(|timestamp| {
                // snowflake id := 40 bit timestamp | 8 bit node id | 16 bit sequence number
                let timestamp_part = (timestamp.as_millis() >> (128 - 40)) as u64;
                let id = self.id.ok_or(message.error(ErrorCode::Abort.into()))?;
                let id_part = (id as u64) << 16;
                let seq_part = self.seq as u64;
                let snowflake = timestamp_part | id_part | seq_part;
                self.seq += 1;
                Ok(generate_ok(common_body, meta, snowflake))
            });
        match result {
            Ok(message) => sender.send(&message)?,
            Err(error_message) => sender.send(&error_message)?,
        };

        Ok(())
    }
}

impl TryFrom<Message<Init>> for GenerateNode {
    type Error = anyhow::Error;

    fn try_from(value: Message<Init>) -> Result<Self, Self::Error> {
        Ok(GenerateNode::new())
    }
}
