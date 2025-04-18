use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_change_password().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let body = serde_json::json!({
        "current_password": Uuid::new_v4().to_string(),
        "new_password": &new_password,
        "new_password_check": &new_password,
    });

    // Act
    let response = app.post_change_password(&body).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act - Part 1 - Log in
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    app.post_login(&login_body).await;

    // Act - Part 2 - Try to change password
    let body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &another_new_password,
    });
    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - \
        the field values must match.</i></p>"
    ));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    // Arrange
    let app = spawn_app().await;
    let wrong_current_password = Uuid::new_v4().to_string();
    let new_password = Uuid::new_v4().to_string();

    // Act - Part 1 - Log in
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    app.post_login(&login_body).await;

    // Act - Part 2 - Try to change password
    let body = serde_json::json!({
        "current_password": &wrong_current_password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });
    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn new_password_must_be_long_enough() {
    // Arrange
    let app = spawn_app().await;
    let new_password = "123456789012"; // Only 12 chars

    // Act - Part 1 - Log in
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    app.post_login(&login_body).await;

    // Act - Part 2 - Try to change password
    let body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });
    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(
        html_page.contains("<p><i>The new password is too short, should be 12 < p < 129.</i></p>")
    );
}

#[tokio::test]
async fn new_password_must_be_short_enough() {
    // Arrange
    let app = spawn_app().await;
    let new_password = "a".repeat(129); // 129 chars

    // Act - Part 1 - Log in
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    app.post_login(&login_body).await;

    // Act - Part 2 - Try to change password
    let body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });
    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(
        html_page.contains("<p><i>The new password is too long, should be 12 < p < 129.</i></p>")
    );
}

#[tokio::test]
async fn logout_clears_session_state() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act - Part 1 - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Change password
    let body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });
    let response = app.post_change_password(&body).await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Part 3 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed.</i></p>"));

    // Act - Part 4 - Log out
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act - Part 5 - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    // Act - Part 6 - Log in again using new password
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &new_password
    });
    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");
}
