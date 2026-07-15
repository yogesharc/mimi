// mod models;
use crate::runtime::runtime;

mod agent_loop;
mod context;
mod events;
mod interactive;
mod models;
mod parser;
mod response;
mod runtime;
mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    runtime().await?;

    Ok(())
}
