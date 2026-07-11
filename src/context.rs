use std::{fs, path::PathBuf};

use crate::parser::{AgentEventItem, Usage};

pub struct Context {
    pub event_logs: Vec<AgentEventItem>,
    pub system_prompt: Option<String>,
    pub usage: Usage,
    pub compact_threshold: u64,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            event_logs: Vec::new(),
            system_prompt: None,
            usage: Usage::default(),
            compact_threshold: 200000,
        }
    }
}

impl Context {
    pub fn build_system_prompt(&mut self) {
        let mut context: String = String::new();

        let system_prompt_path = PathBuf::from("src/prompts/codex_opencode.txt");
        let system_prompt = fs::read_to_string(system_prompt_path);

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
        let upcoming_usage = u64::try_from(new_msg.len()).map_err(|e| e.to_string())?;

        // it should depend on the model context window, will update this later
        if existing_usage + upcoming_usage > self.compact_threshold {
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
