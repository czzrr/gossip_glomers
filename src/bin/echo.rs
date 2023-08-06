use gossip_glomers::main_loop;
use gossip_glomers::Init;
use gossip_glomers::Message;
use gossip_glomers::Node;
use serde::Deserialize;
use serde::Serialize;
use std::io::StdoutLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct Echo {
    msg_id: usize,
}

impl Node<EchoPayload> for Echo {
    fn handle(&mut self, input: Message<EchoPayload>, output_stream: &mut StdoutLock) {
        match input.body.payload.clone() {
            EchoPayload::Echo { echo } => {
                let reply = input.into_reply(EchoPayload::EchoOk { echo }, Some(&mut self.msg_id));
                reply.send(output_stream);
            }
            EchoPayload::EchoOk { .. } => panic!(),
        }
    }

    fn from_init(_init: Init) -> Self {
        Echo { msg_id: 1 }
    }
}

fn main() {
    main_loop::<Echo, _>();
}
