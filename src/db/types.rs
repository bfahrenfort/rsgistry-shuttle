// Types that don't always need to be modified for a use case

use sqlx::FromRow;

#[derive(FromRow)]
pub struct Admin {
    pub id: i32,
    pub username: Option<String>,
    pub token: String,
}
