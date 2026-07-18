use anyhow::{Context as _, Result, bail};
use serde::Serialize;

use super::parser::AgentEventItem;
use super::response::get_response;
use std::io::{self, Write};

use crate::{
    approval::{ApprovalDecision, ApprovalHandler, ApprovalRequest},
    context::Context,
    events::append_events,
    parser::OpenRouterEvents,
    runtime::RunMode,
    tools::{SystemTools, file_search::Search},
};

const COMPACTION_PROMPT: &str = "Create a concise continuation summary of the conversation. Preserve the user's current goal, important decisions, relevant file paths and code details, tool results, errors, and unfinished work. Omit greetings, repetition, and obsolete discussion. Write only the summary needed for another assistant to continue the task without losing context.";

pub async fn run<A: ApprovalHandler>(
    mode: RunMode,
    context: &mut Context<'_>,
    session_id: &str,
    compaction: &mut bool,
    user_msg_queue: &mut Vec<AgentEventItem>,
    search: &Search,
    approval_handler: &mut A,
) -> Result<()> {
    let mut tmp_event_logs: Vec<AgentEventItem> = vec![];

    let mut stop_agent: bool = true;

    loop {
        let mut token_limit_hit = false;

        if !stop_agent {
            token_limit_hit = context
                .exceed_token_usage(None)
                .context("failed to check token usage")?;

            if token_limit_hit {
                *compaction = true;
                let summarize_msg = AgentEventItem::new_user_message(COMPACTION_PROMPT.to_string());
                context.event_logs.push(summarize_msg);
            }
        }

        if token_limit_hit {
            stop_agent = false;
        } else {
            stop_agent = true;
        }
        tmp_event_logs = Vec::new();

        let mut events = get_response(
            context.model.full_identifier(),
            &context.event_logs,
            context.effort.as_ref(),
            &context.system_prompt,
        )
        .await?;

        while let Some(event) = events.recv().await {
            let event = event?;

            if let RunMode::JsonStream = mode {
                print_json_line(&event)?;
            }

            match event {
                OpenRouterEvents::ResponseReasoningSummartPartAdded { .. } => {
                    if let RunMode::Interactive = mode {
                        println!("=======================================");
                        println!("Reasoning:");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseReasoningSummaryTextDelta { delta, .. } => {
                    if let RunMode::Interactive = mode {
                        print!("{delta}");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseContentPartAdded { .. } => {
                    if let RunMode::Interactive = mode {
                        println!("");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseOutputTextDelta { delta, .. } => {
                    if let RunMode::Interactive = mode {
                        print!("{delta}");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseContentPartDone { .. } => {
                    if let RunMode::Interactive = mode {
                        println!("");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseFunctionCallArgumentsDelta { .. } => {
                    if let RunMode::Interactive = mode {
                        print!(".");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseFunctionCallArgumentsDone { .. } => {
                    if let RunMode::Interactive = mode {
                        println!("");
                        let _ = io::stdout().flush();
                    }
                }
                OpenRouterEvents::ResponseOutputItemDone { item, .. } => {
                    match &item {
                        AgentEventItem::Reasoning { summary, .. } => {
                            if !summary.is_empty() {
                                tmp_event_logs.push(item);
                            }
                        }
                        _ => tmp_event_logs.push(item),
                    };
                }
                OpenRouterEvents::ResponseCompleted { response } => {
                    if *compaction {
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

        // println!("tmp event_logs: {tmp_event_logs:?}");

        let mut tool_call_outputs = Vec::new();

        for event in &tmp_event_logs {
            if let AgentEventItem::ToolCall { .. } = event {
                let output =
                    execute_tool_call(event, &search, session_id, approval_handler).await?;

                if let RunMode::JsonStream = mode {
                    print_json_line(&output)?;
                }

                tool_call_outputs.push(output);
            }
        }

        if !tool_call_outputs.is_empty() {
            tmp_event_logs.extend(tool_call_outputs);
            stop_agent = false;
        }

        append_events(session_id, &tmp_event_logs, false)
            .await
            .with_context(|| {
                format!("failed to persist response events for session {session_id}")
            })?;

        if *compaction {
            context.event_logs.clear();
            *compaction = false;
        }
        context.event_logs.extend(tmp_event_logs);

        if !user_msg_queue.is_empty() {
            context.event_logs.extend(user_msg_queue.clone());
            user_msg_queue.clear();
            stop_agent = false
        }

        if stop_agent {
            break;
        }
    }

    Ok(())
}

fn print_json_line(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string(value).context("failed to serialize JSONL output")?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{json}").context("failed to write JSONL output")?;
    stdout.flush().context("failed to flush JSONL output")?;
    Ok(())
}

async fn execute_tool_call<A: ApprovalHandler>(
    item: &AgentEventItem,
    search: &Search,
    session_id: &str,
    approval_handler: &mut A,
) -> Result<AgentEventItem> {
    if let AgentEventItem::ToolCall {
        id,
        call_id,
        name,
        arguments,
        ..
    } = item
    {
        let Some(tool) = SystemTools::variant_from_name(name) else {
            bail!("unknown tool: {name}");
        };

        let search_struct = match &tool {
            SystemTools::SearchFiles | SystemTools::SearchContent => Some(search),
            _ => None,
        };

        let session = match &tool {
            SystemTools::CreateTodoList => Some(session_id),
            _ => None,
        };

        let approved = if tool.requires_approval() {
            let request = ApprovalRequest {
                call_id,
                tool_name: name,
                arguments,
            };

            matches!(
                approval_handler.request_approval(&request)?,
                ApprovalDecision::Approved
            )
        } else {
            true
        };

        let output = if approved {
            SystemTools::execute(&tool, arguments, search_struct, session).await
        } else {
            Ok(serde_json::json!({
                "status": "tool call rejected by user"
            }))
        };

        let output = match output {
            Ok(value) => value,
            Err(e) => {
                // eprintln!("{e}");
                serde_json::json!({"error": e.to_string()})
            }
        };

        let output = serde_json::to_string(&output)
            .with_context(|| format!("failed to serialize output from {name}"))?;

        let tool_call_output = AgentEventItem::ToolCallOutput {
            id: format!("{}_output", id.clone()),
            call_id: call_id.clone(),
            output: output,
        };

        Ok(tool_call_output)
    } else {
        bail!("not a tool call to execute")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{approval::ApprovalDecision, parser::ResponseStatus};

    struct RejectApproval;

    impl ApprovalHandler for RejectApproval {
        fn request_approval(&mut self, _request: &ApprovalRequest<'_>) -> Result<ApprovalDecision> {
            Ok(ApprovalDecision::Rejected)
        }
    }

    #[tokio::test]
    async fn rejected_tool_call_returns_output_without_executing() {
        let item = AgentEventItem::ToolCall {
            id: "item_123".to_string(),
            status: ResponseStatus::Completed,
            call_id: "call_123".to_string(),
            name: "bash".to_string(),
            arguments: r#"{"command":"should-not-run"}"#.to_string(),
        };
        let search = Search::default();
        let mut approval_handler = RejectApproval;

        let output = execute_tool_call(&item, &search, "session_123", &mut approval_handler)
            .await
            .unwrap();

        match output {
            AgentEventItem::ToolCallOutput {
                call_id, output, ..
            } => {
                assert_eq!(call_id, "call_123");
                let output: serde_json::Value = serde_json::from_str(&output).unwrap();
                assert_eq!(output["status"], "tool call rejected by user");
            }
            _ => panic!("expected tool call output"),
        }
    }
}
