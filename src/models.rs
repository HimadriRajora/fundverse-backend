use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct SignupData {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyData {
    pub email: String,
    pub otp: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(FromRow, Serialize)]
pub struct Campaign {
    pub id: u64,
    pub owner_id: u64,
    pub title: String,
    pub description: String,
    pub goal_amount: BigDecimal,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct CampaignData {
    pub title: String,
    pub description: String,
    pub goal_amount: BigDecimal,
}

#[derive(FromRow, Serialize)]
pub struct Pledge {
    pub id: u64,
    pub user_id: u64,
    pub campaign_id: u64,
    pub amount: BigDecimal,
    pub pledged_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct PledgeData {
    pub amount: f64,
}
