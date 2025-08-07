#![cfg(feature = "langfuse")]

use std::collections::HashMap;
use std::env;

use base64::prelude::*;
use opentelemetry_otlp::{Protocol, SpanExporter};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};

use crate::Result;
use crate::error::ConfigError;

#[derive(Debug)]
pub struct LangfuseExporter;

impl LangfuseExporter {
    pub fn new(authorization: String) -> SpanExporter {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Basic {}", authorization),
        );

        SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint("https://us.cloud.langfuse.com/api/public/otel/v1/traces")
            .with_headers(headers)
            .build()
            .expect("valid exporter configuration")
    }

    pub fn from_env() -> Result<SpanExporter> {
        let public_key = env::var("LANGFUSE_PUBLIC_KEY").map_err(ConfigError::Env)?;
        let secret_key = env::var("LANGFUSE_SECRET_KEY").map_err(ConfigError::Env)?;

        let authorization = BASE64_STANDARD.encode(format!("{}:{}", public_key, secret_key));

        Ok(Self::new(authorization))
    }
}
