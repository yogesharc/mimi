// mod models;
mod parser;
mod response;
use parser::ConversationItem;
use response::get_response;

use crate::parser::OpenRouterEvents;

#[tokio::main]
async fn main() -> Result<(), String> {
    let model = String::from("openai/gpt-5.1-codex-mini");
    let input = String::from("Say hello world");

    let mut history: Vec<ConversationItem> = Vec::new();

    let events = get_response(model, input).await?;

    for event in events {
        match event {
            OpenRouterEvents::ResponseOutputTextDelta { delta, .. } => {
                print!("{delta}");
            }
            OpenRouterEvents::ResponseOutputItemDone { item, .. } => {
                history.push(item);
            }
            _ => {}
        }
    }

    println!();
    println!("{history:?}");

    Ok(())
}
