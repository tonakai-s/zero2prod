use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct NewSubFormData {
    name: String,
    email: String,
}

pub async fn subscribe(data: web::Form<NewSubFormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        data.name,
        data.email,
        request_id
    );

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
    .await
    {
        Ok(_) => {
            log::info!(
                "request_id {} - New subscriber details have been saved.",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!(
                "request_id {} - Error inserting the new subscriber: {}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
