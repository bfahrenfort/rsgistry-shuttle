use shuttle_secrets::SecretStore;
use sqlx::PgPool;

#[derive(Clone)]
pub struct MyState {
    pub pool: PgPool,
    pub secrets: SecretStore,
}
