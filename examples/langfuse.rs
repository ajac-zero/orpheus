use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use orpheus::{langfuse::LangfuseExporter, prelude::*};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

fn main() -> anyhow::Result<()> {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(LangfuseExporter::from_env()?)
        .build();

    let tracer = provider.tracer("langfuse-example");

    tracing_subscriber::registry()
        .with(EnvFilter::new("orpheus=trace"))
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    let client = Orpheus::from_env()?;

    let response = client
        .chat([
            Message::system("You are a space pilot assistant"),
            Message::user("Reroute all energy from life support to thrusters"),
        ])
        .model("openai/gpt-4o")
        .top_p(0.95)
        .top_k(5)
        .temperature(0.5)
        .send()?
        .into_content()?;

    tracing::info!("Response: {}", response);

    Ok(())
}
