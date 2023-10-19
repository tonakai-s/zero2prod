use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct NewSubFormData {
    name: String,
    email: String,
}

pub async fn subscribe(data: web::Form<NewSubFormData>, pool: web::Data<PgPool>) -> HttpResponse {
    println!("Name: {}", data.name);
    println!("Email: {}", data.email);

    match sqlx::query!(
        r#"
            INSERT INTO subscriptions(id, email, name, subscribed_at)
            VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Error inserting the new subscriber: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
