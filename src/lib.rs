use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::io::stdin;
use std::io::stdout;
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<P> {
    pub src: String,
    pub dest: String,
    pub body: Body<P>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
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
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Response {
    InitOk { msg_id: usize },
}

pub struct Node {
    pub node_id: usize,
    pub msg_id: usize,
    pub node_ids: Vec<String>,
}

impl Node {
    pub fn new(input_stream: &mut StdinLock, output_stream: &mut StdoutLock) -> Node {
        let inputs =
            serde_json::Deserializer::from_reader(input_stream).into_iter::<Message<Payload>>();

        for input in inputs {
            let input = input.unwrap();
            if let Payload::Req(Request::Init { node_id, node_ids }) = input.body.payload {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(1),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::Resp(Response::InitOk {
                            msg_id: input.body.msg_id.unwrap(),
                        }),
                    },
                };

                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();

                let node_id = node_id[1..].parse().unwrap();
                return Node {
                    node_id,
                    msg_id: 2,
                    node_ids,
                };
            }
        }
        panic!()
    }
}

pub trait NodeHandler<P> {
    fn handle(&mut self, node: &mut Node, input: Message<P>, output_stream: &mut StdoutLock);
}

pub fn main_loop<N, P>(mut node_handler: N)
where
    N: NodeHandler<P>,
    P: DeserializeOwned,
{
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let mut node = Node::new(&mut stdin, &mut stdout);

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();
    for input in inputs {
        let input = input.unwrap();
        node_handler.handle(&mut node, input, &mut stdout);
    }
}
