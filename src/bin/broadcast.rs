use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    Server.run(&mut BroadcastNode::new())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Topology {

}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Broadcast {
    Broadcast { message: i32 },
    Read,
    Topology { topology: serde_json::Value }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BroadcastOk {
    BroadcastOk,
    ReadOk { messages: Vec<i32> },
    TopologyOk,
}

struct BroadcastNode {
    messages: Vec<i32>
}

impl Node<Broadcast, BroadcastOk> for BroadcastNode {
    fn handle_init(&mut self, message: &Message<Init>) -> Result<Message<InitOk>, Message<Error>> {
        Ok(Message::init_ok(
            CommonBody {
                msg_id: None,
                in_reply_to: message.body.common.msg_id,
            },
            Meta::reply(&message.meta),
        ))
    }

    fn handle(&mut self, message: &Message<Broadcast>) -> Result<Message<BroadcastOk>, Message<Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);
        match message.body.custom {
            Broadcast::Broadcast { message } => {
                self.messages.push(message);
                Ok(self.broadcast_ok(common_body, meta))
            },
            Broadcast::Read => Ok(self.read_ok(common_body, meta)),
            Broadcast::Topology { topology: _ } => Ok(self.topology_ok(common_body, meta)),
        }
    }
}

impl BroadcastNode {
    fn new() -> Self {
        Self { messages: Vec::new() }
    }

    fn broadcast_ok(&self, common_body: CommonBody, meta: Meta) -> Message<BroadcastOk> {
        Message {
            meta,
            body: Body {
                common: common_body,
                custom: BroadcastOk::BroadcastOk,
            },
        }
    }

    fn read_ok(&self, common_body: CommonBody, meta: Meta) -> Message<BroadcastOk> {
        Message {
            meta,
            body: Body {
                common: common_body,
                custom: BroadcastOk::ReadOk { messages: self.messages.clone() },
            },
        }
    }

    fn topology_ok(&self, common_body: CommonBody, meta: Meta) -> Message<BroadcastOk> {
        Message {
            meta,
            body: Body {
                common: common_body,
                custom: BroadcastOk::TopologyOk,
            },
        }
    }
}
