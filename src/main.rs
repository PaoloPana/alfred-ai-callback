use std::error::Error;
use alfred_rs::AlfredModule;
use alfred_rs::message::Message;
use alfred_rs::log::warn;
use alfred_rs::tokio;
use serde::Deserialize;

const MODULE_NAME: &str = "ai_callback";
const INPUT_TOPIC: &str = "ai_callback";

#[derive(Deserialize)]
struct AIMessage {
    pub text: String,
    pub command: String
}

fn clean_json(msg_text: String) -> Result<String, String> {
    if msg_text.starts_with('`') {
        if msg_text.starts_with("```") {
            let (_, msg_text) = msg_text.split_once('\n').ok_or("No lines")?;
            let (msg_text, _) = msg_text.rsplit_once('\n').ok_or("No lines")?;
            Ok(msg_text.to_string())
        } else {
            Ok(msg_text[1..msg_text.len() - 1].to_string())
        }
    } else {
        Ok(msg_text)
    }
}

async fn on_input(message: &Message, module: &AlfredModule) -> Result<(), Box<dyn Error>> {
    let msg_text = message.text.clone();
    if msg_text.is_empty() {
        warn!("Empty message text");
        return Ok(());
    }
    let msg_text = clean_json(msg_text)?;
    let ai_message: AIMessage = serde_json::from_str(&msg_text)?;
    if !ai_message.text.is_empty() {
        let (relay_topic, relay_msg) = message.reply(ai_message.text.clone(), message.message_type.clone())?;
        module.send(relay_topic.as_str(), &relay_msg).await?;
    }
    if !ai_message.command.is_empty() {
        let msg_text = ai_message.command;
        let split = msg_text.split(": ").collect::<Vec<&str>>();
        let relay_topic = &split[0][0..];
        let relay_msg_text = &split[1][..split[1].len()];
        let relay_msg = Message {
            text: relay_msg_text.to_string(),
            message_type: message.message_type.clone(),
            ..message.clone()
        };
        module.send(relay_topic, &relay_msg).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut module = AlfredModule::new(MODULE_NAME, env!("CARGO_PKG_VERSION")).await?;
    module.listen(INPUT_TOPIC).await.expect("Error on subscribe");
    loop {
        let (topic, message) = module.receive().await.expect("Error on getting new messages");
        match topic.as_str() {
            INPUT_TOPIC => on_input(&message, &module).await?,
            _ => {
                warn!("Unknown topic {topic}");
            }
        }
    }
}
