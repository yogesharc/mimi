use super::{Modalities, Model, Price, Providers};

pub const GPT_5_4_MINI: Model = Model {
    provider: Providers::OpenAI,
    identifier: "gpt-5.4-mini",
    name: "GPT 5.4 Mini",
    context: 400000,
    input_formats: vec![Modalities::Text, Modalities::Image],
    output_formats: vec![Modalities::Text],
    reasoning: true,
    structured_output: true,
    tool_call: true,
    streaming: true,
    price: Price {
        input: 0.75,
        cached_input: 0.075,
        output: 4.5,
    },
};
