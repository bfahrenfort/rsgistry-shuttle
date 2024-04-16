use aide::axum::IntoApiResponse;
use axum::{extract::State, http::StatusCode, Json};
use cute::c;

use crate::auth::types::Claims;
use crate::state::MyState;
use scaffold::{Entry, EntryWithID, Queue, QueueNew};

// (UNAUTHORIZED) add Program to the database
// Debug and unit test use only!
#[cfg(debug_assertions)]
pub async fn push(
    State(state): State<MyState>,
    Json(data): Json<Entry>,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    // Expanded example:
    // INSERT INTO entries (program_name, doctype, url)
    //     VALUES ($1, $2, $3)
    //     RETURNING (program_name, doctype, url, id)
    let sql = format!(
        "INSERT INTO entries ({}) VALUES ({}) RETURNING {}",
        &Entry::list_fields().join(", "),
        &c![format!("${}", x + 1), for x in 0..(Entry::field_count())].join(", "),
        &EntryWithID::list_fields().join(", "),
    );
    let query = sqlx::query_as::<_, EntryWithID>(&sql);
    match data.bind(query).fetch_one(&state.pool).await {
        Ok(program) => Ok((StatusCode::CREATED, Json(program))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

// Add to queue
// Intended use
// TODO: heavy, heavy validation
pub async fn enqueue(
    State(state): State<MyState>,
    Json(data): Json<QueueNew>,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    // Expanded example:
    // INSERT INTO queue (program_name, doctype, url, request_type)
    //     VALUES ($1, $2, $3, $4)
    //     RETURNING id, program_name, doctype, url, request_type
    let sql = format!(
        "INSERT INTO queue ({}) VALUES ({}) RETURNING {}",
        &QueueNew::list_fields().join(", "),
        &c![format!("${}", x + 1), for x in 0..(QueueNew::field_count())].join(", "),
        &Queue::list_fields().join(", "),
    );

    match data
        .bind(sqlx::query_as::<_, Queue>(&sql))
        .fetch_one(&state.pool)
        .await
    {
        Ok(program) => Ok((StatusCode::CREATED, Json(program))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn auth_push(
    State(state): State<MyState>,
    data: Claims,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    if data.payload.request_type == "create" {
        let id = data.payload.id;
        let shed_queue = Entry::from(data.payload);
        let sql = format!(
            "INSERT INTO entries ({}) VALUES ({}) RETURNING {}",
            &Entry::list_fields().join(", "),
            &c![format!("${}", x + 1), for x in 0..(Entry::field_count())].join(", "),
            &EntryWithID::list_fields().join(", "),
        );

        // Perform whole operation atomically
        let transaction = state
            .pool
            .begin()
            .await
            .expect("Unable to begin transaction");

        // 1. Insert the object designated from the queue into the entries table
        let inserted = shed_queue
            .bind(sqlx::query_as::<_, EntryWithID>(&sql))
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // 2. Remove the queue entry as it's been performed
        sqlx::query("DELETE FROM queue WHERE id=$1")
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(Json(inserted))
    } else {
        // TODO: implement amendments
        Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Unimplemented queue request type: {}",
                data.payload.request_type
            ),
        ))
    }
}
