use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Queue {
    pub id: i32,
    pub program_name: String,
    pub doctype: String,
    pub url: Option<String>,
    pub request_type: String,
}

#[derive(Deserialize, FromRow, Serialize)]
pub struct QueueNew {
    pub program_name: String,
    pub doctype: String,
    pub url: Option<String>,
    pub request_type: String,
}

#[derive(FromRow)]
pub struct Admin {
    pub id: i32,
    pub username: Option<String>,
    pub token: String,
}
