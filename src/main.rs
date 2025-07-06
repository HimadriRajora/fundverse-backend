// src/main.rs

use actix_cors::Cors;
use actix_web::{get, post, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::Serialize;
use sqlx::MySqlPool;
use std::env;

mod auth;
mod campaigns;
mod ml;
mod models;

#[derive(Serialize)]
struct Message {
    message: String,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello from FundVerse".into(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("DB connect failed");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::CONTENT_TYPE,
                        actix_web::http::header::ACCEPT,
                    ])
                    .max_age(3600),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(
                web::scope("/auth")
                    .route("/signup", web::post().to(auth::signup))
                    .route("/verify", web::post().to(auth::verify_email))
                    .route("/login", web::post().to(auth::login)),
            )
            .service(
                web::scope("/api/campaigns")
                    .route("", web::get().to(campaigns::list))
                    .route("", web::post().to(campaigns::create))
                    .route("/{id}", web::put().to(campaigns::update))
                    .route("/{id}/pledge", web::post().to(campaigns::pledge)),
            )
            .service(
                web::scope("/api/ml")
                    .service(ml::translate)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
