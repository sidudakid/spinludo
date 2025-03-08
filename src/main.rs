use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::middleware::Logger;
use sqlx::{PgPool, postgres::PgPoolOptions, types::BigDecimal, FromRow};
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;
use actix_files::Files;
use actix_web::http::header::ContentType;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
    balance: BigDecimal,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Game {
    id: i32,
    player1_id: i32,
    player2_id: Option<i32>,
    entry_fee: BigDecimal,
    owner_cut: BigDecimal,
    status: String,
}

async fn api_documentation() -> impl Responder {
    let html = include_str!("../templates/index.html");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}

async fn landing_page() -> impl Responder {
    let html = include_str!("../templates/landing.html");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
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

#[derive(Debug, Deserialize)]
struct CreateGameRequest {
    entry_fee: BigDecimal,
    owner_cut: BigDecimal,
}

// Create a new game
async fn create_game(
    db: web::Data<PgPool>,
    req: web::Json<CreateGameRequest>,
) -> impl Responder {
    let player1_id = 1; // Example for player 1. Replace with actual player ID.
    let status = "waiting";

    let game = sqlx::query_as::<_, Game>(
        r#"
        INSERT INTO games (player1_id, entry_fee, owner_cut, status)
        VALUES ($1, $2, $3, $4)
        RETURNING id, player1_id, player2_id, entry_fee, owner_cut, status
        "#
    )
    .bind(player1_id)
    .bind(&req.entry_fee)
    .bind(&req.owner_cut)
    .bind(status)
    .fetch_one(&**db)
    .await;

    match game {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(_) => HttpResponse::InternalServerError().json("Error creating game"),
    }
}

#[derive(Debug, Deserialize)]
struct JoinGameRequest {
    player2_id: i32,
}

// Player 2 joins the game
async fn join_game(
    db: web::Data<PgPool>,
    path: web::Path<i32>,
    req: web::Json<JoinGameRequest>,
) -> impl Responder {
    let game_id = path.into_inner();
    let result = sqlx::query(
        r#"
        UPDATE games
        SET player2_id = $1, status = 'active'
        WHERE id = $2 AND player2_id IS NULL
        "#
    )
    .bind(req.player2_id)
    .bind(game_id)
    .execute(&**db)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Game joined successfully"),
        Err(_) => HttpResponse::InternalServerError().json("Error joining game"),
    }
}

// Start the game when both players are present
async fn start_game(
    db: web::Data<PgPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let game_id = path.into_inner();
    let result = sqlx::query(
        r#"
        UPDATE games
        SET status = 'active'
        WHERE id = $1 AND status = 'waiting'
        "#
    )
    .bind(game_id)
    .execute(&**db)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Game started"),
        Err(_) => HttpResponse::InternalServerError().json("Error starting game"),
    }
}

#[derive(Debug, Deserialize)]
struct EndGameRequest {
    winner_id: i32,
}

// End the game, update balances, and determine the winner
async fn end_game(
    db: web::Data<PgPool>,
    path: web::Path<i32>,
    req: web::Json<EndGameRequest>,
) -> impl Responder {
    let game_id = path.into_inner();
    let game = sqlx::query_as::<_, Game>(
        r#"
        SELECT id, player1_id, player2_id, entry_fee, owner_cut, status
        FROM games
        WHERE id = $1
        "#
    )
    .bind(game_id)
    .fetch_one(&**db)
    .await;

    match game {
        Ok(game) => {
            // Calculate the total winnings
            let total_pool = game.entry_fee.clone() * BigDecimal::from(2);
            let owner_share = total_pool.clone() * (game.owner_cut / BigDecimal::from(100));
            let winner_share = total_pool - owner_share.clone();

            // Update players' balances
            let _ = sqlx::query(
                r#"
                UPDATE users
                SET balance = balance - $1
                WHERE id = $2
                "#
            )
            .bind(&game.entry_fee)
            .bind(game.player1_id)
            .execute(&**db)
            .await;

            if let Some(player2_id) = game.player2_id {
                let _ = sqlx::query(
                    r#"
                    UPDATE users
                    SET balance = balance - $1
                    WHERE id = $2
                    "#
                )
                .bind(&game.entry_fee)
                .bind(player2_id)
                .execute(&**db)
                .await;
            }

            let _ = sqlx::query(
                r#"
                UPDATE users
                SET balance = balance + $1
                WHERE id = $2
                "#
            )
            .bind(&winner_share)
            .bind(req.winner_id)
            .execute(&**db)
            .await;

            // Owner receives their cut
            let _ = sqlx::query(
                r#"
                UPDATE users
                SET balance = balance + $1
                WHERE id = $2
                "#
            )
            .bind(&owner_share)
            .bind(game.player1_id)
            .execute(&**db)
            .await;

            HttpResponse::Ok().json("Game ended, balances updated")
        }
        Err(_) => HttpResponse::InternalServerError().json("Error ending game"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let db_pool = init_db().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db_pool.clone()))
            .service(Files::new("/static", "./static").show_files_listing())
            .route("/api/json", web::get().to(api_documentation))
            .route("/", web::get().to(landing_page))
            .service(web::scope("/api")
                .route("/health", web::get().to(health_check))
                .route("/games", web::post().to(create_game))
                .route("/games/{id}/join", web::post().to(join_game))
                .route("/games/{id}/start", web::post().to(start_game))
                .route("/games/{id}/end", web::post().to(end_game))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}