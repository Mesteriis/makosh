import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const root = path.resolve(import.meta.dirname, '../..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');

test('authenticated browser device and session context reach the owner-neutral module envelope', () => {
  const protocol = read(
    'src/platform/runtime_protocol/proto/makosh/runtime/v1/module_client.proto',
  );
  const validation = read(
    'src/platform/runtime_protocol/src/validation/module_client.rs',
  );
  const gateway = read('src/api/gateway/runtime/src/browser/client_rpc.rs');
  const kernel = read('src/kernel/src/platform/gateway.rs');

  assert.match(protocol, /string authenticated_device_id = 8/);
  assert.match(protocol, /string authenticated_client_session_id = 9/);
  assert.match(
    validation,
    /logical_owner_id\.is_empty\(\) != request\.authenticated_client_session_id\.is_empty\(\)/,
  );
  assert.match(gateway, /let device_id = session\.device_id\(\)\.to_owned\(\)/);
  assert.match(gateway, /let session_id = session\.session_id\(\)\.to_owned\(\)/);
  assert.match(
    gateway,
    /handler\(&route, &owner_id, &device_id, &session_id, &body\)/,
  );
  assert.match(
    kernel,
    /authenticated_device_id: authenticated_device_id\.to_owned\(\)/,
  );
  assert.match(
    kernel,
    /authenticated_client_session_id: authenticated_client_session_id\.to_owned\(\)/,
  );
});

test('client payload cannot claim the authenticated device actor', () => {
  const review = read(
    'src/review-task-candidate-api/proto/makosh/review/task_candidate/v1/task_candidate.proto',
  );
  const decision = review
    .split('message DecideReviewTaskCandidateRequestV1')[1]
    .split('message DecideReviewTaskCandidateResponseV1')[0];

  assert.doesNotMatch(decision, /device|actor|owner_id/);
});
