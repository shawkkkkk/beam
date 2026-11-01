# Community-first economics

The user selected a **50-basis-point Beam allocation**: 42 nominal bps for a community allocation, 5 bps for treasury, and 3 bps for founder. Preserve the creator payout and creator-selected wallet system. The funding choice is still open: creator payouts might come from the 42-bps community share or an additional charge. Neither treatment is approved yet. The quoted competitor fee split has not been independently verified.

## Rounding

1. Compute the Beam allocation fee once with Pump's retained ceiling-fee routine at 50 bps.
2. Compute the founder share as `floor(total_fee × 6 / 100)` and the treasury share as `floor(total_fee × 10 / 100)`.
3. Give the remainder to the community.

As an illustration of the allocation math alone, on 100 SOL notional this component is 0.5 SOL: 0.42 community, 0.05 treasury, and 0.03 founder. This is **not an all-in trade quote** while creator payout funding remains undecided. On a one-lamport trade, the rounded component is one lamport, all assigned to the community. Quotes must disclose effective fees on dust trades and every applicable fee component.

`beam_economics.rs` implements only Beam component allocation arithmetic. It is not yet wired into the recovered fee-program CPI, recipient routing or an executable keeper. The current inherited trade code calculates creator fees separately; that describes the present source, **not approval to double-charge in Beam's final policy**. Beam-controlled configuration/vaults must preserve and test creator payout semantics after the funding choice is made. The 42/5/3 split is a requirement for the future Beam implementation, not an enforced live trading fee today.

The requested direction uses the community allocation for holder rewards and
flagship buybacks. If creator payouts are funded from that allocation, those
uses draw from its remainder. A 50/50 split of the available amount is a
provisional test setting, not deployed economics. Holder eligibility, the
flagship mint, recipient vaults, and execution rules have not been selected.

## After graduation

Raydium charges its selected on-chain pool configuration. Beam cannot invent a Raydium share such as 0.10%. Quotes must distinguish Raydium pool fees, creator fees, Beam fees, and migration fees, and must not silently stack the curve's 50 bps on top after graduation. The user selected Raydium AMM v4; the existing gated CPMM migration code is a superseded candidate and does not satisfy that venue requirement.

Recurring buybacks need a collectible fee source. The currently gated CPMM migration candidate burns all LP received by Beam. That removes its position; it does not retain an LP claim to harvest. The selected AMM v4 venue has no Beam-controlled creator or protocol fee switch; its LP fee accrues in pool reserves and is realized through LP withdrawal. Burning all AMM v4 LP would likewise relinquish that claim. Raydium's [Burn & Earn](https://docs.raydium.io/user-flows/burn-and-earn) does not support AMM v4. A Beam 50-bps post-graduation swap fee cannot be assumed from AMM v4. Until an independently reviewed custody/fee-rights design exists, community rewards and buybacks can be funded only from collected pre-graduation proceeds or another explicitly authorized source.

Before activation:

- Specify and test a transparent AMM v4 LP custody/escrow arrangement that preserves principal while defining who may realize fees, or another explicitly supported fee mechanism. Do not describe burned LP as continuing fee income.
- Deny creators, operators and keepers authority to withdraw liquidity principal.
- Set the post-graduation fee budget using the actual Raydium configuration.
- Route flagship community proceeds to a dedicated vault, with public buyback receipts and an explicit burn/retention policy for acquired tokens.
- Bound slippage, price manipulation, account substitutions and keeper reimbursement; isolate execution costs from liquidity principal.

No volume means no revenue or buybacks. A continuing policy does not guarantee returns, price appreciation, execution frequency or perpetual revenue. This is required design work, not completed functionality.
