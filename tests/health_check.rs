use sqlx::{Connection, Executor, PgPool, PgConnection};
use uuid::Uuid;
use std::net::TcpListener;

use zero2prod::configuration::{get_configuration, DatabaseSettings};

pub struct TestApp{
    pub address: String,
    pub connection_pool: PgPool
}

pub async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Faile to read the configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = zero2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        connection_pool
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    println!("Received config: {:?}", config.connection_string());
    
    // Create database
    let mut connection = PgConnection::connect(
        &config.connection_string_without_db()
    )
    .await
    .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate Database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app().await.address;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    let url = format!("{}/health_check", address);

    // Act
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to execute the request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    // Arrange
    let connection = test_app.connection_pool;

    let address = test_app.address;
    let client = reqwest::Client::new();
    let url = format!("{}/subscriptions", &address);

    // Act
    let body = "name=le%20renas&email=rdorneles%40gmail.com";
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT * FROM subscriptions")
        .fetch_one(&connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "rdorneles@gmail.com");
    assert_eq!(saved.name, "le renas");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let url = format!("{}/subscriptions", &address);

    let test_cases = vec![
        ("name=le%20renas", "missing the email"),
        ("email=rdorneles%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute the request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
