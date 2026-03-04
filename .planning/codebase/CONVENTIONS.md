# Coding Conventions

**Analysis Date:** 2025-01-24

## Naming Patterns

**Files:**
- Shell scripts: `kebab-case` or single words (e.g., `ag-durumu`, `yardim`).
- Rust: `snake_case.rs` (standard).
- Slint: `kebab-case.slint` (e.g., `greeter.slint`).

**Functions:**
- Rust: `snake_case` (e.g., `detect_users`, `get_session_command`).
- Shell: Not used heavily as most scripts are single-task.

**Variables:**
- Rust: `snake_case`.
- Shell: `UPPER_CASE` for environment variables and key configurations (e.g., `DOTS_DIR`, `WLR_RENDERER`).

**Types:**
- Rust: `PascalCase`.

## Code Style

**Formatting:**
- Rust: standard `rustfmt`.
- Shell: No specific formatter detected, but scripts are clean and use 4-space indentation or tabs (mixed).

**Linting:**
- Rust: `clippy` (standard).

## Import Organization

**Order:**
1. Standard library imports
2. External crate imports
3. Local module imports (if any)

**Path Aliases:**
- Not detected.

## Error Handling

**Patterns:**
- Rust: Uses `Result` with `Box<dyn std::error::Error>` for the `main` function. Internal errors are logged using `tracing` and displayed to the UI via `app.set_error_message`.
- Shell: In `customize_airootfs.sh`, `set +e` is used explicitly to allow some command failures. Standard error redirection `2>/dev/null || true` is common for service enabling.

## Logging

**Framework:** `tracing` for Rust.

**Patterns:**
- `info!` for progress/success.
- `error!` for failures.

## Comments

**When to Comment:**
- Shell: Used for section headers and explaining "why" (e.g., `# Forces software rendering so it works inside VMs`).
- Rust: Used for doc comments (`///`) on top-level functions.

**JSDoc/TSDoc:**
- Not applicable.

## Function Design

**Size:** Functions are kept small and focused (e.g., `detect_users` is ~20 lines).

**Parameters:** Standard Rust parameter passing.

**Return Values:** Explicit `Result` or `Vec<String>`.

## Module Design

**Exports:** Standard Rust exports.

**Barrel Files:** Not used.

---

*Convention analysis: 2025-01-24*
