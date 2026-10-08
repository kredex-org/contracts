# Contributing to Kredex Contracts

Thanks for helping build the Soroban contracts behind Kredex! This guide covers what's specific to this repo. For the general flow (fork → branch → PR), read the **[org-wide Contributing Guide](https://github.com/kredex-org/.github/blob/main/CONTRIBUTING.md)** first.

> New to Rust or Soroban? That's fine. Start with the [Rust Book](https://doc.rust-lang.org/book/) and the [Soroban docs](https://developers.stellar.org/docs/build/smart-contracts/overview). Ask for help any time in [Discussions](https://github.com/orgs/kredex-org/discussions).

## Setup

```bash
# 1. Install Rust: https://rustup.rs
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy

# 2. Install the Stellar CLI: https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup

# 3. Fork, then clone
git clone https://github.com/<you>/contracts.git
cd contracts
git remote add upstream https://github.com/kredex-org/contracts.git

# 4. Verify everything works
cargo test
```

## Development workflow

```bash
cargo fmt --all                                   # format
cargo clippy --all-targets -- -D warnings         # lint
cargo test                                        # all tests
cargo test -p borrower-reputation                 # one crate
cargo build --target wasm32-unknown-unknown --release   # WASM build
```

`cargo test` and the WASM build **must pass** (CI enforces them). `fmt` and `clippy` currently report warnings in older code, so they are advisory in CI for now. Please don't add new warnings, and PRs that clean existing ones up are very welcome (a great first contribution!).

## Coding guidelines

- **Authorization first.** Every state-changing function must call `require_auth()` on the right address. Explain in your PR how your change preserves access control.
- **Checked arithmetic.** Overflow checks are on in release (`overflow-checks = true`). Prefer `checked_*`/`saturating_*` and handle errors explicitly. Never rely on panics for normal control flow.
- **Storage & TTL.** Use the correct storage type (instance, persistent, temporary) and extend TTLs deliberately. Document your choice.
- **Events.** Emit events for important state changes, so the product can index them.
- **Keep it small.** WASM size matters (`opt-level = "z"`). Avoid heavy dependencies.
- **No `unsafe`, no `std`** in contract code (`#![no_std]`).
- **Document** public functions with `///` doc comments: purpose, auth required, errors.
- **Interface stability.** The [product](https://github.com/kredex-org/product) calls these contracts. Changing a function signature is a breaking change: open an issue first and describe the migration.

## Writing tests

Tests use `soroban-sdk`'s `testutils`. See [`borrower_reputation/src/test.rs`](borrower_reputation/src/test.rs) for a model:

```rust
#[test]
fn test_example() {
    let env = Env::default();
    env.mock_all_auths();
    // register contract, call functions, assert results
}
```

Please cover: the happy path, unauthorized callers, invalid inputs, pause behavior, and edge cases (zero/max values). `test_snapshots/` are generated automatically and git-ignored.

## Pull request checklist

- [ ] `cargo fmt`, `clippy` (no warnings) and `cargo test` pass
- [ ] New or changed behavior has tests
- [ ] Public functions are documented; `docs/CONTRACTS.md` updated if the interface changed
- [ ] No keys, secrets or `.env` files
- [ ] PR title follows [Conventional Commits](https://www.conventionalcommits.org/), e.g. `test(escrow): cover revocation window`

## Changes that need extra care

PRs that alter fund flows, auth, scoring or storage layout need **two maintainer approvals** and a note about upgrade/migration impact. Please discuss them in an issue first.

## Security

Never report vulnerabilities publicly. See the [Security Policy](https://github.com/kredex-org/.github/blob/main/SECURITY.md).

## Need help?

[Discussions](https://github.com/orgs/kredex-org/discussions) · [@kredexweb3](https://x.com/kredexweb3)
