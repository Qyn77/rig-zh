use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use tracing::{info, instrument};


#[instrument(name = "process_user_query")]
pub async fn process_query(user_input: &str) -> anyhow::Result<String> {
    info!("Processing user query");


    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .name("assistant")
        .preamble("You are a helpful assistant.")
        .build();


    // The agent run and its model call emit spans under `process_user_query`.
    let response = agent.prompt(user_input).await?;


    info!("Query processed successfully");
    Ok(response.output())
}
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Sends to http://localhost:4318 unless OTEL_EXPORTER_OTLP_ENDPOINT says otherwise.
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .build()?;


    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name("rig-service").build())
        .build();
    let tracer = provider.tracer("rig-service");


    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();


    let answer = process_query("Hello world!").await?;
    println!("{answer}");


    // Flush buffered spans before exiting.
    provider.shutdown()?;
    Ok(())
}