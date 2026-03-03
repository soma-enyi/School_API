// Security scheme configuration for OpenAPI
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

/// Security scheme modifier to add JWT Bearer authentication
pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Enter your JWT token in the format: Bearer <token>",
                        ))
                        .build(),
                ),
            );
        }
    }
}
