use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not configured");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
