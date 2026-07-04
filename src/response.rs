use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct UserMessage {
    model: String,
    input: Vec<StructuredInput>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct StructuredInput {
    r#type: String,
    role: Role,
    content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    r#type: String,
    text: String,
    annontations: Option<Vec<String>>,
}

impl UserMessage {
    fn new(model: String, input: String) -> Self {
        let content = Content {
            r#type: "input_text".to_string(),
            text: input,
            annontations: None,
        };

        let input = StructuredInput {
            r#type: "message".to_string(),
            role: Role::User,
            content: vec![content],
        };

        UserMessage {
            model,
            input: vec![input],
            stream: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResponseStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseCompleted {
    response: Response,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    id: String,
    output: Vec<Output>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Output {
    r#type: String,
    id: String,
    status: ResponseStatus,
    role: Option<Role>,
    content: Option<Vec<Content>>,
    summary: Option<Vec<String>>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Usage {
    input_tokens: u16,
    output_tokens: u16,
    total_tokens: u16,
    cost: f64,
}

static BASE_URL: &str = "https://openrouter.ai/api/v1/responses";

pub async fn get_response(model: String, input: String) -> Result<(), String> {
    dotenvy::dotenv().ok();

    let api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| "Must have OPENROUTER_API_KEY".to_string())?;

    let req_body = UserMessage::new(model, input);

    let client = reqwest::Client::new();

    let mut res = client
        .post(BASE_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes_stream();

    while let Some(item) = res.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        let chunk_text = String::from_utf8_lossy(&chunk);

        if !chunk_text.trim().starts_with("data") {
            continue;
        } else if chunk_text.contains("[DONE]") {
            break;
        }

        let data = &chunk_text[6..];
        // println!("{data}");

        let parsed_data: Result<ResponseCompleted, serde_json::Error> = serde_json::from_str(data);

        match &parsed_data {
            Ok(v) => {
                let json = serde_json::to_string(&v).map_err(|e| e.to_string())?;
                println!("{}", json)
            }
            Err(e) => eprintln!("{}", e),
        };
    }

    // if !status.is_success() {
    //     return Err(format!("OpenRouter returned {status}: {body}"));
    // }

    Ok(())
}
