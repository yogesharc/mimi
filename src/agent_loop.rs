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

pub async fn run_loop() -> Result<()> {
    let initialize = Instant::now();
    let mut context = Context::default();
    context.build_system_prompt();

    let mut tmp_event_logs: Vec<AgentEventItem> = vec![];
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
        let input = ask_input();

        if input == "exit" {
            break 'outer;
        }

        let new_msg = AgentEventItem::new_user_message(input.clone());

        let is_new_session = match session_id.is_empty() {
            true => {
                session_id = Uuid::now_v7().to_string();
                true
            }
            false => false,
        };

        append_events(&session_id, &vec![new_msg.clone()], is_new_session)
            .await
            .with_context(|| format!("failed to persist user message for session {session_id}"))?;

        // let selected_model_str = String::from("openai/gpt-5.5");

        // let selected_model = get_model(&selected_model_str, &available_models)?;
        // context.model = Some(selected_model);

        let _effort = EffortLevel::Medium;

        context
            .check_token_usage(input)
            .context("failed to check token usage")?;

        context.event_logs.push(new_msg);

        let mut stop_agent: bool;
        let response_time = Instant::now();

        'agent_loop: loop {
            stop_agent = true;
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
                                        continue;
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
                        if let Some(new_usage) = response.usage {
                            context.usage.input_tokens += new_usage.input_tokens;
                            context.usage.output_tokens += new_usage.output_tokens;
                            context.usage.total_tokens += new_usage.total_tokens;
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

            context.event_logs.extend(tmp_event_logs);

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
