use aide::axum::IntoApiResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use cute::c;

use crate::state::MyState;
use scaffold::{Entry, EntryKeyTuple, EntryWithID, Queue};

pub async fn retrieve(
    Path(key_tuple): Path<EntryKeyTuple>,
    State(state): State<MyState>,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    let keys = Entry::get_keys();
    let sql = format!(
        "SELECT * FROM entries WHERE {}",
        c![format!("{} = ${}", keys[x], x + 1), for x in 0..keys.len()].join(" AND ")
    );
    let query = sqlx::query_as::<_, EntryWithID>(&sql);

    let bound = Entry::fetch_bind(Entry::yeet_tuple(key_tuple), query); // So cursed
    match bound.fetch_one(&state.pool).await {
        Ok(program) => Ok((StatusCode::OK, Json(program))),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

pub async fn queue_peek(
    State(state): State<MyState>,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
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
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    println!("queue fetching");
    match sqlx::query_as::<_, Queue>("SELECT * FROM queue")
        .fetch_all(&state.pool)
        .await
    {
        Ok(vec) => Ok((StatusCode::OK, Json(vec))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}
