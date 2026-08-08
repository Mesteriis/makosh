import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('Blob custody release has typed Blob-owned authority without a data-plane delete', async () => {
  const [
    adr,
    blobProto,
    managedProto,
    descriptorProto,
    validation,
    service,
    session,
    release,
    kernelRelease,
    blobClient,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0343-capability-routed-blob-custody-release.md',
        PROJECT_ROOT,
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
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/managed_runtime_control.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/makosh/runtime/v1/recovery.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/platform/runtime_protocol/src/validation/blob.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/data/session.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/runtime/src/release.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/kernel/src/platform/blob/release.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/client/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(adr, /Состояние реализации: реализовано/);
  assert.match(adr, /durable retry после reconnect/);
  assert.match(adr, /Kernel-staged platform configuration/);
  assert.match(blobProto, /message BlobCustodyReleaseGrantV1/);
  assert.match(blobProto, /message BlobCustodyReleaseRequestV1/);
  assert.match(blobProto, /message BlobCustodyReleaseResponseV1/);
  assert.match(blobProto, /custody_release_grace_period_ms/);
  assert.match(blobProto, /BlobBackupClassV1 backup_class = 20/);
  assert.match(managedProto, /ManagedRuntimeBlobCustodyReleaseRequestV1/);
  assert.match(managedProto, /release_blob_custody = 14/);
  assert.match(descriptorProto, /BLOB_QUOTA_OPERATION_V1_RELEASE_CUSTODY = 4/);
  assert.match(validation, /fn valid_release_grant/);
  assert.match(validation, /custody_source_proof_sha256/);
  assert.match(session, /fn validate_signed_release/);
  assert.match(session, /makosh\.blob-custody-release\.v1/);
  assert.match(release, /struct BlobCustodyReleaseLedgerV1/);
  assert.match(release, /reserve_deletion_exact/);
  assert.match(kernelRelease, /impl ManagedRuntimeBlobCustodyReleaseHandler/);
  assert.match(kernelRelease, /current_managed_runtime_matches/);
  assert.match(kernelRelease, /ModuleBlobOperationV1::ReleaseCustody/);
  assert.match(blobClient, /request_managed_blob_custody_release_v2/);
  assert.match(blobClient, /request_next_with_dispatch/);
  const dataOperation = blobProto.match(
    /enum BlobDataOperationV1 \{[\s\S]*?\n\}/,
  )?.[0];
  assert.ok(dataOperation);
  assert.doesNotMatch(dataOperation, /RELEASE|DELETE/);
  assert.match(
    service,
    /Some\(Operation::ReleaseCustody\(request\)\)/,
  );
  assert.doesNotMatch(managedProto, /filesystem|data_socket_path.*release_blob_custody/);
});
