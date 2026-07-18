use std::fs;

use anyhow::{Context as _, Result};

use crate::{
    models::Model,
    parser::{AgentEventItem, EffortLevel, Usage},
};

pub struct Context<'a> {
    pub event_logs: Vec<AgentEventItem>,
    pub system_prompt: Option<String>,
    pub usage: Usage,
    pub compact_threshold_percentage: u64,
    pub model: &'a Model,
    pub effort: Option<EffortLevel>,
}

const SYSTEM_PROMPT: &str = include_str!("prompts/system_prompt.md");

impl<'a> Context<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self {
            event_logs: Vec::new(),
            system_prompt: None,
            usage: Usage::default(),
            compact_threshold_percentage: 10,
            model,
            effort: None,
        }
    }

    pub fn estimate_token_usage(text: &str) -> Result<u64> {
        u64::try_from(text.chars().count().div_ceil(4)).context("failed to calculate token usage")
    }

    pub fn build_system_prompt(&mut self) {
        let mut context = SYSTEM_PROMPT.to_string();

        if let Ok(v) = fs::read_to_string("AGENTS.md") {
            context.push('\n');
            context.push_str(&v);
        }

        let system_tokens = Self::estimate_token_usage(&context).unwrap_or(0);

        self.usage.input_tokens += system_tokens;

        self.system_prompt = Some(context);
    }

    pub fn exceed_token_usage(&self, new_msg: Option<&str>) -> Result<bool> {
        let existing_usage = self.usage.total_tokens;
        let mut upcoming_usage = 0;

        if let Some(msg) = new_msg {
            upcoming_usage = Self::estimate_token_usage(msg)?;
        };

        let context_window = self.model.context_window;

        let useable_context_window = self.compact_threshold_percentage * context_window / 100;

        // println!(
        //     "EXISTING USAGE: {} , UPCOMING: {}, WINDOW: {}",
        //     existing_usage, upcoming_usage, useable_context_window
        // );

        if existing_usage + upcoming_usage > useable_context_window {
            eprintln!("hit usable usage token limit");
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
        let model = get_model(&Some("openai/gpt-5.6-sol".to_string()), &models).unwrap();
        let mut context = Context::new(model);
        let threshold = context.compact_threshold_percentage * model.context_window / 100;
        context.usage.total_tokens = threshold - 1;

        assert!(!context.exceed_token_usage(None).unwrap());
    }

    #[test]
    fn token_limit_is_hit_above_compaction_threshold() {
        let models = all_models();
        let model = get_model(&Some("openai/gpt-5.6-sol".to_string()), &models).unwrap();
        let mut context = Context::new(model);
        let threshold = context.compact_threshold_percentage * model.context_window / 100;
        context.usage.total_tokens = threshold;

        assert!(context.exceed_token_usage(Some("testing")).unwrap());
    }

    #[test]
    fn system_prompt_is_embedded_in_the_binary() {
        let models = all_models();
        let model = get_model(&Some("openai/gpt-5.6-sol".to_string()), &models).unwrap();
        let mut context = Context::new(model);

        context.build_system_prompt();

        assert!(
            context
                .system_prompt
                .as_deref()
                .is_some_and(|prompt| prompt.starts_with("You are a mimi"))
        );
    }
}
