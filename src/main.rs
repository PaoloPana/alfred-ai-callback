use std::collections::HashMap;
use std::error::Error;
use alfred_core::AlfredModule;
use alfred_core::message::Message;
use alfred_core::log::warn;
use alfred_core::tokio;
use serde::{Deserialize, Serialize};

const MODULE_NAME: &str = "ai_callback";
const INPUT_TOPIC: &str = "ai_callback";
const REPLY_TOPIC: &str = "ai_callback.reply";

#[derive(Serialize, Deserialize)]
struct AIMessageRequest {
    pub text: String,
    pub other_info: HashMap<String, String>
}

#[derive(Deserialize)]
struct AIMessageResponse {
    pub text: String,
    pub commands: Vec<String>,
    pub request: Option<String>,
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

fn get_message_from_command(command: &str, message: &Message) -> (String, Message) {
    let msg_text = command;
    let split = msg_text.split(": ").collect::<Vec<&str>>();
    let relay_topic = &split[0][0..];
    let relay_msg_text = &split[1][..split[1].len()];
    let relay_msg = Message {
        text: relay_msg_text.to_string(),
        message_type: message.message_type.clone(),
        ..message.clone()
    };
    (relay_topic.to_string(), relay_msg)
}

async fn on_input(message: &Message, module: &AlfredModule) -> Result<(), Box<dyn Error>> {
    let msg_text = message.text.clone();
    if msg_text.is_empty() {
        warn!("Empty message text");
        return Ok(());
    }
    let msg_text = clean_json(msg_text)?;
    let ai_message: AIMessageResponse = serde_json::from_str(&msg_text)?;
    if ai_message.request.is_some() {
        let ai_request = ai_message.request.unwrap_or_default();
        if !ai_request.is_empty() {
            let (relay_topic, mut relay_msg) = get_message_from_command(ai_request.as_str(), message);
            relay_msg.response_topics.push_front(REPLY_TOPIC.to_string());
            relay_msg.params.insert("ai_request".to_string(), ai_request);
            module.send(relay_topic.as_str(), &relay_msg).await?;
        }
    }
    if !ai_message.text.is_empty() {
        let (relay_topic, relay_msg) = message.reply(ai_message.text.clone(), message.message_type.clone())?;
        module.send(relay_topic.as_str(), &relay_msg).await?;
    }
    if !ai_message.commands.is_empty() {
        for command in ai_message.commands {
            let (relay_topic, relay_msg) = get_message_from_command(command.as_str(), message);
            module.send(relay_topic.as_str(), &relay_msg).await?;
        }
    }
    Ok(())
}

async fn on_reply(message: &Message, module: &AlfredModule) -> Result<(), Box<dyn Error>> {
    let ai_request = message.params.get("ai_request").ok_or("No ai_request")?;
    let ai_response = message.text.clone();
    let user_request = message.params.get("request").ok_or("No user request")?;
    let ai_message = AIMessageRequest {
        text: user_request.to_string(),
        other_info: HashMap::from([(ai_request.to_string(), ai_response)]),
    };
    let mut response_topics = message.response_topics.clone();
    response_topics.push_front(INPUT_TOPIC.to_string());
    let message = Message {
        text: serde_json::to_string(&ai_message)?,
        response_topics,
        ..message.clone()
    };
    module.send("chat", &message).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut module = AlfredModule::new(MODULE_NAME, env!("CARGO_PKG_VERSION")).await?;
    module.listen(INPUT_TOPIC).await.expect("Error on subscribe");
    module.listen(REPLY_TOPIC).await.expect("Error on subscribe");
    loop {
        let (topic, message) = module.receive().await.expect("Error on getting new messages");
        match topic.as_str() {
            INPUT_TOPIC => on_input(&message, &module).await?,
            REPLY_TOPIC => on_reply(&message, &module).await?,
            _ => {
                warn!("Unknown topic {topic}");
            }
        }
    }
}
