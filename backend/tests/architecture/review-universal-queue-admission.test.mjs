import assert from 'node:assert/strict';
import { globSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const absolute = (path) => resolve(PROJECT_ROOT, path);
const read = (path) => readFile(absolute(path), 'utf8');

const exactReviewRoutes = [
  '/makosh.review.attention.client.v1.ReviewAttentionCommandService/Execute',
  '/makosh.review.attention.client.v1.ReviewAttentionQueryService/Query',
  '/makosh.review.note_candidate.v1.ReviewNoteCandidateCommandService/Decide',
  '/makosh.review.note_candidate.v1.ReviewNoteCandidateQueryService/Get',
  '/makosh.review.note_candidate.v1.ReviewNoteCandidateQueryService/List',
  '/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateCommandService/Decide',
  '/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateQueryService/Get',
  '/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateQueryService/List',
  '/makosh.review.task_candidate.v1.ReviewTaskCandidateCommandService/Decide',
  '/makosh.review.task_candidate.v1.ReviewTaskCandidateQueryService/Get',
  '/makosh.review.task_candidate.v1.ReviewTaskCandidateQueryService/List',
].sort();

test('Task 13 remains staged behind Task 10 while preserving admitted Review', async () => {
  const policy = JSON.parse(await read('backend/architecture/policy.json'));
  const implementation = policy.implementation;

  assert.equal(implementation.currentSlice, 'speech_to_text_whisper_admission_v1');
  assert.equal(implementation.productionPackages.length, 283);
  assert.equal(implementation.ownerInventory.businessCapabilities.length, 253);
  assert.equal(implementation.ownerInventory.domains.filter((owner) => owner === 'review').length, 1);
  assert.equal(implementation.productionPackages.filter(({ owner }) => owner === 'review').length, 23);
});

test('Task 13 exposes the exact typed Review routes without a generic target bag', async () => {
  const [attention, note, person, task] = await Promise.all([
    read('backend/src/review-attention-api/src/lib.rs'),
    read('backend/src/review-note-candidate-api/src/lib.rs'),
    read('backend/src/review-person-match-candidate-api/src/lib.rs'),
    read('backend/src/review-task-candidate-api/src/lib.rs'),
  ]);
  const routes = [...`${attention}\n${note}\n${person}\n${task}`.matchAll(/"(\/makosh\.review\.[^"]+Service\/[^"]+)"/g)]
    .map((match) => match[1])
    .sort();

  assert.deepEqual(routes, exactReviewRoutes);
  for (const candidate of [note, task]) {
    assert.match(candidate, /LIST_CONNECT_PATH_V1/);
    assert.doesNotMatch(candidate, /target_domain|target_entity_kind|metadata_json|arbitrary/);
  }
});

test('Task 13 appends FORCE RLS to all older Review stores', async () => {
  const [attention, taskCandidate, noteCandidate] = await Promise.all([
    read('backend/src/review-attention-persistence/src/schema.rs'),
    read('backend/src/review-task-candidate-persistence/src/schema.rs'),
    read('backend/src/review-note-candidate-persistence/src/schema.rs'),
  ]);

  assert.match(attention, /REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V3: u32 = 3/);
  assert.match(attention, /0003_review_attention_owner_rls\.sql/);
  assert.match(taskCandidate, /REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V2: u32 = 2/);
  assert.match(taskCandidate, /0002_review_task_candidate_owner_rls\.sql/);
  assert.match(noteCandidate, /REVIEW_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V2: u32 = 2/);
  assert.match(noteCandidate, /0002_review_note_candidate_owner_rls\.sql/);

  for (const migration of [
    await read('backend/src/review-attention-persistence/migrations/0003_review_attention_owner_rls.sql'),
    await read('backend/src/review-task-candidate-persistence/migrations/0002_review_task_candidate_owner_rls.sql'),
    await read('backend/src/review-note-candidate-persistence/migrations/0002_review_note_candidate_owner_rls.sql'),
  ]) {
    assert.match(migration, /ENABLE ROW LEVEL SECURITY/);
    assert.match(migration, /FORCE ROW LEVEL SECURITY/);
    assert.match(migration, /CREATE POLICY/);
    assert.match(migration, /current_setting\('makosh\.logical_owner_id'/);
  }
});

test('Task 13 compiles Review and removes handwritten REST scaffolds', async () => {
  const [surfaces, adapters, generator] = await Promise.all([
    read('frontend/src/platform/client-runtime/clientSurfaces.ts'),
    read('frontend/src/app/client-surfaces/compiledClientSurfaceAdapters.ts'),
    read('frontend/scripts/generate-proto.mjs'),
  ]);
  const legacyReviewFiles = globSync('src/domains/review/api/**/*', { cwd: absolute('frontend') });

  assert.match(surfaces, /routeId: 'review'[\s\S]*adapterId: 'review-owner'/);
  assert.match(adapters, /'review-owner'/);
  for (const packageName of [
    'review-attention-api',
    'review-note-candidate-api',
    'review-person-match-candidate-api',
    'review-task-candidate-api',
  ]) assert.equal(generator.includes(packageName), true, packageName);
  assert.deepEqual(legacyReviewFiles, []);
});

test('Task 13 requires actual pagination, RLS, decision and restart evidence', async () => {
  const [runner, attention, note, task] = await Promise.all([
    read('backend/scripts/test-authenticated-storage.mjs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/review_attention_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/note_candidate_managed_flow.rs'),
    read('backend/tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_flow.rs'),
  ]);
  for (const name of [
    'managed_review_attention_reaches_gateway_sse_and_replays_after_restart',
    'managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart',
    'managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart',
  ]) assert.equal(runner.includes(name), true, name);

  assert.match(`${attention}\n${note}\n${task}`, /NOBYPASSRLS/);
  assert.match(`${attention}\n${note}\n${task}`, /next.*cursor|cursor.*next/i);
});
