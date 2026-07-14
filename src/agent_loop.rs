use uuid::Uuid;

use anyhow::{Context as _, Result, bail};

use super::parser::AgentEventItem;
use super::response::get_response;
use std::{
    io::{self, Write},
    time::Instant,
};

use crate::{
    context::Context,
    events::append_events,
    models::{all_models, get_model},
    parser::{ContextManagement, EffortLevel, OpenRouterEvents},
    tools::{self, SystemTools},
};

const COMPACTION_PROMPT: &str = "Create a concise continuation summary of the conversation. Preserve the user's current goal, important decisions, relevant file paths and code details, tool results, errors, and unfinished work. Omit greetings, repetition, and obsolete discussion. Write only the summary needed for another assistant to continue the task without losing context.";

pub async fn run_loop() -> Result<()> {
    let initialize = Instant::now();
    let mut context = Context::default();
    context.build_system_prompt();

    let mut tmp_event_logs: Vec<AgentEventItem> = vec![];
    let mut user_msg_queue: Vec<AgentEventItem> = vec![];
    let mut session_id: String = String::new();

    let available_models = all_models();
    let default_model_str = String::from("openai/gpt-5.6-sol");
    let default_model = get_model(&default_model_str, &available_models)?;
    context.model = Some(default_model);

    let context_management = Some(vec![ContextManagement {
        compact_threshold: context.compact_threshold_percentage
            * context.model.unwrap().context_window
            / 100,
        ..Default::default()
    }]);

    let mut search = tools::file_search::Search::default();
    search.index_cwd()?;

    println!("PROGRAM INITIALIZED: {:?}", initialize.elapsed());

    'outer: loop {
        let mut compaction = false;
        let input = ask_input();

        if input == "exit" {
            break 'outer;
        }

        let user_msg = AgentEventItem::new_user_message(input.clone());
        let mut request_msg = user_msg.clone();

        let is_new_session = match session_id.is_empty() {
            true => {
                session_id = Uuid::now_v7().to_string();
                true
            }
            false => {
                let token_limit_hit = context
                    .exceed_token_usage(Some(&input))
                    .context("failed to check token usage")?;

                if token_limit_hit {
                    compaction = true;
                    user_msg_queue.push(user_msg.clone());

                    request_msg = AgentEventItem::new_user_message(COMPACTION_PROMPT.to_string());
                }
                false
            }
        };

        append_events(&session_id, &vec![user_msg], is_new_session)
            .await
            .with_context(|| format!("failed to persist user message for session {session_id}"))?;

        context.event_logs.push(request_msg);

        // let selected_model_str = String::from("openai/gpt-5.5");

        // let selected_model = get_model(&selected_model_str, &available_models)?;
        // context.model = Some(selected_model);

        let _effort = EffortLevel::Medium;

        let mut stop_agent: bool = true;
        let response_time = Instant::now();

        'agent_loop: loop {
            let mut token_limit_hit = false;

            if !stop_agent {
                token_limit_hit = context
                    .exceed_token_usage(None)
                    .context("failed to check token usage")?;

                if token_limit_hit {
                    compaction = true;
                    let summarize_msg =
                        AgentEventItem::new_user_message(COMPACTION_PROMPT.to_string());
                    context.event_logs.push(summarize_msg);
                }
            }

            if token_limit_hit {
                stop_agent = false;
            } else {
                stop_agent = true;
            }
            tmp_event_logs = Vec::new();

            let events = get_response(
                context.model.unwrap().full_identifier(),
                &context.event_logs,
                None,
                &context.system_prompt,
                &context_management,
            )
            .await?;

            for event in events {
                match event {
                    // OpenRouterEvents::ResponseCreated { response } => {}
                    OpenRouterEvents::ResponseOutputItemDone { item, .. } => {
                        match &item {
                            AgentEventItem::ToolCall {
                                id,
                                call_id,
                                name,
                                arguments,
                                ..
                            } => {
                                let Some(tool) = SystemTools::variant_from_name(name) else {
                                    bail!("unknown tool: {name}");
                                };

                                let search_struct = match &tool {
                                    SystemTools::SearchFiles | SystemTools::SearchContent => {
                                        Some(&search)
                                    }
                                    _ => None,
                                };

                                let output = SystemTools::execute(&tool, arguments, search_struct);

                                let output = match output {
                                    Ok(value) => value,
                                    Err(e) => {
                                        eprintln!("{e}");
                                        serde_json::json!({"error": e.to_string()})
                                    }
                                };

                                let output = serde_json::to_string(&output).with_context(|| {
                                    format!("failed to serialize output from {name}")
                                })?;

                                let tool_call_output = AgentEventItem::ToolCallOutput {
                                    id: format!("{}_output", id.clone()),
                                    call_id: call_id.clone(),
                                    output: output,
                                };

                                tmp_event_logs.push(item);
                                tmp_event_logs.push(tool_call_output);
                                stop_agent = false;
                            }
                            AgentEventItem::Reasoning { summary, .. } => {
                                if !summary.is_empty() {
                                    tmp_event_logs.push(item);
                                }
                            }
                            _ => tmp_event_logs.push(item),
                        };
                    }
                    OpenRouterEvents::ResponseCompleted { response } => {
                        if compaction {
                            let summary_tokens = match tmp_event_logs.last() {
                                Some(AgentEventItem::Message { content, .. }) => {
                                    content.iter().try_fold(0_u64, |total, content| {
                                        Context::estimate_token_usage(&content.text)
                                            .map(|tokens| total + tokens)
                                    })?
                                }
                                _ => 0,
                            };

                            context.usage.input_tokens = 0;
                            context.usage.output_tokens = summary_tokens;
                            context.usage.total_tokens = summary_tokens;

                            if let Some(new_usage) = response.usage {
                                context.usage.cost += new_usage.cost;
                            }
                        } else if let Some(new_usage) = response.usage {
                            context.usage.input_tokens = new_usage.input_tokens;
                            context.usage.output_tokens = new_usage.output_tokens;
                            context.usage.total_tokens = new_usage.total_tokens;
                            context.usage.cost += new_usage.cost;
                        }
                    }
                    _ => {}
                }
            }
            println!("");
            println!("tmp event_logs: {tmp_event_logs:?}");

            append_events(&session_id, &tmp_event_logs, false)
                .await
                .with_context(|| {
                    format!("failed to persist response events for session {session_id}")
                })?;

            if compaction {
                context.event_logs.clear();
                compaction = false;
            }
            context.event_logs.extend(tmp_event_logs);

            if !user_msg_queue.is_empty() {
                context.event_logs.extend(user_msg_queue.clone());
                user_msg_queue.clear();
                stop_agent = false
            }

            if stop_agent {
                println!("RESPONSE TIME: {:?}", response_time.elapsed());
                break 'agent_loop;
            }
        }
    }

    Ok(())
}

fn ask_input() -> String {
    let _ = io::stdout().flush();

    let mut input = String::new();
    println!("Ask anything:");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}
