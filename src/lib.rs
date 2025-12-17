use http::StatusCode;
use rand::Rng;
use worker::*;

use crate::{
    config::OAuthConfig,
    http_utils::{error_redirect, make_redirect_uri, parse_url, success_redirect},
    state::{decode_state, encode_state, StateData},
    token::{TokenErrorResponse, TokenResponse},
};

pub(crate) mod config;
pub(crate) mod http_utils;
pub(crate) mod state;
pub(crate) mod token;

async fn handle_oauth_start(req: HttpRequest, env: &Env) -> Result<Response> {
    let config = OAuthConfig::from_env(env)?;

    let url = req.uri().to_string();
    let parsed_url = parse_url(&format!("http://dummy{}", url))?;

    let mut local_port: Option<u16> = None;
    let mut scopes = String::new();

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "local_port" => {
                local_port = value.parse().ok();
            }
            "scopes" => scopes = value.to_string(),
            _ => {}
        }
    }

    let local_port = match local_port {
        Some(port) => port,
        None => {
            return Response::ok("Missing local_port parameter")
                .map(|r| r.with_status(StatusCode::BAD_REQUEST.into()));
        }
    };

    let mut rng = rand::thread_rng();
    let nonce: String = (0..16)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();

    let state_data = StateData { local_port, nonce };

    let state = encode_state(&state_data)?;

    let mut auth_url = parse_url(&config.authorization_url)?;
    let redirect_uri = make_redirect_uri(&req, &config)?;

    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", redirect_uri.as_str())
        .append_pair("response_type", "code")
        .append_pair("state", &state)
        .append_pair("scope", &scopes);

    Response::redirect(auth_url)
}

async fn handle_oauth_callback(req: HttpRequest, env: &Env) -> Result<Response> {
    let config = OAuthConfig::from_env(env)?;

    let url = req.uri().to_string();
    let parsed_url = parse_url(&format!("http://dummy{}", url))?;

    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;
    let mut error_description: Option<String> = None;

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            "error" => error = Some(value.to_string()),
            "error_description" => error_description = Some(value.to_string()),
            _ => {}
        }
    }

    let state_data = match state {
        Some(ref state_token) => decode_state(state_token)?,
        None => {
            return Response::ok("Missing state parameter")
                .map(|r| r.with_status(StatusCode::BAD_REQUEST.into()));
        }
    };

    let local_port = state_data.local_port;

    if let Some(err) = error {
        let error_desc = error_description.unwrap_or_else(|| "Unknown error".to_string());
        let error_msg = format!("{}: {}", err, error_desc);
        return error_redirect(local_port, &error_msg);
    }

    let code = match code {
        Some(c) => c,
        None => return error_redirect(local_port, "Missing authorization code"),
    };

    let redirect_uri = make_redirect_uri(&req, &config)?;

    let token_params = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
        urlencoding::encode(&code),
        urlencoding::encode(redirect_uri.as_str()),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.client_secret)
    );

    let token_request = Request::new_with_init(
        &config.token_url,
        RequestInit::new()
            .with_method(Method::Post)
            .with_body(Some(token_params.into()))
            .with_headers({
                let headers = Headers::new();
                headers.set("Content-Type", "application/x-www-form-urlencoded")?;
                headers
            }),
    )?;

    let mut token_response = Fetch::Request(token_request).send().await?;

    let status = token_response.status_code();
    let body_text = token_response.text().await?;

    if (200..300).contains(&status) {
        match serde_json::from_str::<TokenResponse>(&body_text) {
            Ok(tokens) => success_redirect(local_port, &tokens),
            Err(e) => error_redirect(
                local_port,
                &format!("Failed to parse token response: {}", e),
            ),
        }
    } else {
        let error_msg = match serde_json::from_str::<TokenErrorResponse>(&body_text) {
            Ok(err_resp) => format!(
                "{}: {}",
                err_resp.error,
                err_resp
                    .error_description
                    .unwrap_or_else(|| "No description".to_string())
            ),
            Err(_) => format!("Token request failed with status {}: {}", status, body_text),
        };

        error_redirect(local_port, &error_msg)
    }
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> Result<Response> {
    let config = OAuthConfig::from_env(&env)?;

    match req.method() {
        &http::Method::GET => {
            let path = req.uri().path();
            if path == "/" {
                Response::ok("OK").map(|r| r.with_status(StatusCode::OK.into()))
            } else if path == config.oauth_init_uri_suffix {
                handle_oauth_start(req, &env).await
            } else if path == config.redirect_uri_suffix {
                handle_oauth_callback(req, &env).await
            } else {
                Response::error("Not found", StatusCode::NOT_FOUND.into())
            }
        }
        _ => Response::error("Method not allowed", StatusCode::METHOD_NOT_ALLOWED.into()),
    }
}
