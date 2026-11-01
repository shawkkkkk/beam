# Beam native-SOL ABI fixtures

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
