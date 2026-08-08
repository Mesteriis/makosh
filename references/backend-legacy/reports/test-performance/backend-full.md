# backend-full nextest report

- Generated at: 2026-07-11T03:50:34.631Z
- Source JUnit: `target/nextest/default/junit.xml`
- Total tests: 1566
- Failed tests: 0
- Flaky tests: 1
- Total time: 7933.529s
- Average time: 5.066s
- p95: 12.093s
- p99: 19.758s

## Slowest tests

1. `makosh-backend::v1_communications_archive_inspection::v1_attachment_archive_inspection_reads_local_zip_blob_against_postgres` - 134.497s
2. `makosh-backend::v1_communications_attachment_preview::v1_attachment_preview_reads_bounded_local_pdf_blob_against_postgres` - 133.154s
3. `makosh-backend::v1_communications_attachment_preview::v1_attachment_preview_reads_bounded_local_text_blob_against_postgres` - 119.326s
4. `makosh-backend::v1_communications_attachment_preview::v1_attachment_preview_reads_bounded_local_image_blob_against_postgres` - 119.259s
5. `makosh-backend::graph_api::search::graph_search_returns_matching_nodes` - 32.88s
6. `makosh-backend::graph_api::search::graph_summary_returns_empty_state_for_empty_database` - 26.177s
7. `makosh-backend::gmail_send_api::gmail_send_api_queues_outbox_when_send_scope_enabled_against_postgres` - 24.117s
8. `makosh-backend::zoom_provider_foundation::zoom_remote_transcript_downloads_are_blocked_until_privacy_opt_in_is_enabled` - 23.81s
9. `makosh-backend::graph::graph_store_upserts_node_idempotently_against_postgres` - 21.407s
10. `makosh-backend::graph_api::neighborhood::graph_neighborhood_caps_depth_one_edges_nodes_and_evidence` - 21.302s
