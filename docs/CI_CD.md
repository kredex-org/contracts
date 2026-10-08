# Kredex Contracts — CI/CD Pipelines

This document describes every automated workflow in the `.github/workflows/` folder.

---

## Overview

```
Pull Request opened / updated
  ├── ci.yml          → Format · Lint · Test · Build WASM · Size report
  ├── security.yml    → cargo audit · cargo deny
  └── label.yml       → Auto-label PR by changed files

Push to `main`
  ├── ci.yml          → same as above (without size report)
  ├── security.yml    → same
  └── deploy.yml      → Build + Deploy all 7 contracts to Stellar Testnet

Push a version tag  (e.g. v1.0.0)
  └── release.yml     → Build · Bundle WASMs · Create GitHub Release
```

---

## Workflows

### `ci.yml` — Continuous Integration

Runs on every push to `main` / `develop` and on every PR.

| Job | What it does |
|---|---|
| **fmt** | `cargo fmt --all -- --check` — fails fast on formatting issues |
| **clippy** | `cargo clippy --all-targets -- -D warnings` — lint as errors |
| **test** | `cargo test --workspace` — all unit tests |
| **build-wasm** | `stellar contract build` — compile all 7 contracts to `.wasm` |
| **size-report** | Posts a table of contract sizes as a PR comment *(PRs only)* |

**Artifacts:** WASM files uploaded and kept for 30 days.

---

### `security.yml` — Security Audit

Runs on push to `main`, on PRs, and **every Monday at 08:00 UTC**.

| Step | Tool | Purpose |
|---|---|---|
| `cargo audit` | [rustsec/audit-check](https://github.com/rustsec/audit-check) | Checks all dependencies against the RustSec advisory database |
| `cargo deny` | [cargo-deny-action](https://github.com/EmbarkStudios/cargo-deny-action) | Enforces allowed licenses (MIT, Apache-2.0, BSD, ISC) and bans known-bad crates |

Configuration: [`deny.toml`](../deny.toml)

---

### `deploy.yml` — Testnet Deployment

Runs automatically when Rust source files change on `main`. Can also be triggered manually via **Actions → Run workflow**.

| Input | Default | Description |
|---|---|---|
| `dry_run` | `false` | Build WASM only, skip the actual deploy |

**Required secret:**

```
TESTNET_ADMIN_SECRET  ← the secret key (S...) of the trustlend-admin Stellar account
```

Add it under: **Repository Settings → Secrets and variables → Actions → New repository secret**

**What it does:**
1. Installs `stellar-cli`.
2. Imports the admin identity from the secret.
3. Builds all contracts with `stellar contract build`.
4. Deploys all 7 contracts and prints new contract IDs in the step summary.
5. Uploads WASM artifacts (kept 90 days).

---

### `release.yml` — GitHub Release

Triggered by pushing a version tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

**What it does:**
1. Builds optimized WASM files.
2. Auto-generates release notes including a contract-size table.
3. Creates a GitHub Release with all `.wasm` files attached.
4. Marks as pre-release if the tag contains `alpha`, `beta`, or `rc`.

---

### `label.yml` — PR Auto-Labeller

Automatically applies labels to PRs based on which files changed.

| Files changed | Label applied |
|---|---|
| `borrower_reputation/**` | `contract: reputation` |
| `reputation_nft/**` | `contract: nft` |
| `escrow/**` | `contract: escrow` |
| `lending/**` | `contract: lending` |
| `default_management/**` | `contract: insurance` |
| `liquidity_pool/**` | `contract: liquidity-pool` |
| `oracle_adapter/**` | `contract: oracle` |
| `.github/**` | `area: ci` |
| `docs/**`, `*.md` | `area: docs` |
| `Cargo.toml`, `Cargo.lock` | `area: dependencies` |

Configuration: [`.github/labeler.yml`](labeler.yml)

---

## Required GitHub Settings

### Secrets (Actions)

| Secret | Purpose |
|---|---|
| `TESTNET_ADMIN_SECRET` | Stellar secret key for deploying contracts to testnet |

### Branch Protection (main)

Recommended settings for `main` branch:
- ✅ Require a pull request before merging
- ✅ Require status checks to pass: `fmt`, `clippy`, `test`, `build-wasm`
- ✅ Require branches to be up to date before merging
- ✅ Do not allow bypassing the above settings

### Repository Settings

- ✅ Enable **Discussions** for community Q&A
- ✅ Enable **Issues** with the provided templates
- ✅ Enable **GitHub Actions** (already on by default for public repos)
- ✅ Set **License** to MIT (already committed)
- ✅ Enable **Dependabot** alerts (already configured in `dependabot.yml`)

---

## Adding New Contracts

When you add a new Soroban crate to the workspace:

1. Add it to `Cargo.toml` workspace members — CI will automatically pick it up.
2. Add it to the `deploy.yml` `deploy()` calls.
3. Add a label rule in `.github/labeler.yml`.
4. Document entry points in `docs/CONTRACTS.md`.
