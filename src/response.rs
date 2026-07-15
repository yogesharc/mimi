use crate::parser::{AgentEventItem, EffortLevel, OpenRouterEvents};

use super::parser::ResponseRequest;
use anyhow::{Context, Result, bail};
use futures::StreamExt;
use std::env;
use tokio::sync::mpsc;

static BASE_URL: &str = "https://openrouter.ai/api/v1/responses";

pub async fn get_response(
    model: String,
    input: &Vec<AgentEventItem>,
    effort: Option<&EffortLevel>,
    system_prompt: &Option<String>,
    // context_management: &Option<Vec<ContextManagement>>,
) -> Result<mpsc::Receiver<Result<OpenRouterEvents>>> {
    dotenvy::dotenv().ok();

    let api_key = env::var("OPENROUTER_API_KEY")
        .context("OPENROUTER_API_KEY environment variable is required")?;

    let req_body = ResponseRequest::new(model, input, effort, system_prompt);

    let client = reqwest::Client::new();

    let response = client
        .post(BASE_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await
        .context("failed to send request to OpenRouter")?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .context("failed to read OpenRouter error response")?;
        bail!("OpenRouter request failed with {status}: {body}");
    }

    let (tx, rx) = mpsc::channel(128);

    tokio::spawn(async move {
        let mut event_tx = tx;
        let result = parse_response_events(response, &mut event_tx).await;

        if let Err(error) = result {
            let _ = event_tx.send(Err(error)).await;
        }
    });

    Ok(rx)
}

async fn parse_response_events(
    response: reqwest::Response,
    event_tx: &mut mpsc::Sender<Result<OpenRouterEvents>>,
) -> Result<()> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item.context("failed to read OpenRouter response stream")?;
        let chunk_text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_text);

        while let Some(newline_index) = buffer.find('\n') {
            let line = buffer[..newline_index].trim().to_string();
            buffer = buffer[newline_index + 1..].to_string();

            // println!("{line}");

            if !line.starts_with("data:") {
                continue;
            }

            let data = line.trim_start_matches("data:").trim();

            if data == "[DONE]" {
                return Ok(());
            }

            let event: OpenRouterEvents = serde_json::from_str(data)?;

            if event_tx.send(Ok(event)).await.is_err() {
                return Ok(());
            };
        }
    }

    Ok(())
}
