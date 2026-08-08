import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const COMMUNICATIONS_INGRESS_ROOT = new URL('src/communications-ingress/src/', BACKEND_ROOT);
const COMMUNICATIONS_ATTACHMENT_CONTRACT_ROOT = new URL('src/communications-attachment-contract/src/', BACKEND_ROOT);
const COMMUNICATIONS_API_ROOT = new URL('src/communications-api/src/', BACKEND_ROOT);
const COMMUNICATIONS_DOMAIN_ROOT = new URL('src/communications-domain/src/', BACKEND_ROOT);
const COMMUNICATIONS_PERSISTENCE_ROOT = new URL('src/communications-persistence/src/', BACKEND_ROOT);
const COMMUNICATIONS_RUNTIME_ROOT = new URL('src/communications-runtime/src/', BACKEND_ROOT);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const FORBIDDEN_INTEGRATION_IMPLEMENTATIONS = [
  'makosh_mail_',
  'makosh_telegram_',
  'makosh_whatsapp_',
  'makosh_zulip_',
];
const FORBIDDEN_DOMAIN_IMPLEMENTATIONS = [
  ...FORBIDDEN_INTEGRATION_IMPLEMENTATIONS,
  'makosh_blob_',
];

test('Communications domain does not import integration or Blob implementations', async () => {
  const sources = await rustSources(COMMUNICATIONS_DOMAIN_ROOT);

  assert.ok(sources.length > 0);
  for (const source of sources) {
    for (const implementation of FORBIDDEN_DOMAIN_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden owner implementation ${implementation}`,
      );
    }
  }
});

test('Communications remains isolated after Knowledge owner admission', async () => {
  const [
    policySource,
    ingressSources,
    attachmentContractSources,
    apiSources,
    domainSources,
    persistenceSources,
    runtimeSources,
  ] = await Promise.all([
    readFile(POLICY_PATH, 'utf8'),
    rustSources(COMMUNICATIONS_INGRESS_ROOT),
    rustSources(COMMUNICATIONS_ATTACHMENT_CONTRACT_ROOT),
    rustSources(COMMUNICATIONS_API_ROOT),
    rustSources(COMMUNICATIONS_DOMAIN_ROOT),
    rustSources(COMMUNICATIONS_PERSISTENCE_ROOT),
    rustSources(COMMUNICATIONS_RUNTIME_ROOT),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(
    policy.implementation.currentSlice,
    'call_transcription_managed_conformance_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory, {
    domains: ['communications', 'contacts', 'knowledge', 'review', 'tasks'],
    integrations: ['desktop_call_recording', 'mail'],
    workflows: [
      'attachment_preview',
      'attachment_preview_evidence_replay',
      'attachment_text_extraction',
      'attachment_translation',
      'call_transcription',
      'communication_cross_channel_forward',
      'communication_delivery_intent',
      'communication_explanation',
      'communication_note_candidate_extraction',
      'communication_recipient_suggestion',
      'communication_reply_suggestion',
      'communication_summary',
      'communication_task_candidate_extraction',
      'communication_translation',
      'communications_export',
      'mail_contacts_sync',
      'reviewed_note_candidate_promotion',
      'reviewed_task_candidate_promotion',
    ],
    engines: ['ai', 'attachment_archive_inspection', 'attachment_security', 'speech_to_text'],
    businessCapabilities: [
      'ai.attachment-translation.request.v1',
      'ai.explanation.request.v1',
      'ai.provider.explain.v1',
      'ai.provider.translate.v1',
      'ai.translation.request.v1',
      'attachment-preview-evidence-replay.command.v1',
      'attachment.archive_inspection.v1',
      'attachment.preview.v1',
      'attachment.text_extraction.v1',
      'attachment.translation.v1',
      'attachment_archive_inspection.blob.v1',
      'attachment_archive_inspection.candidate.observe.v1',
      'attachment_archive_inspection.custody-request.publish.v1',
      'attachment_archive_inspection.custody-result.consume.v1',
      'attachment_archive_inspection.safety-state.observe.v1',
      'attachment_archive_inspection.storage.v1',
      'attachment_security.archive-delegation-result.publish.v1',
      'attachment_security.archive-inspection-delegation.v1',
      'attachment_security.blob.v1',
      'attachment_security.candidate.observe.v1',
      'attachment_security.communications-state.observe.v1',
      'attachment_security.storage.v1',
      'attachment_security.text-extraction-delegation-result.publish.v1',
      'attachment_security.text-extraction-delegation.v1',
      'attachment_security.verdict.publish.v1',
      'attachment_text_extraction.translation-source.v1',
      'attachment_translation.blob.v1',
      'attachment_translation.inference.v1',
      'attachment_translation.source_prepared.v1',
      'attachment_translation.source_rejected.v1',
      'attachment_translation.source_requested.v1',
      'attachment_translation.storage.v1',
      'call_transcription.blob.v1',
      'call_transcription.recording_ready.v1',
      'call_transcription.recording_rejected.v1',
      'call_transcription.storage.v1',
      'call_transcription.stt.v1',
      'call_transcription.v1',
      'communication.cross_channel_forward.v1',
      'communication.explanation.v1',
      'communication.note-candidate-extraction.v1',
      'communication.recipient-suggestion.v1',
      'communication.summary.v1',
      'communication.task-candidate-extraction.v1',
      'communication.translation.v1',
      'communication_cross_channel_forward.blob.v1',
      'communication_cross_channel_forward.delivery_rejected.v1',
      'communication_cross_channel_forward.delivery_submit.v1',
      'communication_cross_channel_forward.delivery_submitted.v1',
      'communication_cross_channel_forward.source_prepare.v1',
      'communication_cross_channel_forward.source_prepared.v1',
      'communication_cross_channel_forward.source_rejected.v1',
      'communication_cross_channel_forward.storage.v1',
      'communication_delivery_intent.blob.v1',
      'communication_delivery_intent.ingress_rejected.v1',
      'communication_delivery_intent.ingress_submit.v1',
      'communication_delivery_intent.ingress_submitted.v1',
      'communication_delivery_intent.mail.events.v1',
      'communication_delivery_intent.storage.v1',
      'communication_delivery_intent.telegram.events.v1',
      'communication_delivery_intent.whatsapp.events.v1',
      'communication_delivery_intent.zulip.events.v1',
      'communication_explanation.inference.v1',
      'communication_explanation.source.blob.v1',
      'communication_explanation.source_prepare.v1',
      'communication_explanation.source_prepared.v1',
      'communication_explanation.source_rejected.v1',
      'communication_explanation.storage.v1',
      'communication_note_candidate_extraction.source.blob.v1',
      'communication_note_candidate_extraction.source_prepare.v1',
      'communication_note_candidate_extraction.source_prepared.v1',
      'communication_note_candidate_extraction.source_rejected.v1',
      'communication_note_candidate_extraction.storage.v1',
      'communication_recipient_suggestion.source.blob.v1',
      'communication_recipient_suggestion.source_prepare.v1',
      'communication_recipient_suggestion.source_prepared.v1',
      'communication_recipient_suggestion.source_rejected.v1',
      'communication_recipient_suggestion.storage.v1',
      'communication_summary.inference.v1',
      'communication_summary.source.blob.v1',
      'communication_summary.source_prepare.v1',
      'communication_summary.source_prepared.v1',
      'communication_summary.source_rejected.v1',
      'communication_summary.storage.v1',
      'communication_task_candidate_extraction.source.blob.v1',
      'communication_task_candidate_extraction.source_prepare.v1',
      'communication_task_candidate_extraction.source_prepared.v1',
      'communication_task_candidate_extraction.source_rejected.v1',
      'communication_task_candidate_extraction.storage.v1',
      'communication_translation.inference.v1',
      'communication_translation.source.blob.v1',
      'communication_translation.source_prepare.v1',
      'communication_translation.source_prepared.v1',
      'communication_translation.source_rejected.v1',
      'communication_translation.storage.v1',
      'communications.ai-explanation-source.blob.v1',
      'communications.ai-explanation-source.v1',
      'communications.ai-reply-source.blob.v1',
      'communications.ai-reply-source.v1',
      'communications.ai-summary-source.blob.v1',
      'communications.ai-summary-source.v1',
      'communications.ai-translation-source.blob.v1',
      'communications.ai-translation-source.v1',
      'communications.attachment.blob-admission.observe.v1',
      'communications.attachment.safety-verdict.observe.v1',
      'communications.blob.v1',
      'communications.content.v1',
      'communications.cross-channel-forward-source.blob.v1',
      'communications.cross-channel-forward-source.v1',
      'communications.events.v1',
      'communications.export-source.blob.v1',
      'communications.export-source.v1',
      'communications.export.v1',
      'communications.note-source.blob.v1',
      'communications.note-source.v1',
      'communications.observe.v1',
      'communications.query.v1',
      'communications.recipient-source.blob.v1',
      'communications.recipient-source.v1',
      'communications.saved-search.v1',
      'communications.search.index.v1',
      'communications.sender-insights.v1',
      'communications.storage.v1',
      'communications.task-source.blob.v1',
      'communications.task-source.v1',
      'communications_export.blob.v1',
      'communications_export.events.v1',
      'communications_export.storage.v1',
      'contacts.mail-identity.command.v1',
      'contacts.mail-sync-source.blob-writer.v1',
      'contacts.mail-sync-source.changed.v1',
      'contacts.mail-sync-source.v1',
      'knowledge.reviewed-candidate.blob.v1',
      'knowledge.reviewed-candidate.command.v1',
      'knowledge.reviewed-candidate.created.publisher.v1',
      'knowledge.reviewed-candidate.rejected.publisher.v1',
      'knowledge.storage.v1',
      'mail.address-book.contact-source.blob.v1',
      'mail.address-book.provider.v1',
      'mail.attachment-anchor.consume.v1',
      'mail.attachment-blob-admission.publish.v1',
      'mail.attachment-safety-state.consume.v1',
      'mail.attachment.scan-candidate.publish.v1',
      'mail.blob.v1',
      'mail.communication-observed.publish.v1',
      'mail.contacts-sync.v1',
      'mail.delivery.query.v1',
      'mail.delivery.v1',
      'mail.gmail.credentials.v1',
      'mail.gmail.oauth-refresh.credentials.v1',
      'mail.gmail.oauth-setup.credentials.v1',
      'mail.imap.credentials.v1',
      'mail.oauth.complete.v1',
      'mail.oauth.query.v1',
      'mail.oauth.refresh.v1',
      'mail.oauth.start.v1',
      'mail.smtp.credentials.v1',
      'mail.storage.v1',
      'mail.sync.v1',
      'mail_contacts_sync.contacts.changed.v1',
      'mail_contacts_sync.contacts.command.v1',
      'mail_contacts_sync.contacts.rejected.v1',
      'mail_contacts_sync.contacts.source-prepare.v1',
      'mail_contacts_sync.contacts.source-prepared.v1',
      'mail_contacts_sync.contacts.source-rejected.v1',
      'mail_contacts_sync.contacts.upserted.v1',
      'mail_contacts_sync.mail.entry-observed.v1',
      'mail_contacts_sync.mail.entry-upsert-rejected.v1',
      'mail_contacts_sync.mail.entry-upserted.v1',
      'mail_contacts_sync.mail.fetch-page.v1',
      'mail_contacts_sync.mail.page-completed.v1',
      'mail_contacts_sync.mail.page-rejected.v1',
      'mail_contacts_sync.mail.upsert-entry.v1',
      'mail_contacts_sync.scheduler.receipt.v1',
      'mail_contacts_sync.scheduler.v1',
      'mail_contacts_sync.storage.v1',
      'review.communication-attention.command.v1',
      'review.communication-attention.query.v1',
      'review.communication-attention.realtime.v1',
      'review.communication-attention.storage.v1',
      'review.note-candidate.blob.v1',
      'review.note-candidate.client.v1',
      'review.note-candidate.promotion-result.consumer.v1',
      'review.note-candidate.promotion-result.v1',
      'review.note-candidate.promotion.v1',
      'review.note-candidate.storage.v1',
      'review.note-candidate.submission.v1',
      'review.task-candidate.blob.v1',
      'review.task-candidate.client.v1',
      'review.task-candidate.promotion-result.consumer.v1',
      'review.task-candidate.promotion-result.v1',
      'review.task-candidate.promotion.v1',
      'review.task-candidate.storage.v1',
      'review.task-candidate.submission.v1',
      'reviewed-note-candidate-promotion.source.blob.v1',
      'reviewed_note_candidate_promotion.knowledge-command.publish.v1',
      'reviewed_note_candidate_promotion.knowledge-created.consume.v1',
      'reviewed_note_candidate_promotion.knowledge-rejected.consume.v1',
      'reviewed_note_candidate_promotion.review-approved.consume.v1',
      'reviewed_note_candidate_promotion.review-result.publish.v1',
      'reviewed_note_candidate_promotion.storage.v1',
      'reviewed_task_candidate_promotion.review-approved.consume.v1',
      'reviewed_task_candidate_promotion.review-result.publish.v1',
      'reviewed_task_candidate_promotion.storage.v1',
      'reviewed_task_candidate_promotion.tasks-command.publish.v1',
      'reviewed_task_candidate_promotion.tasks-created.consume.v1',
      'reviewed_task_candidate_promotion.tasks-rejected.consume.v1',
      'tasks.reviewed-candidate.blob.v1',
      'tasks.reviewed-candidate.command.v1',
      'tasks.storage.v1',
    ],
  });
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter((entry) => entry.role === 'integration')
      .map((entry) => entry.name),
    [
      'makosh-mail-api',
      'makosh-mail-core',
      'makosh-mail-imap',
      'makosh-mail-gmail',
      'makosh-mail-smtp',
      'makosh-mail-persistence',
      'makosh-mail-runtime',
      'makosh-mail-assembly',
      'makosh-mail-delivery-intent-contract',
      'makosh-telegram-delivery-intent-contract',
      'makosh-whatsapp-delivery-intent-contract',
      'makosh-zulip-delivery-intent-contract',
      'makosh-ollama-ai-api',
      'makosh-ollama-ai-assembly',
      'makosh-ollama-ai-core',
      'makosh-ollama-ai-http',
      'makosh-ollama-ai-persistence',
      'makosh-ollama-ai-runtime',
      'makosh-mail-retained-evidence-replay-persistence',
      'makosh-mail-retained-evidence-replay-contract',
      'makosh-mail-address-book-contract',
      'makosh-mail-address-book-persistence',
      'makosh-mail-google-people',
      'makosh-mail-carddav',
      'makosh-desktop-call-recording-api',
      'makosh-desktop-call-recording-core',
      'makosh-desktop-call-recording-persistence',
      'makosh-desktop-call-recording-runtime',
      'makosh-desktop-call-recording-assembly',
    ],
    'Mail admission plus provider delivery contracts must remain exact integration build units',
  );

  for (const source of [
    ...ingressSources,
    ...attachmentContractSources,
    ...apiSources,
    ...domainSources,
    ...persistenceSources,
    ...runtimeSources,
  ]) {
    for (const implementation of FORBIDDEN_INTEGRATION_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden provider implementation ${implementation}`,
      );
    }
    assert.ok(!source.content.includes('references/backend-legacy'), `${source.path} uses legacy source`);
    assert.ok(!source.content.includes('references/'), `${source.path} uses reference fallback`);
    assert.doesNotMatch(source.content, /\b(?:HashMap|BTreeMap|serde_json)\b/, `${source.path} uses a generic owner payload shape`);
  }

  const runtime = runtimeSources.map((source) => source.content).join('\n');
  assert.match(runtime, /consume_next_observation_v1/);
  assert.match(runtime, /relay_domain_outbox_once/);
});

