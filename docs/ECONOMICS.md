# Community-first economics

Beam's proposed own fee is 50 basis points: 90% community, 10% platform. This is equivalent to 45 bps and 5 bps of trade value before integer rounding. The user's quoted competitor allocation to its platform is 10 bps; Beam's proposed allocation is half that. The competitor's claim has not been independently verified.

## Rounding

1. Compute the combined fee once with Pump's retained ceiling-fee routine at 50 bps.
2. Compute the platform share as `floor(total_fee / 10)`.
3. Give the remainder to the community.

On 100 SOL notional, the fee is 0.5 SOL: 0.45 community and 0.05 platform. On a one-lamport trade, the rounded fee is one lamport, all assigned to the community. Quotes must disclose effective fees on dust trades.

`beam_economics.rs` implements the allocation. It is not yet wired into the recovered fee-program CPI, recipient routing or an executable keeper. Inherited Pump fee machinery remains until Beam-controlled configuration/vaults replace it. The new rate is not an enforced live trading fee.

The requested direction splits the community share between holder rewards and
flagship buybacks. A 50/50 split within that share is a provisional test setting,
not deployed economics. Holder eligibility, the flagship mint, recipient vaults,
and execution rules have not been selected.

## After graduation

Raydium charges its selected on-chain pool configuration. Beam cannot invent a Raydium share such as 0.10%. Quotes must distinguish pool fees from any Beam fee, and must not silently stack the curve's 50 bps on top after graduation.

Recurring buybacks need a collectible fee source. The currently gated migration candidate burns all LP received by Beam. That removes its position; it does not retain an LP claim to harvest. It cannot fund perpetual buybacks from those burned tokens.

Before activation:

- Implement and test permanently locked principal with community-controlled fee rights, or another explicitly supported fee mechanism.
- Deny creators, operators and keepers authority to withdraw liquidity principal.
- Set the post-graduation fee budget using the actual Raydium configuration.
- Route flagship community proceeds to a dedicated vault, with public buyback receipts and an explicit burn/retention policy for acquired tokens.
- Bound slippage, price manipulation, account substitutions and keeper reimbursement; isolate execution costs from liquidity principal.

No volume means no revenue or buybacks. A continuing policy does not guarantee returns, price appreciation, execution frequency or perpetual revenue. This is required design work, not completed functionality.
