import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('provider-neutral Blob delegation reuses the exact Kernel request-provider resolver', async () => {
  const [protocol, blobSession, requestRouter, blobClient, adr] = await Promise.all([
    read('backend/src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto'),
    read('backend/src/kernel/src/platform/blob/session.rs'),
    read('backend/src/kernel/src/modules/capability/module_request.rs'),
    read('backend/src/platform/blob/client/src/lib.rs'),
    read('docs/adr/ADR-0390-call-recording-custody-and-speech-to-text-boundary.md'),
  ]);

  assert.match(protocol, /ContractReferenceV1 target_request_contract = 10/);
  assert.match(protocol, /resolved_target_owner_id = 3/);
  assert.match(blobSession, /resolve_provider_for_caller/);
  assert.match(requestRouter, /pub\(crate\) fn resolve_provider_for_caller/);
  assert.match(blobClient, /ManagedBlobResolvedProviderCustodyDelegationRequestV1/);
  const resolvedRequest = blobClient.slice(
    blobClient.indexOf('pub struct ManagedBlobResolvedProviderCustodyDelegationRequestV1'),
    blobClient.indexOf('pub struct ManagedBlobCustodyDelegationV1'),
  );
  for (const forbidden of ['target_owner_id', 'target_module_id', 'target_capability_id']) {
    assert.ok(!resolvedRequest.includes(forbidden), `caller-selected target ${forbidden}`);
  }
  assert.match(adr, /Caller не передаёт эти\s+координаты в provider-neutral режиме/);
});

test('module request response Blob target is derived from the authenticated caller grant', async () => {
  const [protocol, requestRouter, adr] = await Promise.all([
    read('backend/src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto'),
    read('backend/src/kernel/src/modules/capability/module_request.rs'),
    read('docs/adr/ADR-0390-call-recording-custody-and-speech-to-text-boundary.md'),
  ]);

  const callerRequest = protocol.slice(
    protocol.indexOf('message ManagedRuntimeModuleRequestRequestV1'),
    protocol.indexOf('message ManagedRuntimeModuleRequestDeliveryV1'),
  );
  assert.match(callerRequest, /string response_blob_capability_id = 5/);
  for (const forbidden of ['response_blob_target_owner_id', 'response_blob_target_module_id']) {
    assert.ok(!callerRequest.includes(forbidden), `caller-selected response target ${forbidden}`);
  }

  assert.match(requestRouter, /fn resolve_response_blob_target/);
  assert.match(requestRouter, /entry\.registration_id\(\) == expectation\.registration_id\(\)/);
  assert.match(requestRouter, /entry\.module_id\(\) == expectation\.module_id\(\)/);
  assert.match(requestRouter, /entry\.grant_epoch\(\) == expectation\.grant_epoch\(\)/);
  assert.match(adr, /Kernel сверяет её с\s+registration, module и grant epoch вызывающего runtime/);
});
