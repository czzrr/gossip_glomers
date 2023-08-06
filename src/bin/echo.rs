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
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct Echo {
    msg_id: usize,
}

impl Node<EchoPayload> for Echo {
    fn handle(&mut self, input: Message<EchoPayload>, output_stream: &mut StdoutLock) {
        match input.body.payload {
            EchoPayload::Echo { echo } => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: EchoPayload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            EchoPayload::EchoOk { .. } => panic!(),
        }
        self.msg_id += 1;
    }

    fn from_init(_init: Init) -> Self {
        Echo { msg_id: 1 }
    }
}

fn main() {
    main_loop::<Echo, _>();
}
