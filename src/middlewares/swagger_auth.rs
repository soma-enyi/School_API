use axum::{
    body::Body,
    http::{Request, Response, StatusCode, header},
    middleware::Next,
};
use base64::{engine::general_purpose, Engine};

const SWAGGER_USERNAME: &str = "admin";
const SWAGGER_PASSWORD: &str = "swagger123";

const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Swagger UI - Login</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: sans-serif; background: #f5f5f5; display: flex; align-items: center; justify-content: center; min-height: 100vh; }
    .card { background: #fff; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 12px rgba(0,0,0,.15); width: 100%; max-width: 360px; }
    h2 { margin-bottom: 1.5rem; color: #333; font-size: 1.25rem; }
    label { display: block; margin-bottom: .25rem; font-size: .875rem; color: #555; }
    input { width: 100%; padding: .6rem .75rem; border: 1px solid #ccc; border-radius: 4px; font-size: 1rem; margin-bottom: 1rem; }
    input:focus { outline: none; border-color: #4a90e2; }
    button { width: 100%; padding: .7rem; background: #4a90e2; color: #fff; border: none; border-radius: 4px; font-size: 1rem; cursor: pointer; }
    button:hover { background: #357abd; }
    .error { color: #c0392b; font-size: .875rem; margin-bottom: 1rem; display: none; }
  </style>
</head>
<body>
  <div class="card">
    <h2>🔒 Swagger UI Access</h2>
    <p id="error" class="error">Invalid username or password.</p>
    <form id="loginForm">
      <label for="username">Username</label>
      <input id="username" type="text" placeholder="admin" autocomplete="username" />
      <label for="password">Password</label>
      <input id="password" type="password" placeholder="Password" autocomplete="current-password" />
      <button type="submit">Login</button>
    </form>
  </div>
  <script>
    document.getElementById('loginForm').addEventListener('submit', function(e) {
      e.preventDefault();
      const u = encodeURIComponent(document.getElementById('username').value);
      const p = encodeURIComponent(document.getElementById('password').value);
      const url = window.location.protocol + '//' + u + ':' + p + '@' + window.location.host + window.location.pathname;
      // Validate first, then navigate
      fetch(window.location.href, {
        headers: { 'Authorization': 'Basic ' + btoa(decodeURIComponent(u) + ':' + decodeURIComponent(p)) }
      }).then(r => {
        if (r.ok) {
          window.location.href = url;
        } else {
          document.getElementById('error').style.display = 'block';
        }
      });
    });
  </script>
</body>
</html>"#;

pub async fn swagger_basic_auth(req: Request<Body>, next: Next) -> Response<Body> {
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(encoded) = auth_str.strip_prefix("Basic ") {
                if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                    if let Ok(credentials) = std::str::from_utf8(&decoded) {
                        if credentials == format!("{}:{}", SWAGGER_USERNAME, SWAGGER_PASSWORD) {
                            return next.run(req).await;
                        }
                    }
                }
            }
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(LOGIN_HTML))
        .unwrap()
}
