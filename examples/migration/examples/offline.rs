use rig::AgentBuilder;
use rig::test_utils::{MockCompletionModel, MockTurn};
use rig::memory::{MemoryStore, load, save, SummaryState, apply_summary};
use rig::memory::recent::{view, LastMessages, TokenBudget};
use rig::message::Message;
use rig::cassette::agent::AgentReplayExt;
use rig::cassette::agent::replay::register_all;
use rig::cassette::effect_log::{EffectLog, EffectLogRecorder};
use rig::handlers::Handlers;
use rig::effect::EffectFamily;

#[derive(Clone)]
struct FailingStore { fail_load: bool }
impl rig::memory::AsyncStore for FailingStore {
    async fn read_log_async(&self, _: &str) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::from(if self.fail_load { std::io::ErrorKind::PermissionDenied } else { std::io::ErrorKind::NotFound }))
    }
    async fn append_log_async(&self, _: &str, _: u64, _: &[u8]) -> Result<(), rig::memory::LogWriteError> {
        Err(std::io::Error::other("simulated failed write").into())
    }
    async fn replace_log_async(&self, _: &str, _: u64, _: &[u8]) -> Result<(), rig::memory::LogWriteError> {
        Err(std::io::Error::other("simulated failed write").into())
    }
    async fn read_file_async(&self, _: &str) -> std::io::Result<Vec<u8>> {
        Err(std::io::ErrorKind::NotFound.into())
    }
    async fn write_file_async(&self, _: &str, _: &[u8]) -> std::io::Result<()> { Ok(()) }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let save_model = MockCompletionModel::text("finished but not saved");
    let failed = AgentBuilder::new(save_model.clone()).memory(FailingStore { fail_load: false })
        .build().prompt("hello").await;
    match failed {
        Err(rig::completion::PromptError::Memory { response: Some(answer), .. }) => assert_eq!(answer.output(), "finished but not saved"),
        other => panic!("expected failed save with finished response: {other:?}"),
    }
    assert_eq!(save_model.request_count(), 1);
    let load_model = MockCompletionModel::text("must not run");
    let failed = AgentBuilder::new(load_model.clone()).memory(FailingStore { fail_load: true })
        .build().prompt("hello").await;
    assert!(matches!(failed, Err(rig::completion::PromptError::Memory { response: None, .. })));
    assert_eq!(load_model.request_count(), 0);
    let store = MemoryStore::default();
    let model = MockCompletionModel::from_turns([MockTurn::text("Hello Ada!"), MockTurn::text("Ada")]);
    let agent = AgentBuilder::new(model).memory(store.clone()).build();
    agent.prompt("My name is Ada.").conversation("user-42").await?;
    assert_eq!(agent.prompt("My name?").conversation("user-42").await?.output(), "Ada");
    let mut conv = load(&store, "user-42")?;
    assert_eq!(conv.messages().len(), 4);
    assert!(load(&store, "other")?.messages().is_empty());
    assert_eq!(view(conv.messages(), &LastMessages(2)).len(), 2);
    assert!(view(conv.messages(), &TokenBudget::new(0)).is_empty());
    let summary = SummaryState { summary: "The user's name is Ada.".into(), ..Default::default() };
    assert!(apply_summary(&mut conv, None, &summary).is_some());
    assert_eq!(conv.messages().len(), 1);
    save(&store, "user-42", &mut conv)?;
    assert_eq!(load(&store, "user-42")?.messages(), conv.messages());

    let recorder = EffectLogRecorder::new();
    let agent = AgentBuilder::new(MockCompletionModel::from_turns([MockTurn::text("Axum, Actix, Rocket")]))
        .owner("replay-example").preamble("You are a helpful assistant.")
        .record_to(recorder.clone()).build();
    let original = agent.prompt("Name three Rust web frameworks.").await?;
    let log = agent.stamp(recorder.take());
    let serialized = serde_json::to_string(&log)?;
    let log: EffectLog = serde_json::from_str(&serialized)?;
    let handlers = Handlers::default();
    register_all(&log, &handlers)?;
    let key = log.iter().find(|r| r.kind.family() == EffectFamily::Completion)
        .ok_or_else(|| anyhow::anyhow!("no completion"))?.key.clone();
    let replay = AgentBuilder::over_handlers(handlers,"replay-example",key)
        .preamble("You are a helpful assistant.").build();
    assert_eq!(replay.prompt("Name three Rust web frameworks.").await?.output(), original.output());
    std::fs::write("frameworks.effects.json", serialized)?;
    let _ = Message::user("ready");
    println!("PASS: failed-save response retention, failed-load no-model-call, memory persistence, conversation isolation, windows, compaction, effect recording and replay");
    Ok(())
}
