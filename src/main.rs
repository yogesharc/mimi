// mod models;
mod agent_loop;
mod context;
mod events;
mod models;
mod parser;
mod response;
mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    agent_loop::run_loop().await?;

    Ok(())
}
