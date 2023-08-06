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

impl<P> Message<P>
where
    P: Serialize,
{
    pub fn into_reply(self, payload: P, msg_id: Option<&mut usize>) -> Self {
        Message {
            src: self.dest,
            dest: self.src,
            body: Body {
                msg_id: msg_id.map(|mid| {
                    let old_mid = *mid;
                    *mid += 1;
                    old_mid
                }),
                in_reply_to: self.body.msg_id,
                payload,
            },
        }
    }

    pub fn send(self, mut output_stream: &mut impl Write) {
        serde_json::to_writer(&mut output_stream, &self).unwrap();
        output_stream.write_all(b"\n").unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        msg_id: usize,
    },
}

pub struct Init {
    pub id: String,
    pub msg_id: usize,
    pub node_ids: Vec<String>,
}

impl Init {
    pub fn new(input_stream: &mut StdinLock, output_stream: &mut StdoutLock) -> Init {
        let inputs =
            serde_json::Deserializer::from_reader(input_stream).into_iter::<Message<InitPayload>>();

        for input in inputs {
            let input = input.unwrap();
            if let InitPayload::Init { node_id, node_ids } = input.body.payload {
                let output = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(1),
                        in_reply_to: input.body.msg_id,
                        payload: InitPayload::InitOk {
                            msg_id: input.body.msg_id.unwrap(),
                        },
                    },
                };

                serde_json::to_writer(&mut *output_stream, &output).unwrap();
                output_stream.write_all(b"\n").unwrap();

                return Init {
                    id: node_id,
                    msg_id: 2,
                    node_ids,
                };
            }
        }
        panic!()
    }
}

pub trait Node<P> {
    fn from_init(init: Init) -> Self;
    fn handle(&mut self, input: Message<P>, output_stream: &mut StdoutLock);
}

pub fn main_loop<N, P>()
where
    N: Node<P>,
    P: DeserializeOwned,
{
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let init = Init::new(&mut stdin, &mut stdout);
    let mut node = N::from_init(init);
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();

    //let (rx, tx) = std::sync::mpsc::channel::<Event<P, IP>>();

    for input in inputs {
        let input = input.unwrap();
        node.handle(input, &mut stdout);
    }
}
