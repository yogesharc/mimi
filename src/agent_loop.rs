use super::parser::ConversationItem;
use super::response::get_response;
use std::io::{self};

use crate::{
    parser::{EffortLevel, OpenRouterEvents},
    tools::{self, SystemTools},
};

pub async fn run_loop() -> Result<(), String> {
    let mut history: Vec<ConversationItem> = Vec::new();
    let mut search = tools::file_search::Search::default();
    search.index_cwd()?;

    'outer: loop {
        let input = ask_input();

        if input == "exit" {
            break 'outer;
        }

        let new_msg = ConversationItem::new_user_message(input);
        history.push(new_msg);

        let model = String::from("openai/gpt-5.5");
        let effort = EffortLevel::Medium;

        let mut stop_agent: bool;

        'agent_loop: loop {
            stop_agent = true;

            let events = get_response(model.clone(), &history, Some(&effort)).await?;

            for event in events {
                match event {
                    OpenRouterEvents::ResponseOutputItemDone { item, .. } => {
                        match &item {
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

                                let search_struct = match &tool {
                                    SystemTools::SearchFiles => Some(&search),
                                    _ => None,
                                };

                                let output = SystemTools::execute(&tool, arguments, search_struct);

                                let output = match output {
                                    Ok(value) => value,
                                    Err(e) => {
                                        eprintln!("{e}");
                                        continue;
                                    }
                                };

                                let output =
                                    serde_json::to_string(&output).map_err(|e| e.to_string())?;

                                let tool_call_output = ConversationItem::TollCallOutput {
                                    id: format!("{}_output", id.clone()),
                                    call_id: call_id.clone(),
                                    output: output,
                                };

                                history.push(item);
                                history.push(tool_call_output);
                                stop_agent = false;
                            }
                            ConversationItem::Reasoning { summary, .. } => {
                                if !summary.is_empty() {
                                    history.push(item);
                                }
                            }
                            _ => history.push(item),
                        };
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
