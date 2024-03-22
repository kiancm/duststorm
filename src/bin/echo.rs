use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Server.run(&mut EchoNode)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename = "echo")]
struct Echo {
    echo: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename = "echo_ok")]
struct EchoOk {
    echo: String,
}

struct EchoNode;

fn echo_ok(common_body: CommonBody, meta: Meta, echo_ok: EchoOk) -> Message<EchoOk> {
    Message {
        meta,
        body: Body {
            common: common_body,
            custom: echo_ok,
        },
    }
}

impl Node<Echo> for EchoNode {
    fn handle_init(
        &mut self,
        message: &Message<Init>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reply = Message::init_ok(
            CommonBody {
                msg_id: None,
                in_reply_to: message.body.common.msg_id,
            },
            Meta::reply(&message.meta),
        );
        sender.send(reply)?;
        Ok(())
    }

    fn handle(
        &mut self,
        message: &Message<Echo>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::reply(&message.meta);
        let reply = echo_ok(
            common_body,
            meta,
            EchoOk {
                echo: message.body.custom.echo.to_string(),
            },
        );
        sender.send(reply)?;
        Ok(())
    }
}
