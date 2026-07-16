use anyhow::{Context, Result, bail};
use serde::{self, Serialize};
use std::collections::HashMap;
pub mod openai;

#[derive(Debug, Serialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum Providers {
    OpenAI,
    Anthropic,
    MoonshotAI,
}

impl Providers {
    fn slug(&self) -> String {
        let slug = match self {
            Providers::OpenAI => "openai",
            Providers::Anthropic => "anthropic",
            Providers::MoonshotAI => "moonshotai",
        };

        slug.to_string()
    }
}

pub enum Modalities {
    Text,
    Image,
    File,
    Audio,
    Video,
}

pub struct Price {
    input: f64,
    cached_input: f64,
    output: f64,
}

pub struct Model {
    pub provider: Providers,
    pub identifier: String,
    pub name: String,
    pub context_window: u64,
    pub input_formats: Vec<Modalities>,
    pub output_formats: Vec<Modalities>,
    pub reasoning: bool,
    pub structured_output: bool,
    pub tool_call: bool,
    pub streaming: bool,
    pub caching: bool,
    pub price: Price,
    updated: String,
}

impl Model {
    pub fn full_identifier(&self) -> String {
        format!("{}/{}", self.provider.slug(), self.identifier)
    }
}

// static MODELS: HashMap<String, Model> = HashMap::new();

pub fn all_models() -> HashMap<String, Model> {
    let mut models: HashMap<String, Model> = HashMap::new();

    for m in openai::models() {
        models.insert(m.full_identifier(), m);
    }

    models
}

pub fn get_model<'a>(
    identifier: &Option<String>,
    models: &'a HashMap<String, Model>,
) -> Result<&'a Model> {
    let identifier_str = match identifier {
        Some(v) => v,
        None => "openai/gpt-5.6-sol", // default model
    };

    models
        .get(identifier_str)
        .with_context(|| format!("unknown model: {identifier_str}"))
}
