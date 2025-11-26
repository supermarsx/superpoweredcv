Repository Guidelines
=====================

Project Structure & Module Organization
---------------------------------------
- `core/src/` holds Rust modules: `profile.rs` (UserProfile schema), `templates.rs` (injection text), `pipeline.rs` (AI/ATS pipeline + metrics), `pdf.rs` (PDF mutators), `red_team.rs` (engine + executors), `main.rs` (CLI & GUI entry point), `gui.rs` (SuperGUI implementation).
- `core/Cargo.toml` manages dependencies; `core/target/` is build output; `spec.md` and `spec-red-teaming-module.md` capture product requirements.

Build, Test, and Development Commands
-------------------------------------
- `cd core && cargo check` – type-check and lint the crate quickly.
- `cd core && cargo build` – compile the binary and library artifacts.
- `cd core && cargo run` – execute the application (launches GUI by default, or CLI with args).

Coding Style & Naming Conventions
---------------------------------
- Rust 2024 edition; prefer `snake_case` for modules/functions, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Derive `Serialize`/`Deserialize` where structs represent API payloads; add `#[serde(...)]` only when needed to match external schema.
- Keep comments minimal and purposeful (explain non-obvious logic or safety decisions).

Testing Guidelines
------------------
- Use `cargo check` as the fast guard; add unit tests under `src/` or integration tests under `tests/` as functionality grows.
- Name tests after the behavior under test (e.g., `generates_variant_hash`, `rejects_missing_template`).
- For PDF mutations, prefer deterministic fixtures and hash assertions to avoid flaky output.

Commit & Pull Request Guidelines
--------------------------------
- Keep commits scoped and descriptive (imperative mood), e.g., `Add stub PDF mutator`, `Wire pipeline executor trait`.
- In PRs, include: purpose/what changed, how to test (`cargo check/run`), and any notes on PDF mutation behavior or outputs. Attach screenshots or sample JSON output when reporting changes are involved.
