#[derive(Debug, thiserror::Error)]
pub enum AuthContractError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token issuance failed")]
    TokenIssuanceFailed,

    #[error("password reset token invalid or expired")]
    InvalidPasswordResetToken,

    #[error("identity provider unavailable")]
    IdentityProviderUnavailable,

    #[error("{0}")]
    Other(String),
}
