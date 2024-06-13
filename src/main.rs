use std::collections::LinkedList;
use alfred_rs::connection::{Receiver, Sender};
use alfred_rs::error::Error;
use alfred_rs::interface_module::InterfaceModule;
use alfred_rs::message::{Message, MessageType};
use alfred_rs::log::warn;
use alfred_rs::pubsub_connection::REQUEST_TOPIC;
use alfred_rs::tokio;

const MODULE_NAME: &'static str = "ai_callback";
const INPUT_TOPIC: &'static str = "ai_callback";
const STT_REQUEST_TOPIC: &'static str = "stt";
const TTS_REQUEST_TOPIC: &'static str = "tts";
const AI_TOPIC: &'static str = "openai";

async fn on_input(message: &mut Message, module: &mut InterfaceModule) -> Result<(), Error> {
    match message.message_type {
        MessageType::TEXT => {
            // TODO: manage errors
            let mut ai_message = message.clone();
            ai_message.request_topic = "openai".to_string();
            module.send(REQUEST_TOPIC.to_string(), &ai_message).await.expect("Error on publish");
        }
        MessageType::AUDIO => {
            let topic = message.response_topics.front().cloned().unwrap();
            message.response_topics = LinkedList::from(
                [
                    // TODO: setup in config.toml file
                    AI_TOPIC.to_string(),
                    TTS_REQUEST_TOPIC.to_string(),
                    topic
                    //"audio_out".to_string()
                ]);
            module.send(STT_REQUEST_TOPIC.to_string(), message).await.expect("Error on publish");
        }
        _ => {
            warn!("Unable to analyse {:?} message type", message.message_type);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let mut module = InterfaceModule::new(MODULE_NAME.to_string()).await?;
    module.listen(INPUT_TOPIC.to_string()).await.expect("Error on subscribe");
    loop {
        let (topic, mut message) = module.receive().await.expect("Error on getting new messages");
        match topic.as_str() {
            INPUT_TOPIC => on_input(&mut message, &mut module).await?,
            _ => {
                warn!("Unknown topic {topic}");
            }
        }
    }
}
