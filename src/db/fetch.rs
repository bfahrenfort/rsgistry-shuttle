use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::types::Queue;
use crate::state::MyState;
use scaffold::EntryWithID;

pub async fn retrieve(
    Path(name): Path<String>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, EntryWithID>("SELECT * FROM programs WHERE program_name = $1")
        .bind(name)
        .fetch_one(&state.pool)
        .await
    {
        Ok(program) => Ok((StatusCode::OK, Json(program))),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

pub async fn queue_peek(
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Queue>("SELECT * FROM queue")
        .fetch_one(&state.pool)
        .await
    {
        Ok(queue_entry) => Ok((StatusCode::OK, Json(queue_entry))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn queue_fetch(
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    println!("queue fetching");
    match sqlx::query_as::<_, Queue>("SELECT * FROM queue")
        .fetch_all(&state.pool)
        .await
    {
        Ok(vec) => Ok((StatusCode::OK, Json(vec))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}
