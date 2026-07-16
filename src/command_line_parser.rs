use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long)]
    pub model: Option<String>,

    #[arg(short, long)]
    pub effort: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Json,
}

pub fn parse_cmd() -> Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}
