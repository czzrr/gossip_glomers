use gossip_glomers::main_loop;
use gossip_glomers::Body;
use gossip_glomers::Message;
use gossip_glomers::Node;
use gossip_glomers::NodeHandler;
use serde::Deserialize;
use serde::Serialize;
use std::io::StdoutLock;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk { id: String },
}

struct UniqueIds;

impl NodeHandler<Payload> for UniqueIds {
    fn handle(&mut self, node: &mut Node, input: Message<Payload>, output_stream: &mut StdoutLock) {
        match input.body.payload {
            Payload::Generate => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(node.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::GenerateOk {
                            id: format!("{}-{}", node.node_id, node.msg_id),
                        },
                    },
                };

                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            Payload::GenerateOk { .. } => panic!(),
        }
        node.msg_id += 1;
    }
}

fn main() {
    main_loop(UniqueIds);
}
