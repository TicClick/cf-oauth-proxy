use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    /// OAuth path suffix where users will land. Example: /steel/oauth
    pub oauth_init_uri_suffix: String,
    /// OAuth callback suffix. Example: /steel/oauth/callback
    pub redirect_uri_suffix: String,
    /// Authorization page URL. Example: https://osu.ppy.sh/oauth/authorize
    pub authorization_url: String,
    /// OAuth token endpoint. Example: https://osu.ppy.sh/oauth/token
    pub token_url: String,
}

impl OAuthConfig {
    pub fn from_env(env: &Env) -> Result<Self> {
        Ok(Self {
            client_id: env.var("OAUTH_CLIENT_ID")?.to_string(),
            client_secret: env.secret("OAUTH_CLIENT_SECRET")?.to_string(),
            oauth_init_uri_suffix: env.var("OAUTH_INIT_URI_SUFFIX")?.to_string(),
            redirect_uri_suffix: env.var("OAUTH_REDIRECT_URI_SUFFIX")?.to_string(),
            authorization_url: env.var("OAUTH_AUTHORIZATION_URL")?.to_string(),
            token_url: env.var("OAUTH_TOKEN_URL")?.to_string(),
        })
    }
}
