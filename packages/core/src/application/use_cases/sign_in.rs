use super::use_case::UseCase;
use crate::application::constants::TOKEN_LIFETIME;
use crate::application::exceptions::authentication::AuthenticationException;
use crate::application::exceptions::base::Exception;
use crate::config::Config;
use crate::utils::{crypto::verify_password, time};
use giong_database::Condition;
use giong_database::Database;
use giong_di::Injectable;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SignInInput {
    pub username: String,
    pub password: String,
}

pub struct SignInOutput {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Injectable)]
pub struct SignInUseCase {
    config: Arc<Config>,
    database: Arc<Database>,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

impl UseCase for SignInUseCase {
    type Input = SignInInput;
    type Output = SignInOutput;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let secret = self
            .config
            .jwt_secret
            .as_deref()
            .ok_or_else(|| Exception::unexpected("JWT secret is not configured"))?;
        let user = self
            .database
            .users
            .find_one(Condition::new().eq("username", input.username.as_str()))
            .await?
            .ok_or(AuthenticationException::InvalidCredentials)?;

        let is_verified = verify_password(input.password.to_owned(), user.password_hash).await?;
        if !is_verified {
            return Err(AuthenticationException::InvalidCredentials.into());
        }

        let access_token = encode(
            &Header::new(Algorithm::HS256),
            &Claims { sub: input.username, exp: time::now()? + TOKEN_LIFETIME },
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(Exception::unexpected)?;

        Ok(SignInOutput { access_token, expires_in: TOKEN_LIFETIME })
    }
}
