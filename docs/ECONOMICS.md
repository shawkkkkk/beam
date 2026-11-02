# Community-first economics

The maintainer decided a **50-basis-point all-in curve fee**: 42 nominal bps for the community allocation, 5 bps for treasury, and 3 bps for founder. Creator payouts come out of the 42 bps community share. There is no additional creator charge. How many of those 42 bps go to creators is unset. The founder figure is 3 bps in the current proposal and is tunable. The design goal is minimal fees, with a very small founder cut, kept as low as possible and lower than pump.fun's published bonding-curve total. The comparison and citations are in [community allocation](COMMUNITY_ALLOCATION.md). Preserve the creator payout and creator-selected wallet system.

## Rounding

1. Compute the Beam allocation fee once with Pump's retained ceiling-fee routine at 50 bps.
2. Compute the founder share as `floor(total_fee × 6 / 100)` and the treasury share as `floor(total_fee × 10 / 100)`.
3. Give the remainder to the community.

As an illustration of the allocation math alone, on 100 SOL notional this component is 0.5 SOL: 0.42 community, 0.05 treasury, and 0.03 founder. That 0.50% is the decided all-in curve total, including whatever creator slice is later set inside the 0.42 community share. It is not collected yet. On a one-lamport trade, the rounded component is one lamport, all assigned to the community. Quotes must disclose effective fees on dust trades, the unset creator slice, and every applicable fee component.

`beam_economics.rs` implements only Beam component allocation arithmetic. It is not yet wired into the recovered fee-program CPI, recipient routing or an executable keeper. The current inherited trade code calculates creator fees separately; that describes the present source, **not approval to add a creator charge on top of the 50 bps curve total**. The decided policy takes the creator payout from the community share. Beam-controlled configuration and vaults must preserve creator-selected wallets and test that payout once the creator's share of the 42 bps is set. The 42/5/3 split is the curve target, not an enforced live trading fee today.

The community allocation, after the configured creator take, funds holder
airdrops of `$BEAM` and a flagship buyback-and-burn of `$BEAM`. `$BEAM` will
launch on Beam and has no mint address yet. A 50/50 split of the remainder is
a provisional test setting. Holder eligibility and buyback venue and timing
are still open. The disabled model is in
[community allocation](COMMUNITY_ALLOCATION.md). It submits no buyback, transfer,
or burn, and it collects no fee.

## After graduation

Raydium charges its selected on-chain pool configuration. The maintainer decision is that a post-graduation Beam fee stacks with the Raydium AMM v4 pool fee. Quotes must show both. This page does not set the post-graduation Beam rate, and the curve's 50 bps figure is the on-pad all-in total. The pinned AMM v4 initializer default and the pump.fun comparison are in [community allocation](COMMUNITY_ALLOCATION.md). The existing gated CPMM migration code is a superseded candidate and does not satisfy the AMM v4 venue.

Recurring buybacks need a collectible fee source. The currently gated CPMM migration candidate burns all LP received by Beam. That removes its position; it does not retain an LP claim to harvest. The selected AMM v4 venue has no Beam-controlled creator or protocol fee switch; its LP fee accrues in pool reserves and is realized through LP withdrawal. Burning all AMM v4 LP would likewise relinquish that claim. Raydium's [Burn & Earn](https://docs.raydium.io/user-flows/burn-and-earn) does not support AMM v4. A Beam 50-bps post-graduation swap fee cannot be assumed from AMM v4. Until an independently reviewed custody/fee-rights design exists, community rewards and buybacks can be funded only from collected pre-graduation proceeds or another explicitly authorized source.

Before activation:

- Specify and test a transparent AMM v4 LP custody/escrow arrangement that preserves principal while defining who may realize fees, or another explicitly supported fee mechanism. Do not describe burned LP as continuing fee income.
- Deny creators, operators and keepers authority to withdraw liquidity principal.
- Set the post-graduation fee budget using the actual Raydium configuration.
- Route flagship community proceeds to a dedicated vault, with public buyback receipts. Acquired `$BEAM` is burned. Execution venue and timing are still open, and no burn instruction is wired.
- Bound slippage, price manipulation, account substitutions and keeper reimbursement; isolate execution costs from liquidity principal.

No volume means no revenue or buybacks. A continuing policy does not guarantee returns, price appreciation, execution frequency or perpetual revenue. This is required design work, not completed functionality.
