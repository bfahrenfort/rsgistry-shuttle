mod auth;
mod db;
mod state;

use axum::{routing::get, routing::post, Router};
use shuttle_runtime::CustomError;
use shuttle_secrets::{SecretStore, Secrets};
use sqlx::PgPool;

use state::MyState;

async fn hello_world() -> &'static str {
    "`glhf` API\n\
        \tDocumentation @ https://github.com/bfahrenfort/shuttle-glhf"
}

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PASSWORD}@localhost:19087/postgres"
    )]
    pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool, secrets };
    #[cfg(debug_assertions)]
    {
        let router = Router::new()
            .route("/", get(hello_world))
            .route("/api/v1/debug/update", post(db::update::push))
            .route("/api/v1/update", get(db::update::auth_push))
            .route("/api/v1/queue/add", post(db::update::enqueue))
            .route("/api/v1/queue/peek", get(db::fetch::queue_peek))
            .route("/api/v1/queue/fetch", get(db::fetch::queue_fetch))
            .route("/api/v1/fetch/:name", get(db::fetch::retrieve))
            .route("/login", post(auth::login))
            .with_state(state);

        Ok(router.into())
    }

    #[cfg(not(debug_assertions))]
    {
        let router = Router::new()
            .route("/", get(hello_world))
            .route("/api/v1/update", get(db::update::auth_push))
            .route("/api/v1/queue/add", post(db::update::enqueue))
            .route("/api/v1/queue/peek", get(db::fetch::queue_peek))
            .route("/api/v1/queue/fetch", get(db::fetch::queue_fetch))
            .route("/api/v1/fetch/:name", get(db::fetch::retrieve))
            .route("/login", post(auth::login))
            .with_state(state);

        Ok(router.into())
    }
}
