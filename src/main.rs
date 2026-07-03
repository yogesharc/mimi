mod models;
mod response;
use response::get_response;

#[tokio::main]
async fn main() {
    let model = String::from("openai/gpt-5.1-codex-mini");
    let input = String::from("Say hello world");

    match get_response(model, input).await {
        Ok(response) => println!("{response}"),
        Err(error) => eprintln!("{error}"),
    }
}
