# Zulip Fixture Test Matrix

Status: proposed.

| Scenario | Provider action | Expected Макошь trace | Phase |
|---|---|---|---:|
| Message observed | Send channel/topic message | `signal.raw.zulip.message.observed` | 1 |
| Message recorded | Send channel/topic message | `communication.message.recorded` | 2 |
| Topic thread mapped | Send two messages in same topic | one conversation/thread evidence chain | 2 |
| Provider send API | Send stream/direct message through REST client | Zulip API response contract | 2 |
| Provider edit API | Edit message through REST client | Zulip API response contract | 2 |
| Provider delete API | Delete message through REST client | Zulip API response contract | 2 |
| Provider reaction API | Add/remove reaction through REST client | Zulip API response contract | 2 |
| Provider upload API | Upload file bytes through REST client | Zulip API upload response contract | 2 |
| Durable command enqueue | Queue Zulip provider command | `communication_provider_commands` row | 2 |
| Durable command retry | Fail and retry Zulip provider command | retrying/completed command lifecycle | 2 |
| Reaction observed | Add emoji reaction | `signal.raw.zulip.reaction.observed` | 2 |
| Reaction materialized | Add emoji reaction | `communication_message_reactions` + `communication.message.updated` | 2 |
| Edit observed | Edit message content/topic | `signal.raw.zulip.message_update.observed` | 2 |
| Edit materialized | Edit message content | `communication_message_versions` + `communication.message.updated` | 2 |
| Delete observed | Delete/tombstone message | `signal.raw.zulip.message_delete.observed` | 2 |
| Delete materialized | Delete/tombstone message | `communication_message_tombstones` + `communication.message.updated` | 2 |
| Attachment observed | Upload file and send link | attachment evidence + quarantine state | 3 |
| Identity trace | Bot/user sends message | provider account -> identity trace candidate | 3 |
| Task candidate | Send action-like text | Review task candidate with evidence | 3 |
| Promotion | Promote Review candidate | target Task created with causation chain | 3 |
| Search projection | Search by unique phrase | search result links to source communication | 3 |
| Full replay | Replay queue events | no duplicated durable records | 4 |

## Required report fields

Every scenario report must include:

```text
scenario_id
run_id
provider
lab_correlation_id
started_at
finished_at
provider_actions[]
provider_events[]
expected_stages[]
observed_stages[]
failures[]
```
