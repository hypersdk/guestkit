// SPDX-License-Identifier: Apache-2.0

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use super::jwt::verify_token;
use super::revoke::is_revoked;
use super::types::AuthUserClaims;
use crate::error::ApiError;
use crate::state::AppState;

pub const AUTH_USER_EXTENSION: &str = "guestkit.auth_user";

pub fn is_public_route(method: &str, path: &str) -> bool {
    matches!(
        (method, path),
        ("GET", "/api/v1/health")
            | ("GET", "/api/v1/auth/config")
            | ("GET", "/api/v1/auth/me")
            | ("GET", "/api/v1/auth/oidc/login")
            | ("GET", "/api/v1/auth/oidc/callback")
            | ("GET", "/api/v1/auth/saml/login")
            | ("POST", "/api/v1/auth/saml/acs")
            | ("POST", "/api/v1/auth/local")
            | ("GET", "/api/v1/settings/sso/saml/metadata")
    )
}

pub fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if !state.config.auth_enabled {
        return Ok(next.run(req).await);
    }

    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    if is_public_route(&method, &path) {
        return Ok(next.run(req).await);
    }

    let token = extract_bearer(req.headers()).ok_or_else(|| ApiError {
        status: StatusCode::UNAUTHORIZED,
        error: "UNAUTHORIZED".into(),
        message: "Bearer token required".into(),
    })?;

    let verified = verify_token(&state.config, &token).map_err(|_| ApiError {
        status: StatusCode::UNAUTHORIZED,
        error: "UNAUTHORIZED".into(),
        message: "invalid or expired token".into(),
    })?;

    if let Some(jti) = &verified.jti {
        let mut redis = state.redis.clone();
        if is_revoked(&mut redis, jti)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
        {
            return Err(ApiError {
                status: StatusCode::UNAUTHORIZED,
                error: "UNAUTHORIZED".into(),
                message: "token revoked".into(),
            });
        }
    }

    req.extensions_mut().insert(verified);
    Ok(next.run(req).await)
}

pub fn user_from_extensions(extensions: &axum::http::Extensions) -> Option<AuthUserClaims> {
    extensions.get::<AuthUserClaims>().cloned()
}

pub fn require_admin(user: &AuthUserClaims) -> Result<(), ApiError> {
    if super::rbac::is_admin(user) {
        Ok(())
    } else {
        Err(ApiError {
            status: StatusCode::FORBIDDEN,
            error: "FORBIDDEN".into(),
            message: "admin role required".into(),
        })
    }
}
