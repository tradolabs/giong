use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::utils::crypto::random_password;
use giong_database::Database;
use giong_database::{Condition, Document};
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct ResetAdminPasswordUseCase {
    database: Arc<Database>,
}

impl UseCase for ResetAdminPasswordUseCase {
    type Input = ();
    type Output = String;

    async fn execute(&self, (): Self::Input) -> Result<Self::Output, Exception> {
        let (password, password_hash) = random_password().await?;

        self.database
            .transaction(async || {
                let condition = Condition::new().eq("username", "admin");
                if self.database.users.exists(condition.clone()).await? {
                    self.database
                        .users
                        .update_one(condition, Document::new().set("password_hash", password_hash))
                        .await?;
                } else {
                    self.database
                        .users
                        .create_one(
                            Document::new()
                                .set("username", "admin")
                                .set("password_hash", password_hash),
                        )
                        .await?;
                }
                Ok(())
            })
            .await?;

        Ok(password)
    }
}
