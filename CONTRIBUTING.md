# Contributing to Teapot

## Code of Conduct

Governed by the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Report issues to [open@inferadb.com](mailto:open@inferadb.com).

## Development Setup

**Requirements:** Rust 1.92+, nightly toolchain (for formatting)

```bash
rustup install 1.92
rustup install nightly
cargo install just
```

**Commands:**

```bash
just          # Run all checks (test, lint, format)
just test     # Run tests
just lint     # Run clippy
just fmt      # Format code
just udeps    # Check unused dependencies
```

## Pull Requests

1. Fork and branch from `main`
2. Run `just` to verify all checks pass
3. Use [Conventional Commits](https://www.conventionalcommits.org/) for messages
4. Update docs if changing public APIs
5. Submit PR with clear description

## Reporting Issues

- **Bugs**: Include version, reproduction steps, expected vs actual behavior
- **Features**: Describe use case and proposed solution
- **Security**: Email [security@inferadb.com](mailto:security@inferadb.com) (not public issues)

## License

Contributions are dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).

## Questions?

- [Discord](https://discord.gg/inferadb)
- [open@inferadb.com](mailto:open@inferadb.com)
