# Course Flow Backend — Architecture Plan (Axum)

---

## Tech Stack

| Component         | Choice                          |
| ----------------- | ------------------------------- |
| **HTTP Framework** | Axum                           |
| **Database**       | PostgreSQL                     |
| **ORM / Queries**  | SQLx                           |
| **Auth**           | JWT (jsonwebtoken crate)       |
| **Password Hashing** | argon2                       |
| **Email**          | Lettre                         |
| **Serialization**  | Serde                          |
| **Validation**     | validator crate                |
| **File Uploads**   | axum-multipart                 |
| **Async Runtime**  | Tokio                          |

---

## Cargo.toml Dependencies

```toml
[dependencies]
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
jsonwebtoken = "9"
argon2 = "0.5"
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
validator = { version = "0.18", features = ["derive"] }
dotenvy = "0.15"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
```

---

## Database Schema

```sql
-- Enums
CREATE TYPE user_role AS ENUM ('student', 'mentor', 'admin');
CREATE TYPE user_status AS ENUM ('pending', 'interview', 'waitlisted', 'accepted', 'rejected', 'active');

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL,
    status user_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Courses
CREATE TABLE courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Course Enrollments
CREATE TABLE course_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    role user_role NOT NULL,
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, course_id)
);

-- Materials
CREATE TABLE materials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    file_url TEXT NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Assignments
CREATE TABLE assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    due_date TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Assignment Submissions
CREATE TABLE assignment_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_url TEXT NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(assignment_id, student_id)
);

-- Attendance
CREATE TABLE attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    status BOOLEAN NOT NULL DEFAULT FALSE,
    marked_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(course_id, student_id, date)
);
```

---

## Project Structure

```
src/
├── main.rs
├── config/
│   ├── mod.rs
│   ├── database.rs
│   └── env.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── course.rs
│   ├── assignment.rs
│   ├── submission.rs
│   ├── material.rs
│   ├── attendance.rs
│   └── enrollment.rs
├── handlers/
│   ├── mod.rs
│   ├── auth.rs
│   ├── admin.rs
│   ├── mentor.rs
│   └── student.rs
├── middleware/
│   ├── mod.rs
│   └── auth.rs            # JWT extractor + role guard
├── services/
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── email_service.rs
│   ├── user_service.rs
│   ├── course_service.rs
│   ├── assignment_service.rs
│   ├── material_service.rs
│   └── attendance_service.rs
├── errors/
│   ├── mod.rs
│   └── app_error.rs
└── utils/
    ├── mod.rs
    └── validators.rs
```

---

