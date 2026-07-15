use crate::{
    context::Context,
    interactive,
    models::{all_models, get_model},
    tools,
};
use anyhow::{Ok, Result};

pub enum RunMode {
    Interactive,
    JsonStream,
}

pub async fn runtime() -> Result<()> {
    let mut context = Context::default();
    context.build_system_prompt();

    let available_models = all_models();
    let default_model_str = String::from("openai/gpt-5.6-sol");
    let default_model = get_model(&default_model_str, &available_models)?;
    context.model = Some(default_model);

    let mut search = tools::file_search::Search::default();
    let _ = search.index_cwd();

    interactive::run(&mut context, &search).await?;

    Ok(())
}
