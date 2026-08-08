#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  closeSync,
  fstatSync,
  fsyncSync,
  lstatSync,
  linkSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { basename, dirname, isAbsolute, join } from 'node:path';

const MAX_CONNECTION_URL_BYTES = 16 * 1024;
const MAX_DUMP_BYTES = 64 * 1024 * 1024 * 1024;
const SAFE_SSL_MODES = new Set(['disable', 'allow', 'prefer', 'require', 'verify-ca', 'verify-full']);

function usage() {
  process.stderr.write('usage: export-postgres-backup.mjs --pg-dump <absolute-pg_dump> --connection-url-file <private-file> --output <absolute-dump-file>\n');
}

function parseArguments(argv) {
  if (argv.length !== 6) return null;
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (!['--pg-dump', '--connection-url-file', '--output'].includes(option)
      || typeof value !== 'string' || value.length === 0 || values.has(option)) {
      return null;
    }
    values.set(option, value);
  }
  return values.size === 3 ? values : null;
}

function requirePrivateRegularFile(path, label, maximumBytes) {
  if (!isAbsolute(path)) throw new Error(`${label} must be an absolute path`);
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isFile() || metadata.size === 0 || metadata.size > maximumBytes) {
    throw new Error(`${label} must be a bounded regular non-symlink file`);
  }
  if ((metadata.mode & 0o077) !== 0) throw new Error(`${label} must not grant group or other access`);
  return metadata;
}

function requireRegularExecutable(path) {
  if (!isAbsolute(path)) throw new Error('pg_dump executable must be an absolute path');
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isFile() || metadata.size === 0 || metadata.size > 1024 * 1024 * 1024) {
    throw new Error('pg_dump executable must be a bounded regular non-symlink file');
  }
  if ((metadata.mode & 0o111) === 0 || (metadata.mode & 0o022) !== 0) {
    throw new Error('pg_dump executable must be executable and not group or other writable');
  }
}

function requirePrivateDirectory(path, label) {
  if (!isAbsolute(path)) throw new Error(`${label} must be an absolute path`);
  const metadata = lstatSync(path);
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
    throw new Error(`${label} must be a non-symlink directory`);
  }
}

function requireAbsentOutput(path) {
  if (!isAbsolute(path)) throw new Error('backup output must be an absolute path');
  requirePrivateDirectory(dirname(path), 'backup output directory');
  try {
    lstatSync(path);
  } catch (error) {
    if (error?.code === 'ENOENT') return;
    throw error;
  }
  throw new Error('backup output already exists');
}

function readConnectionUrl(path) {
  requirePrivateRegularFile(path, 'PostgreSQL connection URL file', MAX_CONNECTION_URL_BYTES);
  const url = readFileSync(path, 'utf8').trim();
  if (url.length === 0 || url.includes('\n') || url.includes('\r')) {
    throw new Error('PostgreSQL connection URL is invalid');
  }
  return url;
}

function parseConnection(urlText) {
  let url;
  try {
    url = new URL(urlText);
  } catch {
    throw new Error('PostgreSQL connection URL is invalid');
  }
  if (!['postgres:', 'postgresql:'].includes(url.protocol)
    || !url.hostname
    || !url.username
    || !url.password
    || url.hash
    || !url.pathname || url.pathname === '/') {
    throw new Error('PostgreSQL connection URL is invalid');
  }
  const database = decode(url.pathname.slice(1));
  const username = decode(url.username);
  const password = decode(url.password);
  const sslmode = url.searchParams.get('sslmode') ?? 'prefer';
  if (url.searchParams.size > (url.searchParams.has('sslmode') ? 1 : 0)
    || !validToken(database)
    || !validToken(username)
    || !validSecret(password)
    || !SAFE_SSL_MODES.has(sslmode)) {
    throw new Error('PostgreSQL connection URL is invalid');
  }
  return {
    host: url.hostname,
    port: url.port || '5432',
    database,
    username,
    password,
    sslmode,
  };
}

