use serde::Deserialize;
use serde::Serialize;
use std::io::StdoutLock;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Body {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Payload {
    Req(Request),
    Resp(Response),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Request {
    Echo { echo: String },
    Init { node_id: String, node_ids: Vec<String> },
    Generate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Response {
    EchoOk { echo: String },
    InitOk { msg_id: usize },
    GenerateOk { id: usize },
}

struct Node {
    node_id: usize,
    msg_id: usize,
    unique_id: usize,
    node_ids: Vec<String>,
}

impl Node {
    pub fn new() -> Node {
        Node { node_id: 0, msg_id: 1, unique_id: 0, node_ids: Vec::new() }
    }

    pub fn handle(&mut self, input: Message, output_stream: &mut StdoutLock) {
        match input.body.payload {
            Payload::Req(req) => {
                match req {
                    Request::Echo { echo } => {
                        let output = Message {
                            src: input.dest,
                            dest: input.src,
                            body: Body {
                                msg_id: Some(self.msg_id),
                                in_reply_to: input.body.msg_id,
                                payload: Payload::Resp(Response::EchoOk { echo }),
                            }
                        };
                        serde_json::to_writer(&mut *output_stream, &output).unwrap();
                        output_stream.write_all(b"\n").unwrap();
                    },
                    Request::Init { node_id, node_ids } => {
                        let output = Message {
                            src: input.dest,
                            dest: input.src,
                            body: Body {
                                msg_id: Some(self.msg_id),
                                in_reply_to: input.body.msg_id,
                                payload: Payload::Resp(Response::InitOk { msg_id: input.body.msg_id.unwrap() }),
                            }
                        };

                        self.node_id = node_id[1..].parse().unwrap();
                        self.unique_id = self.node_id;
                        self.node_ids = node_ids;

                        serde_json::to_writer(&mut *output_stream, &output).unwrap();
                        output_stream.write_all(b"\n").unwrap();
                    },
                    Request::Generate => {
                        let output = Message {
                            src: input.dest,
                            dest: input.src,
                            body: Body {
                                msg_id: Some(self.msg_id),
                                in_reply_to: input.body.msg_id,
                                payload: Payload::Resp(Response::GenerateOk { id: self.unique_id }),
                            }
                        };

                        self.unique_id += self.node_ids.len();

                        serde_json::to_writer(&mut *output_stream, &output).unwrap();
                        output_stream.write_all(b"\n").unwrap(); 
                    },
                }
            },
            Payload::Resp(_) => (),
        }
        self.msg_id += 1;
    }
}

fn main() {
    let stdin = stdin().lock();
    let input_stream = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = stdout().lock();

    let mut node = Node::new();
    for input in input_stream {
        let input = input.unwrap();
        eprintln!(
            "Received message:\n{}",
            serde_json::to_string(&input).unwrap()
        );
        node.handle(input, &mut stdout);
    }
}
