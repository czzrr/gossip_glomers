use gossip_glomers::main_loop;
use gossip_glomers::Body;
use gossip_glomers::Init;
use gossip_glomers::Message;
use gossip_glomers::Node;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::StdoutLock;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Gossip {
        message: usize,
    },
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct Broadcast {
    id: String,
    msg_id: usize,
    data: HashSet<usize>,
    topology: HashMap<String, Vec<String>>,
}

impl Node<BroadcastPayload> for Broadcast {
    fn handle(&mut self, input: Message<BroadcastPayload>, output_stream: &mut StdoutLock) {
        match input.body.payload {
            BroadcastPayload::Broadcast { message } => {
                let broadcast = !self.data.contains(&message);
                self.data.insert(message);

                let output = Message {
                    src: input.dest.clone(),
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: BroadcastPayload::BroadcastOk,
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();

                if broadcast {
                    for node_id in self
                        .topology
                        .get(&self.id)
                        .expect(&format!("topology for node {}", self.id))
                    {
                        self.msg_id += 1;

                        let output = Message {
                            src: input.dest.clone(),
                            dest: node_id.clone(),
                            body: Body {
                                msg_id: Some(self.msg_id),
                                in_reply_to: input.body.msg_id,
                                payload: BroadcastPayload::Gossip { message },
                            },
                        };
                        serde_json::to_writer(&mut *output_stream, &output).unwrap();
                        output_stream.write_all(b"\n").unwrap();
                    }
                }
            }
            BroadcastPayload::BroadcastOk => panic!(),
            BroadcastPayload::Gossip { message } => {
                self.data.insert(message);
            }
            BroadcastPayload::Read => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: BroadcastPayload::ReadOk {
                            messages: self.data.clone(),
                        },
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            BroadcastPayload::ReadOk { .. } => panic!(),
            BroadcastPayload::Topology { topology } => {
                self.topology = topology;

                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: BroadcastPayload::TopologyOk,
                    },
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            }
            BroadcastPayload::TopologyOk => panic!(),
        }
        self.msg_id += 1;
    }

    fn from_init(init: Init) -> Self {
        Broadcast {
            id: init.id,
            msg_id: 1,
            data: HashSet::new(),
            topology: HashMap::new(),
        }
    }
}

fn main() {
    main_loop::<Broadcast, _>();
}
