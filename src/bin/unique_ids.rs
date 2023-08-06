use gossip_glomers::main_loop;
use gossip_glomers::Event;
use gossip_glomers::Init;
use gossip_glomers::Message;
use gossip_glomers::Node;
use serde::Deserialize;
use serde::Serialize;
use std::io::StdoutLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniqueIdsPayload {
    Generate,
    GenerateOk { id: String },
}

struct UniqueIds {
    id: String,
    msg_id: usize,
}

impl Node<UniqueIdsPayload, ()> for UniqueIds {
    fn handle(&mut self, input: Event<UniqueIdsPayload, ()>, output_stream: &mut StdoutLock) {
        let input = match input {
            Event::Message(msg) => msg,
            _ => panic!("UniqueIds node only takes messages"),
        };

        match input.body.payload {
            UniqueIdsPayload::Generate => {
                let reply = input.into_reply(
                    UniqueIdsPayload::GenerateOk {
                        id: format!("{}-{}", self.id, self.msg_id),
                    },
                    Some(&mut self.msg_id),
                );
                reply.send(output_stream);
            }
            UniqueIdsPayload::GenerateOk { .. } => panic!(),
        }
    }

    fn from_init(init: Init, _tx: std::sync::mpsc::Sender<Event<UniqueIdsPayload, ()>>) -> Self {
        UniqueIds {
            id: init.id,
            msg_id: 1,
        }
    }
}

fn main() {
    main_loop::<UniqueIds, _, _>();
}
