# Communication Compliance Suite

Status: proposed.

The Communication Compliance Suite is a provider-neutral test contract for
message integrations. It prevents every new provider from inventing a private
definition of message receipt and bypassing canonical trace requirements.

## Provider states

| State | Meaning |
|---|---|
| `unsupported` | Provider cannot support the capability. |
| `deferred` | Capability is planned but not implemented. |
| `backend_test_required` | Capability has a backend test contract, but the lab runner did not execute that validation. |
| `backend_test_pass` | Backend validation command passed and is recorded with command/evidence. |
| `source_check_required` | Capability requires source wiring evidence that has not been checked by the suite report. |
| `source_check_pass` | Source wiring evidence exists in the expected file/symbol. |
| `execute_required` | Capability requires an executed Lab scenario report. |
| `fixture_pass` | Synthetic/fixture path passes. |
| `lab_pass` | Real provider lab path passes. |
| `live_pass` | External/live account evidence exists. |

## Capability groups

| Group | Required scenarios |
|---|---|
| Receive | message observed, message recorded, replay idempotency |
| Send | outbox queued, provider command executed, delivery completed/failed |
| Conversation | channel/thread/topic mapping, participant mapping |
| Mutation | edit, delete/tombstone, reaction |
| Media | attachment observed, safe transfer, quarantine/scan evidence |
| Identity | provider account to identity trace candidate |
| Intelligence | task/person/document candidate with evidence/confidence |
| Search | message indexed and linked to source communication |
| Trace | provider event to UI/debug trace path |

## Report format

```json
{
  "provider": "zulip",
  "suite_version": 1,
  "generated_at": "2026-06-29T00:00:00Z",
  "capabilities": [
    {
      "name": "receive.message.observed",
      "status": "lab_pass",
      "evidence": [".local/makosh-lab/reports/zulip/...json"]
    }
  ]
}
```

## Required rule

A provider capability is not considered closed unless the report contains
source, confidence/evidence, trace identifiers and replay/idempotency notes.

## Zulip command

Zulip publishes a local compliance report through the single Макошь Lab
entrypoint:

```sh
make makosh-lab ACTION=compliance PROVIDER=zulip
```

The report is written under `.local/makosh-lab/reports/zulip/compliance` and is
derived from scenario contracts, local Lab reports and backend evidence reports.
It is an audit by default. To turn it into a closure gate:

```sh
make makosh-lab ACTION=compliance PROVIDER=zulip REQUIRE_CLOSED=1
```

To refresh backend contract evidence before writing the report:

```sh
make makosh-lab ACTION=compliance PROVIDER=zulip BACKEND=1
```
