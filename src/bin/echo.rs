use std::sync::mpsc::Sender;

use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
    Server.run::<Echo, EchoNode>()?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename = "echo")]
enum Echo {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode;

impl Node<Echo> for EchoNode {
    fn handle(
        &mut self,
        message: &Message<Echo>,
        sender: &Sender<Message<Echo>>,
    ) -> anyhow::Result<()> {
        if let Echo::Echo { ref echo } = message.body.custom {
            let reply = message.reply(Echo::EchoOk { echo: echo.clone() });
            sender.send(reply)?;
        }
        Ok(())
    }
}

impl TryFrom<Message<Init>> for EchoNode {
    type Error = anyhow::Error;

    fn try_from(_value: Message<Init>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
