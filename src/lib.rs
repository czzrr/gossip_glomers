use anyhow::Context;
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

    pub fn send(self, mut output_stream: &mut impl Write) -> anyhow::Result<()> {
        serde_json::to_writer(&mut output_stream, &self).context("serialize message")?;
        output_stream
            .write_all(b"\n")
            .context("write trailing newline")?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}

pub enum Event<P, IP> {
    Message(Message<P>),
    InjectedPayload(IP),
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
    pub node_ids: Vec<String>,
}

impl Init {
    pub fn new(
        input_stream: &mut StdinLock,
        output_stream: &mut StdoutLock,
    ) -> anyhow::Result<Init> {
        let mut inputs =
            serde_json::Deserializer::from_reader(input_stream).into_iter::<Message<InitPayload>>();

        let input = inputs
            .next()
            .expect("init message")
            .context("failed to read init message from stdin")?;
        match input.body.payload.clone() {
            InitPayload::Init { node_id, node_ids } => {
                let msg_id = input.body.msg_id.expect("message id in init message");
                let reply = input.into_reply(InitPayload::InitOk { msg_id }, Some(&mut 0));
                reply.send(output_stream)?;

                return Ok(Init {
                    id: node_id,
                    node_ids,
                });
            }
            _ => panic!(),
        }
    }
}

pub trait Node<P, IP>
where
    Self: Sized,
{
    fn from_init(init: Init, tx: std::sync::mpsc::Sender<Event<P, IP>>) -> anyhow::Result<Self>;
    fn handle(&mut self, event: Event<P, IP>, output_stream: &mut StdoutLock)
        -> anyhow::Result<()>;
}

pub fn main_loop<N, P, IP>() -> anyhow::Result<()>
where
    N: Node<P, IP>,
    P: DeserializeOwned,
    P: Send + 'static,
    IP: Send + 'static,
{
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let init = Init::new(&mut stdin, &mut stdout)?;

    let (tx, rx) = std::sync::mpsc::channel::<Event<P, IP>>();

    let stdin_tx = tx.clone();
    drop(stdin);
    let join_handle = std::thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();
        for input in inputs {
            let input = input.expect("deserialized message");
            if let Err(_) = stdin_tx.send(Event::Message(input)) {
                return anyhow::Ok(());
            }
        }
        Ok(())
    });

    let mut node = N::from_init(init, tx)?;

    for event in rx {
        node.handle(event, &mut stdout)
            .context("failed to handle event")?;
    }
    join_handle
        .join()
        .expect("thread to shut down gracefully")
        .context("hi")?;

    Ok(())
}
