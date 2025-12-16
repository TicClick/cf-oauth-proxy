# cf-oauth-proxy

cf-oauth-proxy is a Cloudflare Worker proxy for concealing `client_secret` of your web app, while letting users get access and refresh tokens through it.

```mermaid
sequenceDiagram
    participant localhost
    participant proxy as cf-oauth-proxy
    participant server as OAuth server

    localhost->>+proxy: GET /oauth (local_port, scopes)
    proxy->>+server: GET /oauth/authorize (client_id, redirect_uri, scope, state)
    Note over server: (approve / deny)
    server-->>-proxy: Redirect to /oauth/callback (state, code)
    proxy->>+server: POST /oauth/token (code, client_secret)
    server-->>-proxy: access_token, refresh_token, expires_in
    proxy-->>-localhost: Redirect to local_port with tokens
```

## Build

https://developers.cloudflare.com/workers/languages/rust/
