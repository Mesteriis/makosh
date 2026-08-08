import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const exportScript = join(backendRoot, 'scripts/export-postgres-backup.mjs');

function temporaryDirectory(prefix) {
  return mkdtempSync(join(realpathSync(tmpdir()), prefix));
}

function runExport(args) {
  return spawnSync(process.execPath, [exportScript, ...args], {
    cwd: backendRoot,
    encoding: 'utf8',
  });
}

function writeFakePgDump(path) {
  writeFileSync(path, `#!${process.execPath}
import { existsSync, readFileSync, writeFileSync } from 'node:fs';

const outputIndex = process.argv.indexOf('--file');
const output = process.argv[outputIndex + 1];
const service = readFileSync(process.env.PGSERVICEFILE, 'utf8');
const pass = readFileSync(process.env.PGPASSFILE, 'utf8');
if (process.env.PGPASSWORD
  || !existsSync(process.env.PGSERVICEFILE)
  || !existsSync(process.env.PGPASSFILE)
  || !service.includes('dbname=makosh_test')
  || !service.includes('user=backup_role')
  || !pass.includes('backup_role')
  || !output) {
  process.exit(71);
}
writeFileSync(output, 'fixture PostgreSQL custom dump');
`, { mode: 0o700 });
  chmodSync(path, 0o700);
}

test('exports a PostgreSQL custom dump without putting credentials in arguments', () => {
  const root = temporaryDirectory('makosh-postgres-backup-');
  try {
    const pgDump = join(root, 'pg_dump');
    const connectionUrl = join(root, 'postgres-url');
    const output = join(root, 'owner-data.dump');
    writeFakePgDump(pgDump);
    writeFileSync(
      connectionUrl,
      'postgresql://backup_role:pass%3Aword@127.0.0.1:5432/makosh_test?sslmode=require\n',
      { mode: 0o600 },
    );
    chmodSync(connectionUrl, 0o600);

    const result = runExport([
      '--pg-dump', pgDump,
      '--connection-url-file', connectionUrl,
      '--output', output,
    ]);

    assert.equal(result.status, 0, result.stderr);
    assert.equal(readFileSync(output, 'utf8'), 'fixture PostgreSQL custom dump');
    assert.equal(statSync(output).mode & 0o777, 0o600);
    assert.match(result.stdout, /^postgres_backup_path=/m);
    assert.match(result.stdout, /^postgres_backup_sha256=[a-f0-9]{64}$/m);
    assert.deepEqual(
      readdirSync(root).filter((entry) => entry.startsWith('.makosh-postgres-backup-')),
      [],
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('refuses an exposed PostgreSQL connection URL before launching pg_dump', () => {
  const root = temporaryDirectory('makosh-postgres-backup-exposed-');
  try {
    const pgDump = join(root, 'pg_dump');
    const connectionUrl = join(root, 'postgres-url');
    const output = join(root, 'owner-data.dump');
    writeFakePgDump(pgDump);
    writeFileSync(connectionUrl, 'postgresql://backup_role:password@127.0.0.1/makosh_test\n', {
      mode: 0o644,
    });
    chmodSync(connectionUrl, 0o644);

    const result = runExport([
      '--pg-dump', pgDump,
      '--connection-url-file', connectionUrl,
      '--output', output,
    ]);

    assert.equal(result.status, 1);
    assert.match(result.stderr, /must not grant group or other access/);
    assert.equal(existsSync(output), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
