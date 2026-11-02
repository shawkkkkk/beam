# Open owner decisions

This is the only list of values the owner has not decided. Other Beam
documents state decided policy and link here. They do not keep a second list
of unset amounts, splits, eligibility rules, venues, or schedules.

This file does not choose a number, a split, a default amount, a recipient, a
venue, or a schedule.

Any code path that needs one of the values below reads an admin-configurable
parameter. The parameter defaults to disabled or unset. While it is disabled
or unset, that path fails closed.

## Decided policy

These values stay as written.

- The curve fee is **0.50% all-in** (50 basis points): **42** bps community,
  **5** bps treasury, and **3** bps founder.
- Creator payouts, including the creator-selected wallet, are paid out of the
  42 bps community share. The curve total stays 0.50%.
- The 42/5/3 rounding in [economics](ECONOMICS.md) stays on the 50 bps
  component: one ceiling fee, founder `floor(total × 6 / 100)`, treasury
  `floor(total × 10 / 100)`, and the remainder to the community bucket. That
  rule does not divide the community bucket between holder rewards and the
  flagship.
- Graduation is Raydium AMM v4 at
  [raydium-io/raydium-amm `91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6`](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6),
  program `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`, 19-account
  `Initialize2`.
- The CPMM migration candidate fails closed.
- PumpSwap stays removed. There is no PumpSwap runtime fallback. Residual
  assumptions remain an engineering audit in [task scope 7](TASKS.md).
- LP is locked permanently. A locked position is not a recurring fee claim.
- Token-2022 trades on the curve. Migration rejects Token-2022 until a safe
  plan lands.
- `$BEAM` is the flagship.
- A separate Slop Cash contributor allocation of **10,000,000 whole `$BEAM`
  tokens** exists. It is disabled by default. It is not part of the 50 bps
  curve fee.
- pump.fun ($PUMP) is the original program author. Beam uses the community
  reconstruction at
  [lalalune/pumpfun-source `5703df02131ab944408e0cf679b4bef9c22e1a3d`](https://github.com/lalalune/pumpfun-source/commit/5703df02131ab944408e0cf679b4bef9c22e1a3d)
  and credits that repository's contributors. See
  [upstream attribution](UPSTREAM.md).

## Creator basis points taken out of the 42 bps community share

Undecided: how many of the 42 community basis points are paid to the creator.
The source is the community share. The count has no chosen value.

Any code path that needs the count uses an admin-configurable parameter. The
parameter defaults to disabled or unset and fails closed while it is disabled
or unset.

## Holder-reward versus flagship split, and the rounding lamport

Undecided: after the creator take, how the remaining community allocation is
divided between holder rewards and the `$BEAM` flagship side, and which of
those two sides receives the rounding lamport.

No ratio is chosen. No side is chosen to receive that lamport. Any code path
that needs the ratio or the rounding side uses admin-configurable parameters
that default to disabled or unset and fail closed while they are disabled or
unset.

The 42/5/3 remainder rule in the decided section is a separate step. It stays
as written.

## Airdrop eligibility

Undecided: who is eligible for a holder airdrop. No cohort, balance rule,
snapshot, or exclusion is chosen.

Any code path that selects recipients uses an admin-configurable parameter
that defaults to disabled or unset and fails closed while it is disabled or
unset.

## Buyback venue and timing

Undecided: where a flagship buyback would execute, when it would run, and
what happens to tokens acquired by that buyback. No venue, schedule, or
disposition is chosen.

Any code path that needs a venue, a time, or a disposition uses
admin-configurable parameters that default to disabled or unset and fail
closed while they are disabled or unset.

## Any post-graduation Beam fee

Undecided: whether Beam charges any fee after graduation, and any amount or
recipient for that fee. No post-graduation Beam amount is chosen.

Raydium's own AMM v4 pool fee remains that pool's configuration. Quotes keep
it separate from the curve's 0.50%.

Any code path that needs a post-graduation Beam fee uses an
admin-configurable parameter that defaults to disabled or unset and fails
closed while it is disabled or unset.

## Slop points source, snapshot, and claim

The 10,000,000 whole `$BEAM` Slop Cash contributor allocation above stays
disabled by default. It is separate from the 50 bps curve fee.

Undecided: the Slop points source, the snapshot, and the claim for that
allocation. No points feed, snapshot, or claim rule is chosen.

Any code path that needs the source, the snapshot, or the claim uses
admin-configurable parameters that default to disabled or unset and fail
closed while they are disabled or unset. The allocation stays disabled until
an admin sets those parameters.

## Other unset values

These are the remaining values the owner has not decided. They live here so
the list stays in one file.

- **`$BEAM` mint address.** The flagship name is decided. The mint address is
  unset. Code that needs the mint uses an admin-configurable parameter that
  defaults to unset and fails closed while it is unset. No address is chosen.
- **Treasury, founder, and community recipient accounts.** They are unset.
  Code that pays them uses admin-configurable parameters that default to
  disabled or unset and fail closed while they are disabled or unset. No
  public key is chosen.
- **Repository-root license.** This repository has no root `LICENSE`. The
  archived pump.fun MIT notice and the third-party notices stay with the
  pinned sources. They do not license new Beam files. No Beam license is
  chosen.
- **Optional three-second buy-side quote.** The prototype stays disabled. Its
  recipient is unset. Code that would pay it uses an admin-configurable
  parameter that defaults to disabled or unset and fails closed while it is
  disabled or unset. No recipient is chosen.
