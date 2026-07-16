use std::io::{self, Write};

use crate::{
    context::Context, events::append_events, parser::AgentEventItem, runtime::RunMode,
    tools::file_search::Search,
};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent_loop;

const COMPACTION_PROMPT: &str = "Create a concise continuation summary of the conversation. Preserve the user's current goal, important decisions, relevant file paths and code details, tool results, errors, and unfinished work. Omit greetings, repetition, and obsolete discussion. Write only the summary needed for another assistant to continue the task without losing context.";

pub async fn run(context: &mut Context<'_>, search: &Search) -> Result<()> {
    let mut session_id: String = String::new();

    loop {
        let mut compaction = false;
        let mut user_msg_queue: Vec<AgentEventItem> = vec![];

        let Some(input) = ask_input()? else {
            break;
        };

        if input.is_empty() {
            print_json_line(
                &serde_json::json!({"type": "error", "error": "You need to type something"}),
            )?;
            continue;
        }

        let command = match serde_json::from_str::<JsonInput>(&input) {
            Ok(command) => command,
            Err(error) => {
                print_json_line(&serde_json::json!({
                    "type": "error",
                    "error": "failed to parse input",
                    "details": error.to_string()
                }))?;
                continue;
            }
        };

        let prompt = match command {
            JsonInput::Prompt { text } => text,
            JsonInput::Approval { call_id, approved } => {
                print_json_line(&serde_json::json!({
                    "type": "error",
                    "error": "approval commands are not implemented",
                    "call_id": call_id,
                    "approved": approved
                }))?;
                continue;
            }
            JsonInput::Exit => {
                print_json_line(&serde_json::json!({"type": "exit", "success": "ok"}))?;
                break;
            }
        };

        let user_msg = AgentEventItem::new_user_message(prompt.clone());
        let mut request_msg = user_msg.clone();

        let is_new_session = match session_id.is_empty() {
            true => {
                session_id = Uuid::now_v7().to_string();
                true
            }
            false => {
                let token_limit_hit = context
                    .exceed_token_usage(Some(&prompt))
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

        agent_loop::run(
            RunMode::JsonStream,
            context,
            &session_id,
            &mut compaction,
            &mut user_msg_queue,
            &search,
        )
        .await?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonInput {
    Prompt { text: String },
    Approval { call_id: String, approved: bool },
    Exit,
}

fn ask_input() -> Result<Option<String>> {
    io::stdout()
        .flush()
        .context("failed to flush JSONL output")?;

    let mut input = String::new();

    let bytes_read = io::stdin()
        .read_line(&mut input)
        .context("failed to read JSONL input")?;

    if bytes_read == 0 {
        return Ok(None);
    }

    Ok(Some(input.trim().to_string()))
}

fn print_json_line(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string(value).context("failed to serialize JSONL output")?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{json}").context("failed to write JSONL output")?;
    stdout.flush().context("failed to flush JSONL output")?;
    Ok(())
}
