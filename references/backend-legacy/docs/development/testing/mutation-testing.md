# Mutation Testing

Макошь uses `cargo-mutants` for targeted mutation analysis.

Command:

- `make mutants`

Current policy:

- use `nextest` as the test tool
- keep this out of the default PR gate because runtime is high
- run it in nightly CI and manually on risky backend work

Because mutation testing is expensive, treat it as a scheduled or pre-merge quality signal, not as a default edit-loop command.
