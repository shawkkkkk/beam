# Community allocation policy

Scope 5 from the [task ledger](TASKS.md). Base commit `4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf` (`main`). This change is a disabled integer model in `programs/beam/src/beam_community_allocation.rs`. It adds one module declaration after the retained program body. It adds no instruction, account, transfer, keeper, or market order.

The model consumes the community integer from `beam_economics::allocate` (42/5/3 nominal bps for community/treasury/founder). Treasury and founder lamports stay on that result. Draft PRs #23, #24, and #26 are related fee work and are not merged here. This branch does not depend on them. Error codes start at 7200 so they stay distinct from Beam policy errors at 7000 and from the 7100 codes proposed on draft PR #24.

Holder eligibility, the flagship mint, recipients, and execution policy remain open. The questions below are for the maintainer. Answering them in review does not enable collection.

## What the model evaluates

- Default: `AllocationModel::disabled()`, creator funding `Undecided`, both shares 0. `divide` returns `Disabled` (7200).
- `CreatorFunding::SeparateCharge`: the whole community integer is divided. A non-zero creator amount is rejected, so this path cannot also deduct a creator payout.
- `CreatorFunding::CommunityRemainder`: the caller supplies an exact creator amount. It is removed first. Holder rewards and flagship buybacks divide only the remainder. A creator amount above the community integer is rejected, including when treasury and founder lamports could cover the difference.
- Share parameters are parts of the **available** community amount per 10_000. They use a different scale from the 42/5/3 trade-component basis points. The provisional test setting is 5_000 / 5_000.
- Rounding: `floor` each configured share. Their gap is 0 or 1 lamport because the shares sum to 10_000. `DustAssignment` gives that lamport to holder rewards or to the flagship leg. A 0-share leg receives 0.
- Conservation: `community = creator_taken + holder_rewards + flagship_buyback`. With the proposed component, `total = treasury + founder + community`.
- A positive share requires a non-zero recipient pubkey. A positive creator deduction requires a non-zero creator pubkey. The all-zero pubkey is unset. A zero share, and a zero creator deduction, may leave that pubkey unset.
- The same pubkey may currently be passed for more than one leg. That behavior is temporary until question 7 is answered.

`divide_proposed_community_leg` only reads `beam_economics::allocate`. On a 100 SOL quote the current helper returns 500_000_000 lamports: 420_000_000 community, 50_000_000 treasury, 30_000_000 founder. An even split of that community integer is 210_000_000 / 210_000_000. Those figures illustrate the arithmetic. They are a component quote while creator funding is unresolved, and they are not an all-in trading fee.

## What stays unwired

The module calculates no creator fee schedule, collects no trade fee, and submits no buyback. It records no entitlement to future volume, price, execution frequency, or perpetual revenue. Inherited Pump fee CPIs are unchanged. Post-graduation funding still depends on a reviewed fee-rights design; burned LP is not treated as income. Scopes 3, 4, and 7 are untouched.

Draft PR #23 changes founder and treasury rounding on some dust trades, which changes the community integer. This splitter conserves whatever community integer `allocate` returns. The 100 SOL illustration is the same under both rounding rules. Draft PR #24 proposes a disabled fee-policy account. These parameters can move there after that account exists. Draft PR #26 exercises policy initialization in a local bank and is a separate runtime lane.

## Questions for the maintainer

1. Which creator-funding config should become the product rule: deduct the creator payout from the community allocation, or keep it as a separate charge?
2. What holder/flagship shares of the available amount should replace the provisional 5_000 / 5_000 test setting?
3. Which leg should receive the single rounding lamport?
4. Who is eligible for holder rewards (minimum balance, snapshot slot, excluded accounts, vesting, claim versus push)?
5. What is the flagship mint, and who may change it?
6. Which accounts receive the holder leg and the flagship leg, and who may rotate those accounts?
7. Must those two recipient accounts be distinct?
8. May one leg be configured at 0 share?
9. What execution policy applies to a future flagship buyback (authority, slippage bound, cadence, price source, and failure behavior)?
10. After a buyback, are acquired flagship tokens burned, held, or distributed?
11. Which balances may fund the community integer before graduation and after graduation? Should a creator amount larger than the community integer fail the quote, cap the creator payout, or switch that trade to a separate charge?
12. Once a Beam fee-policy account exists, should these parameters live on it?
13. Should any later activation wait until the founder/treasury dust cap from draft PR #23 is accepted?
