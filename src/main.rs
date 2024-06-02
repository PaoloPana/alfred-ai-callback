use std::collections::LinkedList;
use alfred_rs::error::Error;
use alfred_rs::connection::{Publisher, Subscriber};
use alfred_rs::message::{Message, MessageType};
use alfred_rs::module::Module;
use alfred_rs::log::warn;
use alfred_rs::tokio;

const MODULE_NAME: &'static str = "ai_callback";
const INPUT_TOPIC: &'static str = "ai_callback";
const STT_REQUEST_TOPIC: &'static str = "audio_out";
const TTS_REQUEST_TOPIC: &'static str = "tts";
const AI_TOPIC: &'static str = "openai";

async fn on_input(message: &mut Message, module: &mut Module) -> Result<(), Error> {
    match message.message_type {
        MessageType::TEXT => {
            // TODO: manage errors
            module.publish(AI_TOPIC.to_string(), message).await.expect("Error on publish");
        }
        MessageType::AUDIO => {
            let topic = message.response_topics.front().cloned().unwrap();
            message.response_topics = LinkedList::from(
                [
                    AI_TOPIC.to_string(),
                //    TTS_REQUEST_TOPIC.to_string(),
                    topic
                ]);
            module.publish(STT_REQUEST_TOPIC.to_string(), message).await.expect("Error on publish");
        }
        MessageType::PHOTO => {
            warn!("Unable to analyse {:?} message type", MessageType::PHOTO);
        }
        MessageType::UNKNOWN => {
            warn!("Unable to analyse {:?} message type", MessageType::UNKNOWN);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let mut module = Module::new(MODULE_NAME.to_string()).await?;
    module.subscribe(INPUT_TOPIC.to_string()).await.expect("Error on subscribe");
    loop {
        let (topic, mut message) = module.get_message().await.expect("Error on getting new messages");
        match topic.as_str() {
            INPUT_TOPIC => on_input(&mut message, &mut module).await?,
            _ => {
                warn!("Unknown topic {topic}");
            }
        }
    }
}
