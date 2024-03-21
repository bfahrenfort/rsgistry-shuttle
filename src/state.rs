use aide::OperationIo;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[derive(Clone, OperationIo)]
pub struct MyState {
    pub pool: PgPool,
    pub secrets: SecretStore,
}
