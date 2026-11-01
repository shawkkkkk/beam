# Beam

Beam is an in-development, Raydium-first Solana launchpad. No on-chain Beam
program or Raydium pool integration has been deployed.

## Current implementation

`beam-core` contains checked, dependency-free accounting for a Pump-style
bonding curve, buy/sell quotes, a transparent launch fee split, and the token
and quote amounts available at graduation. The calculations are a foundation
for an on-chain program; they cannot hold funds or create pools by themselves.

`web/` is a responsive Beam interface preview with original blue, purple, and
black branding. It includes sample token discovery, search, a browser watchlist,
token details, and local token drafts. Market data is illustrative. Wallet
connection, minting, and trading are not live.

The proposed **pre-graduation fee is 0.50%** of gross quote traded:

| Destination | Share of trade | Share of Beam fee |
| --- | ---: | ---: |
| Token rewards / buybacks | 0.40% | 80% |
| Protocol treasury | 0.07% | 14% |
| Founder | 0.03% | 6% |

Fee rounding favors rewards. These numbers are a proposal encoded in the
default `FeePolicy`, not a claim about a live protocol. Post-graduation Raydium
swap fees are governed by the selected Raydium pool configuration and LP
ownership; Beam cannot impose this fee split on all Raydium swaps.

## Develop

Install Rust 1.99.1, then run:

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

For the website, use Node.js 24 and run:

```sh
cd web
npm ci
npm run build
npm run format:check
npm run dev
```

The development server listens on port 5173. The website uses self-hosted fonts
and needs no API key.

The repository has no required secret. See
[the architecture notes](docs/ARCHITECTURE.md) for the on-chain integration
work and limits of current parity.

Implementation is developed through reviewable pull requests. The current
source is not audited and must not be used to custody assets.
