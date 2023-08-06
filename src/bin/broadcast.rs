use gossip_glomers::main_loop;
use gossip_glomers::Body;
use gossip_glomers::Event;
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
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        messages: HashSet<usize>,
    },
}

enum InjectedBroadcastPayload {
    Gossip,
}

struct Broadcast {
    id: String,
    msg_id: usize,
    messages: HashSet<usize>,
    topology: HashMap<String, Vec<String>>,
}

impl Node<BroadcastPayload, InjectedBroadcastPayload> for Broadcast {
    fn handle(
        &mut self,
        event: Event<BroadcastPayload, InjectedBroadcastPayload>,
        output_stream: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match event {
            Event::Message(input) => match input.body.payload.clone() {
                BroadcastPayload::Broadcast { message } => {
                    self.messages.insert(message);

                    let reply = input
                        .clone()
                        .into_reply(BroadcastPayload::BroadcastOk, Some(&mut self.msg_id));
                    reply.send(output_stream);
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
                    let reply =
                        input.into_reply(BroadcastPayload::TopologyOk, Some(&mut self.msg_id));
                    reply.send(output_stream);
                }
                BroadcastPayload::TopologyOk => panic!(),
            },
            Event::InjectedPayload(injected) => match injected {
                InjectedBroadcastPayload::Gossip => {
                    for node_id in self
                        .topology
                        .get(&self.id)
                        .expect(&format!("topology for node {}", self.id))
                    {
                        let msg = Message {
                            src: self.id.clone(),
                            dest: node_id.clone(),
                            body: Body {
                                msg_id: None,
                                in_reply_to: None,
                                payload: BroadcastPayload::Gossip {
                                    messages: self.messages.clone(),
                                },
                            },
                        };
                        msg.send(output_stream);
                    }
                }
            },
        }

        Ok(())
    }

    fn from_init(
        init: Init,
        tx: std::sync::mpsc::Sender<Event<BroadcastPayload, InjectedBroadcastPayload>>,
    ) -> anyhow::Result<Self> {
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if let Err(_) = tx.send(Event::InjectedPayload(InjectedBroadcastPayload::Gossip)) {
                break;
            }
        });

        Ok(Broadcast {
            id: init.id,
            msg_id: 1,
            messages: HashSet::new(),
            topology: HashMap::new(),
        })
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<Broadcast, _, _>()
}
