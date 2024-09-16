use std::{
    fs::{self, File},
    io::{Error, Write},
};

mod jwt_logic;

use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use axum::{
    extract::Form,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio;
use tower_http::trace::TraceLayer;

static DB_PATH: &str = "db/users";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/signin", post(sign_in))
        .route("/register", post(register))
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

fn save_user(data: &UserPassHash) -> Result<(), Error> {
    let json_data = serde_json::to_string(data)?;
    let mut file = File::create(format!("{}/{}", DB_PATH, data.user))?;
    file.write_all(json_data.as_bytes())
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

    let data = UserPassHash {
        user: login.username.clone(),
        hash: pswd_hash.clone(),
    };

    let saved = save_user(&data);

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
    let file_path = format!("{}/{}", DB_PATH, login.username);
    let file_content = match fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Register first".to_string()),
    };
    let user_model: Result<UserPassHash, serde_json::Error> = serde_json::from_str(&file_content);
    let user_model = match user_model {
        Ok(model) => model,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid user data".to_string()),
    };

    match verify_password(&login.password, &user_model.hash) {
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
            (StatusCode::UNAUTHORIZED, "error".to_string())
        }
    }
}
