use clap::Parser;
use giong_core::application::use_cases::run_flow::{RunFlowInput, RunFlowUseCase};
use giong_core::application::use_cases::secrets::{PutSecretInput, PutSecretUseCase};
use giong_core::application::use_cases::upsert_flow::{UpsertFlowInput, UpsertFlowUseCase};
use giong_core::application::use_cases::vaults::CreateVaultUseCase;
use giong_core::config::Config;

mod command;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FlowDocument {
    flow: giong_node::types::node::Flow,
}

use crate::command::AddResource;
use command::Command;

#[derive(Parser)]
#[command(name = "giong", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let application = giong_core::bootstrap::ensure_initialized(Config::default()).await?;
    match cli.command {
        Command::Run { file } => {
            let document: FlowDocument = serde_yaml_ng::from_str(&std::fs::read_to_string(&file)?)?;
            let scope = if document.flow.name.is_none() {
                Some(std::fs::canonicalize(&file)?.to_string_lossy().into_owned())
            } else {
                None
            };
            let run = application
                .execute::<RunFlowUseCase>(RunFlowInput {
                    flow: document.flow,
                    run_id: None,
                    log_timestamp: None,
                    node_id: None,
                    scope,
                })
                .await?;
            println!("Flow {} completed: {}", run.id, run.completed.join(", "));
        }
        Command::SetSecret { vault_id, key } => {
            let value = std::io::read_to_string(std::io::stdin())?;
            application
                .execute::<PutSecretUseCase>(PutSecretInput { vault_id, key, value })
                .await?;
            println!("Stored secret");
        }
        Command::RunNode { flow_id, node_id } => {
            use giong_core::application::types::node_run::RunStatus;
            use giong_core::application::use_cases::{
                get_node_runs::GetNodeRunsUseCase, node_triggers::TriggerNodeUseCase,
            };
            let run = application.execute::<TriggerNodeUseCase>((flow_id, node_id)).await?;
            loop {
                let runs = application.execute::<GetNodeRunsUseCase>(flow_id).await?;
                if let Some(run) =
                    runs.iter().find(|item| item.id == run.id && !item.status.active())
                {
                    println!("{}", serde_json::to_string(run)?);
                    if run.status != RunStatus::Succeeded {
                        anyhow::bail!("Node run failed; see its logs");
                    }
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        Command::DeleteFlow { flow_id } => {
            use giong_core::application::use_cases::delete_flow::DeleteFlowUseCase;
            application.execute::<DeleteFlowUseCase>(flow_id).await?;
            println!("Deleted flow {flow_id}");
        }
        Command::Watch => {
            use giong_core::application::use_cases::node_triggers::{
                DispatchNodeTriggersUseCase, ResumePendingRunsUseCase,
            };
            application.execute::<ResumePendingRunsUseCase>(()).await?;
            loop {
                if let Err(error) = application.execute::<DispatchNodeTriggersUseCase>(()).await {
                    eprintln!("Node triggers: {error}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
        Command::ResetAdminPassword => {
            let password = application.reset_admin_password().await?;
            println!("Admin password: {password}");
        }
        Command::Add(resource) => match resource {
            AddResource::Vault { file } => {
                let input = serde_yaml_ng::from_str(&std::fs::read_to_string(file)?)?;
                let node = application.execute::<CreateVaultUseCase>(input).await?;
                println!("Added vault {} ({})", node.name, node.id);
            }
            AddResource::Flow { name, nodes, inherited_vault_ids } => {
                application
                    .execute::<UpsertFlowUseCase>(UpsertFlowInput {
                        id: None,
                        name: name.clone(),
                        nodes: serde_json::from_str(&nodes)?,
                        inherited_vault_ids,
                    })
                    .await?;
                println!("Added flow {name}");
            }
        },
    }

    Ok(())
}
