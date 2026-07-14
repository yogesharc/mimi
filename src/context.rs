use std::{fs, path::PathBuf};

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
    pub fn build_system_prompt(&mut self) {
        let mut context: String = String::new();

        // let system_prompt_path = PathBuf::from("src/prompts/codex_opencode.txt");
        // let system_prompt = fs::read_to_string(system_prompt_path);
        let system_prompt: Result<String, String> = Ok(String::new());

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

        self.system_prompt = Some(context);

        ()
    }

    pub fn check_token_usage(&self, new_msg: String) -> Result<(), String> {
        let existing_usage = self.usage.total_tokens;
        let upcoming_usage = u64::try_from(new_msg.len() / 4).map_err(|e| e.to_string())?;

        println!("existing_usage: {existing_usage}");
        println!("upcoming_usage: {upcoming_usage}");

        let context_window;

        if let Some(model) = self.model {
            context_window = model.context_window
        } else {
            return Err("No model found".to_string());
        }

        println!("context_window: {context_window}");

        let useable_context_window = self.compact_threshold_percentage * context_window / 100;

        println!("useable_context_window: {useable_context_window}");

        if existing_usage + upcoming_usage > context_window {
            Err("Usage limit Exceed".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn syssy() {
        let system_prompt_path = PathBuf::from("src/prompts/codex_opencode.txt");
        let system_prompt = fs::read_to_string(system_prompt_path).unwrap();

        assert_eq!(true, system_prompt.starts_with("You are a coding agent"))
    }
}
