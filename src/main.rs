use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use dotenvy::dotenv;
use std::env;
use sqlx::MySqlPool;

// Define a serializable struct for our JSON response
#[derive(Serialize)]
struct Message {
    message: String,
}

// Basic GET handler for the "/" route
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello from FundVerse".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from `.env` file
    dotenv().ok(); // This loads DATABASE_URL into std::env:contentReference[oaicite:9]{index=9}

    // Read the DATABASE_URL environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    
    // Create a connection pool to the MySQL database
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database:contentReference[oaicite:10]{index=10}");

    // Start the Actix-web HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Share DB pool with handlers:contentReference[oaicite:11]{index=11}
            .service(index) // Register the index handler for GET "/"
    })
    .bind(("127.0.0.1", 8080))? // Listen on port 8080 on localhost
    .run()
    .await
}
