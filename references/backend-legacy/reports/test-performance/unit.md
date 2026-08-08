# unit nextest report

- Generated at: 2026-06-27T23:18:55.999Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 277
- Failed tests: 0
- Flaky tests: none detected
- Total time: 200.45s
- Average time: 0.724s
- p95: 6.467s
- p99: 9.608s

## Slowest tests

1. `makosh-backend::integrations::telegram::runtime::manager::participants::participants_runtime_tests::sync_provider_roster_snapshots_appends_leave_reconciliation_after_absence_update` - 10.343s
2. `makosh-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_unread_event_reconciles_mark_read_command_and_emits_events` - 9.797s
3. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_edited_event_skips_without_projected_message` - 9.608s
4. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event` - 9.447s
5. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_reaction_changed_event_skips_without_projected_message` - 9.127s
6. `makosh-backend::integrations::telegram::runtime::manager::chat_events::tests::publish_chat_position_event_reconciles_folder_add_and_remove_commands` - 9.061s
7. `makosh-backend::integrations::telegram::runtime::manager::realtime_events::tests::telegram_runtime_event_bridge_skips_broadcast_when_runtime_paused` - 9.033s
8. `makosh-backend::integrations::telegram::runtime::manager::realtime_events::typing_tests::publish_command_reconciled_events_appends_status_and_reconciled_records` - 8.961s
9. `makosh-backend::integrations::telegram::runtime::manager::topic_events::tests::publish_topic_event_reconciles_topic_close_and_appends_runtime_events` - 8.746s
10. `makosh-backend::integrations::telegram::runtime::manager::message_events::tests::publish_message_content_updated_event_skips_without_projected_message` - 8.677s
