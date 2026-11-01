# Beam website preview

This is a static, local product preview adapted from the separate draft PR #3.
It displays sample market data and saves draft ideas in browser storage. Its
creator wallet, donate.gg, callout, and livestream fields are local plans only.
No chain RPC, wallet connection, metadata upload, donation, stream, or trading
action is present. The selected Raydium AMM v4 adapter and Beam fee collection
are not implemented by this preview.

```sh
npm ci --prefix web
npm run build --prefix web
npm run test:unit --prefix web
npm run format:check --prefix web
```

For local browser interaction tests, install a compatible Chromium browser and
run `npm run test:browser --prefix web`. The test starts a loopback Vite preview
and uses synthetic browser state; it does not submit a transaction.

The website and backend contribution lanes are in [the task ledger](../docs/TASKS.md).
