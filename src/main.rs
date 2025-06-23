use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use dotenv::dotenv;
use std::env;
use sqlx::MySqlPool;

mod models;
mod auth;
mod campaigns;

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
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url).await.expect("DB connect failed");

    println!("🚀 FundVerse backend listening on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(
                web::scope("/auth")
                    .route("/signup",  web::post().to(auth::signup))
                    .route("/verify",  web::post().to(auth::verify_email))
                    .route("/login",   web::post().to(auth::login)),
            )
            .service(
                web::scope("/api/campaigns")
                    .route("",              web::get().to(campaigns::list))
                    .route("",              web::post().to(campaigns::create))
                    .route("/{id}",         web::put().to(campaigns::update))
                    .route("/{id}/pledge",  web::post().to(campaigns::pledge)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