function decode(value) {
  try {
    return decodeURIComponent(value);
  } catch {
    throw new Error('PostgreSQL connection URL is invalid');
  }
}

function validToken(value) {
  return typeof value === 'string' && /^[A-Za-z0-9_.-]{1,127}$/.test(value);
}

function validSecret(value) {
  return typeof value === 'string' && value.length > 0 && value.length <= 1024 && !/[\r\n\0]/.test(value);
}

function pgpassEscape(value) {
  return value.replaceAll('\\', '\\\\').replaceAll(':', '\\:');
}

function writePrivateFile(path, contents) {
  writeFileSync(path, contents, { encoding: 'utf8', mode: 0o600, flag: 'wx' });
  chmodSync(path, 0o600);
}

function digestFile(path) {
  const descriptor = openSync(path, 'r');
  try {
    const metadata = fstatSync(descriptor);
    if (!metadata.isFile() || metadata.size === 0 || metadata.size > MAX_DUMP_BYTES) {
      throw new Error('PostgreSQL dump is invalid');
    }
    const bytes = readFileSync(descriptor);
    const after = fstatSync(descriptor);
    if (after.size !== metadata.size) throw new Error('PostgreSQL dump changed while it was read');
    return { size: metadata.size, sha256: createHash('sha256').update(bytes).digest('hex') };
  } finally {
    closeSync(descriptor);
  }
}

function syncFile(path) {
  const descriptor = openSync(path, 'r');
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}

function syncDirectory(path) {
  const descriptor = openSync(path, 'r');
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}

export function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    usage();
    process.exitCode = 2;
    return;
  }
  const pgDump = options.get('--pg-dump');
  const connectionUrlFile = options.get('--connection-url-file');
  const output = options.get('--output');
  let staging = null;
  try {
    requireRegularExecutable(pgDump);
    requireAbsentOutput(output);
    const connection = parseConnection(readConnectionUrl(connectionUrlFile));
    const parent = dirname(output);
    staging = mkdtempSync(join(parent, '.makosh-postgres-backup-'));
    chmodSync(staging, 0o700);
    const serviceFile = join(staging, 'pg_service.conf');
    const passFile = join(staging, 'pgpass');
    const stagedDump = join(staging, `${basename(output)}.dump`);
    writePrivateFile(serviceFile, `[makosh_backup]\nhost=${connection.host}\nport=${connection.port}\ndbname=${connection.database}\nuser=${connection.username}\nsslmode=${connection.sslmode}\n`);
    writePrivateFile(passFile, `${pgpassEscape(connection.host)}:${connection.port}:${pgpassEscape(connection.database)}:${pgpassEscape(connection.username)}:${pgpassEscape(connection.password)}\n`);
    const result = spawnSync(pgDump, [
      '--format=custom', '--no-owner', '--no-privileges', '--file', stagedDump,
      '--dbname=service=makosh_backup',
    ], {
      env: { HOME: staging, LC_ALL: 'C', PGPASSFILE: passFile, PGSERVICEFILE: serviceFile },
      encoding: 'utf8',
      stdio: ['ignore', 'ignore', 'pipe'],
    });
    if (result.error || result.status !== 0) {
      throw new Error('pg_dump failed without publishing a backup');
    }
    chmodSync(stagedDump, 0o600);
    const dump = digestFile(stagedDump);
    syncFile(stagedDump);
    linkSync(stagedDump, output);
    syncDirectory(parent);
    process.stdout.write(`postgres_backup_path=${output}\npostgres_backup_size_bytes=${dump.size}\npostgres_backup_sha256=${dump.sha256}\n`);
  } catch (error) {
    process.stderr.write(`postgres-backup: ${error.message}\n`);
    process.exitCode = 1;
  } finally {
    if (staging) rmSync(staging, { recursive: true, force: true });
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
