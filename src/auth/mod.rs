pub mod types;

use aide::axum::IntoApiResponse;
use axum::{extract::State, Json};
use jsonwebtoken::{encode, Header};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use std::time::SystemTime;

use crate::db::types::Admin;
use crate::state::MyState;
use scaffold::Queue;
use types::*;

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let mut key = [0u8; 64];
    OsRng.fill_bytes(&mut key);
    Keys::new(&key)
});

pub async fn login(
    State(state): State<MyState>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoApiResponse, impl IntoApiResponse> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let tokens: Vec<String> = sqlx::query_as::<_, Admin>("SELECT * FROM admins")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| AuthError::WrongCredentials)?
        .into_iter()
        .map(|e| e.token)
        .collect();

    if !(tokens.contains(&payload.client_secret)
        || state.secrets.get("ROOT_ADMIN").unwrap() == payload.client_secret)
    {
        return Err(AuthError::WrongCredentials);
    }
    // add 5 minutes to current unix epoch time as expiry date/time
    let exp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 300;
    let exp = usize::try_from(exp).unwrap();

    match payload.queue_id {
        Some(id) => {
            // If they already know what id to use from queue_peek, then just push it
            let pusher = sqlx::query_as::<_, Queue>("SELECT * FROM queue WHERE id=$1")
                .bind(id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| AuthError::BadQueue)?;

            let record = Claims {
                payload: pusher,
                exp,
            };

            let token = encode(&Header::default(), &record, &KEYS.encoding)
                .map_err(|_| AuthError::TokenCreation)?;

            Ok(Json(AuthBody::new(token)))
        }
        None => {
            let queue_peek = sqlx::query_as::<_, Queue>("SELECT * FROM queue")
                .fetch_all(&state.pool)
                .await
                .map_err(|_| AuthError::BadQueue)?;

            let record = Claims {
                payload: queue_peek[0].clone(),
                exp,
            };

            let token = encode(&Header::default(), &record, &KEYS.encoding)
                .map_err(|_| AuthError::TokenCreation)?;

            Ok(Json(AuthBody::new(token)))
        }
    }
}
