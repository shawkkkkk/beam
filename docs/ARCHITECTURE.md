# Beam implementation path

## Scope and reference

Pump Main in `lalalune/pumpfun-source` is a reconstructed Solana program.
Beam uses its curve quote and reserve-update behavior as a reference. Exact
binary or byte-for-byte identity is impossible once the program ID, migration
instructions, fee routing, and Mayhem behavior change. Functional parity for
the launch flow is the practical target; each instruction needs tests against
reference vectors before it is called compatible.

The current Rust crate is pure accounting only. It does not create mints,
accept trades, distribute rewards, migrate liquidity, or deploy a program.
There is no PumpSwap or Mayhem dependency in Beam.

## On-chain work to implement

1. Create a Beam Anchor program with a distinct program ID, mint and curve
   accounts, metadata, authorities, and explicit token program support. Check
   account ownership, PDA seeds, actual vault balances, overflow, and slippage.
2. Port the tested curve math into `create`, `buy`, and `sell` instructions.
   Compare outputs and account state against Pump Main fixtures, including
   partial fills, rounding, completion, and error cases. The core currently
   models only fixed token buy and sell quantities.
3. Implement on-chain fee vaults and claim or buyback rules. The 0.40% reward
   allocation needs an explicit recipient and auditable distribution policy;
   a quoted allocation alone is not a payout.
4. Replace PumpSwap migration with a CPI to Raydium's legacy AMM v4
   `Initialize2`, matching the requested pre-PumpSwap destination. Use the
   canonical deployed program ID, config, authority, pool, vault, target-order,
   LP mint, fee destination, and market seed accounts. Verify the exact
   deployed version's account list and pool creation fee before building the
   instruction. The current Raydium source accepts a 19-account initializer
   without an OpenBook market, while its README still documents market
   creation; this discrepancy needs a deployed-program check. AMM v4's
   initializer currently requires classic SPL Token mints, so launches must
   reject incompatible mint extensions. Transfer the completed curve's
   reserved tokens and net quote into the pool atomically.
5. Burn the LP tokens minted to Beam's migration authority in the same
   transaction, after validating the Raydium pool and minted LP balance. This
   makes liquidity non-withdrawable; it also means Beam cannot claim LP fees
   as a revenue stream. Test this policy against the actual deployed AMM.
6. Run local validator and devnet integration tests for create, buy, sell,
   graduation, failed CPI rollback, fee claims, and pool liquidity custody.
   Obtain independent security review before mainnet use.

Raydium AMM v4's source currently identifies mainnet program
`675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`. The post-graduation swap
fee and liquidity behavior come from that deployed program, not from Beam's
`FeePolicy` constant.

The launch fee proposal is 0.50% total, with 0.40% routed to rewards or
buybacks, 0.07% to a protocol treasury, and 0.03% to the founder. This is a
deliberate proposal for review, not yet an on-chain guarantee.
