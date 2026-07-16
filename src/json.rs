use std::io::{self, Write};

use crate::{
    context::Context,
    events::append_events,
    parser::{AgentEventItem, EffortLevel},
    runtime::{self, RunMode},
    tools::file_search::Search,
};
use anyhow::{Context as _, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::agent_loop;

const COMPACTION_PROMPT: &str = "Create a concise continuation summary of the conversation. Preserve the user's current goal, important decisions, relevant file paths and code details, tool results, errors, and unfinished work. Omit greetings, repetition, and obsolete discussion. Write only the summary needed for another assistant to continue the task without losing context.";

pub async fn run(context: &mut Context<'_>, search: &Search) -> Result<()> {
    let mut session_id: String = String::new();

    loop {
        let mut compaction = false;
        let mut user_msg_queue: Vec<AgentEventItem> = vec![];

        let input = ask_input();

        if input.is_empty() {
            println!(
                "{}",
                serde_json::json!({"type": "error", "error": "You need to type something"})
            );
            continue;
        }

        let parsed_input = serde_json::from_str::<JsonInput>(&input);

        if parsed_input.is_err() {
            println!(
                "{}",
                serde_json::json!({"type": "error", "error": "failed to parse input"})
            );
            continue;
        };

        let parsed_input = parsed_input?;

        if let InputType::Exit = parsed_input.r#type {
            println!("{}", serde_json::json!({"type": "exit", "success": "ok"}));
            break;
        };

        let mut prompt = String::new();

        if let InputType::Prompt = parsed_input.r#type {
            prompt = match parsed_input.text {
                Some(v) => v,
                None => {
                    println!(
                        "{}",
                        serde_json::json!({"type": "error", "error": "prompt is missing"})
                    );
                    continue;
                }
            };
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
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Prompt,
    Approval,
    Exit,
}
#[derive(Deserialize)]
pub struct JsonInput {
    pub r#type: InputType,
    pub text: Option<String>,
    pub call_id: Option<String>,
}

fn ask_input() -> String {
    let _ = io::stdout().flush();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}
