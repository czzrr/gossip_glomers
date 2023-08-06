use gossip_glomers::main_loop;
use gossip_glomers::Body;
use gossip_glomers::Init;
use gossip_glomers::Message;
use gossip_glomers::Node;
use serde::Deserialize;
use serde::Serialize;
use std::io::StdoutLock;
use std::io::Write;

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

impl Node<UniqueIdsPayload> for UniqueIds {
    fn handle(&mut self, input: Message<UniqueIdsPayload>, output_stream: &mut StdoutLock) {
        match input.body.payload {
            UniqueIdsPayload::Generate => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: UniqueIdsPayload::GenerateOk {
                            id: format!("{}-{}", self.id, self.msg_id),
                        },
                    },
                };

                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            UniqueIdsPayload::GenerateOk { .. } => panic!(),
        }
        self.msg_id += 1;
    }

    fn from_init(init: Init) -> Self {
        UniqueIds {
            id: init.id,
            msg_id: 1,
        }
    }
}

fn main() {
    main_loop::<UniqueIds, _>();
}
