use std::io::{self, Write};

use crate::{
    approval::{ApprovalDecision, ApprovalHandler, ApprovalRequest},
    context::Context,
    events::append_events,
    parser::AgentEventItem,
    runtime::RunMode,
    tools::file_search::Search,
};
use anyhow::{Context as _, Ok, Result};
use uuid::Uuid;

use crate::agent_loop;

const COMPACTION_PROMPT: &str = "Create a concise continuation summary of the conversation. Preserve the user's current goal, important decisions, relevant file paths and code details, tool results, errors, and unfinished work. Omit greetings, repetition, and obsolete discussion. Write only the summary needed for another assistant to continue the task without losing context.";

pub async fn run(context: &mut Context<'_>, search: &Search) -> Result<()> {
    println!("\n============================");
    println!("||     MIMI v0.1  ^_^     ||");
    println!("============================");
    println!("");

    let mut session_id: String = String::new();
    let mut approval_handler = InteractiveApprovalHandler;

    loop {
        let mut compaction = false;
        let mut user_msg_queue: Vec<AgentEventItem> = vec![];

        let input = ask_input();
        if input == "exit" {
            break;
        } else if input.is_empty() {
            println!("You need to type something");
            continue;
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

        // let _effort = EffortLevel::Medium;

        println!("============== ASSISTANT ==============");
        agent_loop::run(
            RunMode::Interactive,
            context,
            &session_id,
            &mut compaction,
            &mut user_msg_queue,
            &search,
            &mut approval_handler,
        )
        .await?;
    }
    Ok(())
}

struct InteractiveApprovalHandler;

impl ApprovalHandler for InteractiveApprovalHandler {
    fn request_approval(&mut self, request: &ApprovalRequest<'_>) -> Result<ApprovalDecision> {
        println!("\n========== APPROVAL REQUIRED ==========");
        println!("Tool: {}", request.tool_name);
        println!("=======================================");
        println!("Arguments: {}", request.arguments);
        println!("=======================================");
        loop {
            print!("Allow this tool call? [y/N]: ");
            io::stdout()
                .flush()
                .context("failed to flush approval prompt")?;

            let mut input = String::new();
            let bytes_read = io::stdin()
                .read_line(&mut input)
                .context("failed to read approval response")?;

            if bytes_read == 0 {
                return Ok(ApprovalDecision::Rejected);
            }

            match input.trim().to_ascii_lowercase().as_str() {
                "y" | "yes" => return Ok(ApprovalDecision::Approved),
                "n" | "no" | "" => return Ok(ApprovalDecision::Rejected),
                _ => println!("Reply yes or no."),
            }
        }
    }
}

fn ask_input() -> String {
    let _ = io::stdout().flush();

    let mut input = String::new();

    println!("\n=======================================");
    println!("Ask anything:");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}
