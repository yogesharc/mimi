use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::{
    models::Model,
    parser::{AgentEventItem, Usage},
};

pub struct Context<'a> {
    pub event_logs: Vec<AgentEventItem>,
    pub system_prompt: Option<String>,
    pub usage: Usage,
    pub compact_threshold_percentage: u64,
    pub model: Option<&'a Model>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            event_logs: Vec::new(),
            system_prompt: None,
            usage: Usage::default(),
            compact_threshold_percentage: 10,
            model: None,
        }
    }
}

impl Context<'_> {
    pub fn estimate_token_usage(text: &str) -> Result<u64> {
        u64::try_from(text.chars().count().div_ceil(4)).context("failed to calculate token usage")
    }

    pub fn build_system_prompt(&mut self) {
        let mut context: String = String::new();

        // let system_prompt_path = PathBuf::from("src/prompts/codex_opencode.txt");
        // let system_prompt = fs::read_to_string(system_prompt_path);
        let system_prompt: Result<String> = Ok(String::new());

        match system_prompt {
            Ok(v) => context.push_str(&v),
            Err(e) => eprintln!("could not find system prompt text: {e}"),
        }

        let agents_md_path = PathBuf::from("AGENTS.md");
        let agents_md = fs::read_to_string(agents_md_path);

        match agents_md {
            Ok(v) => context.push_str(&v),
            Err(e) => eprintln!("could not find agents md file: {e}"),
        }

        if context.is_empty() {
            self.system_prompt = None;
            return;
        }

        let system_tokens = Self::estimate_token_usage(&context).unwrap_or(0);

        self.usage.input_tokens += system_tokens;

        self.system_prompt = Some(context);

        ()
    }

    pub fn exceed_token_usage(&self, new_msg: Option<&str>) -> Result<bool> {
        let existing_usage = self.usage.total_tokens;
        let mut upcoming_usage = 0;

        if let Some(msg) = new_msg {
            upcoming_usage = Self::estimate_token_usage(msg)?;
        };

        let context_window;

        if let Some(model) = self.model {
            context_window = model.context_window
        } else {
            bail!("no model found");
        }

        let useable_context_window = self.compact_threshold_percentage * context_window / 100;

        // println!(
        //     "EXISTING USAGE: {} , UPCOMING: {}, WINDOW: {}",
        //     existing_usage, upcoming_usage, useable_context_window
        // );

        if existing_usage + upcoming_usage > useable_context_window {
            println!("HIT TOKEN LIMIT");
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::{all_models, get_model};

    #[test]
    fn token_limit_is_not_hit_below_compaction_threshold() {
        let models = all_models();
        let model = get_model("openai/gpt-5.6-sol", &models).unwrap();
        let mut context = Context::default();
        context.model = Some(model);
        let threshold = context.compact_threshold_percentage * model.context_window / 100;
        context.usage.total_tokens = threshold - 1;

        assert!(!context.exceed_token_usage(None).unwrap());
    }

    #[test]
    fn token_limit_is_hit_above_compaction_threshold() {
        let models = all_models();
        let model = get_model("openai/gpt-5.6-sol", &models).unwrap();
        let mut context = Context::default();
        context.model = Some(model);
        let threshold = context.compact_threshold_percentage * model.context_window / 100;
        context.usage.total_tokens = threshold;

        assert!(context.exceed_token_usage(Some("testing")).unwrap());
    }

    #[test]
    fn syssy() {
        let system_prompt_path = PathBuf::from("src/prompts/codex_opencode.txt");
        let system_prompt = fs::read_to_string(system_prompt_path).unwrap();

        assert_eq!(true, system_prompt.starts_with("You are a coding agent"))
    }
}
