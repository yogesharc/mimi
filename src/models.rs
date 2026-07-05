use serde::{self, Serialize};
mod openai;

#[derive(Debug, Serialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum Providers {
    OpenAI,
    Anthropic,
    MoonshotAI,
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

pub struct Model<'a> {
    provider: Providers,
    identifier: &'a str,
    name: &'a str,
    context: u16,
    input_formats: Vec<Modalities>,
    output_formats: Vec<Modalities>,
    reasoning: bool,
    structured_output: bool,
    tool_call: bool,
    streaming: bool,
    caching: bool,
    price: Price,
    updated: &'a str,
}

impl Model<'_> {
    fn get_identifier(&self) -> String {
        format!("{}/{}", self.provider, self.identifier)
    }
}
