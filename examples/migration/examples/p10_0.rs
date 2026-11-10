use std::collections::BTreeSet;
use std::io::BufRead;
use rig::completion::CompletionRequest;
use rig::providers::openai::{self, OpenAI};
use rig::run::{AgentRun, AgentRunStep, ModelTurn, ModelTurnOutcome, InvalidToolCallAction, PendingToolCall, TurnPolicy};
use rig::tool::{ToolSet, ToolContext, ToolExecutionError, ToolResult};

static EXECUTIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// Demonstration only: no bank or payment service is called.
#[rig::rig_tool(description = "Simulate a transfer; never moves real money")]
fn transfer_funds(account: String, amount: u64) -> Result<String, ToolExecutionError> {
    EXECUTIONS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(format!("SIMULATED: transferred ${amount} to {account}"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::test_utils::{MockCompletionModel, MockTurn};
    let offline = std::env::args().any(|arg| arg == "--offline");
    let model = if offline {
        MockCompletionModel::from_turns([
            MockTurn::tool_call("call-1", "transfer_funds", serde_json::json!({"account":"B-2", "amount":500})),
            MockTurn::text("The simulated transfer was reviewed."),
        ]).erase()
    } else {
        OpenAI::from_env()?.completion(openai::GPT_5_5).erase()
    };
    let mut tools = ToolSet::default();
    tools.add_tool(TransferFunds);
    let definitions = tools.tool_definitions();
    let names: BTreeSet<String> = definitions.iter().map(|d| d.name.to_string()).collect();
    let policy = TurnPolicy::new(names, None, None)?;
    let checkpoint = std::path::Path::new("run.json");
    let mut run = AgentRun::new("Simulate transferring $500 to account B-2.").max_turns(4);
    let mut approved = 0;
    loop {
        match run.next_step()? {
            AgentRunStep::CallModel { prompt, history, .. } => {
                let response = model.call(CompletionRequest::new(prompt)
                    .messages(history)
                    .preamble("You are a demonstration banking assistant. Use the simulation tool.")
                    .tools(definitions.clone())).await?;
                let mut outcome = run.model_response(ModelTurn::from_policy(&response, policy.clone()))?;
                while let ModelTurnOutcome::NeedsResolution(_) = outcome {
                    outcome = run.resolve_invalid_tool_call(InvalidToolCallAction::fail())?;
                }
            }
            AgentRunStep::CallTools { .. } => {
                std::fs::write(checkpoint, serde_json::to_vec(&run)?)?;
                let mut resumed: AgentRun = serde_json::from_slice(&std::fs::read(checkpoint)?)?;
                let AgentRunStep::CallTools { calls } = resumed.next_step()? else {
                    anyhow::bail!("expected pending calls after resume");
                };
                let mut answers = Vec::new();
                for pending in calls {
                    match pending {
                        PendingToolCall::Execute(call) => {
                            println!("approve {}({})? [y/N]", call.name(), call.arguments());
                            let mut answer = String::new();
                            std::io::stdin().lock().read_line(&mut answer)?;
                            let result = if answer.trim() == "y" {
                                approved += 1;
                                tools.execute(call.name(), call.arguments().to_string(), &mut ToolContext::new()).await
                            } else {
                                ToolResult::skipped("denied by the reviewer")
                            };
                            answers.push(call.answer(result));
                        }
                        PendingToolCall::Malformed(call) => answers.push(call.answer(None)),
                        _ => anyhow::bail!("unsupported pending-call variant"),
                    }
                }
                resumed.answer_all(answers)?;
                // Persist answered state too; answered slots are not offered again after resume.
                std::fs::write(checkpoint, serde_json::to_vec(&resumed)?)?;
                run = serde_json::from_slice(&std::fs::read(checkpoint)?)?;
            }
            AgentRunStep::Done(response) => {
                assert_eq!(EXECUTIONS.load(std::sync::atomic::Ordering::SeqCst), approved);
                println!("{} (executed {approved} simulated transfers)", response.output());
                return Ok(());
            }
        }
    }
}
