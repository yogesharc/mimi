use super::parser::ConversationItem;
use super::response::get_response;
use std::io::{self};

use crate::{parser::OpenRouterEvents, tools::SystemTools};

pub async fn run_loop() -> Result<(), String> {
    let mut history: Vec<ConversationItem> = Vec::new();

    'outer: loop {
        let input = ask_input();

        if input == "exit" {
            break 'outer;
        }

        let new_msg = ConversationItem::new_user_message(input);
        history.push(new_msg);

        let model = String::from("openai/gpt-5.1-codex-mini");

        let mut stop_agent: bool;

        'agent_loop: loop {
            stop_agent = true;

            let events = get_response(model.clone(), &history).await?;

            for event in events {
                match event {
                    OpenRouterEvents::ResponseOutputItemDone { item, .. } => {
                        // println!("{item:?}");

                        let output_item: Option<ConversationItem> = match &item {
                            ConversationItem::ToolCall {
                                id,
                                call_id,
                                name,
                                arguments,
                                ..
                            } => {
                                let Some(tool) = SystemTools::variant_from_name(name) else {
                                    return Err(format!("Unknown tool: {name}"));
                                };

                                let output = SystemTools::execute(&tool, arguments)?;
                                let output =
                                    serde_json::to_string(&output).map_err(|e| e.to_string())?;

                                Some(ConversationItem::TollCallOutput {
                                    id: format!("{}_output", id.clone()),
                                    call_id: call_id.clone(),
                                    output: output,
                                })
                            }
                            _ => None,
                        };

                        history.push(item);

                        if let Some(tool_call_output) = output_item {
                            history.push(tool_call_output);
                            stop_agent = false;
                        }
                    }
                    _ => {}
                }
            }
            println!("");
            println!("history: {history:?}");

            if stop_agent {
                break 'agent_loop;
            }
        }
    }

    Ok(())
}

fn ask_input() -> String {
    let mut input = String::new();
    println!("Ask anything:");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}