## App State (Shared)

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: String,
    pub mail_config: MailConfig,
}
```

---

## Axum Router Setup

```rust
// main.rs
use axum::{Router, middleware};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let state = AppState::init().await;

    let app = Router::new()
        .nest("/api/auth", auth_routes())
        .nest("/api/admin", admin_routes())
        .nest("/api/mentor", mentor_routes())
        .nest("/api/student", student_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## API Endpoints

### 1. Auth — `/api/auth` (Public)

| Method | Endpoint                    | Description                                        |
| ------ | --------------------------- | -------------------------------------------------- |
| POST   | `/api/auth/register/student` | Student registration (status → pending, triggers interview mail) |
| POST   | `/api/auth/register/mentor`  | Mentor registration with course preference (status → pending, needs admin validation) |
| POST   | `/api/auth/login`            | Login for all roles, returns JWT                    |
| PUT    | `/api/auth/change-password`  | Change password for logged-in user (protected)      |

### 2. Admin — `/api/admin` (Admin Only)

| Method | Endpoint                       | Description                          |
| ------ | ------------------------------ | ------------------------------------ |
| GET    | `/api/admin/dashboard`          | Admin dashboard stats (totals, pending approvals) |
| **Student Management** | | |
| GET    | `/api/admin/students`           | List all students with status        |
| GET    | `/api/admin/students/:id`       | View single student details          |
| PUT    | `/api/admin/students/:id/waitlist` | Move student to waitlist (after interview) |
| PUT    | `/api/admin/students/:id/accept`   | Accept student → sends acceptance mail |
| PUT    | `/api/admin/students/:id/reject`   | Reject student → sends rejection mail  |
| **Mentor Management** | | |
| GET    | `/api/admin/mentors`            | List all mentors                     |
| GET    | `/api/admin/mentors/:id`        | View single mentor details           |
| PUT    | `/api/admin/mentors/:id`        | Update mentor details                |
| DELETE | `/api/admin/mentors/:id`        | Delete a mentor                      |
| PUT    | `/api/admin/mentors/:id/accept` | Validate/accept mentor               |
| PUT    | `/api/admin/mentors/:id/reject` | Reject mentor                        |
| **Course Management** | | |
| POST   | `/api/admin/courses`            | Create a new course                  |
| GET    | `/api/admin/courses`            | List all courses                     |
| PUT    | `/api/admin/courses/:id`        | Update a course                      |
| DELETE | `/api/admin/courses/:id`        | Delete a course                      |

### 3. Mentor — `/api/mentor` (Mentor Only)

| Method | Endpoint                              | Description                        |
| ------ | ------------------------------------- | ---------------------------------- |
| GET    | `/api/mentor/dashboard`                | Mentor dashboard (assigned courses, stats) |
| **Courses** | | |
| GET    | `/api/mentor/courses`                  | List mentor's assigned courses     |
| GET    | `/api/mentor/courses/:id/students`     | List students enrolled in a course |
| **Materials** | | |
| POST   | `/api/mentor/courses/:id/materials`    | Upload material to a course        |
| GET    | `/api/mentor/courses/:id/materials`    | List materials for a course        |
| DELETE | `/api/mentor/materials/:id`            | Delete a material                  |
| **Assignments** | | |
| POST   | `/api/mentor/courses/:id/assignments`  | Create assignment for a course     |
| GET    | `/api/mentor/courses/:id/assignments`  | List assignments for a course      |
| PUT    | `/api/mentor/assignments/:id`          | Update an assignment               |
| DELETE | `/api/mentor/assignments/:id`          | Delete an assignment               |
| GET    | `/api/mentor/assignments/:id/submissions` | View submissions for an assignment |
| **Attendance** | | |
| POST   | `/api/mentor/courses/:id/attendance`   | Mark attendance for students       |
| GET    | `/api/mentor/courses/:id/attendance`   | View attendance records            |

### 4. Student — `/api/student` (Student Only)

| Method | Endpoint                               | Description                        |
| ------ | -------------------------------------- | ---------------------------------- |
| GET    | `/api/student/dashboard`                | Student dashboard (enrolled courses, upcoming assignments) |
| **Courses** | | |
| GET    | `/api/student/courses`                  | List enrolled courses              |
| GET    | `/api/student/courses/:id`              | View course details                |
| **Materials** | | |
| GET    | `/api/student/courses/:id/materials`    | View/download materials for a course |
| **Assignments** | | |
| GET    | `/api/student/courses/:id/assignments`  | List assignments for a course      |
| GET    | `/api/student/assignments/:id`          | View single assignment detail      |
| POST   | `/api/student/assignments/:id/submit`   | Submit an assignment               |
| **Attendance** | | |
| GET    | `/api/student/courses/:id/attendance`   | View own attendance for a course   |

---

## JWT Auth Middleware (Axum Extractor)

```rust
// middleware/auth.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       // user id
    pub role: String,    // "student" | "mentor" | "admin"
    pub exp: usize,
}

pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Role,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing token"})),
            ))?;

        let app_state = AppState::from_ref(state);
        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(app_state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid token"})),
        ))?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
            role: token_data.claims.role.parse().unwrap(),
        })
    }
}
```

---

## Role Guard Middleware

```rust
// middleware/auth.rs

pub async fn require_role(
    required: Role,
    auth_user: AuthUser,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    if auth_user.role != required {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Insufficient permissions"})),
        ));
    }
    Ok(next.run(request).await)
}
```

---

## Request/Response Flow

```
Client Request
     │
     ▼
Axum Router (matches route)
     │
     ▼
Tower Middleware (CORS, Tracing)
     │
     ▼
Role Guard Layer (checks JWT + role)
     │
     ▼
Handler (extracts Path, Query, Json)
     │
     ▼
Service Layer (business logic)
     │
     ▼
SQLx (database queries)
     │
     ▼
JSON Response
```

---

## User Lifecycle Flows

### Student Lifecycle

```
Register → Interview Mail Sent → Admin Waitlists →
  ├─ Accept → Acceptance Mail + Credentials → Login → Dashboard
  └─ Reject → Rejection Mail
```

### Mentor Lifecycle

```
Register (with course) → Admin Validates →
  ├─ Accept → Mail + Credentials → Login → Mentor Dashboard
  └─ Reject → Rejection Mail
```

---

## Email Triggers

| Event                     | Email Sent                       |
| ------------------------- | -------------------------------- |
| Student registers         | Interview scheduled mail         |
| Student moved to waitlist | Waitlist confirmation mail       |
| Student accepted          | Acceptance mail + login details  |
| Student rejected          | Rejection mail                   |
| Mentor accepted by admin  | Acceptance mail + login details  |
| Mentor rejected           | Rejection mail                   |

---

## Endpoint Count

| Module    | Endpoints |
| --------- | --------- |
| Auth      | 4         |
| Admin     | 14        |
| Mentor    | 12        |
| Student   | 8         |
| **Total** | **38**    |

---

## Implementation Order

1. **Config + AppState + main.rs** — Server boots up
2. **Models** — All structs + enums
3. **Auth** — Register, Login, JWT, Password hashing
4. **Middleware** — JWT extractor, Role guard
5. **Admin handlers + services**
6. **Mentor handlers + services**
7. **Student handlers + services**
8. **Email service**
