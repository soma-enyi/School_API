# School Backend API

A Rust backend API for managing schools and students, built with Axum and PostgreSQL.

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

The server will start on `http://0.0.0.0:8080`

You should see output like:
```
INFO school_backend: Connecting to database...
INFO school_backend: Database connected successfully!
INFO school_backend: Database health check passed!
INFO school_backend: Server starting on 0.0.0.0:8080
```

### Production Build

```bash
cargo build --release
./target/release/school-backend
```

## API Endpoints

### Schools

- `GET /api/schools` - List all schools
- `GET /api/schools/:id` - Get a specific school by ID
- `POST /api/schools` - Create a new school

Example POST request:
```bash
curl -X POST http://localhost:8080/api/schools \
  -H "Content-Type: application/json" \
  -d '{"name":"Test School","address":"123 Main St"}'
```

### Students

- `GET /api/students` - List all students
- `GET /api/students/:id` - Get a specific student by ID
- `POST /api/students` - Create a new student

Example POST request:
```bash
curl -X POST http://localhost:8080/api/students \
  -H "Content-Type: application/json" \
  -d '{"name":"John Doe","email":"john@example.com","school_id":1}'
```

## Database Schema

### Schools Table
- `id` - Serial primary key
- `name` - VARCHAR(255), required
- `address` - TEXT, optional

### Students Table
- `id` - Serial primary key
- `name` - VARCHAR(255), required
- `email` - VARCHAR(255), required, unique
- `school_id` - Foreign key to schools table, optional

## Features

- RESTful API with Axum web framework
- PostgreSQL database with SQLx for type-safe queries
- CORS enabled for cross-origin requests
- Structured logging with tracing
- Connection pooling for optimal performance
- Health check on startup to verify database connectivity

## Troubleshooting

### Database Connection Issues

If you see "password authentication failed", ensure:
1. PostgreSQL is running: `sudo systemctl status postgresql`
2. Password is set correctly: `sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';"`
3. `.env` file has the correct DATABASE_URL

### Port Already in Use

If port 8080 is already in use, you can change it in `src/main.rs`:
```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 8080)); // Change 8080 to your preferred port
```
