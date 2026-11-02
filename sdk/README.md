# Beam native-SOL ABI fixtures

`idl/beam.json` is the Anchor 0.31 IDL regenerated from `programs/beam`.
The Anchor module name remains `pump_recovered`; the address is the undeployed
Beam placeholder. Regenerate it with `scripts/assemble_beam_idl.py` from an
`idl-build` test log. PDA seeds are not inside that file: Anchor's resolver
does not compile for this source. `idl/beam.pdas.json` lists the seeds and
fixed addresses, and the compatibility test derives each one with
`Pubkey::find_program_address`, including the AMM v4 helpers in
`beam_amm_v4`. The dependency files in `programs/beam/idls/` are not this
program's IDL.

`fixtures/gated-migration.json` keeps the fail-closed CPMM instructions as a
superseded section. Those account metas use synthetic keys, not the compiled
devnet or mainnet CPMM ids. The `ammV4` section is the pinned 19-account
`Initialize2` ABI at mainnet program `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`,
the same program id draft PR #38 executes. There is no Beam migration
instruction for that ABI yet. `enabled: true` remains representable on the
superseded CPMM instruction. The handler still rejects it.

`programs/beam/tests/idl_compatibility.rs` checks that IDL against the program
types, the legacy `fixtures/native-sol.json` encodings, and the gated
migration fixture.

`beam-native.mjs` is a dependency-free, local-only encoder for the retained
legacy `create`, `buy`, and `sell` instruction data and ordered Anchor account
metas at Beam contract commit `65c96b5`. It also decodes the raw
`CompleteEvent` payload and the current 166-byte `BondingCurve` account. Run:

```sh
node --test sdk/beam-native.test.mjs
```

The golden instruction bytes in `fixtures/native-sol.json` came from
`beam_program::instruction::{Create,Buy,Sell}.data()`. The signer/writable
flags and order came from
`beam_program::accounts::{Create,LegacyBuy,LegacySell}.to_account_metas(None)`
under Anchor 0.31.1. The event bytes came from `CompleteEvent::data()` and the
curve bytes from `BondingCurve::try_serialize`. The fixture uses synthetic
public keys and `.invalid` metadata URL; it is never a transaction example.

These helpers **do not** derive or verify PDAs, associated token accounts,
fee recipients, token program ownership, remaining accounts, signatures, or
on-chain state. In particular, inherited Pump `fee_config` and `fee_program`
accounts in legacy buy/sell have not been rehosted for Beam, and the handlers
expect additional validated remaining accounts. The builders return base
account metadata for inspection and testing, not an executable Beam client.
The development program ID is not deployed. No RPC, wallet, signing, or live
trade code is present.

`decodeCurrentBondingCurve` intentionally rejects historical shorter curve
layouts that the Rust program can read; it accepts only exactly 166 bytes of
the current serialized layout. Neither decoder verifies the account owner or
that a payload came from a trusted transaction. Check those separately when a
real client is designed.
