use gossip_glomers::main_loop;
use gossip_glomers::Init;
use gossip_glomers::Message;
use gossip_glomers::Node;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::StdoutLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Gossip {
        messages: HashSet<usize>,
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
    messages: HashSet<usize>,
    topology: HashMap<String, Vec<String>>,
}

impl Node<BroadcastPayload> for Broadcast {
    fn handle(&mut self, input: Message<BroadcastPayload>, output_stream: &mut StdoutLock) {
        match input.body.payload.clone() {
            BroadcastPayload::Broadcast { message } => {
                self.messages.insert(message);

                let reply = input
                    .clone()
                    .into_reply(BroadcastPayload::BroadcastOk, Some(&mut self.msg_id));
                reply.send(output_stream);

                for node_id in self
                    .topology
                    .get(&self.id)
                    .expect(&format!("topology for node {}", self.id))
                {
                    let mut reply = input.clone().into_reply(
                        BroadcastPayload::Gossip {
                            messages: self.messages.clone(),
                        },
                        Some(&mut self.msg_id),
                    );
                    reply.dest = node_id.clone();
                    reply.body.in_reply_to = None;
                    reply.send(output_stream);
                }
            }
            BroadcastPayload::BroadcastOk => panic!(),
            BroadcastPayload::Gossip { messages } => {
                self.messages.extend(messages);
            }
            BroadcastPayload::Read => {
                let reply = input.into_reply(
                    BroadcastPayload::ReadOk {
                        messages: self.messages.clone(),
                    },
                    Some(&mut self.msg_id),
                );
                reply.send(output_stream);
            }
            BroadcastPayload::ReadOk { .. } => panic!(),
            BroadcastPayload::Topology { topology } => {
                self.topology = topology;
                let reply = input.into_reply(BroadcastPayload::TopologyOk, Some(&mut self.msg_id));
                reply.send(output_stream);
            }
            BroadcastPayload::TopologyOk => panic!(),
        }
        self.msg_id += 1;
    }

    fn from_init(init: Init) -> Self {
        Broadcast {
            id: init.id,
            msg_id: 1,
            messages: HashSet::new(),
            topology: HashMap::new(),
        }
    }
}

fn main() {
    main_loop::<Broadcast, _>();
}
