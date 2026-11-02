# Low-usage PR review proposal

This proposal is **inactive**. The accompanying workflow has only a manual
`workflow_dispatch` trigger and read permissions. The draft PR must not be
merged or activated without a separate maintainer review. It does not approve,
mark ready, enable auto-merge, merge, label, comment, or change repository
settings. Feature development and live deployment remain paused.

## Current repository facts (2 November 2026)

- `shawkkkkk/beam` is public. The repository API reports
  `allow_auto_merge: false`; native auto-merge is currently disabled.
- The read-only rulesets endpoint returned an empty list. The connected GitHub
  App received `403 Resource not accessible by integration` for main's branch
  protection endpoint, so required reviews/checks and merge-queue policy are
  **unknown**. An empty ruleset list does not prove there is no protection.
- Existing workflows are contract, website, native SDK/service, and AMM v4
  local-validator tests. They request read permissions. Website browser tests
  are separate from hosted website CI; a green check is not full product
  validation.
- The repository owner is `shawkkkkk`. This proposal creates no bot account,
  personal token, GitHub App installation, or secret.

## What the draft supplies

`.github/workflows/pr-review-snapshot.yml` reads one PR on manual request. It
prints its head SHA, draft/mergeable API values, and the first page of check
runs and reviews in the run summary. It uses the built-in ephemeral
`GITHUB_TOKEN` with `contents: read`, `pull-requests: read`, and `checks: read`.
It does not check out or execute PR code and does not treat a label, green job,
or review count as approval. API results are a point-in-time snapshot; a new
head invalidates any previous assessment. The first-page cap also means the
summary cannot prove every check or review was inspected.

## Activation decision, separately required

The least complex future route is GitHub's native auto-merge after maintainers
verify a protection policy with required human review and the exact relevant
status checks. GitHub says native auto-merge waits for those requirements;
the repository setting must be enabled first. Do not assume a merge queue is
configured for this personal repository. If a PR changes contract funds,
fees, authorization, workflows, migrations, or dependency trust, require
deliberate human review of its combined tree and applicable runtime tests.

If maintainers later want an event-driven *report*, a separately reviewed
change could use `pull_request` events without write permission. If they want
a workflow to *enable* auto-merge, that requires a separate action-time
approval for `pull-requests: write`, a trusted event design, repository
auto-merge setting, and verified protection rules. Never run or check out
untrusted PR code in a privileged workflow. Do not add `pull_request_target`
for this read-only task; GitHub documents its elevated trust and current
public-repository event-policy limits. `gh pr merge --match-head-commit` is an
immediate race guard for one action, not a permanent pin on later native
auto-merge. A new head must be reviewed again.

Slop automatic task closure is unrelated to GitHub approval or merging. No
workflow should self-approve, bypass protections, use admin override, silently
merge a draft, or infer production readiness from source/host/mock checks.

References: [GitHub native auto-merge](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/automatically-merging-a-pull-request),
[GitHub `pull_request_target` security](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target),
and [`gh pr merge` options](https://cli.github.com/manual/gh_pr_merge).
