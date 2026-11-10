#![allow(dead_code, unused_imports, unused_variables)]
use rig::prelude::*;
use rig::providers::openai::{self,OpenAI};
use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt};
use std::io::Write;


#[derive(Clone)]
struct MessageOnlyLayer;


impl<S> tracing_subscriber::Layer<S> for MessageOnlyLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        use tracing::field::{Field, Visit};


        struct MessageVisitor(Option<String>);


        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = Some(format!("{value:?}"));
                }
            }
        }


        let mut visitor = MessageVisitor(None);
        event.record(&mut visitor);


        if let Some(msg) = visitor.0 {
            let level = event.metadata().level();
            let _ = writeln!(std::io::stdout(), "{level} {}", msg.trim_matches('"'));
        }
    }
}


fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(MessageOnlyLayer)
        .init();
}
async fn block_0()->anyhow::Result<()> {
let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
    .preamble("You are a helpful assistant.")
    .record_content_telemetry(true) // every run of this agent
    .build();


// Or for a single run:
let response = agent.prompt("Hello!").record_content_telemetry(true).await?;
Ok(())
}
async fn block_1()->anyhow::Result<()> {
tracing_subscriber::fmt().init();
Ok(())
}
async fn block_2()->anyhow::Result<()> {
tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,rig=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
Ok(())
}
async fn block_10()->anyhow::Result<()> {
use rig::telemetry::{GenAiOperation, SpanBuilder};


let span = SpanBuilder::new("my-provider", "my-model", GenAiOperation::Chat)
    .streaming(false)
    .build();
let _guard = span.enter();
Ok(())
}
async fn block_11()->anyhow::Result<()> {
use std::sync::Arc;
use rig::completion::CompletionRequest;
use rig::observe::{AdapterContext, ObservationLog, Subject};
use rig::providers::openai::{self, OpenAI};


let log = Arc::new(ObservationLog::with_capacity(256));
let context = AdapterContext::new(log.clone(), Subject::scoped("checkout"), "summarize-1");


let model = OpenAI::from_env()?.completion(openai::GPT_5_5);
model.call_observed(CompletionRequest::new("Hello!"), context).await?;


for observation in log.trace().observations {
    println!("{:?}", observation.action);
}
Ok(())
}
fn main() {}