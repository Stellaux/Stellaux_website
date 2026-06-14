use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::auth::domain::error::AuthContractError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthIdentityProvider {
    ServerJwt,
    SupabaseJwt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCredential {
    pub user_id: Uuid,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordResetGrant {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
pub trait UserCredentialRepository: Send + Sync {
    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<StoredCredential>, AuthContractError>;
}

#[async_trait]
pub trait PasswordResetTokenRepository: Send + Sync {
    async fn consume(&self, token: &str) -> Result<Option<PasswordResetGrant>, AuthContractError>;
}

#[async_trait]
pub trait SessionTokenIssuer: Send + Sync {
    async fn issue_for_user(
        &self,
        user_id: Uuid,
        role: &str,
        provider: AuthIdentityProvider,
    ) -> Result<String, AuthContractError>;
}
