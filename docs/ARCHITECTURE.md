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
4. Replace PumpSwap migration with a CPI to Raydium CP-Swap pool initialization.
   Use the canonical CP-Swap program and configuration accounts, mint ordering,
   vault PDAs, LP mint, fee recipient, token programs, and pool start time from
   the deployed Raydium version. Verify required pool creation fees and token
   extension support before permitting a launch. Transfer the completed
   curve's reserved tokens and net quote into the pool atomically.
5. Define an enforceable LP policy before migration: a lock or a permanent
   liquidity position with clearly documented fee rights. Raydium CP-Swap's
   minimum LP lock in `initialize` does not lock the remaining LP minted to
   the creator. Track LP ownership and fee distribution on chain.
6. Run local validator and devnet integration tests for create, buy, sell,
   graduation, failed CPI rollback, fee claims, and pool liquidity custody.
   Obtain independent security review before mainnet use.

Raydium CP-Swap's source describes permissionless pool creation and a 100-unit
minimum LP lock in its initializer. Those details are version-dependent. The
post-graduation swap fee and LP fee rights come from Raydium's configuration,
not from the Beam `FeePolicy` constant.

The launch fee proposal is 0.50% total, with 0.40% routed to rewards or
buybacks, 0.07% to a protocol treasury, and 0.03% to the founder. This is a
deliberate proposal for review, not yet an on-chain guarantee.
