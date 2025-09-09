use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use std::env;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(username: &str) -> Result<String> {
    // .env 로드 (이미 로드돼 있으면 no-op)
    dotenv().ok();
    let secret = env::var("SECRET_KEY")?;

    let expiration = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600; // 1시간 유효
    let claims = Claims {
        sub: username.to_string(),
        exp: expiration as usize,
    };
 
    let key = EncodingKey::from_secret(secret.as_bytes());
    let token = encode(&Header::default(), &claims, &key)?;
    Ok(token)
}
 
pub fn decode_jwt(token: &str) -> Result<String> {
    dotenv().ok();
    let secret = env::var("SECRET_KEY")?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let data = decode::<Claims>(token, &key, &Validation::default())?;
    Ok(data.claims.sub)
}
