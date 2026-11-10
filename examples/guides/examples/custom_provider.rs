#![allow(dead_code, unused_imports, unused_variables)]
use serde::{Serialize,Deserialize};
use serde_json::json;
use rig::prelude::*;
mod block4 {
use super::*;
use rig::wire::Secret;


/// Where Acme lives and the key it takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcmeConfig {
    pub api_key: Secret,
    pub base_url: String,
}


impl AcmeConfig {
    pub fn new(api_key: impl Into<Secret>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.acme.test".to_owned(),
        }
    }


    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}


/// Acme's generate endpoint, for one model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcmeChat {
    pub config: AcmeConfig,
    pub model: String,
}
}
pub use block4::*;
mod block5 {
use super::*;
use rig::completion::options::{Mapping, OptionFields, OptionMap};
use rig::completion::{Accepts, CompletionRequest, Media, ReplayTarget};
use rig::message::Api;


impl ReplayTarget for AcmeChat {
    fn api(&self) -> Api {
        Api::from_static("acme.generate")
    }


    fn provider(&self) -> &str {
        "acme"
    }


    fn model(&self) -> &str {
        &self.model
    }


    /// Text only: no images, and no tool calling.
    fn accepts(&self, _model: &str) -> Accepts {
        Accepts {
            tools: false,
            ..Accepts::TEXT
        }
    }


    /// No media at all: text documents are sent as their text.
    fn encodes(&self, _model: &str, _media: Media<'_>) -> bool {
        false
    }


    fn map_options(&self, _request: &CompletionRequest, fields: OptionFields<'_>) -> OptionMap {
        let OptionFields {
            reasoning,
            cache,
            service_tier,
            verbosity,
            parallel_tool_calls,
            top_p,
            seed,
            stop,
            cache_key,
        } = fields;
        const NO_FIELD: &str = "Acme has no such field";
        OptionMap {
            reasoning: Mapping::of(reasoning, |_| Mapping::unsupported(NO_FIELD)),
            cache: Mapping::of(cache, |_| Mapping::unsupported(NO_FIELD)),
            service_tier: Mapping::of(service_tier, |_| Mapping::unsupported(NO_FIELD)),
            verbosity: Mapping::of(verbosity, |_| Mapping::unsupported(NO_FIELD)),
            parallel_tool_calls: Mapping::of(parallel_tool_calls, |_| {
                Mapping::unsupported(NO_FIELD)
            }),
            top_p: Mapping::of(top_p, |top_p| Mapping::Send(json!({ "top_p": top_p }))),
            seed: Mapping::of(seed, |seed| Mapping::Send(json!({ "seed": seed }))),
            stop: Mapping::of_stop(stop, |stop| Mapping::Send(json!({ "stop": stop }))),
            cache_key: Mapping::unrouted(cache_key),
        }
    }
}
}
pub use block5::*;
mod block6 {
use super::*;
use rig::completion::{FinishReason, Usage};
use rig::operation::{Block, Completion, Finish};
use rig::providers::internal::wire::classify_untyped_line;
use rig::wire::{Decoder, Flow, Out, WireEvent, WireFrame};
use rig::ProviderError;


/// One Acme reply document, or one line of a streamed reply.
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub usage: Option<AcmeUsage>,
    // Preserve vendor-specific fields for CompletionResponse::raw.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AcmeUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}


#[derive(Debug, Default)]
pub struct AcmeDecoder;


impl<'id> Decoder<'id, Completion> for AcmeDecoder {
    type Event = Chunk;


    fn classify(&self, frame: WireFrame) -> WireEvent<Chunk> {
        classify_untyped_line(frame.as_str().as_bytes())
    }


    fn decode(
        &mut self,
        chunk: Chunk,
        mut out: Out<'id, Completion>,
    ) -> Result<Flow, ProviderError> {
        if !chunk.text.is_empty() {
            out.run(Block::Text, &chunk.text)?;
        }
        if !chunk.done {
            return Ok(Flow::More);
        }
        out.end_run()?;
        let usage = chunk.usage.map_or_else(Usage::new, |usage| {
            Usage::new()
                .input_tokens(usage.input_tokens)
                .output_tokens(usage.output_tokens)
        });
        Ok(out.end(Finish {
            usage,
            reason: Some(FinishReason::Stop),
            response_id: chunk.id,
            model: chunk.model,
            ..Finish::default()
        }))
    }
}
}
pub use block6::*;
mod block7 {
use super::*;
use rig::operation::Completion;
use rig::wire::document::{Reassemble, Serves};
use serde_json::{Map, Value};


#[derive(Default)]
pub struct AcmeDocument {
    text: String,
    last: Map<String, Value>,
}


impl Reassemble<Chunk> for AcmeDocument {
    fn absorb(&mut self, event: &Chunk) {
        let Ok(Value::Object(chunk)) = serde_json::to_value(event) else {
            return;
        };
        if let Some(text) = chunk.get("text").and_then(Value::as_str) {
            self.text.push_str(text);
        }
        self.last = chunk;
    }


