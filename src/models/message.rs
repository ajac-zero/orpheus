use open_responses::{
    AssistantContentPart, AssistantMessageItemParam, DeveloperMessageItemParam,
    InputContentPart, InputImageContentParamAutoParam, InputItem, InputTextContentParam,
    OutputTextContentParam, SystemMessageItemParam, UserMessageItemParam,
};

/// An ergonomic message builder that converts to `open_responses::InputItem`.
#[derive(Debug, Clone)]
pub struct Message {
    role: Role,
    content: Vec<ContentPart>,
}

#[derive(Debug, Clone)]
enum Role {
    System,
    Developer,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
enum ContentPart {
    Text(String),
    Image { url: String },
}

impl Message {
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    pub fn developer(text: impl Into<String>) -> Self {
        Self {
            role: Role::Developer,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    /// Add an image to this message.
    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        self.content.push(ContentPart::Image { url: url.into() });
        self
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Message::user(value)
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Message::user(value)
    }
}

impl From<Message> for InputItem {
    fn from(msg: Message) -> Self {
        match msg.role {
            Role::User => {
                let content: Vec<InputContentPart> = msg
                    .content
                    .into_iter()
                    .map(|p| match p {
                        ContentPart::Text(text) => {
                            InputContentPart::InputText(InputTextContentParam {
                                type_: "input_text".into(),
                                text,
                            })
                        }
                        ContentPart::Image { url } => {
                            InputContentPart::InputImage(InputImageContentParamAutoParam {
                                type_: "input_image".into(),
                                image_url: url,
                                detail: None,
                            })
                        }
                    })
                    .collect();
                InputItem::UserMessage(UserMessageItemParam {
                    type_: "message".into(),
                    role: "user".into(),
                    content: Some(content),
                })
            }
            Role::System => {
                let content: Vec<InputContentPart> = msg
                    .content
                    .into_iter()
                    .filter_map(|p| match p {
                        ContentPart::Text(text) => {
                            Some(InputContentPart::InputText(InputTextContentParam {
                                type_: "input_text".into(),
                                text,
                            }))
                        }
                        _ => None,
                    })
                    .collect();
                InputItem::SystemMessage(SystemMessageItemParam {
                    type_: "message".into(),
                    role: "system".into(),
                    content: Some(content),
                })
            }
            Role::Developer => {
                let content: Vec<InputContentPart> = msg
                    .content
                    .into_iter()
                    .filter_map(|p| match p {
                        ContentPart::Text(text) => {
                            Some(InputContentPart::InputText(InputTextContentParam {
                                type_: "input_text".into(),
                                text,
                            }))
                        }
                        _ => None,
                    })
                    .collect();
                InputItem::DeveloperMessage(DeveloperMessageItemParam {
                    type_: "message".into(),
                    role: "developer".into(),
                    content: Some(content),
                })
            }
            Role::Assistant => {
                let content: Vec<AssistantContentPart> = msg
                    .content
                    .into_iter()
                    .filter_map(|p| match p {
                        ContentPart::Text(text) => {
                            Some(AssistantContentPart::OutputText(OutputTextContentParam {
                                type_: "output_text".into(),
                                text,
                                annotations: None,
                            }))
                        }
                        _ => None,
                    })
                    .collect();
                InputItem::AssistantMessage(AssistantMessageItemParam {
                    type_: "message".into(),
                    role: "assistant".into(),
                    content: Some(content),
                })
            }
        }
    }
}
