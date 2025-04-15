use actix_web::{HttpResponse, http::header::ContentType};

pub async fn home() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        // Includes file contents at compile time!!!
        .body(include_str!("./home/home.html"))
}
