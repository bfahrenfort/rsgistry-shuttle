use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Expanded example:
    // INSERT INTO entries (program_name, doctype, url)
    //     VALUES ($1, $2, $3)
    //     RETURNING (program_name, doctype, url, id)
    let sql = [
        "INSERT INTO entries (",
        &Entry::list_fields().join(", "),
        ") \
            VALUES (",
        &c![format!("${}", x + 1), for x in 0..(Entry::field_count())].concat(),
        ") \
            RETURNING ",
        &EntryWithID::list_fields().join(", "),
    ]
    .concat();
    let query = sqlx::query_as::<_, EntryWithID>(&sql);
    match data.bind(query).fetch_one(&state.pool).await {
        Ok(program) => Ok((StatusCode::CREATED, Json(program))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

// Add to queue
// Intended use
// TODO heavy, heavy validation
pub async fn enqueue(
    State(state): State<MyState>,
    Json(data): Json<QueueNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Expanded example:
    // INSERT INTO queue (program_name, doctype, url, request_type)
    //     VALUES ($1, $2, $3, $4)
    //     RETURNING id, program_name, doctype, url, request_type
    let sql = [
        "INSERT INTO queue (",
        &QueueNew::list_fields().join(", "),
        ") \
            VALUES (",
        &c![format!("${}", x + 1), for x in 0..(QueueNew::field_count())].concat(),
        ") \
            RETURNING ",
        &Queue::list_fields().join(", "),
    ]
    .concat();

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
) -> Result<impl IntoResponse, impl IntoResponse> {
    if data.payload.request_type == "create" {
        let id = data.payload.id;
        let shed_queue = Entry::from(data.payload);
        let sql = [
            "INSERT INTO entries (",
            &Entry::list_fields().join(", "),
            ") \
            VALUES (",
            &c![format!("${}", x + 1), for x in 0..(Entry::field_count())].concat(),
            ") \
            RETURNING ",
            &EntryWithID::list_fields().join(", "),
        ]
        .concat();
        match shed_queue
            .bind(sqlx::query_as::<_, EntryWithID>(&sql))
            .fetch_one(&state.pool)
            .await
        {
            Ok(_) => match sqlx::query("DELETE FROM queue WHERE id=$1")
                .bind(id)
                .execute(&state.pool)
                .await
            {
                Ok(_) => Ok(Json(shed_queue)),
                Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
            },
            Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        }
    } else {
        Err((StatusCode::BAD_REQUEST, "haha".to_string()))
    }
}
