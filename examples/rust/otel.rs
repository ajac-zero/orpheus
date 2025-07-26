#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.98"
//! opentelemetry = "0.30.0"
//! opentelemetry_sdk = "0.30.0"
//! opentelemetry-stdout = "0.30.0"
//! tracing = "0.1.41"
//! tracing-opentelemetry = "0.31.0"
//! tracing-subscriber = { version = "0.3.19", features = ["env-filter"] }
//! orpheus = { version = "0.1.1", features = ["otel"] }
//! ```
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use orpheus::prelude::*;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

fn main() -> anyhow::Result<()> {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();

    let tracer = provider.tracer("otel-example");

    tracing_subscriber::registry()
        .with(EnvFilter::new("orpheus=trace"))
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    let client = Orpheus::from_env()?;

    let stream = client
        .chat("hiii")
        .model("openai/gpt-4o")
        .top_p(0.95)
        .top_k(5)
        .temperature(0.5)
        .stream()?;

    for _ in stream {
        continue;
    }

    Ok(())
}
