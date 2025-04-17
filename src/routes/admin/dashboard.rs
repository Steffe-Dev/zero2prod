use crate::{
    session_state::TypedSession,
    utility::{e500, see_other},
};
use actix_web::{
    HttpResponse,
    http::header::ContentType,
    web,
};
use anyhow::{Context, Ok};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // The deserialisation of the type we want from the session store (`Uuid` in this case)
    // could fail.
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return core::result::Result::Ok(see_other("/login"));
    };
    core::result::Result::Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
            </head>
            <body>
                <p>Welcome {username}!</p>
                <p>Available actions:</p>
                <ol>
                <li><a href="/admin/password">Change password</a></li>
                </ol>
            </body>
            </html>
            "#
            )),
    )
}

async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
