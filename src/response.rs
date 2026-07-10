use crate::parser::{AgentEventItem, EffortLevel, OpenRouterEvents};

use super::parser::ResponseRequest;
use futures::StreamExt;
use std::env;

static BASE_URL: &str = "https://openrouter.ai/api/v1/responses";

pub async fn get_response(
    model: String,
    input: &Vec<AgentEventItem>,
    effort: Option<&EffortLevel>,
) -> Result<Vec<OpenRouterEvents>, String> {
    dotenvy::dotenv().ok();

    let api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| "Must have OPENROUTER_API_KEY".to_string())?;

    let req_body = ResponseRequest::new(model, input, effort);

    let client = reqwest::Client::new();

    let response = client
        .post(BASE_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.map_err(|e| e.to_string())?;
        return Err(format!("OpenRouter request failed with {status}: {body}"));
    }

    let mut res = response.bytes_stream();

    let mut events: Vec<OpenRouterEvents> = vec![];
    let mut buffer = String::new();

    while let Some(item) = res.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        let chunk_text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_text);

        while let Some(newline_index) = buffer.find('\n') {
            let line = buffer[..newline_index].trim().to_string();
            buffer = buffer[newline_index + 1..].to_string();

            println!("{line}");

            if !line.starts_with("data:") {
                continue;
            }

            let data = line.trim_start_matches("data:").trim();

            if data == "[DONE]" {
                return Ok(events);
            }

            let event = match serde_json::from_str::<OpenRouterEvents>(data) {
                Ok(event) => event,
                Err(e) => {
                    eprintln!("failed to parse event: {e}");
                    eprintln!("failed item raw data: {data}");
                    continue;
                }
            };

            events.push(event);
        }
    }

    Ok(events)
}
