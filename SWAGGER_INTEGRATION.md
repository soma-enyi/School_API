# Swagger/OpenAPI Integration Guide

## Overview

This document describes the Swagger/OpenAPI 3.0 integration that has been added to the School Management API.

## What's Been Implemented

### 1. Dependencies Added
- `utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }`
- `utoipa-swagger-ui = { version = "6.0", features = ["axum"] }`
- `proptest = "1.4"` (dev dependency for property-based testing)

### 2. Documentation Module (`src/docs/`)
- **`mod.rs`**: Contains the main `ApiDoc` struct with OpenAPI configuration
- **`security.rs`**: Implements JWT Bearer authentication scheme for Swagger UI

### 3. Annotated Models
All request/response models have been annotated with `#[derive(ToSchema)]` and include:
- Field-level examples and format hints
- Validation constraints (min_length, pattern, etc.)
- Struct-level example JSON

**Annotated Models:**
- User, UserResponse, UserRole
- RegisterRequest, LoginRequest, AuthResponse
- RefreshTokenRequest, TokenResponse
- Student, Mentor, School
- ErrorResponse (standardized error format)

### 4. Annotated Controllers
The following controllers have been fully annotated with `#[utoipa::path]`:

**Authentication Controller** (10 endpoints):
- POST `/auth/admin/register`
- POST `/auth/admin/login`
- POST `/auth/student/register`
- POST `/auth/student/login`
- POST `/auth/mentor/register`
- POST `/auth/mentor/login`
- POST `/auth/refresh`
- POST `/auth/logout` (protected)
- GET `/auth/me` (protected)
- POST `/auth/verify` (protected)

**Admin Controller** (5 endpoints):
- GET `/admin/dashboard` (protected)
- GET `/admin/users` (protected)
- GET `/admin/statistics` (protected)
- POST `/admin/users/{user_id}/deactivate` (protected)
- POST `/admin/users/{user_id}/activate` (protected)

**School Controller** (6 endpoints):
- GET `/admin/schools` (protected)
- POST `/admin/schools/create` (protected)
- GET `/admin/schools/{school_id}` (protected)
- PUT `/admin/schools/{school_id}` (protected)
- DELETE `/admin/schools/{school_id}` (protected)
- GET `/admin/schools/{school_id}/statistics` (protected)

### 5. Swagger UI Integration
Swagger UI is mounted at `/docs` and serves the OpenAPI specification at `/api-docs/openapi.json`.

## How to Use

### 1. Build the Project
```bash
cargo build --release
```

### 2. Run the Server
```bash
cargo run
```

The server will start on `http://localhost:3000`

### 3. Access Swagger UI
Open your browser and navigate to:
```
http://localhost:3000/docs
```

### 4. Test Endpoints

#### Testing Public Endpoints
1. Navigate to the "Authentication" section
2. Try the `/auth/admin/register` endpoint:
   - Click "Try it out"
   - Fill in the request body with example data
   - Click "Execute"

#### Testing Protected Endpoints
1. First, login using `/auth/admin/login` or similar
2. Copy the `access_token` from the response
3. Click the "Authorize" button at the top of Swagger UI
4. Enter: `Bearer <your_access_token>`
5. Click "Authorize"
6. Now you can test protected endpoints like `/admin/dashboard`

## API Documentation Features

### Security
- JWT Bearer authentication is documented
- Protected endpoints show a lock icon
- "Authorize" button allows you to set your token for all requests

### Request/Response Examples
- All models include example JSON
- Field descriptions and constraints are visible
- Response status codes are documented (200, 201, 400, 401, 403, 404, 500)

### Error Responses
All error responses follow a standardized format:
```json
{
  "error": "ErrorType",
  "message": "Human-readable error message",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Architecture

### OpenAPI Generation Flow
```
Models (ToSchema) → Controllers (utoipa::path) → ApiDoc → Swagger UI
```

1. **Models** derive `ToSchema` trait with field annotations
2. **Controllers** use `#[utoipa::path]` macro to document endpoints
3. **ApiDoc** aggregates all paths and schemas
4. **Swagger UI** renders the interactive documentation

### File Structure
```
src/
├── docs/
│   ├── mod.rs          # ApiDoc configuration
│   └── security.rs     # JWT security scheme
├── models/
│   ├── user.rs         # User models with ToSchema
│   ├── student.rs      # Student models with ToSchema
│   ├── mentor.rs       # Mentor models with ToSchema
│   └── school.rs       # School models with ToSchema
├── controllers/
│   ├── auth.rs         # Auth endpoints with utoipa::path
│   ├── admin.rs        # Admin endpoints with utoipa::path
│   └── school.rs       # School endpoints with utoipa::path
└── utils/
    └── errors.rs       # ErrorResponse with ToSchema
```

## Adding New Endpoints

To add documentation for a new endpoint:

### 1. Annotate the Model
```rust
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MyRequest {
    #[schema(example = "example value")]
    pub field: String,
}
```

### 2. Annotate the Handler
```rust
#[utoipa::path(
    post,
    path = "/api/my-endpoint",
    tag = "MyTag",
    request_body = MyRequest,
    responses(
        (status = 200, description = "Success", body = MyResponse),
        (status = 400, description = "Bad request", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])  // If endpoint requires auth
    )
)]
pub async fn my_handler(
    Json(payload): Json<MyRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Implementation
}
```

### 3. Add to ApiDoc
In `src/docs/mod.rs`, add the path to the `paths()` section:
```rust
paths(
    // ... existing paths ...
    crate::controllers::MyController::my_handler,
),
```

### 4. Add Schema to Components
If you have new models, add them to the `components()` section:
```rust
components(
    schemas(
        // ... existing schemas ...
        crate::models::MyRequest,
        crate::models::MyResponse,
    )
)
```

## Troubleshooting

### Rust-Analyzer Errors
You may see false errors from rust-analyzer about "struct is not supported in traits or impls". These are parsing issues with the utoipa macro and can be ignored. The code will compile correctly.

### Compilation Issues
If you encounter compilation errors:
1. Ensure all dependencies are correctly added to `Cargo.toml`
2. Run `cargo clean` and then `cargo build`
3. Check that all model imports are correct in `src/docs/mod.rs`

### Swagger UI Not Loading
1. Verify the server is running on port 3000
2. Check that `/docs` route is accessible
3. Look for errors in the server logs

## Next Steps

### Remaining Controllers
The following controllers still need annotation (follow the same pattern):
- Student Controller endpoints
- Mentor Controller endpoints
- Health check endpoint

### Testing
Optional property-based tests can be added to verify:
- All routes are documented
- Schema completeness
- Security annotation consistency
- Error response consistency

## Benefits

1. **Interactive Documentation**: Developers can test endpoints directly from the browser
2. **Always Up-to-Date**: Documentation is generated from code, so it stays in sync
3. **Type Safety**: Compile-time validation ensures documentation matches implementation
4. **Standardization**: Consistent error responses and authentication across all endpoints
5. **Developer Experience**: Easy to understand API structure and test endpoints

## References

- [utoipa Documentation](https://docs.rs/utoipa/)
- [utoipa-swagger-ui Documentation](https://docs.rs/utoipa-swagger-ui/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
