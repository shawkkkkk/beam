# Beam feature parity ledger

This records evidence from the pinned on-chain reconstruction and Beam's
development source as of 1 November 2026. It is a work ledger, not a claim of
Pump website or binary identity. The archived source includes some companion
programs; copying them into `upstream/` does not deploy or integrate them.

| User-facing behavior | On-chain source coverage | Beam implementation and tests | External work before a usable feature |
| --- | --- | --- | --- |
| Launch, curve buy and sell | Retained [create/buy/sell handlers](../programs/beam/src/lib.rs) and curve state. | Anchor source and selected host ABI/math tests; no end-to-end launch. | Beam fee-program compatibility, complete instruction/runtime tests, IDL, client and deployment. |
| Creator fee and chosen payout address | Curve [creator state](../programs/beam/src/trade_types.rs), [direct collection](../programs/beam/src/creator/collect_creator_fee.rs), and [sharing distribution](../programs/beam/src/creator/distribute_creator_fees.rs) are retained. Creation accepts a creator public key. | Source retained; trade calculation still uses inherited Pump fee CPI. No distinct `payout_wallet` field or user-editable change flow is evidenced by the launchpad ABI. The current CPMM migration burns Beam's LP and does not establish post-graduation creator fees. | Rehost/verify fee configuration, creator vault and sharing authorities; specify wallet-selection UX and permitted changes; test payout and post-graduation AMM v4 fee rights. |
| Donate.gg charity and donations | Pinned [donation relay](../upstream/pumpfun-source/programs/donation-relay/src/lib.rs) and [Pump Fees donation instructions](../upstream/pumpfun-source/programs/pump-fees/src/lib.rs) exist separately from Pump Main. | Archived source only; no Beam donation instruction, configured relay, charity catalog or client flow. | Confirm authorized relay/service access, recipient and settlement rules, fee-program integration, attribution, mock tests and a separate release decision. No real donations now. |
| Callouts | No explicit callout instruction or field identified in the pinned on-chain programs. | No Beam service or client. | Define product behavior and independently implement an authenticated service/UI. Do not infer original app code from mint metadata URI. |
| Livestreaming | No explicit livestream instruction or field identified in the pinned on-chain programs. | No Beam media service or client. | Define moderation, media provider, authorization, storage and UI; test locally. No live stream activation now. |
| Graduation venue | Archived PumpSwap path is blocked; the Beam [CPMM adapter](../programs/beam/src/beam_raydium.rs) is retained as superseded source. | Host checks cover selected accounts/math. Activation and execution of CPMM now fail closed under every build feature. The user selected **Raydium AMM v4**; no AMM v4 adapter exists yet. | Implement the [official AMM v4 `Initialize2` ABI](https://github.com/raydium-io/raydium-amm/blob/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6/program/src/processor.rs#L549-L621), classic SPL Token-only mint gate, fee/custody policy and runtime tests. OpenBook is [no longer required](https://docs.raydium.io/products/amm-v4/overview). |
| Mayhem | [Beam policy](../programs/beam/src/beam_policy.rs) rejects creation/enablement while retaining inert ABI fields. | Host policy checks. | Keep excluded and test every relevant handler/runtime path. |
| Beam 42/5/3 fee component | [Arithmetic](../programs/beam/src/beam_economics.rs) allocates 50 bps among community, treasury and founder. | Unit tests only; no Beam-controlled fee collection or vaults. Creator fees, Raydium pool fees and migration charges are separate. | Enforce authorized config and recipients, preserve creator fee routing, verify exact quotes/accounting and test custody. Community holder/buyback 50/50 is provisional. |
| Market charts, wallet, social UI | On-chain events and metadata URI can be inputs, but are not a website or indexer. | No web/SDK/API on canonical main at this ledger revision; [separate PR #3](https://github.com/shawkkkkk/beam/pull/3) contains a static sample-data preview **and a conflicting alternate Anchor prototype**. Neither is a live client for the canonical contract. | Review the web subtree selectively; independently implement indexer, freshness/verification, wallet client and UX. Never present fixtures as live data or merge the alternate program wholesale. |

Only Raydium AMM v4, removal of Mayhem, and the specified Beam fee policy are
intentional product differences requested so far. They require changes to the
program binary and account/instruction behavior, so whole-program byte identity
is impossible. For every other feature, seek observable behavior and interface
compatibility, document unknowns, preserve licenses, and test each change.

Raydium's current [AMM v4 fee documentation](https://docs.raydium.io/products/amm-v4/fees)
describes an LP fee realized through LP withdrawal. Beam cannot set that pool's
fee or its protocol recipient; the [official program source](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6)
is an ABI reference, not proof of deployed-bytecode identity. AMM v4 does not
support Token-2022 or [Raydium Burn & Earn](https://docs.raydium.io/user-flows/burn-and-earn).
Burning all LP cannot fund continuing post-graduation buybacks or creator fees.
Those features need a separate, reviewed fee-rights design. Beam's retained
curve supports a restricted Token-2022 subset, but AMM v4 does not accept that
mint program. This is an unresolved compatibility conflict, **not** an approved
removal of Token-2022 launches; their graduation must remain unavailable until
an explicit compatible plan is reviewed.
