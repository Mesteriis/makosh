import { access, lstat, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

const toolchain = process.argv[2] ?? '1.97.0';
const backendDirectory = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const composeFile = join(
  backendDirectory,
  'tests/support/events/nats/jwt/compose.yaml',
);
const fixtureDirectory = await mkdtemp(join(tmpdir(), 'makosh-nats-jwt-'));
const composeEnvironment = {
  ...process.env,
  MAKOSH_NATS_JWT_FIXTURE_DIR: fixtureDirectory,
};

try {
  await run('docker', ['compose', '-f', composeFile, 'up', '--detach', '--wait'], composeEnvironment);
  const credentials = await fixturePaths(fixtureDirectory);
  await buildManagedRuntimes(composeEnvironment);
  await run(
    'cargo',
    [
      `+${toolchain}`,
      'test',
      '--locked',
      '-p',
      'makosh-events-jetstream-testkit',
      'tests::jwt_live::jwt_runtime_credential_is_verified_and_broker_fences_subjects',
      '--',
      '--ignored',
      '--exact',
      '--test-threads=1',
    ],
    {
      ...composeEnvironment,
      MAKOSH_NATS_JWT_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE: credentials.accountPublicKey,
      MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE: credentials.accountSigningSeed,
      MAKOSH_NATS_JWT_EVENT_HUB_CREDS_FILE: credentials.eventHubCredentials,
    },
  );
  await runResolverPublisherConformance(credentials, composeEnvironment);
  await runAuthorityResolverConformance(credentials, composeEnvironment);
  await runManagedAuthorityCredentialConformance(credentials, composeEnvironment);
  await verifyRevocationDisconnect(credentials, composeEnvironment);
} catch (error) {
  await run(
    'docker',
    ['compose', '-f', composeFile, 'logs', 'fixture', 'nats'],
    composeEnvironment,
    true,
  );
  throw error;
} finally {
  await run('docker', ['compose', '-f', composeFile, 'down', '--volumes'], composeEnvironment, true);
  await rm(fixtureDirectory, { force: true, recursive: true });
}

async function fixturePaths(directory) {
  const paths = {
    accountPublicKey: join(directory, 'account-public-key'),
    accountSigningSeed: join(directory, 'account-signing.seed'),
    eventHubCredentials: join(directory, 'event-hub.creds'),
    eventsAccountJwt: join(directory, 'events.jwt'),
    resolverUpdateCredentials: join(directory, 'resolver-update.creds'),
  };
  await Promise.all([
    validatePublicFile(paths.accountPublicKey),
    validatePrivateFile(paths.accountSigningSeed),
    validatePrivateFile(paths.eventHubCredentials),
    validateAccountJwtFile(paths.eventsAccountJwt),
    validatePrivateFile(paths.resolverUpdateCredentials),
  ]);
  return paths;
}

async function runResolverPublisherConformance(credentials, environment) {
  await run(
    'cargo',
    [
      `+${toolchain}`,
      'test',
      '--locked',
      '-p',
      'makosh-events-jetstream-testkit',
      'tests::resolver_update_live::system_account_publishes_a_bound_account_jwt',
      '--',
      '--ignored',
      '--exact',
      '--test-threads=1',
    ],
    {
      ...environment,
      MAKOSH_NATS_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE: credentials.accountPublicKey,
      MAKOSH_NATS_JWT_ACCOUNT_JWT_FILE: credentials.eventsAccountJwt,
      MAKOSH_NATS_JWT_RESOLVER_UPDATE_CREDS_FILE: credentials.resolverUpdateCredentials,
    },
  );
}

async function runAuthorityResolverConformance(credentials, environment) {
  await run(
    'cargo',
    [
      `+${toolchain}`,
      'test',
      '--locked',
      '-p',
      'makosh-events-jetstream-testkit',
      'tests::authority_runtime::resolver_update_live::authority_runtime_updates_the_resolver_only_with_vault_system_credentials',
      '--',
      '--ignored',
      '--exact',
      '--test-threads=1',
    ],
    {
      ...environment,
      MAKOSH_NATS_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE: credentials.accountPublicKey,
      MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE: credentials.accountSigningSeed,
      MAKOSH_NATS_JWT_ACCOUNT_JWT_FILE: credentials.eventsAccountJwt,
      MAKOSH_NATS_JWT_RESOLVER_UPDATE_CREDS_FILE: credentials.resolverUpdateCredentials,
    },
  );
}

async function buildManagedRuntimes(environment) {
  await run(
    'cargo',
    [
      `+${toolchain}`,
      'build',
      '--locked',
      '-p',
      'makosh-vault-runtime',
      '-p',
      'makosh-events-authority-runtime',
    ],
    environment,
  );
}

async function runManagedAuthorityCredentialConformance(credentials, environment) {
  await run(
    'cargo',
    [
      `+${toolchain}`,
      'test',
      '--locked',
      '-p',
      'makosh-kernel-recovery-testkit',
      'tests::events_authority_vault::managed_jwt::kernel_managed_authority_delivers_a_broker_accepted_jwt',
      '--',
      '--ignored',
      '--exact',
      '--test-threads=1',
    ],
    {
      ...environment,
      MAKOSH_EVENTS_MANAGED_JWT_TEST: '1',
      MAKOSH_NATS_JWT_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE: credentials.accountPublicKey,
      MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE: credentials.accountSigningSeed,
      MAKOSH_NATS_JWT_EVENT_HUB_CREDS_FILE: credentials.eventHubCredentials,
      MAKOSH_VAULT_RUNTIME_BIN: join(backendDirectory, 'target/debug/makosh-vault-runtime'),
      MAKOSH_EVENTS_AUTHORITY_RUNTIME_BIN: join(
        backendDirectory,
        'target/debug/makosh-events-authority-runtime',
      ),
    },
  );
}

async function verifyRevocationDisconnect(credentials, environment) {
  const ready = join(fixtureDirectory, 'runtime-revocation-ready');
  const proceed = join(fixtureDirectory, 'runtime-revocation-proceed');
  const test = run(
    'cargo',
    [
      `+${toolchain}`,
      'test',
      '--locked',
      '-p',
      'makosh-events-jetstream-testkit',
      'tests::jwt_revocation_live::resolver_claim_update_disconnects_an_active_runtime',
      '--',
      '--ignored',
      '--exact',
      '--test-threads=1',
    ],
    {
      ...environment,
      MAKOSH_NATS_JWT_TEST_ENDPOINT: 'nats://127.0.0.1:43223',
      MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE: credentials.accountPublicKey,
      MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE: credentials.accountSigningSeed,
      MAKOSH_NATS_JWT_REVOCATION_READY_FILE: ready,
      MAKOSH_NATS_JWT_REVOCATION_PROCEED_FILE: proceed,
    },
  );
  await waitForFile(ready);
  const runtimePublicKey = (await readFile(ready, 'utf8')).trim();
  if (!/^U[A-Z0-9]{55}$/.test(runtimePublicKey)) {
    throw new Error('runtime revocation marker has an invalid public NKey');
  }
  await addRuntimeRevocation(environment, runtimePublicKey);
  await refreshEventsAccountJwt(environment);
  await runAuthorityResolverConformance(credentials, environment);
  await writeFile(proceed, 'revoke published\n', { mode: 0o600 });
  await test;
}

async function addRuntimeRevocation(environment, runtimePublicKey) {
  await run(
    'docker',
    [
      'compose', '-f', composeFile, 'exec', '-T', 'authority', 'nsc', '--all-dirs', '/fixture/nsc',
      'keys', 'migrate',
    ],
    environment,
  );
  await run(
    'docker',
    [
      'compose', '-f', composeFile, 'exec', '-T', 'authority', 'nsc', '--all-dirs', '/fixture/nsc',
      'revocations', 'add-user', '--account', 'events', '--user-public-key', runtimePublicKey,
      '--at', '0',
    ],
    environment,
  );
}

async function refreshEventsAccountJwt(environment) {
  await run(
    'docker',
    [
      'compose', '-f', composeFile, 'exec', '-T', 'authority', 'sh', '-ec',
      'nsc --all-dirs /fixture/nsc describe account --name events --raw > /fixture/events.jwt',
    ],
    environment,
  );
}

async function waitForFile(path) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      await access(path);
      return;
    } catch {
      await new Promise((resolveWait) => setTimeout(resolveWait, 50));
    }
  }
  throw new Error(`timed out waiting for runtime marker ${path}`);
}

async function validatePublicFile(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error(`JWT fixture ${path} must be a regular file`);
  }
  const value = (await readFile(path, 'utf8')).trim();
  if (!/^A[A-Z0-9]{55}$/.test(value)) {
    throw new Error(`JWT fixture ${path} has an invalid account public key`);
  }
}

async function validatePrivateFile(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o077) !== 0) {
    throw new Error(`JWT fixture ${path} must be a private regular file`);
  }
}

async function validateAccountJwtFile(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size === 0 || metadata.size > 16384) {
    throw new Error(`JWT fixture ${path} must be a bounded regular Account JWT`);
  }
  const value = (await readFile(path, 'utf8')).trim();
  if (value.split('.').length !== 3) {
    throw new Error(`JWT fixture ${path} must have three JWT segments`);
  }
}

function run(command, argumentsList, environment, allowFailure = false) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, argumentsList, {
      cwd: backendDirectory,
      env: environment,
      stdio: 'inherit',
    });
    child.once('error', rejectRun);
    child.once('exit', (code, signal) => {
      if (code === 0 || allowFailure) {
        resolveRun();
        return;
      }
      rejectRun(new Error(`${command} exited with ${signal ?? `code ${code}`}`));
    });
  });
}
