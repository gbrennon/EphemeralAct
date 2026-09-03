# Development

Requires Rust (see [`rust-toolchain.toml`](../rust-toolchain.toml)),
[`just`](https://github.com/casey/just), and `cargo-llvm-cov` for coverage
(`just tools` installs the extras).

## Common tasks

| Recipe | What it does |
|---|---|
| `just build` | Build the project |
| `just run *args` | Run the binary (`cargo run -- run ...`) |
| `just test` | Run all tests with coverage (threshold: 80%) |
| `just test-local` | Run tests without coverage |
| `just lint` | `cargo clippy -- -D warnings` (zero warnings enforced) |
| `just fmt` / `just fmt-check` | Format / check formatting |
| `just install` | Install release binary to `~/.cargo/bin` |

CI also lints Forgejo Actions workflows with `actionlint` (`just
lint-workflows`) and uses `lefthook` pre-commit hooks (`just install-hooks`).
