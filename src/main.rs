use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap as Map;
use std::io::stdin;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MessageBody {
    #[serde(rename = "type")]
    ty: String,
    #[serde(flatten)]
    opt: Map<String, Value>,
}

fn main() {
    let mut de = serde_json::Deserializer::from_reader(stdin());

    let node_id = init(&mut de);
    dbg!(node_id);

    loop {
        let msg = get_message(&mut de);
        match msg {
            Ok(msg) => {
                eprintln!(
                    "Received message:\n{}",
                    serde_json::to_string(&msg).unwrap()
                );
                match handle_message(&msg) {
                    Ok(_) => (),
                    _ => break,
                }
            }
            Err(_) => break,
        }
    }
}

fn get_message(
    de: &mut serde_json::Deserializer<serde_json::de::IoRead<std::io::Stdin>>,
) -> Result<Message, serde_json::Error> {
    let msg = Message::deserialize(de);
    msg
}

fn send_message(msg: &Message) {
    println!("{}", serde_json::to_string(&msg).unwrap());

}

fn init(de: &mut serde_json::Deserializer<serde_json::de::IoRead<std::io::Stdin>>) -> String {
    let msg = get_message(de).unwrap();
    let node_id = msg
        .body
        .opt
        .get("node_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    respond_init(&msg, node_id.clone());

    node_id
}

fn handle_message(msg: &Message) -> Result<(), ()> {
    let body = &msg.body;
    match body.ty.as_str() {
        "echo" => {
            let mut body2 = body.clone();
            body2.ty = "echo_ok".to_owned();
            body2.opt.insert("in_reply_to".to_owned(), body2.opt.get("msg_id").unwrap().clone());
            let msg2 = Message {
                src: msg.dest.clone(),
                dest: msg.src.clone(),
                body: body2
            };
            send_message(&msg2);

            Ok(())
        }
        _ => Err(()),
    }
}

fn respond_init(msg: &Message, node_id: String) {
    let body = MessageBody {
        ty: "init_ok".to_owned(),
        opt: Map::from([(
            "in_reply_to".to_owned(),
            Value::from(msg.body.opt.get("msg_id").unwrap().as_i64().unwrap()),
        )]),
    };
    let msg2 = Message {
        src: node_id,
        dest: msg.src.clone(),
        body,
    };
    send_message(&msg2);
}
