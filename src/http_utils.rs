use worker::*;

use crate::{config::OAuthConfig, token::TokenResponse};

pub fn parse_url(url_str: &str) -> Result<url::Url> {
    url::Url::parse(url_str).map_err(|e| Error::RustError(e.to_string()))
}

pub fn make_redirect_uri(req: &HttpRequest, config: &OAuthConfig) -> Result<url::Url> {
    let host = req
        .uri()
        .host()
        .ok_or_else(|| Error::RustError("Missing host from URI".to_owned()))?;

    let scheme = req
        .uri()
        .scheme_str()
        .ok_or_else(|| Error::RustError("Missing scheme from URI".to_owned()))?;

    let url_str = match req.uri().port_u16() {
        Some(port) => format!(
            "{}://{}:{}{}",
            scheme, host, port, config.redirect_uri_suffix
        ),
        None => format!("{}://{}{}", scheme, host, config.redirect_uri_suffix),
    };

    url::Url::parse(&url_str).map_err(|e| Error::RustError(e.to_string()))
}

pub fn error_redirect(local_port: u16, error_msg: &str) -> Result<Response> {
    let redirect_url = url::Url::parse(&format!(
        "http://localhost:{}/?status=error&error={}",
        local_port,
        urlencoding::encode(error_msg)
    ))?;
    Response::redirect(redirect_url)
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

    Response::redirect(redirect_url)
}
