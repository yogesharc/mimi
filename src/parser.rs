use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseRequest {
    model: String,
    input: Vec<ConversationItem>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConversationItem {
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<Vec<Value>>,
}

impl ConversationItem {
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

impl ResponseRequest {
    pub fn new(model: String, input: String) -> Self {
        ResponseRequest {
            model,
            input: vec![ConversationItem::new_user_message(input)],
            stream: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseCreatedItem {
    id: String,
    created_at: u64,
    model: String,
    status: ResponseStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseCompletedItem {
    id: String,
    completed_at: u64,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    cost: f64,
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
        item: ConversationItem,
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
    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone {
        output_index: u16,
        item: ConversationItem,
    },
    #[serde(rename = "response.completed")]
    ResponseCompleted { response: ResponseCompletedItem },
}

// struct Parser {}

// impl Parser {
//     fn new() -> Self {
//         Self {}
//     }

//     fn parse_open_router_events(event: String) -> OpenRouterEvents {
//         let v = match event.get("type")) {
//             "response.created" => handle_response_created(event)
//             _ => {}
//         }
//     }

//     fn handle_response_created(event: Value) -> OpenRouterEvents::ResponseCreated {
//         let value = serde_json::from_value::<OpenRouterEvents::ResponseCreated>(event).map_err(|e| e.to_string())?;
//     }
// }
