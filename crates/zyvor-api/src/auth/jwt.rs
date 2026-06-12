// SPDX-License-Identifier: Apache-2.0

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::types::AuthUserClaims;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    role: String,
    provider: String,
    iss: String,
    exp: usize,
    iat: usize,
}

pub fn issue_token(config: &Config, user: &AuthUserClaims) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + config.jwt_expiry_secs();
    let claims = JwtClaims {
        sub: user.sub.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        provider: user.provider.clone(),
        iss: config.jwt_issuer.clone(),
        exp,
        iat: now,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
}

pub fn verify_token(config: &Config, token: &str) -> Result<AuthUserClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[config.jwt_issuer.as_str()]);
    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )?;
    Ok(AuthUserClaims {
        sub: data.claims.sub,
        email: data.claims.email,
        name: data.claims.name,
        role: data.claims.role,
        provider: data.claims.provider,
    })
}
