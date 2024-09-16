mod jwt_logic;
use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{
    extract::Form,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

extern crate database as db;

use serde::{Deserialize, Serialize};
use tokio;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/signin", post(sign_in))
        .route("/register", post(register))
        .route("/users", get(get_all_users))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UserPassHash {
    user: String,
    hash: String,
}

async fn get_all_users() -> impl IntoResponse {
    let users = db::repo::users::get_all_users();
    let usernames: Vec<String> = users.into_iter().map(|user| user.username).collect();
    (StatusCode::OK, Json(usernames)).into_response()
}

fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(OsRng);
    println!("salt: {}", salt.as_str());
    let hash_pass = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash_pass)
}

fn verify_password(password: &str, hash: &str) -> Result<(), password_hash::Error> {
    let argon = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    let resp = argon.verify_password(password.as_bytes(), &parsed_hash);
    resp
}

async fn register(Form(login): Form<Login>) -> (StatusCode, String) {
    let pswd_hash_w = hash_password(&login.password.as_str());

    let pswd_hash = match pswd_hash_w {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::BAD_REQUEST, "Unknown_error".to_string()),
    };

    let saved = db::repo::users::create_new_user(&login.username, &pswd_hash.clone());

    let response = format!(
        "Username: {}, Password: {}, Hash: {}",
        login.username, login.password, pswd_hash
    );

    match saved {
        Ok(_) => (StatusCode::OK, response),
        Err(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "User not created".to_string(),
        ),
    }
}

async fn sign_in(Form(login): Form<Login>) -> (StatusCode, String) {
    let user_result = db::repo::users::get_user_by_name(&login.username);

    match user_result {
        Ok(Some(user)) => match verify_password(&login.password, &user.password_hash) {
            Ok(()) => {
                println!("Password is correct");
                match jwt_logic::create_jwt(&login.username, 60) {
                    Ok(token) => (StatusCode::OK, token),
                    Err(e) => {
                        println!("JWT generation error: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Token generation error".to_string(),
                        )
                    }
                }
            }
            Err(_) => {
                println!("Wrong password");
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
        },
        Ok(None) => {
            println!("User not found");
            (StatusCode::UNAUTHORIZED, "User not found".to_string())
        }
        Err(e) => {
            println!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        }
    }
}
