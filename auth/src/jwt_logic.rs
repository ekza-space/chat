use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use sha2::Sha384;

pub fn create_jwt(user_name: &str, exp_min: i64) -> Result<String, String> {
    let key = Hmac::<Sha384>::new_from_slice(b"secret").map_err(|_| "Invalid key length")?;

    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };

    let now = Utc::now();
    let exp = now + Duration::minutes(exp_min);

    let mut claims = BTreeMap::new();
    claims.insert("user_name", user_name.to_string());
    claims.insert("iat", now.timestamp().to_string());
    claims.insert("exp", exp.timestamp().to_string());

    let token = Token::new(header, claims)
        .sign_with_key(&key)
        .map_err(|_| "Failed to sign token")?;

    Ok(token.into())
}
