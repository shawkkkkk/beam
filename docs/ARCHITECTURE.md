# Beam implementation path

## Scope and reference

Pump Main in `lalalune/pumpfun-source` is a reconstructed Solana program.
Beam uses its curve quote and reserve-update behavior as a reference. Exact
binary or byte-for-byte identity is impossible once the program ID, migration
instructions, fee routing, and Mayhem behavior change. Functional parity for
the launch flow is the practical target; each instruction needs tests against
reference vectors before it is called compatible.

`beam-core` is pure accounting. `programs/beam/` now contains an experimental
Anchor `create`, `buy`, and `sell` path for a classic SPL Token mint with six
decimals. `create` mints a fixed supply into a launch PDA's token vault and
removes mint authority. Buys and sells use the shared curve math; quote and
accrued fee balances stay in the launch account. The caller currently chooses
curve reserves, and the program has no token metadata, fee claims, Raydium
migration, or deployment keypair. Its ID is a development placeholder. No
on-chain program has been deployed. There is no PumpSwap or Mayhem dependency.

## On-chain work to implement

1. Add immutable or tightly governed launch parameters, metadata creation,
   exact-quote buys, partial fills, and parity fixtures from Pump Main. Validate
   account state and errors as well as arithmetic. The current program covers
   only exact-token buys and sells, with caller-provided curve parameters.
2. Implement authorized on-chain fee claims or buybacks and a documented
   rewards distribution policy. The current fee amounts accrue in each launch
   account but cannot be withdrawn. A recorded 0.40% allocation is not a
   payout. Prove SOL escrow and rent invariants across claims and migration.
3. Replace PumpSwap migration with a CPI to Raydium's legacy AMM v4
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
4. Burn the LP tokens minted to Beam's migration authority in the same
   transaction, after validating the Raydium pool and minted LP balance. This
   makes liquidity non-withdrawable; it also means Beam cannot claim LP fees
   as a revenue stream. Test this policy against the actual deployed AMM.
5. Run local validator and devnet integration tests for create, buy, sell,
   graduation, failed CPI rollback, fee claims, and pool liquidity custody.
   Obtain independent security review before mainnet use.

Raydium AMM v4's source currently identifies mainnet program
`675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`. The post-graduation swap
fee and liquidity behavior come from that deployed program, not from Beam's
`FeePolicy` constant.

The launch fee proposal is 0.50% total, with 0.40% routed to rewards or
buybacks, 0.07% to a protocol treasury, and 0.03% to the founder. This is a
deliberate proposal for review, not yet an on-chain guarantee.
