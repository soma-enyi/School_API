# School Backend API

A Rust backend API for managing schools and students, built with Axum and PostgreSQL.

## Features

- RESTful API with Axum web framework
- PostgreSQL database with SQLx for type-safe queries
- JWT-based authentication with role-based access control
- **Interactive Swagger/OpenAPI 3.0 documentation**
- CORS enabled for cross-origin requests
- Structured logging with tracing
- Connection pooling for optimal performance
- Standardized error responses

## Prerequisites

- Rust (latest stable version)
- PostgreSQL 17 or higher
- Ubuntu/Debian Linux (or similar)

## Installation & Setup

### 1. Install PostgreSQL

```bash
sudo apt update
sudo apt install -y postgresql postgresql-contrib
```

### 2. Start PostgreSQL Service

```bash
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### 3. Set Up Database

```bash
# Set password for postgres user
sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';"

# Create the database
sudo -u postgres createdb school_db

# Run migrations
sudo -u postgres psql -d school_db -f migrations/001_init.sql
```

### 4. Configure Environment

The `.env` file is already configured with:
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/school_db
```

You can modify this if you use different credentials.

## Running the Application

### Development Mode

```bash
cargo run
```

The server will start on `http://localhost:3000`

You should see output like:
```
INFO school_backend: Database migrations completed successfully
INFO school_backend: Server running on http://127.0.0.1:3000
```

### Production Build

```bash
cargo build --release
./target/release/school-backend
```

## API Documentation

### Swagger UI (Interactive Documentation)

Once the server is running, access the interactive API documentation at:

**http://localhost:3000/docs**

The Swagger UI provides:
- Complete API endpoint documentation
- Interactive "Try it out" functionality
- Request/response schemas with examples
- Authentication support (JWT Bearer tokens)
- Organized by tags (Health, Authentication, Admin, Student, Mentor, School)

### OpenAPI Specification

The raw OpenAPI 3.0 specification is available at:

**http://localhost:3000/api-docs/openapi.json**

## API Endpoints

### Health Check
- `GET /health` - Check API health status

### Authentication

**Public Endpoints:**
- `POST /auth/admin/register` - Register new admin user
- `POST /auth/admin/login` - Admin login
- `POST /auth/student/register` - Register new student user
- `POST /auth/student/login` - Student login
- `POST /auth/mentor/register` - Register new mentor user
- `POST /auth/mentor/login` - Mentor login
- `POST /auth/refresh` - Refresh access token

**Protected Endpoints:**
- `POST /auth/logout` - Logout (requires authentication)
- `GET /auth/me` - Get current user profile
- `POST /auth/verify` - Verify token validity

### Admin Operations (Requires Admin Role)
- `GET /admin/dashboard` - Get admin dashboard
- `GET /admin/users` - List all users
- `GET /admin/statistics` - Get user statistics
- `POST /admin/users/{user_id}/activate` - Activate user account
- `POST /admin/users/{user_id}/deactivate` - Deactivate user account

### School Management (Requires Admin Role)
- `GET /admin/schools` - List all schools
- `POST /admin/schools/create` - Create new school
- `GET /admin/schools/{school_id}` - Get school details
- `PUT /admin/schools/{school_id}` - Update school
- `DELETE /admin/schools/{school_id}` - Delete school
- `GET /admin/schools/{school_id}/statistics` - Get school statistics

### Student Operations (Requires Student Role)
- `GET /student/dashboard` - Get student dashboard
- `GET /student/profile` - Get student profile
- `GET /student/courses` - Get enrolled courses
- `POST /student/assignments/{assignment_id}/submit` - Submit assignment
- `GET /student/grades` - Get grades
- `POST /student/messages/mentor` - Send message to mentor

### Mentor Operations (Requires Mentor Role)
- `GET /mentor/dashboard` - Get mentor dashboard
- `GET /mentor/profile` - Get mentor profile
- `GET /mentor/students` - Get assigned students
- `GET /mentor/students/{student_id}/progress` - Get student progress
- `POST /mentor/assignments/{assignment_id}/grade` - Grade assignment
- `POST /mentor/assignments/create` - Create new assignment
- `POST /mentor/messages/student/{student_id}` - Send message to student
- `GET /mentor/courses/{course_id}/assignments` - Get course assignments

## Authentication

The API uses JWT (JSON Web Tokens) for authentication.

### Getting a Token

1. Register or login using the appropriate endpoint:
```bash
curl -X POST http://localhost:3000/auth/admin/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@school.com","password":"your_password"}'
```

2. The response will include an `access_token`:
```json
{
  "user": {...},
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Using the Token

Include the token in the Authorization header for protected endpoints:

```bash
curl -X GET http://localhost:3000/admin/dashboard \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

### Using Swagger UI with Authentication

1. Click the "Authorize" button in Swagger UI
2. Enter: `Bearer YOUR_ACCESS_TOKEN`
3. Click "Authorize"
4. Now you can test protected endpoints directly in the browser

## Database Schema

### Users Table
- `id` - UUID primary key
- `email` - VARCHAR, required, unique
- `password_hash` - VARCHAR, required
- `first_name` - VARCHAR, required
- `last_name` - VARCHAR, required
- `role` - VARCHAR (admin, student, mentor), required
- `is_active` - BOOLEAN, default true
- `created_at` - TIMESTAMP
- `updated_at` - TIMESTAMP

### Schools Table (Placeholder)
- `id` - UUID primary key
- `name` - VARCHAR, required
- `location` - VARCHAR
- `principal` - VARCHAR

### Students Table (Placeholder)
- `id` - UUID primary key
- `user_id` - Foreign key to users table
- `school_id` - Foreign key to schools table
- `enrollment_date` - VARCHAR

### Mentors Table (Placeholder)
- `id` - UUID primary key
- `user_id` - Foreign key to users table
- `school_id` - Foreign key to schools table
- `specialization` - VARCHAR

## Error Responses

All error responses follow a standardized format:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error message",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

Common HTTP status codes:
- `200 OK` - Successful GET, PUT, DELETE
- `201 Created` - Successful POST
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Valid token but insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict (e.g., email already exists)
- `500 Internal Server Error` - Server error

## Troubleshooting

### Database Connection Issues

If you see "password authentication failed", ensure:
1. PostgreSQL is running: `sudo systemctl status postgresql`
2. Password is set correctly: `sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';"`
3. `.env` file has the correct DATABASE_URL

### Port Already in Use

If port 3000 is already in use, you can change it in `src/main.rs`:
```rust
let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?; // Change 3000 to your preferred port
```

### Swagger UI Not Loading

1. Ensure the server is running on port 3000
2. Navigate to `http://localhost:3000/docs` (not `/swagger`)
3. Check server logs for any errors
4. Verify the OpenAPI spec is accessible at `http://localhost:3000/api-docs/openapi.json`

### Rust-Analyzer Errors

You may see false errors from rust-analyzer about "struct is not supported in traits or impls" related to utoipa macros. These are parsing issues and can be ignored - the code will compile correctly.

## Documentation

- **API Documentation**: See [SWAGGER_INTEGRATION.md](SWAGGER_INTEGRATION.md) for detailed information about the Swagger/OpenAPI integration
- **Authentication**: See [AUTH_DOCUMENTATION.md](AUTH_DOCUMENTATION.md) for authentication details
- **Interactive API Docs**: http://localhost:3000/docs (when server is running)

## Technology Stack

- **Framework**: Axum 0.7
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: bcrypt
- **API Documentation**: utoipa + utoipa-swagger-ui
- **Logging**: tracing + tracing-subscriber
- **Async Runtime**: Tokio
