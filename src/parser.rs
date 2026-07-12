use crate::tools;

use super::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Reasoning<'a> {
    effort: &'a EffortLevel,
}

#[derive(Debug, Serialize)]
pub struct ResponseRequest<'a> {
    model: String,
    input: &'a Vec<AgentEventItem>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
    reasoning: Option<Reasoning<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_management: Option<Vec<ContextManagement>>,
}

#[derive(Debug, Serialize)]
pub struct ContextManagement {
    r#type: String,
    compact_threshold: u64,
}

impl Default for ContextManagement {
    fn default() -> Self {
        Self {
            r#type: "compaction".to_string(),
            compact_threshold: 200000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AgentEventItem {
    #[serde(rename = "message")]
    Message {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<ResponseStatus>,
        role: Role,
        content: Vec<Content>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        status: ResponseStatus,
        summary: Vec<Value>,
        encrypted_content: Option<String>,
        format: Option<String>,
    },
    #[serde(rename = "function_call")]
    ToolCall {
        id: String,
        status: ResponseStatus,
        call_id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "function_call_output")]
    TollCallOutput {
        id: String,
        call_id: String,
        output: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<Vec<Value>>,
}

impl AgentEventItem {
    pub fn new_user_message(input: String) -> Self {
        let content = Content {
            content_type: "input_text".to_string(),
            text: input,
            annotations: None,
        };

        Self::Message {
            id: None,
            status: None,
            role: Role::User,
            content: vec![content],
        }
    }
}

impl<'a> ResponseRequest<'a> {
    pub fn new(
        model: String,
        input: &'a Vec<AgentEventItem>,
        effort: Option<&'a EffortLevel>,
        system_prompt: &'a Option<String>,
    ) -> Self {
        let sys_tool_definitions = tools::SystemTools::all()
            .iter()
            .map(|tool| tool.definition())
            .collect();

        let reasoning = effort.map(|eff| Reasoning { effort: eff });
        let context_management = ContextManagement::default();

        ResponseRequest {
            model,
            input,
            stream: true,
            tools: Some(sys_tool_definitions),
            tool_choice: Some("auto".to_string()),
            reasoning,
            instructions: system_prompt,
            context_management: Some(vec![context_management]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseCreatedItem {
    pub id: String,
    created_at: u64,
    model: String,
    status: ResponseStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseCompletedItem {
    id: String,
    completed_at: u64,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Usage {
    pub input_tokens: u64,
    // pub cached_input_tokens: u64,
    pub output_tokens: u64,
    // pub reasoning_tokens: u64,
    pub total_tokens: u64,
    pub cost: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OpenRouterEvents {
    #[serde(rename = "response.created")]
    ResponseCreated { response: ResponseCreatedItem },
    #[serde(rename = "response.in_progress")]
    ResponseInProgress { response: ResponseCreatedItem },
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded {
        output_index: u64,
        item: AgentEventItem,
    },
    #[serde(rename = "response.content_part.added")]
    ResponseContentPartAdded {
        output_index: u16,
        item_id: String,
        part: Content,
    },
    #[serde(rename = "response.output_text.delta")]
    ResponseOutputTextDelta {
        output_index: u16,
        item_id: String,
        content_index: u16,
        delta: String,
    },
    #[serde(rename = "response.output_text.done")]
    ResponseOutputTextDone {
        output_index: u16,
        item_id: String,
        content_index: u16,
        text: String,
    },
    #[serde(rename = "response.content_part.done")]
    ResponseContentPartDone {
        output_index: u16,
        item_id: String,
        content_index: u16,
        part: Content,
    },
    #[serde(rename = "response.reasoning_summary_part.added")]
    ResponseReasoningSummartPartAdded { summary_index: u16, part: Content },
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ResponseReasoningSummaryTextDelta {
        output_index: u16,
        item_id: String,
        summary_index: u16,
        delta: String,
    },
    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone {
        output_index: u16,
        item: AgentEventItem,
    },
    #[serde(rename = "response.completed")]
    ResponseCompleted { response: ResponseCompletedItem },
}
