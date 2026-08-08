#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import {
  chmodSync, lstatSync, mkdtempSync, readFileSync, rmSync, writeFileSync,
} from 'node:fs';
import { dirname, isAbsolute, join } from 'node:path';

const MAX_BYTES = 64 * 1024 * 1024 * 1024;
const SAFE_SSL_MODES = new Set(['disable', 'allow', 'prefer', 'require', 'verify-ca', 'verify-full']);

function parse(argv) {
  if (argv.length !== 8) return null;
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const [option, value] = [argv[index], argv[index + 1]];
    if (!['--pg-restore', '--psql', '--connection-url-file', '--input'].includes(option) || !value || values.has(option)) return null;
    values.set(option, value);
  }
  return values.size === 4 ? values : null;
}

function regular(path, label, maximumBytes = MAX_BYTES) {
  if (!isAbsolute(path)) throw new Error(`${label} must be an absolute path`);
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isFile() || metadata.size === 0 || metadata.size > maximumBytes) {
    throw new Error(`${label} must be a bounded regular non-symlink file`);
  }
  return metadata;
}

function executable(path, label) {
  const metadata = regular(path, label, 1024 * 1024 * 1024);
  if ((metadata.mode & 0o111) === 0 || (metadata.mode & 0o022) !== 0) throw new Error(`${label} must be executable and not group or other writable`);
}

function connection(path) {
  const metadata = regular(path, 'PostgreSQL connection URL file', 16 * 1024);
  if ((metadata.mode & 0o077) !== 0) throw new Error('PostgreSQL connection URL file must not grant group or other access');
  const text = readFileSync(path, 'utf8').trim();
  let url;
  try { url = new URL(text); } catch { throw new Error('PostgreSQL connection URL is invalid'); }
  const database = decode(url.pathname.slice(1));
  const username = decode(url.username);
  const password = decode(url.password);
  const sslmode = url.searchParams.get('sslmode') ?? 'prefer';
  if (!['postgres:', 'postgresql:'].includes(url.protocol) || !url.hostname || !url.password || !token(database) || !token(username) || !secret(password) || !SAFE_SSL_MODES.has(sslmode)) {
    throw new Error('PostgreSQL connection URL is invalid');
  }
  return { host: url.hostname, port: url.port || '5432', database, username, password, sslmode };
}

function decode(value) { try { return decodeURIComponent(value); } catch { throw new Error('PostgreSQL connection URL is invalid'); } }
function token(value) { return /^[A-Za-z0-9_.-]{1,127}$/.test(value); }
function secret(value) { return value.length > 0 && value.length <= 1024 && !/[\r\n\0]/.test(value); }
function escape(value) { return value.replaceAll('\\', '\\\\').replaceAll(':', '\\:'); }

function privateFile(path, value) { writeFileSync(path, value, { encoding: 'utf8', mode: 0o600, flag: 'wx' }); chmodSync(path, 0o600); }

function command(binary, args, environment, failure) {
  const result = spawnSync(binary, args, { env: environment, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  if (result.error || result.status !== 0) throw new Error(failure);
  return (result.stdout ?? '').trim();
}

export function main(argv = process.argv.slice(2)) {
  const options = parse(argv);
  if (!options) {
    process.stderr.write('usage: restore-postgres-backup.mjs --pg-restore <absolute-pg_restore> --psql <absolute-psql> --connection-url-file <private-file> --input <private-dump>\n');
    process.exitCode = 2;
    return;
  }
  let staging = null;
  try {
    const pgRestore = options.get('--pg-restore');
    const psql = options.get('--psql');
    const input = options.get('--input');
    executable(pgRestore, 'pg_restore executable');
    executable(psql, 'psql executable');
    const dump = regular(input, 'PostgreSQL restore input');
    if ((dump.mode & 0o077) !== 0) throw new Error('PostgreSQL restore input must not grant group or other access');
    const target = connection(options.get('--connection-url-file'));
    staging = mkdtempSync(join(dirname(input), '.makosh-postgres-restore-'));
    chmodSync(staging, 0o700);
    const service = join(staging, 'pg_service.conf');
    const pass = join(staging, 'pgpass');
    privateFile(service, `[makosh_restore]\nhost=${target.host}\nport=${target.port}\ndbname=${target.database}\nuser=${target.username}\nsslmode=${target.sslmode}\n`);
    privateFile(pass, `${escape(target.host)}:${target.port}:${escape(target.database)}:${escape(target.username)}:${escape(target.password)}\n`);
    const environment = { HOME: staging, LC_ALL: 'C', PGPASSFILE: pass, PGSERVICEFILE: service };
    const empty = command(psql, ['--tuples-only', '--no-align', '--dbname=service=makosh_restore', '--command=SELECT count(*) = 0 FROM pg_catalog.pg_tables WHERE schemaname NOT IN (\'pg_catalog\', \'information_schema\')'], environment, 'cannot verify PostgreSQL restore target');
    if (empty !== 't') throw new Error('PostgreSQL restore target is not empty');
    command(pgRestore, ['--no-owner', '--no-privileges', '--exit-on-error', '--single-transaction', '--dbname=service=makosh_restore', input], environment, 'pg_restore failed without a partial-recovery success claim');
    const ledger = command(psql, ['--tuples-only', '--no-align', '--dbname=service=makosh_restore', '--command=SELECT count(*) = 1 FROM information_schema.tables WHERE table_schema = \'makosh_platform\' AND table_name = \'storage_migration_ledger\''], environment, 'cannot validate restored PostgreSQL migration ledger');
    if (ledger !== 't') throw new Error('restored PostgreSQL migration ledger is unavailable');
    process.stdout.write(`postgres_restore_input=${input}\npostgres_restore_size_bytes=${dump.size}\n`);
  } catch (error) {
    process.stderr.write(`postgres-restore: ${error.message}\n`);
    process.exitCode = 1;
  } finally {
    if (staging) rmSync(staging, { recursive: true, force: true });
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
