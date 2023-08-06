use gossip_glomers::main_loop;
use gossip_glomers::Event;
use gossip_glomers::Init;
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

impl Node<EchoPayload, ()> for Echo {
    fn handle(
        &mut self,
        input: Event<EchoPayload, ()>,
        output_stream: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let input = match input {
            Event::Message(msg) => msg,
            _ => panic!("Echo node only takes messages"),
        };

        match input.body.payload.clone() {
            EchoPayload::Echo { echo } => {
                let reply = input.into_reply(EchoPayload::EchoOk { echo }, Some(&mut self.msg_id));
                reply.send(output_stream);
            }
            EchoPayload::EchoOk { .. } => panic!(),
        }

        Ok(())
    }

    fn from_init(
        _init: Init,
        _tx: std::sync::mpsc::Sender<Event<EchoPayload, ()>>,
    ) -> anyhow::Result<Self> {
        Ok(Echo { msg_id: 1 })
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<Echo, _, _>()
}
