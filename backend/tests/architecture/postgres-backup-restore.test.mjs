import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const script = join(backendRoot, 'scripts/restore-postgres-backup.mjs');

function temporaryDirectory(prefix) {
  return mkdtempSync(join(realpathSync(tmpdir()), prefix));
}

function run(args) {
  return spawnSync(process.execPath, [script, ...args], { cwd: backendRoot, encoding: 'utf8' });
}

function writeExecutable(path, source) {
  writeFileSync(path, `#!${process.execPath}\n${source}`, { mode: 0o700 });
  chmodSync(path, 0o700);
}

function fixtures(root, empty = true, ledger = true) {
  const psql = join(root, 'psql');
  const pgRestore = join(root, 'pg_restore');
  const connectionUrl = join(root, 'postgres-url');
  const input = join(root, 'owner-data.dump');
  writeExecutable(psql, `
if (!process.env.PGPASSFILE || process.argv.includes('--password')) process.exit(71);
process.stdout.write(process.argv.join(' ').includes('storage_migration_ledger') ? '${ledger ? 't' : 'f'}\\n' : '${empty ? 't' : 'f'}\\n');
`);
  writeExecutable(pgRestore, `
if (!process.env.PGSERVICEFILE || process.argv.includes('--clean') || process.argv.some((value) => value.includes('pass@'))) process.exit(72);
process.exit(0);
`);
  writeFileSync(connectionUrl, 'postgresql://restore_role:pass@127.0.0.1/makosh_restore?sslmode=require\n', { mode: 0o600 });
  writeFileSync(input, 'custom dump', { mode: 0o600 });
  chmodSync(connectionUrl, 0o600);
  chmodSync(input, 0o600);
  return { psql, pgRestore, connectionUrl, input };
}

test('restores only into an empty PostgreSQL target without credentials in arguments', () => {
  const root = temporaryDirectory('makosh-postgres-restore-');
  try {
    const values = fixtures(root);
    const result = run([
      '--pg-restore', values.pgRestore, '--psql', values.psql,
      '--connection-url-file', values.connectionUrl, '--input', values.input,
    ]);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^postgres_restore_size_bytes=11$/m);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('refuses a non-empty PostgreSQL target before pg_restore', () => {
  const root = temporaryDirectory('makosh-postgres-restore-nonempty-');
  try {
    const values = fixtures(root, false);
    const result = run([
      '--pg-restore', values.pgRestore, '--psql', values.psql,
      '--connection-url-file', values.connectionUrl, '--input', values.input,
    ]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /target is not empty/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects a restored database without the Storage migration ledger', () => {
  const root = temporaryDirectory('makosh-postgres-restore-ledger-');
  try {
    const values = fixtures(root, true, false);
    const result = run([
      '--pg-restore', values.pgRestore, '--psql', values.psql,
      '--connection-url-file', values.connectionUrl, '--input', values.input,
    ]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /migration ledger is unavailable/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
