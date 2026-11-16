use crate::application::exceptions::base::Exception;
use crate::application::services::node_service::NodeService;
use crate::application::types::node_execution::NodeExecution;
use crate::application::use_cases::{
    create_flow::{CreateFlowInput, CreateFlowUseCase},
    create_node::{CreateNodeInput, CreateNodeUseCase},
    create_secret::{CreateSecretInput, CreateSecretUseCase},
    create_vault::{CreateVaultInput, CreateVaultUseCase},
    delete_flow::DeleteFlowUseCase,
    delete_node::DeleteNodeUseCase,
    delete_secret::{DeleteSecretInput, DeleteSecretUseCase},
    delete_vault::DeleteVaultUseCase,
    get_flows::GetFlowsUseCase,
    get_node_details::{GetNodeDetailsUseCase, NodeDetails},
    get_node_logs::{GetNodeLogsInput, GetNodeLogsUseCase},
    get_secret::{GetSecretInput, GetSecretUseCase},
    get_secrets::{GetSecretsUseCase, SecretInfo},
    get_vaults::GetVaultsUseCase,
    reset_admin_password::ResetAdminPasswordUseCase,
    resume_pending_executions::ResumePendingExecutionsUseCase,
    sign_in::{SignInInput, SignInOutput, SignInUseCase},
    update_flow::{UpdateFlowInput, UpdateFlowUseCase},
    update_node::{UpdateNodeInput, UpdateNodeUseCase},
    update_secret::{UpdateSecretInput, UpdateSecretUseCase},
    update_vault::{UpdateVaultInput, UpdateVaultUseCase},
    use_case::UseCase,
};
use crate::config::Config;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::entities::secret_entity::SecretEntity;
use giong_database::entities::vault_entity::VaultEntity;
use giong_di::Container;
use std::sync::Arc;
use uuid::Uuid;

pub struct Application {
    config: Arc<Config>,
    container: Arc<Container>,
}

impl Application {
    pub fn new(container: Arc<Container>) -> Result<Self, Exception> {
        let config = container.get::<Config>()?;
        Ok(Self { config, container })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn reset_admin_password(&self) -> Result<String, Exception> {
        self.container.get::<ResetAdminPasswordUseCase>()?.execute(()).await
    }

    pub async fn sign_in(&self, username: &str, password: &str) -> Result<SignInOutput, Exception> {
        self.container
            .get::<SignInUseCase>()?
            .execute(SignInInput { username: username.to_owned(), password: password.to_owned() })
            .await
    }

    pub async fn get_flows(&self) -> Result<Vec<FlowEntity>, Exception> {
        self.container.get::<GetFlowsUseCase>()?.execute(()).await
    }

    pub async fn create_flow(&self, input: CreateFlowInput) -> Result<FlowEntity, Exception> {
        self.container.get::<CreateFlowUseCase>()?.execute(input).await
    }

    pub async fn create_node(&self, input: CreateNodeInput) -> Result<NodeEntity, Exception> {
        self.container.get::<CreateNodeUseCase>()?.execute(input).await
    }

    pub async fn update_node(&self, input: UpdateNodeInput) -> Result<NodeEntity, Exception> {
        self.container.get::<UpdateNodeUseCase>()?.execute(input).await
    }

    pub async fn execute_node(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
    ) -> Result<NodeExecution, Exception> {
        self.container.get::<NodeService>()?.enqueue_execution(flow_id, node_id).await
    }

    pub async fn delete_flow(&self, flow_id: Uuid) -> Result<(), Exception> {
        self.container.get::<DeleteFlowUseCase>()?.execute(flow_id).await
    }

    pub async fn delete_node(&self, flow_id: Uuid, node_id: Uuid) -> Result<(), Exception> {
        self.container.get::<DeleteNodeUseCase>()?.execute((flow_id, node_id)).await
    }

    pub async fn update_flow(&self, input: UpdateFlowInput) -> Result<FlowEntity, Exception> {
        self.container.get::<UpdateFlowUseCase>()?.execute(input).await
    }

    pub async fn get_node_details(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
    ) -> Result<NodeDetails, Exception> {
        self.container.get::<GetNodeDetailsUseCase>()?.execute((flow_id, node_id)).await
    }

    pub async fn get_node_logs(
        &self,
        input: GetNodeLogsInput,
    ) -> Result<Option<String>, Exception> {
        self.container.get::<GetNodeLogsUseCase>()?.execute(input).await
    }

    pub async fn get_secret(
        &self,
        vault_id: Uuid,
        secret_id: Uuid,
    ) -> Result<SecretEntity, Exception> {
        self.container
            .get::<GetSecretUseCase>()?
            .execute(GetSecretInput { vault_id, secret_id })
            .await
    }

    pub async fn get_vaults(&self) -> Result<Vec<VaultEntity>, Exception> {
        self.container.get::<GetVaultsUseCase>()?.execute(()).await
    }

    pub async fn create_vault(&self, input: CreateVaultInput) -> Result<VaultEntity, Exception> {
        self.container.get::<CreateVaultUseCase>()?.execute(input).await
    }

    pub async fn update_vault(&self, input: UpdateVaultInput) -> Result<VaultEntity, Exception> {
        self.container.get::<UpdateVaultUseCase>()?.execute(input).await
    }

    pub async fn delete_vault(&self, vault_id: Uuid) -> Result<(), Exception> {
        self.container.get::<DeleteVaultUseCase>()?.execute(vault_id).await
    }

    pub async fn get_secrets(&self, vault_id: Uuid) -> Result<Vec<SecretInfo>, Exception> {
        self.container.get::<GetSecretsUseCase>()?.execute(vault_id).await
    }

    pub async fn create_secret(
        &self,
        vault_id: Uuid,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<CreateSecretUseCase>()?
            .execute(CreateSecretInput { vault_id, key, value })
            .await
    }

    pub async fn update_secret(
        &self,
        vault_id: Uuid,
        secret_id: Uuid,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<UpdateSecretUseCase>()?
            .execute(UpdateSecretInput { vault_id, secret_id, value })
            .await
    }

    pub async fn delete_secret(&self, vault_id: Uuid, secret_id: Uuid) -> Result<(), Exception> {
        self.container
            .get::<DeleteSecretUseCase>()?
            .execute(DeleteSecretInput { vault_id, secret_id })
            .await
    }

    pub async fn resume_pending_executions(&self) -> Result<(), Exception> {
        self.container.get::<ResumePendingExecutionsUseCase>()?.execute(()).await
    }
}