    fn finish(self) -> Value {
        if self.last.is_empty() {
            return Value::Null;
        }
        let mut document = self.last;
        document.insert("text".to_owned(), Value::String(self.text));
        Value::Object(document)
    }
}


impl Serves<Completion> for AcmeDocument {}
}
pub use block7::*;
mod block8 {
use super::*;
use rig::completion::options::{request_params, RawAt};
use rig::completion::CompletionRequest;
use rig::error::EncodeError;
use rig::http_client::{HeaderValue, Request};
use rig::message::{AssistantContent, Message, UserContent};
use rig::operation::Completion;
use rig::wire::{Descriptor, Encoded, Framing, Mode, Wire, WireFrame};
use serde_json::{Map, Value};


impl AcmeChat {
    /// Acme's own fields: model, messages, sampling and the stream flag.
    fn base(&self, request: &CompletionRequest, mode: Mode) -> Result<Map<String, Value>, EncodeError> {
        if !request.tools.is_empty() {
            return Err(EncodeError::request("Acme does not support tools"));
        }
        let mut messages = Vec::new();
        for message in &request.chat_history {
            let (role, content) = match message.as_ref() {
                Message::System { content } => ("system", content.clone()),
                Message::User { content } => ("user", user_text(content)?),
                Message::Assistant(turn) => ("assistant", assistant_text(&turn.content)),
            };
            messages.push(json!({ "role": role, "content": content }));
        }
        let mut body = Map::new();
        // `prepare` has already resolved the model: the request's override, or ours.
        let model = request.model.clone().unwrap_or_else(|| self.model.clone());
        body.insert("model".into(), json!(model));
        body.insert("messages".into(), json!(messages));
        body.insert("stream".into(), json!(mode == Mode::Streaming));
        if let Some(temperature) = request.temperature {
            body.insert("temperature".into(), json!(temperature));
        }
        if let Some(max_tokens) = request.max_tokens {
            body.insert("max_tokens".into(), json!(max_tokens));
        }
        Ok(body)
    }
}


/// The text of a user message. `accepts` and `encodes` mean the adapter
/// has already replaced anything else.
fn user_text(content: &[UserContent]) -> Result<String, EncodeError> {
    let mut texts = Vec::new();
    for part in content {
        match part {
            UserContent::Text(text) => texts.push(text.text()),
            _ => return Err(EncodeError::request("Acme takes text only")),
        }
    }
    Ok(texts.join("\n"))
}


fn assistant_text(content: &[AssistantContent]) -> String {
    content
        .iter()
        .filter_map(|block| match block {
            AssistantContent::Text(text) => Some(text.text()),
            _ => None,
        })
        .collect()
}


impl Wire for AcmeChat {
    type Op = Completion;
    type Payload = Encoded;
    type Frame = WireFrame;
    type Event = Chunk;
    type Decoder<'id> = AcmeDecoder;
    type Reassembler = AcmeDocument;


    fn describe(&self) -> Descriptor<'_> {
        Descriptor::new("acme").model(self.model.as_str()).replay(self)
    }


