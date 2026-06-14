use uuid::Uuid;

use crate::domains::auth::domain::ports::AuthIdentityProvider;

/// Stable caller/session contract that downstream domains can rely on without
/// coupling themselves to the transport-level middleware implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContract {
    pub user_id: Uuid,
    pub role: String,
    pub provider: AuthIdentityProvider,
    pub audience: SessionAudience,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAudience {
    CustomerClient,
    InternalDashboard,
    Integration,
}
