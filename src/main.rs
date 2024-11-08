use alfred_rs::AlfredModule;
use alfred_rs::connection::{Receiver, Sender};
use alfred_rs::error::Error;
use alfred_rs::message::Message;
use alfred_rs::log::warn;
use alfred_rs::tokio;

const MODULE_NAME: &str = "ai_callback";
const INPUT_TOPIC: &str = "ai_callback";

async fn on_input(message: &Message, module: &mut AlfredModule) -> Result<(), Error> {
    let msg_text = message.text.as_str();
    let (relay_topic, relay_msg) = if msg_text.starts_with('`') && msg_text.ends_with('`') && msg_text.contains(": ") {
        let msg_text = msg_text.to_string();
        let split = msg_text.split(": ").collect::<Vec<&str>>();
        let relay_topic = &split[0][1..];
        let relay_msg_text = &split[1][..split[1].len() - 1];
        let relay_msg = Message {
            text: relay_msg_text.to_string(),
            message_type: message.message_type.clone(),
            ..message.clone()
        };
        (relay_topic.to_string(), relay_msg)
    } else {
        message.reply(message.text.clone(), message.message_type.clone())?
    };
    module.send(relay_topic.as_str(), &relay_msg).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let mut module = AlfredModule::new(MODULE_NAME).await?;
    module.listen(INPUT_TOPIC).await.expect("Error on subscribe");
    loop {
        let (topic, message) = module.receive().await.expect("Error on getting new messages");
        match topic.as_str() {
            INPUT_TOPIC => on_input(&message, &mut module).await?,
            _ => {
                warn!("Unknown topic {topic}");
            }
        }
    }
}
