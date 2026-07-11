// mod models;
mod agent_loop;
mod context;
mod events;
mod parser;
mod response;
mod tools;

#[tokio::main]
async fn main() -> Result<(), String> {
    agent_loop::run_loop().await?;

    Ok(())
}
