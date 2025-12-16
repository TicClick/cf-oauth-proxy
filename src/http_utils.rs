use http::StatusCode;
use worker::*;

use crate::token::TokenResponse;

pub fn redirect_to(new_location: &str) -> Result<Response> {
    Response::empty().map(|r| {
        r.with_status(StatusCode::FOUND.into())
            .with_headers([("Location", new_location)].iter().collect())
    })
}

pub fn parse_url(url_str: &str) -> Result<url::Url> {
    url::Url::parse(url_str).map_err(|e| Error::RustError(e.to_string()))
}

pub fn error_redirect(local_port: u16, error_msg: &str) -> Result<Response> {
    let redirect_url = format!(
        "http://localhost:{}/?status=error&error={}",
        local_port,
        urlencoding::encode(error_msg)
    );
    redirect_to(&redirect_url)
}

pub fn success_redirect(local_port: u16, tokens: &TokenResponse) -> Result<Response> {
    let mut redirect_url = url::Url::parse(&format!("http://localhost:{}/", local_port))
        .map_err(|e| Error::RustError(e.to_string()))?;

    redirect_url
        .query_pairs_mut()
        .append_pair("status", "ok")
        .append_pair("access_token", &tokens.access_token);

    if let Some(ref refresh_token) = tokens.refresh_token {
        redirect_url
            .query_pairs_mut()
            .append_pair("refresh_token", refresh_token);
    }

    redirect_to(redirect_url.as_str())
}
