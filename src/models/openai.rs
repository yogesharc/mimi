use super::{Modalities, Model, Price, Providers};

pub fn models() -> Vec<Model> {
    let gpt_5_4_mini = Model {
        provider: Providers::OpenAI,
        identifier: "gpt-5.4-mini".to_string(),
        name: "GPT 5.4 Mini".to_string(),
        context_window: 400000,
        input_formats: vec![Modalities::Text, Modalities::Image],
        output_formats: vec![Modalities::Text],
        reasoning: true,
        structured_output: true,
        tool_call: true,
        streaming: true,
        caching: true,
        price: Price {
            input: 0.75,
            cached_input: 0.075,
            output: 4.5,
        },
        updated: "today".to_string(),
    };

    let gpt_5_6_sol = Model {
        provider: Providers::OpenAI,
        identifier: "gpt-5.6-sol".to_string(),
        name: "GPT 5.6 Sol".to_string(),
        context_window: 400000,
        input_formats: vec![Modalities::Text, Modalities::Image, Modalities::File],
        output_formats: vec![Modalities::Text],
        reasoning: true,
        structured_output: true,
        tool_call: true,
        streaming: true,
        caching: true,
        price: Price {
            input: 5.00,
            cached_input: 0.5,
            output: 30.00,
        },
        updated: "today".to_string(),
    };

    vec![gpt_5_4_mini, gpt_5_6_sol]
}
