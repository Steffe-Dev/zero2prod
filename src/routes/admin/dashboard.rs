use crate::{authentication::UserId, utility::e500};
use actix_web::{HttpResponse, http::header::ContentType, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    // The deserialisation of the type we want from the session store (`Uuid` in this case)
    // could fail.
    let username = get_username(**user_id, &pool).await.map_err(e500)?;
    Ok(HttpResponse::Ok()
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
                <li>
                    <a href="/admin/password">Change password</a>
                </li>
                <li>
                    <form name="logoutForm" action="/admin/logout" method="post">
                        <input type="submit" value="Logout">
                    </form>
                </li>
                </ol>
            </body>
            </html>
            "#
        )))
}

pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
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
