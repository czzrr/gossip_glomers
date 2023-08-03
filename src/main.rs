use serde::Deserialize;
use serde::Serialize;
use serde_json::StreamDeserializer;
use serde_json::de::IoRead;
use std::io::StdinLock;
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
    pub fn new<'de>(input_stream: &mut StreamDeserializer<'_, IoRead<StdinLock>, Message>, output_stream: &mut StdoutLock) -> Node {
        for input in input_stream {
            let input = input.unwrap();
            if let Payload::Req(Request::Init { node_id, node_ids }) = input.body.payload {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(1),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::Resp(Response::InitOk { msg_id: input.body.msg_id.unwrap() }),
                    }
                };

                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();

                let node_id = node_id[1..].parse().unwrap();
                return Node { node_id, msg_id: 2, unique_id: node_id, node_ids };
            }
        }
        panic!()
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
                    _ => panic!(),
                }
            },
            Payload::Resp(_) => panic!(),
        }
        self.msg_id += 1;
    }
}

fn main() {
    let stdin = stdin().lock();
    let mut input_stream = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = stdout().lock();

    let mut node = Node::new(&mut input_stream, &mut stdout);
    for input in input_stream {
        let input = input.unwrap();
        eprintln!(
            "Received message:\n{}",
            serde_json::to_string(&input).unwrap()
        );
        node.handle(input, &mut stdout);
    }
}
