use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::middleware::Logger;
use sqlx::{PgPool, postgres::PgPoolOptions};
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
    balance: f64,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct Game {
    id: i32,
    player1_id: i32,
    player2_id: Option<i32>,
    entry_fee: f64,
    owner_cut: f64,
    status: String,
}

async fn init_db() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("Server is running!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let db_pool = init_db().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health_check))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

