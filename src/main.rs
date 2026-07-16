// mod models;
use crate::runtime::runtime;

mod agent_loop;
mod command_line_parser;
mod context;
mod events;
mod interactive;
mod json;
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
