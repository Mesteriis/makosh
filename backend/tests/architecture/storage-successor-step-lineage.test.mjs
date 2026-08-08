import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0311-storage-successor-bundle-step-lineage.md',
    PROJECT_ROOT,
  ),
  executor: new URL(
    'src/platform/storage/postgres/src/migrations/execution.rs',
    BACKEND_ROOT,
  ),
  liveTest: new URL(
    'tests/support/storage/src/tests/postgres_live.rs',
    BACKEND_ROOT,
  ),
  authenticatedRunner: new URL(
    'scripts/test-authenticated-storage.mjs',
    BACKEND_ROOT,
  ),
};

test('Storage successor bundles inherit exact steps without replaying DDL', async () => {
  const [adr, executor, liveTest, authenticatedRunner] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );

  assert.match(adr, /Gate `storage_successor_step_lineage_v1`/);
  assert.match(adr, /predecessor acceptance без DDL replay/);
  assert.match(adr, /digest drift rejection/);
  assert.match(adr, /future bundle revision rejection/);

  assert.match(executor, /enum RecordedStepLineageV1/);
  assert.match(executor, /Exact/);
  assert.match(executor, /Predecessor/);
  assert.match(executor, /Missing/);
  assert.match(
    executor,
    /SELECT bundle_revision, step_digest[\s\S]*owner_id = \$1 AND step_revision = \$2/,
  );
  assert.doesNotMatch(
    executor,
    /WHERE owner_id = \$1 AND bundle_revision = \$2 AND step_revision = \$3/,
  );
  assert.match(
    executor,
    /RecordedStepLineageV1::Predecessor[\s\S]*record_step/,
  );

  assert.match(
    liveTest,
    /assert_successor_bundle_inherits_exact_steps/,
  );
  assert.match(
    liveTest,
    /fn successor_bundle_inherits_exact_steps_without_ddl_replay/,
  );
  assert.match(liveTest, /apply predecessor bundle/);
  assert.match(liveTest, /inherit predecessor step and apply successor step/);
  assert.match(liveTest, /reapply exact successor bundle/);
  assert.match(
    authenticatedRunner,
    /MAKOSH_STORAGE_TEST_DATABASE_URL: await postgres_test_database_url\(secrets\)/,
  );
  assert.doesNotMatch(authenticatedRunner, /console\.log\(.*databaseUrl/);
});
