use duststorm::*;

fn main() -> std::io::Result<()> {
    let mut echo_server = Server {
        node: Box::new(EchoNode),
    };
    echo_server.run()?;
    Ok(())
}

struct EchoNode;

fn echo_ok(common_body: CommonBody, meta: Meta, echo_ok: EchoOk) -> Message {
    Message {
        meta,
        body: Body {
            common: common_body,
            custom: CustomBody::EchoOk(echo_ok),
        },
    }
}

impl Node for EchoNode {
    fn handle_init(&mut self, message: &Message) -> Result<Message, Message> {
        Ok(Message::init_ok(
            CommonBody {
                msg_id: None,
                in_reply_to: message.body.common.msg_id,
            },
            Meta::flip(&message.meta),
        ))
    }

    fn handle(&mut self, message: &Message) -> Result<Message, Message> {
        let common_body = CommonBody {
            msg_id: None,
            in_reply_to: message.body.common.msg_id,
        };
        let meta = Meta::flip(&message.meta);
        match &message.body.custom {
            CustomBody::Echo(Echo { echo }) => Ok(echo_ok(
                common_body,
                meta,
                EchoOk {
                    echo: echo.to_string(),
                },
            )),
            _ => Err(Message::error(
                common_body,
                meta,
                Error {
                    code: ErrorCode::MalformedRequest,
                },
            )),
        }
    }
}
