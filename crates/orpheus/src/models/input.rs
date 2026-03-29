use open_responses::{FunctionCallOutputItemParam, InputItem};

use crate::models::Message;

/// A collection of input items to send to the API.
pub struct Input(pub Vec<InputItem>);

impl Input {
    pub fn push(&mut self, item: InputItem) {
        self.0.push(item);
    }

    pub fn push_function_output(&mut self, call_id: impl Into<String>, output: impl Into<String>) {
        self.0
            .push(InputItem::FunctionCallOutput(FunctionCallOutputItemParam {
                type_: "function_call_output".into(),
                call_id: call_id.into(),
                output: Some(output.into()),
            }));
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Input(vec![Message::user(value).into()])
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Input(vec![Message::user(value).into()])
    }
}

impl From<Message> for Input {
    fn from(value: Message) -> Self {
        Input(vec![value.into()])
    }
}

impl From<Vec<Message>> for Input {
    fn from(value: Vec<Message>) -> Self {
        Input(value.into_iter().map(Into::into).collect())
    }
}

impl From<&Vec<Message>> for Input {
    fn from(value: &Vec<Message>) -> Self {
        Input(value.iter().cloned().map(Into::into).collect())
    }
}

impl<const N: usize> From<[Message; N]> for Input {
    fn from(value: [Message; N]) -> Self {
        Input(value.into_iter().map(Into::into).collect())
    }
}

impl From<&Input> for Input {
    fn from(value: &Input) -> Self {
        Input(value.0.clone())
    }
}

impl From<Vec<InputItem>> for Input {
    fn from(value: Vec<InputItem>) -> Self {
        Input(value)
    }
}

impl serde::Serialize for Input {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
