// src/auth.rs

use actix_web::{web, HttpResponse};
use sqlx::MySqlPool;
use crate::models::{User, SignupData, VerifyData, LoginData};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use chrono::Utc;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use jsonwebtoken::{encode, Header, EncodingKey};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

// In‐memory OTP store (for dev/testing)
lazy_static! {
    static ref OTP_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: u64,
    username: String,
    exp: usize,
}

// ─────────── SIGNUP ───────────
pub async fn signup(
    pool: web::Data<MySqlPool>,
    form: web::Json<SignupData>,
) -> HttpResponse {
    let pool = pool.get_ref();

    // 1) Check for duplicate username/email
    if sqlx::query!("SELECT id FROM users WHERE username = ?", form.username)
        .fetch_optional(pool).await.unwrap().is_some()
    {
        return HttpResponse::Conflict().body("Username already taken");
    }
    if sqlx::query!("SELECT id FROM users WHERE email = ?", form.email)
        .fetch_optional(pool).await.unwrap().is_some()
    {
        return HttpResponse::Conflict().body("Email already registered");
    }

    // 2) Hash and insert
    let hashed = hash(&form.password, DEFAULT_COST).unwrap();
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, is_verified, created_at)
        VALUES (?, ?, ?, FALSE, NOW())
        "#,
        form.username, form.email, hashed
    )
    .execute(pool).await.unwrap();

    // 3) Generate & store OTP
    let otp = rand::thread_rng().gen_range(100000..999999).to_string();
    OTP_STORE.lock().unwrap().insert(form.email.clone(), otp.clone());

    // 4) Build & send email via STARTTLS on Mailtrap sandbox
    let smtp_host = std::env::var("SMTP_HOST").unwrap();
    let smtp_port: u16 = std::env::var("SMTP_PORT").unwrap().parse().unwrap();
    let smtp_user = std::env::var("SMTP_USERNAME").unwrap();
    let smtp_pass = std::env::var("SMTP_PASSWORD").unwrap();
    let from_addr = std::env::var("EMAIL_FROM").unwrap();

    let email = Message::builder()
        .from(from_addr.parse().unwrap())
        .to(form.email.parse().unwrap())
        .subject("Your FundVerse OTP")
        .body(format!("Your OTP code is: {}", otp))
        .unwrap();

    let creds = Credentials::new(smtp_user, smtp_pass);
    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(info) => println!("✉️  OTP email sent: {:#?}", info),
        Err(e)  => eprintln!("❌  SMTP send error: {:?}", e),
    }

    HttpResponse::Ok().body("Signup successful! Check Mailtrap for your OTP.")
}

// ─────────── VERIFY ───────────
pub async fn verify_email(
    pool: web::Data<MySqlPool>,
    form: web::Json<VerifyData>,
) -> HttpResponse {
    let pool = pool.get_ref();

    // Fetch user
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_verified, created_at \
         FROM users WHERE email = ?"
    )
    .bind(&form.email)
    .fetch_optional(pool)
    .await
    .unwrap();

    if user.is_none() {
        return HttpResponse::BadRequest().body("No such user");
    }

    // Check OTP
    let mut store = OTP_STORE.lock().unwrap();
    match store.remove(&form.email) {
        Some(code) if code == form.otp.trim() => {
            // Mark verified
            sqlx::query!(
                "UPDATE users SET is_verified = TRUE WHERE email = ?",
                form.email
            )
            .execute(pool).await.unwrap();

            HttpResponse::Ok().body("Email verified successfully")
        }
        _ => HttpResponse::BadRequest().body("Invalid or expired OTP"),
    }
}

// ─────────── LOGIN ───────────
pub async fn login(
    pool: web::Data<MySqlPool>,
    form: web::Json<LoginData>,
) -> HttpResponse {
    let pool = pool.get_ref();

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_verified, created_at \
         FROM users WHERE username = ?"
    )
    .bind(&form.username)
    .fetch_optional(pool)
    .await
    .unwrap();

    let user = match user {
        Some(u) => u,
        None    => return HttpResponse::Unauthorized().body("Invalid credentials"),
    };

    if !user.is_verified {
        return HttpResponse::Unauthorized().body("Email not verified");
    }
    if !verify(&form.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    // Create JWT
    let exp = Utc::now().timestamp() as usize + 86_400;
    let claims = Claims { sub: user.id, username: user.username.clone(), exp };
    let secret = std::env::var("JWT_SECRET").unwrap();
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .unwrap();

    // Set HttpOnly cookie
    let cookie = actix_web::cookie::Cookie::build("auth_token", token)
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({"message":"Login successful"}))
}