test('Communications attachment schemas have one contract owner and no compatibility facade', async () => {
  const [ingress, api, attachment] = await Promise.all([
    readFile(new URL('src/communications-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-attachment-contract/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.doesNotMatch(ingress, /attachment_(?:blob|safety|anchor)_v1/);
  assert.doesNotMatch(api, /attachment_wire/);
  assert.match(attachment, /pub mod blob_admission_v1/);
  assert.match(attachment, /pub mod safety_verdict_v1/);
  assert.match(attachment, /pub mod anchor_recorded_v1/);
  assert.match(attachment, /pub mod lifecycle_v1/);
});

test('Communications custody transfer keeps source receipts private and uses only the Blob client port', async () => {
  const [persistenceSources, runtimeSources] = await Promise.all([
    rustSources(COMMUNICATIONS_PERSISTENCE_ROOT),
    rustSources(COMMUNICATIONS_RUNTIME_ROOT),
  ]);
  const custody = persistenceSources.find((source) => source.path.endsWith('/custody_transfer.rs'));
  assert.ok(custody, 'Communications custody persistence is required');
  assert.match(custody.content, /communications_body_custody_transfers/);
  assert.match(custody.content, /source_custody_proof/);

  const runtime = runtimeSources.map((source) => source.content).join('\n');
  assert.match(runtime, /request_managed_blob_custody_transfer/);
  assert.doesNotMatch(runtime, /makosh_blob_service|BlobContentLifecycleStore/);

  for (const source of runtimeSources.filter((source) => source.path.includes('/query'))) {
    assert.doesNotMatch(source.content, /source_blob_ref|source_custody_proof/);
  }
});

async function rustSources(directory) {
  const entries = await readdir(directory, { recursive: true, withFileTypes: true });
  return Promise.all(entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map(async (entry) => {
      const parent = entry.parentPath;
      const path = parent.startsWith(directory.pathname)
        ? join(parent, entry.name)
        : join(directory.pathname, parent, entry.name);
      return { path, content: await readFile(path, 'utf8') };
    }));
}
