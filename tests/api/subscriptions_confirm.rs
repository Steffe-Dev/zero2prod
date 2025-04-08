use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};
use zero2prod::domain::SubcriptionToken;

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn confirmations_with_non_existant_token_are_rejected_with_a_401() {
    // Arrange
    let app = spawn_app().await;
    let token = SubcriptionToken::generate();

    // Act
    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token={}",
        &app.address,
        token.as_ref()
    ))
    .await
    .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn confirmations_with_invalid_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;
    let token = "mytoken";

    // Act
    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token={}",
        &app.address, token
    ))
    .await
    .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(email_request);

    // Act
    let response = reqwest::get(links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_a_confirmation_link_confirms_the_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(email_request);

    // Act
    reqwest::get(links.html).await.unwrap();

    // Assert
    let saved_record = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.pg_pool)
        .await
        .expect("Failed to fetch the saved subscription.");

    assert_eq!(saved_record.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved_record.name, "le guin");
    assert_eq!(saved_record.status, "confirmed");
}

#[tokio::test]
async fn clicking_a_confirmation_link_twice_has_no_further_effect() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(email_request);

    // Act
    reqwest::get(links.html.clone()).await.unwrap();
    reqwest::get(links.html).await.unwrap();

    // Assert
    let saved_record = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.pg_pool)
        .await
        .expect("Failed to fetch the saved subscription.");

    assert_eq!(saved_record.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved_record.name, "le guin");
    assert_eq!(saved_record.status, "confirmed");
}
