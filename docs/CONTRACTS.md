# Contract Reference

Reference for the Kredex Soroban contracts (soroban-sdk 22). Entry points are listed from the source; see each crate's `src/lib.rs` for parameters and doc comments.

> ⚠️ **Testnet only. Not audited.**

## Deployed on Stellar Testnet

| Contract | ID | Explorer |
| :-- | :-- | :-- |
| Reputation | `CDPALR5OWSO2HFSTB262IPUJNGRDVJOE5AGODXPVSSRPWVRAYK5Q6BOV` | [View](https://stellar.expert/explorer/testnet/contract/CDPALR5OWSO2HFSTB262IPUJNGRDVJOE5AGODXPVSSRPWVRAYK5Q6BOV) |
| Reputation NFT | `CBHSAFAS2V4M6WGF65XBGWG3BNGCC22JQDZWLRRJQW75467IX2JZYLZT` | [View](https://stellar.expert/explorer/testnet/contract/CBHSAFAS2V4M6WGF65XBGWG3BNGCC22JQDZWLRRJQW75467IX2JZYLZT) |
| Escrow | `CBNF4KK4JHQ5UUUC4W65WHA3WOB2FWCIRG3L2R3M5TEKKE6ZPSMUEAP5` | [View](https://stellar.expert/explorer/testnet/contract/CBNF4KK4JHQ5UUUC4W65WHA3WOB2FWCIRG3L2R3M5TEKKE6ZPSMUEAP5) |
| Lending | `CAJ5VLQZ2ZKOCVQKFX7LFYF36LNLCDT6QJT4STAZFQ5P23TZZ4UBEGXQ` | [View](https://stellar.expert/explorer/testnet/contract/CAJ5VLQZ2ZKOCVQKFX7LFYF36LNLCDT6QJT4STAZFQ5P23TZZ4UBEGXQ) |
| Default Management | `CDXUYZL5IJNGN742IZTC7H4IBKNEGTSMZACO76LXTVJSDVXXF2NSBOED` | [View](https://stellar.expert/explorer/testnet/contract/CDXUYZL5IJNGN742IZTC7H4IBKNEGTSMZACO76LXTVJSDVXXF2NSBOED) |
| Liquidity Pool | `CBUNFNEDSL3R2QTHLJVG4LOTRPYWTY557WZJIBGAMCGP2YDAHNOUKBCA` | [View](https://stellar.expert/explorer/testnet/contract/CBUNFNEDSL3R2QTHLJVG4LOTRPYWTY557WZJIBGAMCGP2YDAHNOUKBCA) |
| Oracle Adapter | `CBDXC63B7PO2DPZWZ2UGRTRCHNO2YIS5QP6SKRWJIJAPZQG5B3X4F2A3` | [View](https://stellar.expert/explorer/testnet/contract/CBDXC63B7PO2DPZWZ2UGRTRCHNO2YIS5QP6SKRWJIJAPZQG5B3X4F2A3) |

## Entry points

### `borrower_reputation`
Tracks each borrower's score, tier, KYC tier and loan totals.
- **Admin / lifecycle:** `initialize`, `get_admins`, `get_lending_contract`, `is_paused`, `pause`, `unpause`
- **Profiles:** `init_borrower`, `has_profile`, `get_profile`, `bump_profile_ttl`
- **Scoring:** `add_reputation_event`, `update_loan_totals`, `calculate_max_loan`, `calculate_interest_rate`
- **Risk controls:** `freeze_account`, `unfreeze_account`, `is_frozen`, `get_kyc_tier`, `set_kyc_tier`
- **NFT badges:** `has_badge`, `mint`

### `reputation_nft`
Soulbound badge NFTs. Transfers are intentionally disabled.
`initialize`, `get_admin`, `get_minter`, `set_minter`, `pause`, `unpause`, `mint`, `has_badge`, `get_badge`, `get_tier`, `transfer`, `transfer_from`, `approve`

### `lending`
Loan lifecycle. Interest math lives in [`lending/src/interest_rate.rs`](../lending/src/interest_rate.rs).
- **Admin / lifecycle:** `initialize`, `get_admins`, `get_usdc_token`, `is_paused`, `pause`, `unpause`
- **Lifecycle:** `create_loan_request`, `approve_loan`, `revoke_approval`, `activate_loan`, `fund_from_pool`, `record_payment`, `mark_defaulted`
- **Queries:** `get_loan`, `get_loan_count`, `is_overdue`, `days_overdue`, `get_borrower_loans`, `get_lender_loans`, `get_payment_count`, `get_payment`, `get_active_borrowings`, `get_active_lendings`
- **Maintenance:** `bump_loan_ttl`

### `escrow`
Holds funds with a revocation window.
`initialize`, `get_admins`, `get_usdc_token`, `is_paused`, `pause`, `unpause`, `create_hold`, `revoke_hold`, `confirm_disbursement`, `bump_hold_ttl`, `is_within_revocation_window`, `get_hold`, `get_escrow_count`

### `default_management`
`initialize`, `get_admins`, `get_usdc_token`, `is_paused`, `pause`, `unpause`, `record_default`, `get_default_record`, `get_insurance_balance`, `add_to_insurance`, `trigger_insurance_payout`, `bump_default_ttl`, `get_insurance_event`, `get_insurance_event_count`

### `liquidity_pool`
`initialize`, `deposit`, `withdraw`, `deposit_collateral`, `withdraw_collateral`, `allocate_to_loan`, `record_repayment`, `get_pool_metrics`

### `oracle_adapter`
`initialize`, `set_price`, `get_price`

## Design notes

- **Pausable & multi-admin.** Core contracts support `pause`/`unpause`; sensitive actions such as `freeze_account` require distinct admins (see `test_freeze_requires_distinct_admins`).
- **Reputation drives pricing.** Higher tier ⇒ higher max loan and lower interest rate.
- **TTL management.** Persistent records expose `bump_*_ttl` so they can be kept alive.
- **Release profile** is size-optimized (`opt-level = "z"`, LTO, `panic = "abort"`, `overflow-checks = true`). A `release-with-logs` profile keeps debug assertions for testnet debugging.

## Test coverage

| Crate | Tests |
| :-- | :-- |
| `borrower_reputation` | ✅ 16 tests |
| `reputation_nft`, `lending`, `escrow`, `default_management`, `liquidity_pool`, `oracle_adapter` | ❌ **none yet. Help wanted!** |

See [CONTRIBUTING.md](../CONTRIBUTING.md) to add some.
