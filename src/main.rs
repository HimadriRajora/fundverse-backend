use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use dotenv::dotenv;
use std::env;
use sqlx::MySqlPool;

mod models;
mod auth;

#[derive(Serialize)]
struct Message {
    message: String,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello from FundVerse".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env variables
    dotenv().ok();

    // Enable the logger (reads RUST_LOG env var)
    env_logger::init();

    // Read DATABASE_URL and connect
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Print a startup message
    println!("🚀 FundVerse backend listening on http://127.0.0.1:8080");

    // Build and run the server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())               // << enable request logging
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(
                web::scope("/auth")
                    .route("/signup", web::post().to(auth::signup))
                    .route("/verify", web::post().to(auth::verify_email))
                    .route("/login", web::post().to(auth::login)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
