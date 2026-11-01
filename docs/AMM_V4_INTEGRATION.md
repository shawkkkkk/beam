# Raydium AMM v4 integration boundary

Beam selected Raydium AMM v4 for graduation. This page tracks the development
dependency chain. The current Beam migration instruction is a gated, superseded
CPMM candidate. **No Beam instruction invokes AMM v4, and migration is not enabled.**

The ABI source pin is `raydium-io/raydium-amm@91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6`.
It is source evidence, not proof that a deployed Raydium program has matching
bytes. The archived Pump and CPMM snapshots remain provenance references.

| Step | Current local implementation | Still required before activation |
| --- | --- | --- |
| Account ABI | `programs/beam/src/beam_amm_v4.rs` derives the 19 `Initialize2` metas and validates actual classic SPL mint/source accounts, canonical Beam signer ATAs, WSOL backing, current config fee, and planned initial LP. | Fix and persist the market seed, store an authorized fee cap, preflight immediately before CPI, and check account aliases and all setup identities. Token-2022 graduation needs a separately reviewed compatible design. |
| Funding | `programs/beam/src/beam_amm_v4_funding.rs` computes checked curve liquidity, creator/protocol liabilities, donor surplus, migration charge, and caller-funded setup budget. | Read all values from trusted on-chain accounts. Bind the existing migration-fee recipient to stored policy. Do not use balance-only refund math without exact account and reserve checks. |
| Staged settlement | `programs/beam/src/beam_amm_v4_settlement.rs` checks three keyed observations: before setup funding, immediately before CPI, and after CPI. It counts curve-base-vault and curve-SOL funding once, then exact source, pool-vault, named fee-destination, migration-recipient and LP deltas. | Capture the observations from authenticated AccountInfos. Decode Raydium pool state and verify owner, status, mint/vault/LP keys, reserve changes, and LP custody. Bind every setup-rent payment to its created account; the current cap checks only the total. A caller-supplied `receipt_written` boolean is not replay protection. |
| Runtime evidence | Host unit tests cover the source helpers. A separate local ProgramTest branch exercises a mock AMM v4 processor. | Test the actual Beam migration handler and real pinned Raydium program in an isolated local runtime, including late CPI error and outer rollback. Never use captured original Pump binaries. |
| Client and indexing | AMM v4 ABI fixtures are source-only; the preview labels migration and liquidity unavailable. | Add a namespaced AMM v4 receipt/event, authenticated indexer mapping, SDK builder against the final account ABI, and verified-liquidity state. |

The planned transfer sequence is:

1. Verify the curve backing and separate fee/rent liabilities. Capture the
   initial balances and any donor surplus.
2. Move exactly the planned base amount from the curve's canonical base-token
   vault to Beam's base source and the planned pool quote from the curve to
   Beam's canonical WSOL source. Fund setup from the caller's bounded
   contribution. Capture the
   prepared balances. A WSOL source must be synced: lamports equal its native
   rent reserve plus token amount.
3. Invoke AMM v4 with the Beam PDA signer. Check the source debits, pool vault
   deposits, Raydium's canonical WSOL creation-fee destination, and received
   LP against the pinned integer-sqrt rule. Pay any existing migration fee only
   to its authorized recipient and verify the curve retains liabilities and
   donor surplus. Capture final balances and write an authenticated one-time
   receipt within the instruction.

The disabled helpers model this sequence; they do not perform it. Their balance
snapshots are values supplied by a caller. An executable adapter must bind
every value to the intended account, reject aliases before transfers, and
verify each destination's state. The setup-cost cap alone cannot prove rent
went only to canonical Raydium-created accounts. Incoming unrelated funds can mask overspend
in balance-only checks. A precreated pool or donated source tokens can block
migration and must not be adopted or swept to a caller.

AMM v4 does not natively accept Token-2022 mints or reproduce creator fees on
direct external swaps. LP disposition, post-graduation fee rights, creator
funding within or beyond the 42-bps community allocation, the internal
holder/buyback split, and production recipients remain undecided. Do not
enable migration or claim recurring buybacks from LP that Beam does not retain.

Host validation from the repository root:

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

These checks establish source integrity and deterministic host behavior only.
They do not exercise Beam's migration entrypoint or Raydium's deployed binary.
