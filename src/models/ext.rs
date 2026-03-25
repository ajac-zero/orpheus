use open_responses::{ContentPart, FunctionCall, OutputItem, ResponseResource};

/// Extension trait for convenient access to response data.
pub trait ResponseExt {
    /// Extract the concatenated output text from all message output items.
    fn output_text(&self) -> Option<String>;

    /// Extract all function calls from the response output.
    fn function_calls(&self) -> Vec<&FunctionCall>;
}

impl ResponseExt for ResponseResource {
    fn output_text(&self) -> Option<String> {
        let mut text = String::new();
        for item in &self.output {
            if let OutputItem::Message(msg) = item {
                for part in &msg.content {
                    if let ContentPart::OutputText(t) = part {
                        if let Some(ref s) = t.text {
                            text.push_str(s);
                        }
                    }
                }
            }
        }
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn function_calls(&self) -> Vec<&FunctionCall> {
        self.output
            .iter()
            .filter_map(|item| {
                if let OutputItem::FunctionCall(fc) = item {
                    Some(fc)
                } else {
                    None
                }
            })
            .collect()
    }
}
