mod api;
mod db;
mod pages;
mod routes;

use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = db::create_pool()
        .await
        .expect("Failed to connect to PostgreSQL");

    db::migrate(&pool)
        .await
        .expect("Failed to run database migrations");

    let state = AppState { pool: pool.clone() };

    let app = routes::public_routes()
        .with_state(state)
        .nest_service("/assets", ServeDir::new("public/assets"));

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().expect("PORT must be a valid number");`r`n    let address = SocketAddr::from(([0, 0, 0, 0], port));

    println!("==========================================");
    println!("SHRI KRISHNA TYRE HOUSE");
    println!("==========================================");
    println!("Database: PostgreSQL");
    println!("Website: http://0.0.0.0:{}", port);
    println!("==========================================");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to start server");

    axum::serve(listener, app).await.expect("Server failed");
}

