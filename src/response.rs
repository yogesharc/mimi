use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
struct RequestBody {
    model: String,
    input: String,
}

static BASE_URL: &str = "https://openrouter.ai/api/v1/responses";

pub async fn get_response(model: String, input: String) -> Result<String, String> {
    dotenvy::dotenv().ok();

    let api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| "Must have OPENROUTER_API_KEY".to_string())?;

    let client = reqwest::Client::new();
    let req_body = RequestBody { model, input };

    let res = client
        .post(BASE_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let body = res.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("OpenRouter returned {status}: {body}"));
    }

    Ok(body)
}
