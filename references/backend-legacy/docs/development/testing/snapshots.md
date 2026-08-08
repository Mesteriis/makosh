# Snapshot Testing

Макошь uses `insta` for stable output snapshots.

Current baseline target:

- `backend/tests/snapshot_smoke.rs`

Commands:

- `make snapshot-test`
- `make snapshot-accept`

Acceptance flow:

1. Run `make snapshot-test`
2. Inspect `.snap.new` output if the snapshot changed
3. Accept the update with `make snapshot-accept`

Snapshot tests are intended for:

- JSON payloads
- Connect/HTTP response shapes
- parsing output
- event payloads
- Markdown generation
