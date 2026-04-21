use axum::{
    body::Body,
    http::{Request, Response, StatusCode, header},
    middleware::Next,
};
use base64::{engine::general_purpose, Engine};

const SWAGGER_USERNAME: &str = "admin";
const SWAGGER_PASSWORD: &str = "swagger123";

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
        .header(header::WWW_AUTHENTICATE, r#"Basic realm="Swagger UI""#)
        .body(Body::from("Unauthorized"))
        .unwrap()
}
