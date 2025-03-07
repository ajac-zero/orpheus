pub mod chat;
pub mod embed;
pub mod message;
pub mod prompt;

use std::collections::HashMap;

pub type ExtrasMap = Option<HashMap<String, String>>;
