# ephact

> [!NOTE]
> If you are viewing this repository elsewhere, please be aware that it may be
> a read-only mirror. The original repository lives on Codeberg:
> [https://codeberg.org/gbrennon/ephact](https://codeberg.org/gbrennon/ephact).

`ephact` is a Rust crate that runs actions and workflows locally, in ephemeral
repositories, isolated from your operating system. Workflows execute inside
throwaway containers; by default nothing touches the host filesystem, and
network access is opt-in. Safe by default, real only when you ask for it.

## Quick start

```sh
cargo run -- run --repo /path/to/repo --workflow .github/workflows/ci.yml --event push
```

Built on Rust edition **2024**; the toolchain comes from
[`rust-toolchain.toml`](rust-toolchain.toml).

---

## 1. Using ephact

CLI flags, event simulation, secrets and inputs, and the opt-in flags that let
runs touch real containers or the network are documented in
[`docs/usage.md`](docs/usage.md).

## 2. Architecture

The crate follows hexagonal architecture, with `domain`, `application`,
`infrastructure`, and `presentation` layers under `src/`. Ports and adapters,
and how the layers depend on each other, are documented in
[`docs/architecture.md`](docs/architecture.md).

---

## 3. Development

Building, testing, linting, coverage, and the `just` recipes that drive them
are documented in [`docs/development.md`](docs/development.md).
