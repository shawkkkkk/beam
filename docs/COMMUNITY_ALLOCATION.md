# Community allocation policy

Scope 5 from the [task ledger](TASKS.md). Base commit `4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf` (`main`). The model is `programs/beam/src/beam_community_allocation.rs`. It adds one module declaration after the retained program body. It adds no instruction, account, transfer, burn, keeper, or market order. Error codes start at 7200.

The curve charge decided below is the proposal in `beam_economics.rs`. Those constants were not changed here. The inherited Pump fee CPI still calculates creator and protocol fees on the retained trade path. That CPI is not this schedule and it is not approval to add a second creator charge. Collection is not live.

## Design principle

The core goal is minimal fees, far lower than pump.fun's published bonding-curve total. The founder takes a very minimal cut. Total Beam fees stay as low as possible. The current founder figure is 3 trade basis points (0.03% of the quote when the 50 bps component rounds evenly). That founder share is tunable. This document does not change it.

## Maintainer decisions

- **Curve total.** 50 bps all-in on the curve: 42 community, 5 treasury, 3 founder. Creator payouts come out of the community share. There is no additional creator charge. `creator_funding(AdditionalCharge)` returns `AdditionalChargeRejected` (7205).
- **Creator size.** How many of the 42 bps go to creators is still unset. The default model selects from-community and leaves the size `None`. Arithmetic rejects `None` (`CreatorBpsUnset`, 7201). A set value must be `<= 42`. The lamports are `floor(community_integer * bps / 42)`, which is always within the community integer. No creator rate is hard-coded.
- **Flagship coin.** `$BEAM`, a token that will itself launch on the Beam launchpad. It has no mint address yet. Arithmetic requires a caller-supplied mint pubkey and rejects the all-zero pubkey (`UnsetFlagshipMint`, 7207). No address is hard-coded.
- **Holder-rewards leg.** An airdrop of `$BEAM` to holders. The leg's lamports are a budget for that airdrop. No airdrop or transfer is wired.
- **Flagship leg.** Buyback-and-burn of `$BEAM`. Acquired tokens are burned. `Hold` is not a stored disposition; `accept_flagship_disposition(Hold)` returns `HoldDispositionRejected` (7208). No buyback order and no burn instruction are wired.
- **Slop Cash supply.** 10,000,000 whole `$BEAM` tokens are allocated to Slop Cash contributors, pro rata by Slop points. This is token supply, not part of the 50 bps fee. It is not funded and it is not live. `divide_slop_contributor_supply` stays disabled.

`AllocationModel::disabled()` is still the default. Selecting from-community does not enable collection.

## Proposed curve numbers

From `beam_economics.rs` on this branch:

| Constant | Value | Meaning when the 50 bps fee rounds evenly |
| --- | ---: | --- |
| `TOTAL_FEE_BPS` | 50 | 0.50% of the quote, rounded once with the retained ceiling routine |
| `COMMUNITY_SHARE_BPS` | 8,400 | 84% of that fee, the nominal 42 bps community share, including the creator slice |
| `TREASURY_SHARE_BPS` | 1,000 | 10% of that fee, nominal 5 bps |
| `FOUNDER_SHARE_BPS` | 600 | 6% of that fee, nominal 3 bps; tunable |

On a 100 SOL quote the helper returns 500,000,000 lamports: 420,000,000 community, 50,000,000 treasury, and 30,000,000 founder. The community integer is then divided only after a configured creator bps is set. Treasury and founder are not reduced to cover the creator.

The retained `Global.pool_migration_fee` is a separate configured migration charge. This change does not set it. pump.fun's published graduation fee is a different product's number, cited below.

## Comparison with published pump.fun fees

Source: [pump.fun Fees](https://pump.fun/docs/fees), last updated 17 October 2026.

| Charge | Published pump.fun figure | Beam curve proposal |
| --- | --- | --- |
| Bonding curve / on-pad total | 1.25% (0.30% creator, 0.95% protocol, 0% LP) | 0.50% all-in (42/5/3 bps). Founder is 0.03%. The creator slice sits inside the 42 bps and has no chosen number yet. |
| App platform fee | +0.45% of the trade on the pump.fun web and mobile apps; 0.40% when a referral code is applied. In addition to the table above. Terminal and third-party interfaces do not charge it. | No app platform fee is specified here. |
| Canonical PumpSwap, by market cap | 1.25% at the low end of the published schedule, down to 0.30% at the top tier | Not a Beam venue. |
| Graduation | 0.015 SOL to PumpSwap | Not copied. Beam's migration fee field is unchanged. |

0.50% is lower than the published 1.25% bonding-curve total, and lower than that total plus the 0.45% app fee. It is a target for the curve component. It is not a live charge, and it is not a promise about price or volume.

## After graduation

A post-graduation Beam fee stacks with the Raydium AMM v4 pool fee. Quotes have to show both. This model does not set the post-graduation Beam rate and does not collect either fee.

The pinned AMM v4 source initializer sets the default swap fee to 25/10,000 (0.25% of input) and the pnl share to 12/100 of that fee:

- [`Fees::initialize` at `91739d8`](https://github.com/raydium-io/raydium-amm/blob/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6/program/src/state.rs) (`swap_fee_numerator = 25`, `swap_fee_denominator = 10_000`, `pnl_numerator = 12`, `pnl_denominator = 100`).

That 0.25% is the initializer default in the pinned source. A live pool stores its own numerators. Raydium's [AMM v4 fees](https://docs.raydium.io/products/amm-v4/fees) describe the same defaults: 0.25% gross, of which 12% (0.03% of volume) is protocol pnl and the rest stays in the pool for LPs. Beam does not set that pool fee. The in-repo pin is an ABI reference, not a deployed-bytecode match. Burning the LP position gives up the LP portion of the pool fee. The superseded CPMM candidate is not this fee.

## Slop contributor supply

`SLOP_CONTRIBUTOR_WHOLE_TOKENS` is 10,000,000. Mint decimals are unset, so the helper counts whole tokens, not base units.

When arithmetic is explicitly requested, contributor `i` receives `floor(allocation * points_i / total_points)`. The leftover whole tokens stay unassigned (`allocation = assigned + unassigned`). A zero point total is rejected. The disabled Slop entrypoint does not write shares.

This allocation is not added to the 50 bps fee and it is not a funded balance.

## Questions still open

1. How many of the 42 community bps go to creators? The parameter must be set before arithmetic, and it must be `<= 42`. No number is chosen.
2. What holder/flagship shares of the amount left after that creator take should replace the provisional 5,000/5,000 test setting?
3. Which of those two legs receives the single rounding lamport?
4. Who is eligible for the `$BEAM` holder airdrop?
5. Where and when does a flagship buyback execute?
6. For the Slop supply: what is the points source, the snapshot time, who is eligible, and how is a claim made?

Recipient accounts, whether those accounts must differ, a zero-share leg, where the parameters are stored, and the founder/treasury dust cap in draft PR #23 were not decided in this reply. Draft PRs #23, #24, and #26 are not merged here. The splitter conserves whatever community integer `allocate` returns.
