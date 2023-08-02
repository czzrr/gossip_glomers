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

#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
    Init { node_id: String, node_ids: Vec<String> },
    InitOk { msg_id: usize },
}

struct Node {
    id: usize,
}

impl Node {
    pub fn new() -> Node {
        Node { id: 1 }
    }

    pub fn handle(&mut self, input: Message, output_stream: &mut StdoutLock) {
        match input.body.payload {
            Payload::Echo { echo } => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::EchoOk { echo },
                    }
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            },
            Payload::EchoOk { .. } => (),
            Payload::Init { .. } => {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::InitOk { msg_id: input.body.msg_id.unwrap() },
                    }
                };
                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();
            },
            Payload::InitOk { .. } => unreachable!(),
        }
        self.id += 1;
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
