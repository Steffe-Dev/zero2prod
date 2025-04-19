use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use reqwest::Url;
use secrecy::SecretString;
use wiremock::MockServer;
use zero2prod::startup::{Application, get_connection_pool};

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use zero2prod::configuration::DatabaseSettings;
use zero2prod::telemetry;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub pg_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn login(&self, app: &TestApp) -> reqwest::Response {
        let login_body = serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        });
        app.post_login(&login_body).await
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut OsRng);
        // Match params of the default password
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to create test user.");
    }
}

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let endpoint = format!("{}/subscriptions", &self.address);
        self.api_client
            .post(endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_newsletters(&self) -> reqwest::Response {
        let endpoint = format!("{}/admin/newsletters", &self.address);
        self.api_client
            .get(endpoint)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_newsletters_html(&self) -> String {
        self.get_newsletters()
            .await
            .text()
            .await
            .expect("Should return html as text")
    }

    pub async fn post_newsletters<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        let endpoint = format!("{}/admin/newsletters", &self.address);
        self.api_client
            .post(endpoint)
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        let endpoint = format!("{}/login", &self.address);
        self.api_client
            .post(endpoint)
            // This `reqwest` method makes sure that the body is URL-encoded
            // and the `Content-Type` header is set accordingly
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_login_html(&self) -> String {
        let endpoint = format!("{}/login", &self.address);
        self.api_client
            .get(&endpoint)
            .send()
            .await
            .expect("Failed to execute request")
            .text()
            .await
            .expect("Could not get the response as text")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        let endpoint = format!("{}/admin/password", &self.address);
        self.api_client
            .post(endpoint)
            // This `reqwest` method makes sure that the body is URL-encoded
            // and the `Content-Type` header is set accordingly
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        let endpoint = format!("{}/admin/password", &self.address);
        self.api_client
            .get(&endpoint)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password()
            .await
            .text()
            .await
            .expect("Should return html as text")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        let endpoint = format!("{}/admin/dashboard", &self.address);
        self.api_client
            .get(&endpoint)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard()
            .await
            .text()
            .await
            .expect("Could not get the response as text")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        let endpoint = format!("{}/admin/logout", &self.address);
        self.api_client
            .post(endpoint)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let confirmation_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&confirmation_link).unwrap();
            // We don't want to call random web API's
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        // Parse the body as JSON, startign from raw bytes
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
        let text_link = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks {
            html: html_link,
            plain_text: text_link,
        }
    }
}

/// Spin up an instance of the web server and return its address (i.e. http://localhost:XXXXX)
pub async fn spawn_app() -> TestApp {
    // The first time `initialise` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = zero2prod::configuration::get_configuration().expect("Failed to read config");
        c.database.database_name = Uuid::new_v4().to_string();
        // Assign random OS port
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build app");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);

    // Tokio spins up a new runtime for each test, shutting down and cleaning up
    // after the test ran. Therefore, no cleanup needed.
    let _ = tokio::spawn(app.run_until_stopped());

    let api_client = reqwest::Client::builder()
        // Prevent `reqwest` from automatically redirecting
        // for test purposes
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .expect("Failed to build client");

    let test_app = TestApp {
        address,
        pg_pool: get_connection_pool(&configuration.database),
        email_server,
        port,
        test_user: TestUser::generate(),
        api_client,
    };
    test_app.test_user.store(&test_app.pg_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create DB
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::from("password"),
        ..config.clone()
    };
    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate DB
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
pub async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // This mock is scoped to only this function, it gets dropped
    // at the end, and it's expectations are eagerly validated
    // This is due to `mount_as_scoped`
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

pub async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
