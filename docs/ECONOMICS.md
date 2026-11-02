# Community-first economics

The user selected a **50-basis-point Beam allocation**: 42 nominal bps for a community allocation, 5 bps for treasury, and 3 bps for founder. That 0.50% is the all-in curve fee. Preserve the creator payout and creator-selected wallet system. Creator payouts are paid out of the 42 bps community share. How many of those 42 bps go to creators is unset; that count, and every other unset owner value, is only in [open owner decisions](OPEN_DECISIONS.md). The quoted competitor fee split has not been independently verified.

## Rounding

1. Compute the Beam allocation fee once with Pump's retained ceiling-fee routine at 50 bps.
2. Compute the founder share as `floor(total_fee × 6 / 100)` and the treasury share as `floor(total_fee × 10 / 100)`.
3. Give the remainder to the community.

As an illustration of the allocation math alone, on 100 SOL notional this component is 0.5 SOL: 0.42 community, 0.05 treasury, and 0.03 founder. This 0.50% curve component is all-in. The illustration does not assign a creator line, because the creator basis-point count inside the 42 bps is unset. On a one-lamport trade, the rounded component is one lamport, all assigned to the community bucket. That dust rule divides the 50 bps component. It does not divide the community bucket between holder rewards and the flagship. Quotes must disclose effective fees on dust trades and every applicable fee component.

`beam_economics.rs` implements only Beam component allocation arithmetic. It is not yet wired into the recovered fee-program CPI, recipient routing or an executable keeper. The current inherited trade code calculates creator fees separately; collection on the retained path is still that inherited CPI. Beam-controlled configuration must pay creators from the 42 bps community share once an admin sets the creator basis-point count. Until that parameter is set it stays disabled or unset and the path fails closed. The 42/5/3 split is the curve policy. It is not an enforced live trading fee today.

The community allocation funds holder rewards and `$BEAM` flagship buybacks
after the creator take. `$BEAM` is the flagship. Every value the owner has
not set is only in [open owner decisions](OPEN_DECISIONS.md).

## After graduation

Raydium charges its selected on-chain pool configuration. Beam cannot invent a Raydium share such as 0.10%. Quotes must distinguish Raydium pool fees, creator fees, Beam fees, and migration fees, and must not silently stack the curve's 50 bps on top after graduation. The user selected Raydium AMM v4; the existing gated CPMM migration code is a superseded candidate and does not satisfy that venue requirement.

Recurring buybacks need a collectible fee source. LP is locked permanently. The currently gated CPMM migration candidate burns all LP received by Beam, and a permanent lock likewise leaves no LP withdrawal. That position is not an LP fee claim. The selected AMM v4 venue has no Beam-controlled creator or protocol fee switch; its LP fee accrues in pool reserves and is realized through LP withdrawal. Raydium's [Burn & Earn](https://docs.raydium.io/user-flows/burn-and-earn) does not support AMM v4. Any post-graduation Beam fee is unset. See [open owner decisions](OPEN_DECISIONS.md). Until an admin sets an authorized source, community rewards and buybacks can be funded only from collected pre-graduation proceeds.

Before activation:

- Keep LP locked permanently. Creators, operators, and keepers have no authority to withdraw liquidity principal. Do not describe locked or burned LP as continuing fee income.
- Leave every unset owner value disabled or unset until an admin sets it. The path fails closed while the parameter is disabled or unset. The values are only in [open owner decisions](OPEN_DECISIONS.md). Raydium's pool configuration remains the pool's own fee.
- Bound slippage, price manipulation, account substitutions and keeper reimbursement; isolate execution costs from liquidity principal.

No volume means no revenue or buybacks. A continuing policy does not guarantee returns, price appreciation, execution frequency or perpetual revenue. This is required design work, not completed functionality.
