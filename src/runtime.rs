use crate::{
    command_line_parser::{Commands, parse_cmd},
    context::Context,
    interactive, json,
    models::{all_models, get_model},
    parser::EffortLevel,
    tools,
};
use anyhow::{Ok, Result};

pub enum RunMode {
    Interactive,
    JsonStream,
}

pub async fn runtime() -> Result<()> {
    let cli = parse_cmd()?;

    let model_arg = cli.model;
    let effort_arg = cli.effort;
    let mode_arg = cli.command;

    let available_models = all_models();
    let selected_model = get_model(&model_arg, &available_models)?;
    let mut context = Context::new(selected_model);
    context.build_system_prompt();

    let mut effort: Option<EffortLevel> = None;

    if let Some(eff) = effort_arg {
        effort = match eff.as_str() {
            "minimal" => Some(EffortLevel::Minimal),
            "low" => Some(EffortLevel::Low),
            "medium" => Some(EffortLevel::Medium),
            "high" => Some(EffortLevel::High),
            _ => None,
        }
    }

    context.effort = effort;

    let mut search = tools::file_search::Search::default();
    search.index_cwd()?;

    match mode_arg {
        Some(Commands::Json) => json::run(&mut context, &search).await?,
        None => interactive::run(&mut context, &search).await?,
    }

    Ok(())
}
