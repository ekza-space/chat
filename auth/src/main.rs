use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use axum::{
    extract::Form,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/token", post(token))
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

async fn register(Form(login): Form<Login>) -> String {
    let pswd_hash = hash_password(&login.password.as_str());
    format!(
        "Username: {}, Password: {}, Hash: {}",
        login.username,
        login.password,
        pswd_hash.unwrap()
    )
    // TODO: store hash to db
}

async fn token(Form(login): Form<Login>) -> String {
    let hash = "hash from db".to_string();
    match verify_password(&login.password, &hash) {
        Ok(()) => {
            println!("Password is correct");
            "jwt token".to_string()
        }
        Err(_) => {
            println!("Wrong password");
            "error".to_string()
        }
    }
}
