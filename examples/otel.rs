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
