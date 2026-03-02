# Contributing to School Backend API

Thank you for considering contributing to this project! We welcome contributions from everyone.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/school-backend.git
   cd school-backend
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/original-owner/school-backend.git
   ```

## Development Setup

Follow the setup instructions in [README.md](README.md) to get your development environment ready.

## Making Changes

### 1. Create a Branch

Create a new branch for your feature or bugfix:
```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/add-teacher-model` for new features
- `fix/database-connection-error` for bug fixes
- `docs/update-api-documentation` for documentation updates

### 2. Code Style

This project follows Rust best practices:

- Run `cargo fmt` before committing to format your code
- Run `cargo clippy` to catch common mistakes and improve code quality
- Write clear, descriptive commit messages
- Add comments for complex logic

### 3. Testing

Before submitting your changes:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Build the project
cargo build

# Run the application
cargo run
```

Test your changes manually using curl or a REST client:
```bash
# Test endpoint
curl http://localhost:8080/api/schools
```

### 4. Commit Your Changes

Write clear commit messages following this format:
```
type: brief description

Detailed explanation of what changed and why (if needed)
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Example:
```bash
git add .
git commit -m "feat: add teacher model and endpoints"
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title describing the change
- Description of what was changed and why
- Any related issue numbers (e.g., "Fixes #123")

## Project Structure

```
school-backend/
├── src/
│   ├── controllers/     # Request handlers
│   ├── models/          # Data structures
│   ├── routes/          # API route definitions
│   ├── services/        # Business logic and database queries
│   ├── middlewares/     # Custom middleware
│   ├── utils/           # Helper functions
│   └── main.rs          # Application entry point
├── migrations/          # Database migrations
└── README.md
```

## Adding New Features

### Adding a New Model

1. Create model file in `src/models/`
2. Define structs with `Serialize`, `Deserialize`, and `FromRow` derives
3. Export in `src/models/mod.rs`

### Adding New Endpoints

1. Create service file in `src/services/` with database queries
2. Create controller file in `src/controllers/` with request handlers
3. Create route file in `src/routes/` with endpoint definitions
4. Register routes in `src/routes/mod.rs`

### Database Migrations

When adding new tables or modifying schema:

1. Create a new migration file: `migrations/00X_description.sql`
2. Write SQL for creating/altering tables
3. Document the migration in your PR

## Code Review Process

1. All submissions require review before merging
2. Address any feedback from reviewers
3. Keep your branch up to date with main:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

## Reporting Issues

When reporting bugs, include:
- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, PostgreSQL version)
- Error messages or logs

## Questions?

Feel free to open an issue for questions or discussions about the project.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
