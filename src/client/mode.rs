use std::sync::Arc;

pub trait Mode: std::marker::Sync + Send + Clone {
    fn new() -> Self;
    fn runtime(&self) -> Option<Arc<tokio::runtime::Runtime>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Sync {
    pub rt: Arc<tokio::runtime::Runtime>,
}

impl Mode for Sync {
    fn new() -> Self {
        let rt = Arc::new(tokio::runtime::Runtime::new().expect("create tokio runtime"));
        Self { rt }
    }

    fn runtime(&self) -> Option<Arc<tokio::runtime::Runtime>> {
        Some(self.rt.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Async;

impl Mode for Async {
    fn new() -> Self {
        Self
    }
}
