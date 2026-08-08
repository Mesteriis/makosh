# Макошь legacy backend reference

This directory contains the complete Rust backend workspace as it existed before
the clean-room rewrite started on 2026-07-15.

It is reference material, not production code:

- do not add it to the new Cargo workspace;
- do not depend on its crates from new code;
- use it only to recover verified behavior, event names, provider fixtures and
  security constraints;
- port behavior through a new contract and a focused test, never by moving an
  owner tree wholesale;
- treat a provider as supported only when an executable fixture or isolated
  runtime test proves it.

The preserved workspace can be inspected from this directory. Its build state
and containers were intentionally removed before archival. Vault and provider
session state were not moved or deleted.

Legacy operational material is preserved beside the workspace:

- `Makefile` and `scripts/`;
- `bacon.toml`, `deny.toml` and `.pre-commit-config.yaml`;
- previous GitHub CI, nightly, Pages workflows and pull request template;
- `docs/` with the complete previous documentation tree, archived ADR,
  generated wiki, product/domain specifications, status and testing guides;
- `canonical-evidence-final-report.md` and `reports/` with historical
  implementation/performance evidence;
- `frontend/README.md` with the previous full-stack, API and packaging
  instructions.

These files document the old command surface. They are not expected to run from
this directory: the active frontend and root infrastructure were intentionally
not duplicated into the reference tree. Do not invoke them as clean-room
validation or restore them to the repository root without a new executable
contract.

Only ADR-0200…ADR-0206 and the minimal clean-room architecture summaries remain
active under the repository-root `docs/` directory.
