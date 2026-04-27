use std::str::FromStr;

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub async fn create_pool(database_url: &str) -> PgPool {
    let options = PgConnectOptions::from_str(database_url).expect("Invalid database URL");

    PgPoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .expect("Failed to connect to database")
}

#[allow(unused)]
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}
