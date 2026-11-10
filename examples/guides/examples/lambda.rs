use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


fn build_entertainer() -> anyhow::Result<Agent> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();
    Ok(agent)
}


async fn entertain(agent: &Agent, prompt: &str) -> anyhow::Result<String> {
    Ok(agent.prompt(prompt).await?.output())
}
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct Request {
    prompt: String,
}


#[derive(Serialize)]
struct Response {
    joke: String,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let agent = build_entertainer()?;
    let agent = &agent;


    run(service_fn(move |event: LambdaEvent<Request>| async move {
        let joke = entertain(agent, &event.payload.prompt).await?;
        Ok::<_, Error>(Response { joke })
    }))
    .await
}