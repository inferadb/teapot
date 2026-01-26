# Teapot

Rust-native terminal UI framework following the Elm Architecture, inspired by Bubble Tea.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Program                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐             │
│  │  Model  │───▶│ Update  │───▶│  View   │             │
│  │ (state) │    │ (logic) │    │(render) │             │
│  └─────────┘    └─────────┘    └─────────┘             │
│       ▲              │                                  │
│       │              ▼                                  │
│       │         ┌─────────┐                            │
│       └─────────│   Cmd   │                            │
│                 │(effects)│                            │
│                 └─────────┘                            │
└─────────────────────────────────────────────────────────┘
```

## Error Handling

Use **thiserror** for structured error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("validation failed: {message}")]
    Validation { field: String, message: String },
}
```

**Why thiserror (not snafu or anyhow):**
- Library crate - consumers must match on error variants
- Simple, user-facing errors - no deep chains needing backtraces
- anyhow erases types, preventing `match err { Error::Cancelled => ... }`

## Code Style

**Builders:** Use `bon` for complex builders (TaskProgressView, Field types). Use fluent API for simpler types (Style).

**Modern patterns:**
- `let-else` for early returns
- `is_none_or`/`is_some_and` over `match` for Option predicates
- `map_or` over `map().unwrap_or()`
- `&& let` chains for nested conditionals

**Forbidden:**
- `unsafe` code (denied via `[lints.rust]` in Cargo.toml)
- `.unwrap()` in production code (allowed in tests)
- `panic!`, `todo!()`, `unimplemented!()`

## Testing

```bash
cargo +1.92 test                                    # Unit + doc tests
cargo +1.92 clippy --all-targets -- -D warnings    # Lints
cargo +nightly fmt                                  # Format (nightly required)
cargo +nightly udeps --all-targets                  # Unused dependency detection
```
