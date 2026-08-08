import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('call transcription persistence is owner-local, restart-safe, and content-private', async () => {
  const [policySource, workspace, manifest, schema, repository, jobs, outbox, realtime, tickets, adr] =
    await Promise.all([
      read('backend/architecture/policy.json'),
      read('backend/Cargo.toml'),
      read('backend/src/call-transcription-persistence/Cargo.toml'),
      read('backend/src/call-transcription-persistence/migrations/0001_call_transcription.sql'),
      read('backend/src/call-transcription-persistence/src/repository.rs'),
      read('backend/src/call-transcription-persistence/src/jobs.rs'),
      read('backend/src/call-transcription-persistence/src/outbox.rs'),
      read('backend/src/call-transcription-persistence/src/realtime.rs'),
      read('backend/src/call-transcription-persistence/src/tickets.rs'),
      read('docs/adr/ADR-0390-call-recording-custody-and-speech-to-text-boundary.md'),
    ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert.deepEqual(packages.get('makosh-call-transcription-persistence'), {
    name: 'makosh-call-transcription-persistence',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'persistence',
  });
  assert.match(workspace, /"src\/call-transcription-persistence"/);
  assert.match(manifest, /owner = "call_transcription"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /makosh-call-transcription-api/);
  assert.match(manifest, /makosh-call-transcription-core/);
  assert.match(manifest, /makosh-storage-protocol/);
  assert.doesNotMatch(
    manifest,
    /makosh-communications|makosh-desktop-call-recording|makosh-speech-to-text|makosh-whisper/,
  );

  for (const table of [
    'call_transcription_runs',
    'call_transcription_inbox',
    'call_transcription_jobs',
    'call_transcription_outbox',
    'call_transcription_realtime',
    'call_transcription_read_tickets',
  ]) {
    assert.match(schema, new RegExp(`makosh_data\\.${table}`));
  }
  for (const required of [
    'request_fingerprint',
    'source_receipt_sha256',
    'stt_request_digest',
    'stt_result_receipt_sha256',
    'artifact_receipt_sha256',
    'runtime_generation',
    'grant_epoch',
    'lease_fence',
    'client_session_sha256',
  ]) {
    assert.match(schema, new RegExp(required));
  }
  for (const forbidden of [
    'audio_bytes',
    'transcript_text',
    'segment_text',
    'custody_proof',
    'provider_id',
    'provider_name',
    'model_id',
    'model_name',
    'filesystem_path',
    'communications_',
    'telegram_',
  ]) {
    assert.doesNotMatch(schema, new RegExp(forbidden));
  }

  assert.match(repository, /persist_recording_ingress/);
  assert.match(repository, /call_transcription_inbox/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /insert_outbox/);
  assert.match(repository, /append_realtime/);
  assert.match(jobs, /claim_next_job/);
  assert.match(jobs, /FOR UPDATE SKIP LOCKED/);
  assert.match(jobs, /recover_expired_jobs/);
  assert.match(jobs, /materialize_transcript/);
  assert.match(outbox, /unpublished_outbox/);
  assert.match(realtime, /realtime_after/);
  assert.match(tickets, /device_actor_sha256/);
  assert.match(tickets, /client_session_sha256/);
  assert.match(tickets, /used_at_unix_seconds IS NULL/);
  assert.match(tickets, /TicketUsed/);
  assert.match(adr, /makosh-call-transcription-persistence/);
  assert(policy.implementation.ownerInventory.businessCapabilities.includes(
    'call_transcription.storage.v1',
  ));
});
