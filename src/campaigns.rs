use actix_web::{web, HttpResponse};
use sqlx::MySqlPool;
use crate::models::{Campaign, CampaignData, Pledge, PledgeData};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page:  Option<u32>,
    pub limit: Option<u32>,
}

pub async fn list(
    pool: web::Data<MySqlPool>,
    query: web::Query<Pagination>,
) -> HttpResponse {
    let limit  = query.limit.unwrap_or(10) as i64;
    let offset = ((query.page.unwrap_or(1) - 1) * limit as u32) as i64;

    match sqlx::query_as!(
        Campaign,
        r#"
        SELECT id, owner_id, title, description,
               goal_amount, created_at
          FROM campaigns
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(_)   => HttpResponse::InternalServerError().body("DB error"),
    }
}

pub async fn create(
    pool: web::Data<MySqlPool>,
    data: web::Json<CampaignData>,
) -> HttpResponse {
    let owner_id = 1u64;

    let res = sqlx::query!(
        r#"
        INSERT INTO campaigns (owner_id, title, description, goal_amount)
        VALUES (?, ?, ?, ?)
        "#,
        owner_id,
        data.title,
        data.description,
        data.goal_amount
    )
    .execute(pool.get_ref())
    .await;

    if let Err(_) = res {
        return HttpResponse::InternalServerError().body("Insert failed");
    }
    let id = res.unwrap().last_insert_id();

    match sqlx::query_as!(
        Campaign,
        r#"
        SELECT id, owner_id, title, description,
               goal_amount, created_at
          FROM campaigns
         WHERE id = ?
        "#,
        id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => HttpResponse::Created().json(row),
        Err(_)  => HttpResponse::InternalServerError().body("Fetch failed"),
    }
}

pub async fn update(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    data: web::Json<CampaignData>,
) -> HttpResponse {
    let id = *path;

    if let Err(_) = sqlx::query!(
        r#"
        UPDATE campaigns
           SET title = ?, description = ?, goal_amount = ?
         WHERE id = ?
        "#,
        data.title,
        data.description,
        data.goal_amount,
        id
    )
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError().body("Update failed");
    }

    match sqlx::query_as!(
        Campaign,
        r#"
        SELECT id, owner_id, title, description,
               goal_amount, created_at
          FROM campaigns
         WHERE id = ?
        "#,
        id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => HttpResponse::Ok().json(row),
        Err(_)  => HttpResponse::InternalServerError().body("Fetch failed"),
    }
}

pub async fn pledge(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    data: web::Json<PledgeData>,
) -> HttpResponse {
    let campaign_id = *path;
    let user_id = 1u64;

    let res = sqlx::query!(
        r#"
        INSERT INTO pledges (user_id, campaign_id, amount)
        VALUES (?, ?, ?)
        "#,
        user_id,
        campaign_id,
        data.amount
    )
    .execute(pool.get_ref())
    .await;

    if let Err(_) = res {
        return HttpResponse::InternalServerError().body("Insert pledge failed");
    }
    let last_id = res.unwrap().last_insert_id();

    match sqlx::query_as!(
        Pledge,
        r#"
        SELECT id, user_id, campaign_id,
               amount AS "amount!: _",
               pledged_at
          FROM pledges
         WHERE id = ?
        "#,
        last_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => HttpResponse::Created().json(row),
        Err(_)  => HttpResponse::InternalServerError().body("Fetch pledge failed"),
    }
}

pub async fn list_owned(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> HttpResponse {
    let user_id = *path;

    match sqlx::query_as!(
        Campaign,
        r#"
        SELECT id, owner_id, title, description,
               goal_amount, created_at
          FROM campaigns
         WHERE owner_id = ?
         ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(_)   => HttpResponse::InternalServerError().body("DB error"),
    }
}

pub async fn list_pledged(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> HttpResponse {
    let user_id = *path;

    match sqlx::query_as!(
        Campaign,
        r#"
        SELECT c.id, c.owner_id, c.title, c.description,
               c.goal_amount, c.created_at
          FROM campaigns c
          JOIN pledges p ON p.campaign_id = c.id
         WHERE p.user_id = ?
         GROUP BY c.id
         ORDER BY c.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(_)   => HttpResponse::InternalServerError().body("DB error"),
    }
}
