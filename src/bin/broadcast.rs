use gossip_glomers::main_loop;
use gossip_glomers::Body;
use gossip_glomers::Message;
use gossip_glomers::Node;
use gossip_glomers::NodeHandler;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::StdoutLock;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast { message: usize },
    BroadcastOk,
    Read,
    ReadOk { messages: HashSet<usize> },
    Topology { topology: HashMap<String, Vec<String>> },
    TopologyOk,
}

struct Broadcast {
    messages_received: HashSet<usize>,
}

impl NodeHandler<Payload> for Broadcast {
    fn handle(&mut self, node: &mut Node, input: Message<Payload>, output_stream: &mut StdoutLock) {
        match input.body.payload {
            Payload::Broadcast { message } => {
                self.messages_received.insert(message);

                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(node.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::BroadcastOk,
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            Payload::BroadcastOk { .. } => panic!(),
            Payload::Read => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(node.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::ReadOk { messages: self.messages_received.clone() },
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            },
            Payload::ReadOk { .. } => panic!(),
            Payload::Topology { topology: _ } => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(node.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::TopologyOk,
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            },
            Payload::TopologyOk => panic!(),
        }
        node.msg_id += 1;
    }
}

fn main() {
    main_loop(Broadcast { messages_received: HashSet::new() });
}
