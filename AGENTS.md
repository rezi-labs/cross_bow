# Agent Guidelines for cross_bow

## Build/Test/Lint Commands
- **Verify everything**: `just verify` (runs lint + test)
- **Test all**: `just test` → `cargo test`
- **Test single**: `cargo test test_name` or `cargo test --test integration_test_name`
- **Lint**: `just lint` → `cargo fmt --check` + `cargo clippy`
- **Format**: `just fmt` → `cargo fmt` + `cargo fix`
- **Run locally**: `just run` (starts DB + runs app)

## Code Style
- **Formatting**: Use `cargo fmt` (enforced by CI). Always run `just fmt` before committing
- **Linting**: Fix all `cargo clippy` warnings. Run `just verify` to check
- **Naming**: snake_case (functions/variables), PascalCase (types), SCREAMING_SNAKE_CASE (constants)
- **Error handling**: Use `Result<T, E>` with custom error types (WebhookError, DatabaseError, ValidationError)
- **Async**: Use `async fn` for all I/O operations with Tokio runtime
- **Database**: Use `sqlx` with compile-time checked queries and parameterized statements
- **Imports**: Group stdlib → external crates → internal modules (separated by blank lines)

## Architecture
- **Framework**: Actix-web + HTMX + Maud templates + PostgreSQL
- **Frontend**: Tailwind CSS + DaisyUI (Swiss design theme in assets/)
- **Project structure**: models/, handlers/, services/, templates/, middleware/, db/
- **Testing**: Unit tests in modules, integration tests in tests/ directory