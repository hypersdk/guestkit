// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

use super::types::{AuthUserClaims, OidcDiscovery, OidcSettings};
use crate::config::Config;

#[derive(Debug, Deserialize)]
struct RawDiscovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    userinfo_endpoint: Option<String>,
    #[serde(default)]
    jwks_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
}

pub struct OidcLoginRequest {
    pub state: String,
    pub code_verifier: String,
    pub authorize_url: String,
}

pub fn build_login_request(
    config: &Config,
    oidc: &OidcSettings,
    discovery: &OidcDiscovery,
) -> Result<OidcLoginRequest> {
    let mut state_bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    let mut verifier_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

    let redirect_uri = config.oidc_redirect_uri();
    let mut url = Url::parse(&discovery.authorization_endpoint)
        .context("invalid authorization_endpoint from discovery")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("response_type", "code");
        pairs.append_pair("client_id", &oidc.client_id);
        pairs.append_pair("redirect_uri", &redirect_uri);
        pairs.append_pair("scope", &oidc.scopes.join(" "));
        pairs.append_pair("state", &state);
        pairs.append_pair("code_challenge", &challenge);
        pairs.append_pair("code_challenge_method", "S256");
    }

    Ok(OidcLoginRequest {
        state,
        code_verifier,
        authorize_url: url.to_string(),
    })
}

pub async fn fetch_discovery(client: &Client, issuer_url: &str) -> Result<OidcDiscovery> {
    let issuer = issuer_url.trim_end_matches('/');
    let url = format!("{issuer}/.well-known/openid-configuration");
    let raw: RawDiscovery = client
        .get(&url)
        .send()
        .await
        .context("OIDC discovery request failed")?
        .error_for_status()
        .context("OIDC discovery returned error status")?
        .json()
        .await
        .context("OIDC discovery JSON parse failed")?;

    Ok(OidcDiscovery {
        issuer: raw.issuer,
        authorization_endpoint: raw.authorization_endpoint,
        token_endpoint: raw.token_endpoint,
        userinfo_endpoint: raw.userinfo_endpoint,
        jwks_uri: raw.jwks_uri,
    })
}

pub async fn exchange_code(
    client: &Client,
    config: &Config,
    oidc: &OidcSettings,
    discovery: &OidcDiscovery,
    code: &str,
    code_verifier: &str,
) -> Result<AuthUserClaims> {
    let redirect_uri = config.oidc_redirect_uri();
    let mut form = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri.as_str()),
        ("client_id", oidc.client_id.as_str()),
        ("code_verifier", code_verifier),
    ];
    let client_secret = oidc.client_secret.as_deref().unwrap_or("");
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret));
    }

    let token: TokenResponse = client
        .post(&discovery.token_endpoint)
        .form(&form)
        .send()
        .await
        .context("OIDC token exchange failed")?
        .error_for_status()
        .context("OIDC token endpoint error")?
        .json()
        .await
        .context("OIDC token JSON parse failed")?;

    if let Some(endpoint) = &discovery.userinfo_endpoint {
        let info: UserInfo = client
            .get(endpoint)
            .bearer_auth(&token.access_token)
            .send()
            .await
            .context("OIDC userinfo request failed")?
            .error_for_status()
            .context("OIDC userinfo error")?
            .json()
            .await
            .context("OIDC userinfo JSON parse failed")?;

        let sub = info
            .sub
            .or(info.preferred_username.clone())
            .ok_or_else(|| anyhow!("OIDC userinfo missing sub"))?;
        return Ok(AuthUserClaims {
            sub,
            email: info.email,
            name: info.name.or(info.preferred_username),
            role: config.default_role().to_string(),
            provider: "oidc".into(),
        });
    }

    Err(anyhow!(
        "OIDC provider did not expose userinfo_endpoint; configure an IdP with userinfo"
    ))
}
