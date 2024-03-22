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

impl Node<Echo> for EchoNode {
    fn handle_init(
        &mut self,
        message: &Message<Init>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reply = message.reply(InitOk::InitOk);
        sender.send(reply)?;
        Ok(())
    }

    fn handle(
        &mut self,
        message: &Message<Echo>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reply = message.reply(EchoOk {
            echo: message.body.custom.echo.clone(),
        });
        sender.send(reply)?;
        Ok(())
    }
}