    fn encode(&self, request: CompletionRequest, mode: Mode) -> Result<Encoded, EncodeError> {
        let body = request_params(self, &request, |_| self.base(&request, mode), RawAt::Top, &[])?;
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key.expose()))
                .map_err(EncodeError::request)?;
        authorization.set_sensitive(true);
        let http = Request::post(format!("{}/v1/generate", self.config.base_url))
            .header("content-type", "application/json")
            .header("authorization", authorization)
            .body(body.into_body())?;
        let framing = match mode {
            Mode::Unary => Framing::Whole,
            Mode::Streaming => Framing::Ndjson,
        };
        Ok(Encoded::new(http, framing).with_route(Some("/v1/generate")))
    }


    fn decoder<'id>(&self) -> Self::Decoder<'id> {
        AcmeDecoder
    }
}
}
pub use block8::*;
mod block9 {
use super::*;
use rig::driver::Model;
use rig::http_client::{DynHttpClient, HttpClientExt};
use rig::wire::Secret;


#[derive(Clone)]
pub struct Acme {
    config: AcmeConfig,
    http: DynHttpClient,
}


impl Acme {
    pub fn new(api_key: impl Into<Secret>) -> Self {
        Self {
            config: AcmeConfig::new(api_key),
            http: rig::rig_reqwest::shared(),
        }
    }


    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self::new(std::env::var("ACME_API_KEY")?))
    }


    /// Send through another HTTP client: a configured reqwest client,
    /// middleware, or your own `HttpClientExt`.
    pub fn with_http(mut self, http: impl HttpClientExt + 'static) -> Self {
        self.http = DynHttpClient::new(http);
        self
    }


    pub fn completion(&self, model: impl Into<String>) -> Model<AcmeChat> {
        let wire = AcmeChat {
            config: self.config.clone(),
            model: model.into(),
        };
        Model::new(wire, self.http.clone())
    }
}
}
pub use block9::*;
async fn block0()->anyhow::Result<()> {
use rig::providers::openai::OpenAIConfig;
use rig::providers::openai::wire::{Dialect, Quirks};


/// Acme's gateway speaks Chat Completions but rejects `stream_options` and
/// has no structured output.
const ACME: Dialect = Dialect::gateway("acme", "https://api.acme.test/v1", "ACME_API_KEY")
    .with_quirks(Quirks::openai().without_stream_usage().without_response_format());


let acme = OpenAIConfig::from_env_with(&ACME)?.client();
let model = acme.completion("acme-large");


let agent = AgentBuilder::new(model).preamble("You are terse.").build();
let answer = agent.prompt("Say hello.").await?.output();
Ok(())
}
async fn block1()->anyhow::Result<()> {
use rig::providers::anthropic::{self, AnthropicConfig, Dialect};


const ACME_MESSAGES: Dialect = anthropic::compatible(
    "acme",
    "https://api.acme.test/anthropic",
    "ACME_API_KEY",
    Some("ACME_BASE_URL"),
);


let model = AnthropicConfig::from_env_with(&ACME_MESSAGES)?
    .client()
    .completion("acme-large");
let response = model.call("Say hello.").await?;
println!("{}", response.text());
Ok(())
}
async fn block10()->anyhow::Result<()> {
use futures::StreamExt;
use rig::streaming::{Item, StreamEvent};


let model = Acme::from_env()?.completion("acme-large");


// One request, no agent loop.
let response = model.call("Say hello.").await?;
println!("{} ({:?} output tokens)", response.text(), response.usage.output_tokens);


// The same request, streamed through the same decoder.
let mut stream = model.stream("Count to five.")?;
while let Some(item) = stream.next().await {
    if let Item::Event(StreamEvent::Text { text, .. }) = item? {
        print!("{text}");
    }
}


// As an agent model.
let agent = AgentBuilder::new(model).preamble("You are terse.").build();
println!("{}", agent.prompt("What is Rust?").await?.output());
Ok(())
}
mod block12 {
use super::*;
use rig::operation::{CallFragment, Completion};
use rig::wire::Out;
use rig::ProviderError;


/// One streamed tool-call delta, as an OpenAI-style API sends it.
fn write_call_delta(
    out: &mut Out<'_, Completion>,
    index: usize,
    id: Option<&str>,
    name: Option<&str>,
    arguments: &str,
) -> Result<(), ProviderError> {
    out.fragment(Some(index), CallFragment { id, name, arguments: Some(arguments) })
}


/// The provider's end of the reply: close the open calls, then `out.end(..)`.
fn close_calls(out: &mut Out<'_, Completion>) -> Result<(), ProviderError> {
    out.finish_open()
}
}
#[tokio::main]
async fn main()->anyhow::Result<()> {
use futures::stream;
use rig::driver::{Exchange, Model, Opened, Opening, Transport};
use rig::wire::{Encoded, Wire, WireFrame};


/// Replays one recorded reply body for any HTTP wire.
#[derive(Clone)]
struct Replay(&'static str);


impl<W: Wire<Payload = Encoded, Frame = WireFrame>> Transport<W> for Replay {
    fn send(&self, payload: Encoded, _exchange: Exchange) -> Opening<WireFrame> {
        let frames = payload.framing.split(self.0.as_bytes());
        Opening::ready(Opened::new(stream::iter(frames.into_iter().map(Ok))))
    }
}


let wire = AcmeChat {
    config: AcmeConfig::new("test-key"),
    model: "acme-large".to_owned(),
};
let streamed = concat!(
    r#"{"text":"Hel"}"#, "\n",
    r#"{"text":"lo"}"#, "\n",
    r#"{"id":"gen_1","done":true,"usage":{"input_tokens":3,"output_tokens":2},"vendor_metric":7}"#, "\n",
);
let model = Model::new(wire, Replay(streamed));


let response = model.stream("hi")?.finish().await?;
assert_eq!(response.text(), "Hello");
assert_eq!(response.usage.output_tokens, Some(2));
assert_eq!(response.raw["text"], "Hello");
assert_eq!(response.raw["vendor_metric"], 7);
Ok(())
}