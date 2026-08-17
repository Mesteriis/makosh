import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Blob authority separates ephemeral operations from durable custody', async () => {
  const [recoveryProto, blobProto, kernelSession, vaultLease, contentFormat, metadataCodec] =
    await Promise.all([
      readFile(
        new URL(
          'src/platform/runtime_protocol/proto/makosh/runtime/v1/recovery.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL(
          'src/platform/runtime_protocol/proto/makosh/runtime/v1/blob_runtime.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/kernel/src/platform/blob/session.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/platform/blob/runtime/src/vault/lease.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/platform/blob/runtime/src/storage/format.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/platform/blob/runtime/src/metadata/codec.rs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  assert.match(recoveryProto, /enum BlobQuotaOperationV1/);
  assert.match(recoveryProto, /string custody_scope_id = 2;/);
  assert.match(recoveryProto, /repeated BlobQuotaOperationV1 allowed_operations = 3;/);
  assert.match(blobProto, /string custody_scope_id = 22;/);
  assert.match(blobProto, /string custody_scope_id = 21;/);
  assert.match(blobProto, /string target_custody_scope_id = 20;/);

  assert.match(kernelSession, /entry\.request\(\)\.allows\(value\)/);
  assert.match(kernelSession, /BLOB_CONTENT_KEY_SCHEMA_REVISION/);
  assert.doesNotMatch(kernelSession, /key_revision:\s*expectation\.grant_epoch/);

  assert.match(vaultLease, /opaque_reference_scope\(reference, fence\.custody\(\)\)/);
  const scopeFunction = vaultLease.match(
    /fn opaque_reference_scope[\s\S]*?(?=\nfn valid_record_id)/,
  )?.[0];
  assert.ok(scopeFunction, 'Vault Blob authority scope function is required');
  assert.match(scopeFunction, /custody\.custody_scope_id\(\)/);
  assert.match(scopeFunction, /reference\.reference_id\(\)/);
  assert.doesNotMatch(
    scopeFunction,
    /registration_id|runtime_instance_id|runtime_generation|grant_epoch|capability_id/,
  );
  assert.match(contentFormat, /const LEGACY_MAGIC: &\[u8; 8\] = b"HBLBENC2"/);
  assert.match(contentFormat, /const CHUNKED_MAGIC: &\[u8; 8\] = b"HBLBENC3"/);
  assert.match(contentFormat, /associated_data\(reference, custody, key_revision\)/);
  assert.match(contentFormat, /chunk_associated_data[\s\S]*offset[\s\S]*plaintext_len/);
  assert.doesNotMatch(contentFormat, /BlobAccessFenceV1|registration_id|runtime_instance_id|grant_epoch/);
  assert.match(metadataCodec, /const MAGIC: &\[u8; 8\] = b"HBLBM002"/);
  assert.match(metadataCodec, /custody\.custody_scope_id\(\)/);
  assert.doesNotMatch(metadataCodec, /BlobAccessFenceV1|registration_id|runtime_instance_id|grant_epoch/);
});

test('Blob conformance proves rotation survival and legacy fail-closed behavior', async () => {
  const [storageTests, metadataTests, liveDataPath] = await Promise.all([
    readFile(
      new URL(
        'tests/support/blob/src/tests/encrypted_storage.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/blob/src/tests/metadata_lifecycle.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/blob_service/data_path.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(
    storageTests,
    /encrypted_content_survives_runtime_registration_and_grant_rotation/,
  );
  assert.match(storageTests, /store_open_rejects_legacy_ciphertext_before_serving_requests/);
  assert.match(
    metadataTests,
    /aggregate_quota_survives_registration_and_grant_rotation_for_one_custody_scope/,
  );
  assert.match(metadataTests, /lifecycle_open_rejects_legacy_metadata_before_serving_requests/);
  assert.match(liveDataPath, /"blob-content\.write"/);
  assert.match(liveDataPath, /"blob-content\.read"/);
  assert.match(liveDataPath, /"blob-runtime-v1"/);
  assert.match(liveDataPath, /"blob-runtime-v2"/);
  assert.match(liveDataPath, /"owner-1\.content\.v1"/);
});
