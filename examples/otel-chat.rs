use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use orpheus::prelude::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

fn main() -> anyhow::Result<()> {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let telemetry = tracing_opentelemetry::layer().with_tracer(provider.tracer("otel-chat"));
    let subscriber = Registry::default()
        .with(EnvFilter::new("orpheus=trace"))
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber)?;

    let client = Orpheus::from_env()?;

    let response = client
        .chat("hiii")
        .model("openai/gpt-4o")
        .top_p(0.95)
        .top_k(5)
        .temperature(0.5)
        .send()?
        .into_content()?;

    tracing::info!("Response: {}", response);

    Ok(())
}
