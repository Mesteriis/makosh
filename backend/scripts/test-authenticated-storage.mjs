import { execFile, spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { access, chmod, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const toolchain = process.argv[2] || '1.97.0';
const project = `makosh-storage-authenticated-${process.pid}`;
const compose = ['compose', '--project-name', project, '-f', 'development/authenticated/compose.yaml'];
const focusedTest = process.env.MAKOSH_STORAGE_AUTHENTICATED_TEST_FILTER?.trim();
const managedTest = process.env.MAKOSH_STORAGE_MANAGED_TEST_FILTER?.trim();
const schedulerPostgresTest = process.env.MAKOSH_SCHEDULER_POSTGRES_TEST_FILTER?.trim();
const delayedDeliveryPostgresTest =
  process.env.MAKOSH_COMMUNICATION_DELAYED_DELIVERY_POSTGRES_TEST_FILTER?.trim();
const deliveryIntentPostgresTest =
  process.env.MAKOSH_COMMUNICATION_DELIVERY_INTENT_POSTGRES_TEST_FILTER?.trim();
const crossChannelForwardPostgresTest =
  process.env.MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES_TEST_FILTER?.trim();
const telegramCallsTest = process.env.MAKOSH_TELEGRAM_CALLS_POSTGRES_TEST_FILTER?.trim();
const personsPostgresTest = process.env.MAKOSH_PERSONS_POSTGRES_TEST_FILTER?.trim();
const mailPersonsSyncPostgresTest =
  process.env.MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_TEST_FILTER?.trim();
const reviewPersonMatchCandidatePostgresTest =
  process.env.MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_POSTGRES_TEST_FILTER?.trim();
const identityResolutionPostgresTest =
  process.env.MAKOSH_IDENTITY_RESOLUTION_POSTGRES_TEST_FILTER?.trim();
const projectionPostgresTest =
  process.env.MAKOSH_PROJECTION_POSTGRES_TEST_FILTER?.trim();
const reviewedPersonMatchPromotionPostgresTest =
  process.env.MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_POSTGRES_TEST_FILTER?.trim();
const keepContour = process.env.MAKOSH_STORAGE_KEEP_CONTOUR === '1';
const authenticatedTests = [
  'authenticated_revoke_fences_the_real_pool_and_postgres_role',
  'authenticated_runtime_revokes_the_exact_staged_binding_through_vault',
  'authenticated_admin_console_requires_a_file_backed_credential',
  'authenticated_pgbouncer_reloads_the_storage_owned_database_include',
  'authenticated_runtime_readiness_accepts_the_resolved_platform_credential',
  'authenticated_runtime_applies_the_kernel_staged_binding_to_pgbouncer',
  'authenticated_inherited_runtime_bootstraps_real_platform_services',
  'authenticated_runtime_accepts_only_the_vault_delivered_role_credential',
  'authenticated_runtime_applies_the_exact_bound_owner_migration_bundle',
  'authenticated_runtime_bootstraps_the_platform_postgres_schema',
  'authenticated_runtime_reconciles_roles_for_the_kernel_staged_binding',
];

async function run(command, args, options = {}) {
  if (process.env.MAKOSH_STORAGE_UNBUFFERED === '1') {
    await new Promise((resolve, reject) => {
      const child = spawn(command, args, { stdio: 'inherit', ...options });
      child.once('error', reject);
      child.once('exit', (code) => {
        if (code === 0) resolve();
        else reject(new Error(`${command} exited with ${code ?? 'signal'}`));
      });
    });
    return;
  }
  await execFileAsync(command, args, { encoding: 'utf8', ...options });
}

async function create_secret_files() {
  const directory = await mkdtemp(join(tmpdir(), 'makosh-storage-auth-'));
  const ports = await allocate_loopback_ports();
  await chmod(directory, 0o700);
  const postgresPath = await create_secret_file(directory, 'postgres-admin-password');
  const pgbouncerPath = await create_secret_file(directory, 'pgbouncer-admin-password');
  const runtimeDirectory = join(directory, 'runtime');
  const storageDirectory = join(runtimeDirectory, 'storage');
  const pgbouncerDirectory = join(storageDirectory, 'pgbouncer');
  const pgbouncerAuthDirectory = join(pgbouncerDirectory, 'auth');
  await mkdir(pgbouncerAuthDirectory, { recursive: true, mode: 0o700 });
  await chmod(runtimeDirectory, 0o700);
  await chmod(storageDirectory, 0o700);
  await chmod(pgbouncerDirectory, 0o700);
  await chmod(pgbouncerAuthDirectory, 0o700);
  const databasesPath = join(pgbouncerDirectory, 'databases.ini');
  await writeFile(databasesPath, '[databases]\n', { mode: 0o600 });
  await chmod(databasesPath, 0o600);
  return {
    directory,
    postgresPath,
    pgbouncerPath,
    storageDirectory,
    pgbouncerDirectory,
    pgbouncerAuthDirectory,
    databasesPath,
    authPath: join(pgbouncerAuthDirectory, 'users.txt'),
    ...ports,
  };
}

async function allocate_loopback_ports() {
  const reservations = await Promise.all(
    ['nats', 'postgres', 'pgbouncer', 'clamav'].map(() => reserve_loopback_port()),
  );
  const [natsPort, postgresPort, pgbouncerPort, clamavPort] = reservations.map(
    ({ port }) => port,
  );
  await Promise.all(
    reservations.map(
      ({ server }) =>
        new Promise((resolve, reject) => {
          server.close((error) => (error ? reject(error) : resolve()));
        }),
    ),
  );
  return { natsPort, postgresPort, pgbouncerPort, clamavPort };
}

async function reserve_loopback_port() {
  const server = createServer();
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  const address = server.address();
  if (!address || typeof address === 'string') {
    server.close();
    throw new Error('loopback port reservation is unavailable');
  }
  return { port: address.port, server };
}

async function create_secret_file(directory, name) {
  const path = join(directory, name);
  await writeFile(path, `${randomBytes(32).toString('hex')}\n`, { mode: 0o600 });
  await chmod(path, 0o600);
  return path;
}

function compose_environment(secrets) {
  return {
    ...process.env,
    MAKOSH_STORAGE_POSTGRES_SECRET_FILE: secrets.postgresPath,
    MAKOSH_STORAGE_PGBOUNCER_SECRET_FILE: secrets.pgbouncerPath,
    MAKOSH_STORAGE_PGBOUNCER_DATABASES_DIRECTORY: secrets.pgbouncerDirectory,
    MAKOSH_STORAGE_PGBOUNCER_AUTH_DIRECTORY: secrets.pgbouncerAuthDirectory,
    MAKOSH_STORAGE_PGBOUNCER_RUNTIME_UID: String(process.getuid()),
    MAKOSH_STORAGE_AUTHENTICATED_NATS_PORT: String(secrets.natsPort),
    MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT: String(secrets.postgresPort),
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT: String(secrets.pgbouncerPort),
    MAKOSH_ATTACHMENT_SECURITY_CLAMAV_PORT: String(secrets.clamavPort),
  };
}

async function start_contour(secrets) {
  allocate_runtime_files(secrets);
  await mkdir(secrets.pgbouncerDirectory, { recursive: true, mode: 0o700 });
  await mkdir(secrets.pgbouncerAuthDirectory, { recursive: true, mode: 0o700 });
  await chmod(secrets.pgbouncerDirectory, 0o700);
  await chmod(secrets.pgbouncerAuthDirectory, 0o700);
  await writeFile(secrets.databasesPath, '[databases]\n', { mode: 0o600 });
  await chmod(secrets.databasesPath, 0o600);
  await rm(secrets.authPath, { force: true });
  await run('docker', [...compose, 'up', '--detach', '--wait'], {
    env: compose_environment(secrets),
  });
  const { stdout } = await execFileAsync('docker', [...compose, 'ps', '--quiet', 'postgres'], {
    encoding: 'utf8',
    env: compose_environment(secrets),
  });
  const container = stdout.trim();
  if (!/^[a-f0-9]{12,64}$/i.test(container)) throw new Error('authenticated PostgreSQL container is unavailable');
  secrets.postgresContainer = container;
  const { stdout: natsStdout } = await execFileAsync(
    'docker',
    [...compose, 'ps', '--quiet', 'nats'],
    {
      encoding: 'utf8',
      env: compose_environment(secrets),
    },
  );
  const natsContainer = natsStdout.trim();
  if (!/^[a-f0-9]{12,64}$/i.test(natsContainer)) {
    throw new Error('authenticated NATS container is unavailable');
  }
  secrets.natsContainer = natsContainer;
}

function allocate_runtime_files(secrets) {
  const contour = randomBytes(8).toString('hex');
  secrets.runtimeDirectory = join(secrets.directory, `runtime-${contour}`);
  secrets.storageDirectory = join(secrets.runtimeDirectory, 'storage');
  secrets.pgbouncerDirectory = join(secrets.storageDirectory, 'pgbouncer');
  secrets.pgbouncerAuthDirectory = join(secrets.pgbouncerDirectory, 'auth');
  secrets.databasesPath = join(secrets.pgbouncerDirectory, 'databases.ini');
  secrets.authPath = join(secrets.pgbouncerAuthDirectory, 'users.txt');
}

async function stop_contour(secrets) {
  if (keepContour) return;
  await run('docker', [...compose, 'down', '--volumes', '--remove-orphans'], {
    env: compose_environment(secrets),
  });
}

async function run_conformance(secrets) {
  try {
    if (projectionPostgresTest) {
      await run_projection_postgres_conformance(secrets, projectionPostgresTest);
      return;
    }
    if (identityResolutionPostgresTest) {
      await run_identity_resolution_postgres_conformance(secrets, identityResolutionPostgresTest);
      return;
    }
    if (reviewedPersonMatchPromotionPostgresTest) {
      await run_reviewed_person_match_promotion_postgres_conformance(
        secrets,
        reviewedPersonMatchPromotionPostgresTest,
      );
      return;
    }
    if (reviewPersonMatchCandidatePostgresTest) {
      await run_review_person_match_candidate_postgres_conformance(
        secrets,
        reviewPersonMatchCandidatePostgresTest,
      );
      return;
    }
    if (mailPersonsSyncPostgresTest) {
      await run_mail_persons_sync_postgres_conformance(secrets, mailPersonsSyncPostgresTest);
      return;
    }
    if (personsPostgresTest) {
      await run_persons_postgres_conformance(secrets, personsPostgresTest);
      return;
    }
    if (schedulerPostgresTest) {
      await run_scheduler_postgres_conformance(secrets, schedulerPostgresTest);
      return;
    }
    if (delayedDeliveryPostgresTest) {
      await run_delayed_delivery_postgres_conformance(
        secrets,
        delayedDeliveryPostgresTest,
      );
      return;
    }
    if (deliveryIntentPostgresTest) {
      await run_delivery_intent_postgres_conformance(
        secrets,
        deliveryIntentPostgresTest,
      );
      return;
    }
    if (crossChannelForwardPostgresTest) {
      await run_cross_channel_forward_postgres_conformance(
        secrets,
        crossChannelForwardPostgresTest,
      );
      return;
    }
    if (telegramCallsTest) {
      await run_telegram_calls_conformance(secrets, telegramCallsTest);
      return;
    }
    for (const test of focusedTest ? [focusedTest] : managedTest ? [] : authenticatedTests) {
      await start_contour(secrets);
      try {
        await run('cargo', [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'test',
    '--locked',
    '-p',
    'makosh-storage-testkit',
    '--',
    '--ignored',
    test,
    '--test-threads=1',
    ], {
    env: {
      ...process.env,
      MAKOSH_STORAGE_TEST_DATABASE_URL: await postgres_test_database_url(secrets),
      MAKOSH_STORAGE_AUTHENTICATED_TEST: '1',
      MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE: secrets.pgbouncerPath,
      MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE: secrets.postgresPath,
      MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST: '127.0.0.1',
      MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT: String(secrets.pgbouncerPort),
      MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST: '127.0.0.1',
      MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT: String(secrets.postgresPort),
      MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE: secrets.databasesPath,
      MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE: secrets.authPath,
      MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_CONTAINER: secrets.postgresContainer,
    },
        });
      } finally {
        await stop_contour(secrets);
      }
    }
    if (!focusedTest) {
      await run_managed_process_conformance(secrets);
    }
  } catch (error) {
    print_test_diagnostics(error);
    throw error;
  }
}

async function run_projection_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_projection_conf_${process.pid}_${randomBytes(6).toString('hex')}`;
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    const args = [
      `+${toolchain}`, '--config', 'build.rustc-wrapper=""', 'test', '--locked',
      '-p', 'makosh-mail-persons-sync-persistence-testkit', '--test',
      'search_timeline_graph_postgres_live', '--', '--ignored',
    ];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, {
      env: {
        ...process.env,
        MAKOSH_PROJECTION_POSTGRES_URL: await postgres_database_url(secrets, database),
      },
    });
  } finally {
    if (created) {
      await postgres_admin_sql(
        secrets,
        'makosh_storage_authenticated',
        `DROP DATABASE ${database} WITH (FORCE)`,
      ).catch(() => undefined);
    }
    await stop_contour(secrets);
  }
}

async function run_identity_resolution_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_identity_res_conformance_${process.pid}_${randomBytes(8).toString('hex')}`;
  const sentinel = randomBytes(32).toString('hex');
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    await postgres_admin_sql(
      secrets,
      database,
      `CREATE TABLE public.makosh_identity_resolution_disposable_sentinel (sentinel_id SMALLINT PRIMARY KEY CHECK (sentinel_id = 1), token TEXT NOT NULL); INSERT INTO public.makosh_identity_resolution_disposable_sentinel VALUES (1, '${sentinel}')`,
    );
    const args = [
      `+${toolchain}`, '--config', 'build.rustc-wrapper=""', 'test', '--locked',
      '-p', 'makosh-mail-persons-sync-persistence-testkit', '--test',
      'identity_resolution_postgres_live', '--', '--ignored',
    ];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, {
      env: {
        ...process.env,
        MAKOSH_IDENTITY_RESOLUTION_POSTGRES_URL: await postgres_database_url(secrets, database),
        MAKOSH_IDENTITY_RESOLUTION_DISPOSABLE_DATABASE: database,
        MAKOSH_IDENTITY_RESOLUTION_DISPOSABLE_SENTINEL: sentinel,
      },
    });
  } finally {
    if (created) {
      await postgres_admin_sql(
        secrets,
        'makosh_storage_authenticated',
        `DROP DATABASE ${database} WITH (FORCE)`,
      ).catch(() => undefined);
    }
    await stop_contour(secrets);
  }
}

async function run_reviewed_person_match_promotion_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_reviewed_pm_conf_${process.pid}_${randomBytes(8).toString('hex')}`;
  const sentinel = randomBytes(32).toString('hex');
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    await postgres_admin_sql(secrets, database, `CREATE TABLE public.makosh_reviewed_person_match_promotion_disposable_sentinel (sentinel_id SMALLINT PRIMARY KEY CHECK (sentinel_id = 1), token TEXT NOT NULL); INSERT INTO public.makosh_reviewed_person_match_promotion_disposable_sentinel VALUES (1, '${sentinel}')`);
    const args = [`+${toolchain}`, '--config', 'build.rustc-wrapper=""', 'test', '--locked', '-p', 'makosh-mail-persons-sync-persistence-testkit', '--test', 'reviewed_person_match_promotion_postgres_live', '--', '--ignored'];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, { env: { ...process.env, MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_POSTGRES_URL: await postgres_database_url(secrets, database), MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_DISPOSABLE_DATABASE: database, MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_DISPOSABLE_SENTINEL: sentinel } });
  } finally {
    if (created) await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `DROP DATABASE ${database} WITH (FORCE)`).catch(() => undefined);
    await stop_contour(secrets);
  }
}

async function postgres_admin_sql(secrets, database, sql) {
  if (!/^[a-z0-9_]+$/u.test(database)) throw new Error('invalid disposable database identity');
  await run('docker', [
    'exec', secrets.postgresContainer, 'sh', '-ceu',
    'export PGPASSWORD="$(cat /run/secrets/storage_postgres_admin_password)"; exec psql --username=makosh_postgres_admin --dbname="$1" --set=ON_ERROR_STOP=1 --command="$2"',
    '--', database, sql,
  ]);
}

async function run_persons_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_persons_conformance_${process.pid}_${randomBytes(8).toString('hex')}`;
  const sentinel = randomBytes(32).toString('hex');
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    await postgres_admin_sql(
      secrets,
      database,
      `CREATE TABLE public.makosh_persons_disposable_sentinel (sentinel_id SMALLINT PRIMARY KEY CHECK (sentinel_id = 1), token TEXT NOT NULL); INSERT INTO public.makosh_persons_disposable_sentinel VALUES (1, '${sentinel}')`,
    );
    const args = [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-persons-persistence-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
    ];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, {
      env: {
        ...process.env,
        MAKOSH_PERSONS_POSTGRES_URL: await postgres_database_url(secrets, database),
        MAKOSH_PERSONS_DISPOSABLE_DATABASE: database,
        MAKOSH_PERSONS_DISPOSABLE_SENTINEL: sentinel,
      },
    });
  } finally {
    if (created) {
      await postgres_admin_sql(
        secrets,
        'makosh_storage_authenticated',
        `DROP DATABASE ${database} WITH (FORCE)`,
      ).catch(() => undefined);
    }
    await stop_contour(secrets);
  }
}

async function run_mail_persons_sync_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_mail_persons_sync_conformance_${process.pid}_${randomBytes(8).toString('hex')}`;
  const sentinel = randomBytes(32).toString('hex');
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    await postgres_admin_sql(
      secrets,
      database,
      `CREATE TABLE public.makosh_mail_persons_sync_disposable_sentinel (sentinel_id SMALLINT PRIMARY KEY CHECK (sentinel_id = 1), token TEXT NOT NULL); INSERT INTO public.makosh_mail_persons_sync_disposable_sentinel VALUES (1, '${sentinel}')`,
    );
    const args = [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-mail-persons-sync-persistence-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
    ];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, {
      env: {
        ...process.env,
        MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL: await postgres_database_url(secrets, database),
        MAKOSH_MAIL_PERSONS_SYNC_DISPOSABLE_DATABASE: database,
        MAKOSH_MAIL_PERSONS_SYNC_DISPOSABLE_SENTINEL: sentinel,
      },
    });
  } finally {
    if (created) {
      await postgres_admin_sql(
        secrets,
        'makosh_storage_authenticated',
        `DROP DATABASE ${database} WITH (FORCE)`,
      ).catch(() => undefined);
    }
    await stop_contour(secrets);
  }
}

async function run_review_person_match_candidate_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  const database = `makosh_review_pm_conformance_${process.pid}_${randomBytes(8).toString('hex')}`;
  const sentinel = randomBytes(32).toString('hex');
  let created = false;
  try {
    await postgres_admin_sql(secrets, 'makosh_storage_authenticated', `CREATE DATABASE ${database}`);
    created = true;
    await postgres_admin_sql(
      secrets,
      database,
      `CREATE TABLE public.makosh_review_person_match_candidate_disposable_sentinel (sentinel_id SMALLINT PRIMARY KEY CHECK (sentinel_id = 1), token TEXT NOT NULL); INSERT INTO public.makosh_review_person_match_candidate_disposable_sentinel VALUES (1, '${sentinel}')`,
    );
    const args = [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-mail-persons-sync-persistence-testkit',
      '--test',
      'review_person_match_candidate_postgres_live',
      '--',
      '--ignored',
    ];
    if (test !== 'all') args.push(test);
    args.push('--test-threads=1');
    await run('cargo', args, {
      env: {
        ...process.env,
        MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_POSTGRES_URL:
          await postgres_database_url(secrets, database),
        MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_DISPOSABLE_DATABASE: database,
        MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_DISPOSABLE_SENTINEL: sentinel,
      },
    });
  } finally {
    if (created) {
      await postgres_admin_sql(
        secrets,
        'makosh_storage_authenticated',
        `DROP DATABASE ${database} WITH (FORCE)`,
      ).catch(() => undefined);
    }
    await stop_contour(secrets);
  }
}

async function run_scheduler_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-scheduler-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_SCHEDULER_POSTGRES_URL: await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function run_delayed_delivery_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-communication-delayed-delivery-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_COMMUNICATION_DELAYED_DELIVERY_POSTGRES_URL:
          await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function run_delivery_intent_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-communication-delivery-intent-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_COMMUNICATION_DELIVERY_INTENT_POSTGRES_URL:
          await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function run_cross_channel_forward_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-communication-cross-channel-forward-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES_URL:
          await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function run_telegram_calls_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-telegram-calls-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_TELEGRAM_CALLS_POSTGRES_URL: await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function postgres_test_database_url(secrets) {
  return postgres_database_url(secrets, 'makosh_storage_authenticated');
}

async function postgres_database_url(secrets, database) {
  if (!/^[a-z0-9_]+$/u.test(database)) throw new Error('invalid PostgreSQL database identity');
  const password = (await readFile(secrets.postgresPath, 'utf8')).trim();
  const databaseUrl = new URL(
    `postgres://makosh_postgres_admin@127.0.0.1/${database}`,
  );
  databaseUrl.password = password;
  databaseUrl.port = String(secrets.postgresPort);
  databaseUrl.searchParams.set('sslmode', 'disable');
  return databaseUrl.toString();
}

async function run_managed_process_conformance(secrets) {
  const focusedPersonMatchCandidate = managedTest?.startsWith(
    'managed_review_person_match_candidate_',
  ) ?? false;
  const focusedCalendar = managedTest === 'managed_calendar_lifecycle_reminder_replays_and_restarts_with_owner_rls';
  const focusedOrganizations = managedTest === 'managed_organizations_lifecycle_replays_and_restarts_with_owner_rls';
  const focusedRelationships = managedTest === 'managed_relationships_lifecycle_replays_and_restarts_with_owner_rls';
  const focusedProjects = managedTest === 'managed_projects_lifecycle_replays_and_restarts_with_owner_rls';
  const focusedDecisions = managedTest === 'managed_decisions_lifecycle_replays_and_restarts_with_owner_rls';
  const focusedDocuments = managedTest === 'managed_documents_lifecycle_custody_replays_and_restarts_with_owner_rls';
  const focusedObligationCandidate = managedTest === 'managed_obligation_candidate_promotes_to_actual_obligation_and_replays';
  const focusedProjection = [
    'managed_search_timeline_graph_project_query_replay_and_restart',
    'managed_memory_consistency_risk_project_query_replay_and_restart',
  ].includes(managedTest);
  const focusedManaged = focusedPersonMatchCandidate || focusedCalendar || focusedOrganizations || focusedRelationships || focusedProjects || focusedDecisions || focusedDocuments || focusedObligationCandidate || focusedProjection;
  const tdjsonFixture = focusedManaged ? '' : await compile_tdjson_fixture(secrets);
  const tgcallsFixture = focusedManaged ? '' : await compile_tgcalls_fixture(secrets);
  const textExtractionOcr = focusedManaged
    ? { runner: '', english: '', russian: '' }
    : await prepare_attachment_text_extraction_ocr();
  const whisperStt = focusedManaged
    ? { runner: '', model: '', testWav: '' }
    : await prepare_whisper_stt(secrets);
  const focusedBuild = focusedProjection ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-search-runtime',
    '-p',
    'makosh-timeline-runtime',
    '-p',
    'makosh-graph-runtime',
    '-p',
    'makosh-memory-runtime',
    '-p',
    'makosh-consistency-runtime',
    '-p',
    'makosh-risk-runtime',
  ] : focusedObligationCandidate ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-blob-service',
    '-p',
    'makosh-communications-runtime',
    '-p',
    'makosh-communications-export-runtime',
    '-p',
    'makosh-review-obligation-candidate-runtime',
    '-p',
    'makosh-reviewed-obligation-candidate-promotion-runtime',
    '-p',
    'makosh-obligations-runtime',
  ] : focusedCalendar ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-scheduler-runtime',
    '-p',
    'makosh-calendar-runtime',
  ] : focusedDocuments ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-blob-service',
    '-p',
    'makosh-documents-runtime',
  ] : focusedProjects ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-projects-runtime',
  ] : focusedDecisions ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-decisions-runtime',
  ] : focusedRelationships ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-relationships-runtime',
  ] : focusedOrganizations ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-organizations-runtime',
  ] : focusedPersonMatchCandidate ? [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-communications-runtime',
    '-p',
    'makosh-communications-export-runtime',
    '-p',
    'makosh-blob-service',
    '-p',
    'makosh-persons-runtime',
    '-p',
    'makosh-identity-resolution-runtime',
    '-p',
    'makosh-review-person-match-candidate-runtime',
    '-p',
    'makosh-reviewed-person-match-candidate-promotion-runtime',
  ] : [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'makosh-vault-runtime',
    '-p',
    'makosh-storage-runtime',
    '-p',
    'makosh-scheduler-runtime',
    '-p',
    'makosh-communications-runtime',
    '-p',
    'makosh-communications-export-runtime',
    '-p',
    'makosh-communication-delivery-intent-runtime',
    '-p',
    'makosh-communication-cross-channel-forward-runtime',
    '-p',
    'makosh-communication-delayed-delivery-runtime',
    '-p',
    'makosh-communication-bulk-action-runtime',
    '-p',
    'makosh-communication-reply-suggestion-runtime',
    '-p',
    'makosh-communication-summary-runtime',
    '-p',
    'makosh-communication-translation-runtime',
    '-p',
    'makosh-communication-explanation-runtime',
    '-p',
    'makosh-communication-recipient-suggestion-runtime',
    '-p',
    'makosh-communication-task-candidate-runtime',
    '-p',
    'makosh-communication-note-candidate-runtime',
    '-p',
    'makosh-attachment-security-runtime',
    '-p',
    'makosh-attachment-archive-inspection-runtime',
    '-p',
    'makosh-attachment-text-extraction-runtime',
    '-p',
    'makosh-attachment-translation-runtime',
    '-p',
    'makosh-attachment-preview-runtime',
    '-p',
    'makosh-attachment-preview-evidence-replay-runtime',
    '-p',
    'makosh-review-attention-runtime',
    '-p',
    'makosh-review-task-candidate-runtime',
    '-p',
    'makosh-reviewed-task-candidate-promotion-runtime',
    '-p',
    'makosh-tasks-runtime',
    '-p',
    'makosh-persons-runtime',
    '-p',
    'makosh-review-person-match-candidate-runtime',
    '-p',
    'makosh-reviewed-person-match-candidate-promotion-runtime',
    '-p',
    'makosh-mail-persons-sync-runtime',
    '-p',
    'makosh-mail-persons-sync-assembly',
    '-p',
    'makosh-review-note-candidate-runtime',
    '-p',
    'makosh-reviewed-note-candidate-promotion-runtime',
    '-p',
    'makosh-knowledge-runtime',
    '-p',
    'makosh-ai-inference-runtime',
    '-p',
    'makosh-ollama-ai-runtime',
    '-p',
    'makosh-speech-to-text-runtime',
    '-p',
    'makosh-whisper-stt-runtime',
    '-p',
    'makosh-desktop-call-recording-runtime',
    '-p',
    'makosh-call-transcription-runtime',
    '-p',
    'makosh-mail-runtime',
    '-p',
    'makosh-telegram-runtime',
    '-p',
    'makosh-zulip-runtime',
    '-p',
    'makosh-whatsapp-runtime',
    '-p',
    'makosh-blob-service',
    '--features',
    'makosh-mail-runtime/conformance-test-support,makosh-zulip-runtime/conformance-test-support',
  ];
  await run('cargo', focusedBuild);
  for (const test of managedTest ? [managedTest] : [
    'managed_storage_binary_bootstraps_through_live_vault',
    'managed_scheduler_crash_uses_storage_control_successor_provisioning',
    'managed_communications_domain_starts_with_owner_local_storage_and_events',
    'managed_communications_export_workflow_starts_with_owner_local_storage_and_events',
    'managed_delivery_intent_reaches_gateway_sse_and_replays_after_restart',
    'managed_cross_channel_forward_reaches_delivery_intent_and_replays_after_restart',
    'managed_delayed_delivery_starts_with_scheduler_and_delivery_intent',
    'managed_bulk_action_reaches_gateway_sse_and_replays_after_restart',
    'managed_attachment_security_engine_starts_with_exact_signed_contracts',
    'managed_archive_inspection_reaches_gateway_sse_and_replays_after_restart',
    'managed_attachment_text_extraction_completes_through_gateway_and_replays_after_restart',
    'managed_attachment_translation_reaches_source_ai_and_gateway_sse',
    'managed_attachment_preview_reaches_gateway_blob_sse_and_replays_after_restart',
    'managed_attachment_preview_evidence_replay_runtime_starts_with_exact_signed_contracts',
    'managed_attachment_preview_evidence_replay_restores_expired_sources_to_browser_preview',
    'managed_review_attention_reaches_gateway_sse_and_replays_after_restart',
    'managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart',
    'managed_tasks_lifecycle_replays_and_restarts_with_owner_rls',
    'managed_persons_bootstrap_is_control_responsive_and_requires_exact_consumer',
    'managed_persons_command_is_atomic_replayable_restart_and_control_close_safe',
    'managed_mail_persons_sync_actual_binary_bootstraps_exact_private_contour',
    'empty_start_provider_resync_without_contacts_import',
    'managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart',
    'managed_knowledge_lifecycle_search_replays_and_restarts_with_owner_rls',
    'managed_calendar_lifecycle_reminder_replays_and_restarts_with_owner_rls',
    'managed_organizations_lifecycle_replays_and_restarts_with_owner_rls',
    'managed_relationships_lifecycle_replays_and_restarts_with_owner_rls',
    'managed_projects_lifecycle_replays_and_restarts_with_owner_rls',
    'managed_decisions_lifecycle_replays_and_restarts_with_owner_rls',
    'managed_documents_lifecycle_custody_replays_and_restarts_with_owner_rls',
    'managed_search_timeline_graph_project_query_replay_and_restart',
    'managed_obligation_candidate_promotes_to_actual_obligation_and_replays',
    'managed_communications_ai_source_is_event_only_and_revision_fenced',
    'managed_reply_suggestion_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_summary_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_translation_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse',
    'managed_recipient_suggestion_reaches_gateway_sse_and_replays_after_restart',
    'managed_ai_inference_routes_to_ollama_and_replays_after_restart',
    'managed_ollama_ai_runtime_replays_provider_unavailable_without_second_http_attempt',
    'managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart',
    'managed_speech_to_text_whisper_bootstrap_fails_closed_and_stops_promptly',
    'managed_speech_to_text_whisper_private_surfaces_reject_malformed_provider_output',
    'managed_desktop_recording_reaches_blob_event_gateway_sse_and_restart',
    'managed_call_transcription_reaches_recording_stt_gateway_blob_and_restarts',
    'managed_mail_runtime_uses_kernel_leases_and_route_specific_admission',
    'managed_mail_credential_rotation_quiesces_until_settings_successor',
    'managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event',
    'managed_mail_gmail_runtime_mutates_once_and_replays_event_without_private_payload',
    'managed_mail_gmail_oauth_rotates_credentials_once_and_fails_closed',
    'managed_mail_gmail_oauth_route_is_fenced_by_owner_revoke',
    'managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission',
    'managed_zulip_account_rotation_and_retirement_use_settings_successors',
    'managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_zulip_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_zulip_private_surfaces_reject_malformed_provider_output',
    'managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing',
    'managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_whatsapp_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_whatsapp_private_surfaces_reject_malformed_host_output',
    'managed_telegram_runtime_uses_kernel_leases_and_event_only_communications_handoff',
    'managed_telegram_core_operational_projection_is_restart_safe',
    'managed_telegram_folder_reassignment_converges_after_partial_provider_failure',
    'managed_telegram_automation_route_is_durable_and_provider_side_effect_free',
    'managed_telegram_call_history_route_is_durable_and_replayable',
    'managed_telegram_runtime_bootstrap_fails_closed_and_stops_promptly',
    'managed_telegram_private_surfaces_reject_malformed_provider_output',
    'managed_telegram_real_tdlib_reaches_qr_authorization',
    'managed_telegram_real_tgcalls_audio_device_conformance',
    'managed_call_evidence_survives_nats_outage_and_replays_through_gateway_sse',
  ]) {
    await start_contour(secrets);
    try {
      await run('cargo', [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'test',
    '--locked',
    '-p',
    'makosh-kernel-recovery-testkit',
    '--',
    '--ignored',
    test,
    '--test-threads=1',
  ], {
    env: {
      ...authenticated_environment(secrets),
      MAKOSH_VAULT_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-vault-runtime`,
      MAKOSH_STORAGE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-storage-runtime`,
      MAKOSH_SCHEDULER_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-scheduler-runtime`,
      MAKOSH_SCHEDULER_LIVE_NATS_ENDPOINT: `nats://127.0.0.1:${secrets.natsPort}`,
      MAKOSH_COMMUNICATIONS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communications-runtime`,
      MAKOSH_COMMUNICATIONS_EXPORT_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communications-export-runtime`,
      MAKOSH_COMMUNICATION_DELIVERY_INTENT_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-delivery-intent-runtime`,
      MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-cross-channel-forward-runtime`,
      MAKOSH_COMMUNICATION_DELAYED_DELIVERY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-delayed-delivery-runtime`,
      MAKOSH_COMMUNICATION_BULK_ACTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-bulk-action-runtime`,
      MAKOSH_COMMUNICATION_REPLY_SUGGESTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-reply-suggestion-runtime`,
      MAKOSH_COMMUNICATION_SUMMARY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-summary-runtime`,
      MAKOSH_COMMUNICATION_TRANSLATION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-translation-runtime`,
      MAKOSH_COMMUNICATION_EXPLANATION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-explanation-runtime`,
      MAKOSH_COMMUNICATION_RECIPIENT_SUGGESTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-recipient-suggestion-runtime`,
      MAKOSH_COMMUNICATION_TASK_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-task-candidate-runtime`,
      MAKOSH_COMMUNICATION_NOTE_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-communication-note-candidate-runtime`,
      MAKOSH_ATTACHMENT_SECURITY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-security-runtime`,
      MAKOSH_ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-archive-inspection-runtime`,
      MAKOSH_ATTACHMENT_TEXT_EXTRACTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-text-extraction-runtime`,
      MAKOSH_ATTACHMENT_TRANSLATION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-translation-runtime`,
      MAKOSH_ATTACHMENT_PREVIEW_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-preview-runtime`,
      MAKOSH_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-attachment-preview-evidence-replay-runtime`,
      MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER: textExtractionOcr.runner,
      MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_ENG: textExtractionOcr.english,
      MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_RUS: textExtractionOcr.russian,
      MAKOSH_REVIEW_ATTENTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-attention-runtime`,
      MAKOSH_REVIEW_TASK_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-task-candidate-runtime`,
      MAKOSH_REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-reviewed-task-candidate-promotion-runtime`,
      MAKOSH_TASKS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-tasks-runtime`,
      MAKOSH_REVIEW_OBLIGATION_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-obligation-candidate-runtime`,
      MAKOSH_REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-reviewed-obligation-candidate-promotion-runtime`,
      MAKOSH_OBLIGATIONS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-obligations-runtime`,
      MAKOSH_PERSONS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-persons-runtime`,
      MAKOSH_IDENTITY_RESOLUTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-identity-resolution-runtime`,
      MAKOSH_SEARCH_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-search-runtime`,
      MAKOSH_TIMELINE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-timeline-runtime`,
      MAKOSH_GRAPH_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-graph-runtime`,
      MAKOSH_MEMORY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-memory-runtime`,
      MAKOSH_CONSISTENCY_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-consistency-runtime`,
      MAKOSH_RISK_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-risk-runtime`,
      MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-person-match-candidate-runtime`,
      MAKOSH_REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-reviewed-person-match-candidate-promotion-runtime`,
      MAKOSH_MAIL_PERSONS_SYNC_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-mail-persons-sync-runtime`,
      MAKOSH_MAIL_PERSONS_SYNC_ASSEMBLY_BIN: `${process.cwd()}/target/debug/makosh-mail-persons-sync-assembly`,
      MAKOSH_REVIEW_NOTE_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-note-candidate-runtime`,
      MAKOSH_REVIEWED_NOTE_CANDIDATE_PROMOTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-reviewed-note-candidate-promotion-runtime`,
      MAKOSH_KNOWLEDGE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-knowledge-runtime`,
      MAKOSH_CALENDAR_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-calendar-runtime`,
      MAKOSH_ORGANIZATIONS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-organizations-runtime`,
      MAKOSH_RELATIONSHIPS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-relationships-runtime`,
      MAKOSH_PROJECTS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-projects-runtime`,
      MAKOSH_DECISIONS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-decisions-runtime`,
      MAKOSH_DOCUMENTS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-documents-runtime`,
      MAKOSH_AI_INFERENCE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-ai-inference-runtime`,
      MAKOSH_OLLAMA_AI_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-ollama-ai-runtime`,
      MAKOSH_SPEECH_TO_TEXT_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-speech-to-text-runtime`,
      MAKOSH_WHISPER_STT_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-whisper-stt-runtime`,
      MAKOSH_DESKTOP_CALL_RECORDING_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-desktop-call-recording-runtime`,
      MAKOSH_CALL_TRANSCRIPTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-call-transcription-runtime`,
      MAKOSH_WHISPER_STT_RUNNER: whisperStt.runner,
      MAKOSH_WHISPER_STT_MODEL: whisperStt.model,
      MAKOSH_WHISPER_STT_TEST_WAV: whisperStt.testWav,
      MAKOSH_MAIL_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-mail-runtime`,
      MAKOSH_TELEGRAM_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-telegram-runtime`,
      MAKOSH_ZULIP_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-zulip-runtime`,
      MAKOSH_WHATSAPP_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-whatsapp-runtime`,
      MAKOSH_TELEGRAM_TDJSON_FIXTURE: tdjsonFixture,
      MAKOSH_TELEGRAM_TGCALLS_FIXTURE: tgcallsFixture,
      MAKOSH_BLOB_SERVICE_BIN: `${process.cwd()}/target/debug/makosh-blob-service`,
      MAKOSH_COMMUNICATIONS_LIVE_NATS_ENDPOINT: `nats://127.0.0.1:${secrets.natsPort}`,
    },
      });
    } finally {
      await stop_contour(secrets);
    }
  }
}

async function prepare_attachment_text_extraction_ocr() {
  const root = process.env.MAKOSH_TEST_ATTACHMENT_TEXT_EXTRACTION_OCR_ROOT
    || join(process.cwd(), '..', '.local', 'dev-native', 'attachment-text-extraction-ocr');
  const resources = {
    runner: join(root, 'tesseract-runner'),
    english: join(root, 'eng.traineddata'),
    russian: join(root, 'rus.traineddata'),
  };
  try {
    await Promise.all(Object.values(resources).map((path) => access(path)));
  } catch {
    await run('./scripts/build-attachment-text-extraction-ocr-macos.sh', [
      '--output-dir',
      root,
    ]);
  }
  await Promise.all(Object.values(resources).map((path) => access(path)));
  return resources;
}

async function prepare_whisper_stt(secrets) {
  const root = process.env.MAKOSH_TEST_WHISPER_STT_ROOT
    || join(process.cwd(), '..', '.local', 'dev-native', 'whisper-stt');
  const resources = {
    runner: join(root, 'whisper-cli'),
    model: join(root, 'ggml-base.bin'),
    testWav: join(secrets.directory, 'whisper-stt-test.wav'),
  };
  try {
    await Promise.all([access(resources.runner), access(resources.model)]);
  } catch {
    await run('./scripts/build-whisper-stt-macos.sh', [
      '--output-dir',
      root,
    ]);
  }
  await Promise.all([access(resources.runner), access(resources.model)]);
  const source = join(secrets.directory, 'whisper-stt-test.aiff');
  await run('/usr/bin/say', [
    '-v',
    'Samantha',
    '-r',
    '120',
    '-o',
    source,
    'Макошь clean room transcription. Макошь managed speech engine. Макошь private audio evidence.',
  ]);
  await run('/usr/bin/afconvert', [
    '-f',
    'WAVE',
    '-d',
    'LEI16@16000',
    '-c',
    '1',
    source,
    resources.testWav,
  ]);
  await canonicalize_whisper_test_wav(resources.testWav);
  await access(resources.testWav);
  const preflightOutput = join(secrets.directory, 'whisper-stt-preflight');
  await run(resources.runner, [
    '--model',
    resources.model,
    '--file',
    resources.testWav,
    '--threads',
    '4',
    '--language',
    'en',
    '--output-json',
    '--output-file',
    preflightOutput,
    '--no-prints',
  ], { env: {} });
  const preflight = JSON.parse(await readFile(`${preflightOutput}.json`, 'utf8'));
  if (!Array.isArray(preflight.transcription) || preflight.transcription.length === 0) {
    throw new Error('synthetic Whisper STT fixture produced no transcription');
  }
  return resources;
}

async function canonicalize_whisper_test_wav(path) {
  const source = await readFile(path);
  if (source.length < 44 || source.toString('ascii', 0, 4) !== 'RIFF'
      || source.toString('ascii', 8, 12) !== 'WAVE') {
    throw new Error('Whisper test fixture is not a RIFF/WAVE file');
  }
  let format;
  let samples;
  for (let offset = 12; offset + 8 <= source.length;) {
    const kind = source.toString('ascii', offset, offset + 4);
    const size = source.readUInt32LE(offset + 4);
    const start = offset + 8;
    const end = start + size;
    if (end > source.length) throw new Error(`truncated WAV ${kind} chunk`);
    if (kind === 'fmt ') format = source.subarray(start, end);
    if (kind === 'data') samples = source.subarray(start, end);
    offset = end + (size % 2);
  }
  if (!format || format.length < 16 || !samples || samples.length === 0
      || format.readUInt16LE(0) !== 1 || format.readUInt16LE(2) !== 1
      || format.readUInt32LE(4) !== 16_000 || format.readUInt16LE(14) !== 16
      || samples.length % 2 !== 0) {
    throw new Error('Whisper test fixture is not PCM S16LE mono 16000 Hz');
  }
  const canonical = Buffer.alloc(44 + samples.length);
  canonical.write('RIFF', 0, 'ascii');
  canonical.writeUInt32LE(36 + samples.length, 4);
  canonical.write('WAVEfmt ', 8, 'ascii');
  canonical.writeUInt32LE(16, 16);
  canonical.writeUInt16LE(1, 20);
  canonical.writeUInt16LE(1, 22);
  canonical.writeUInt32LE(16_000, 24);
  canonical.writeUInt32LE(32_000, 28);
  canonical.writeUInt16LE(2, 32);
  canonical.writeUInt16LE(16, 34);
  canonical.write('data', 36, 'ascii');
  canonical.writeUInt32LE(samples.length, 40);
  samples.copy(canonical, 44);
  await writeFile(path, canonical, { mode: 0o600 });
}

async function compile_tdjson_fixture(secrets) {
  const source = join(
    process.cwd(),
    'tests',
    'fixtures',
    'telegram-tdjson',
    'tdjson.c',
  );
  const output = join(secrets.directory, 'libmakosh-telegram-tdjson-fixture.dylib');
  const linkMode = process.platform === 'darwin' ? '-dynamiclib' : '-shared';
  await run('cc', [
    linkMode,
    '-fPIC',
    '-Wall',
    '-Wextra',
    '-Werror',
    source,
    '-o',
    output,
  ]);
  return output;
}

async function compile_tgcalls_fixture(secrets) {
  const source = join(
    process.cwd(),
    'tests',
    'fixtures',
    'telegram-tgcalls',
    'bridge.c',
  );
  const output = join(secrets.directory, 'libmakosh-telegram-tgcalls-fixture.dylib');
  const linkMode = process.platform === 'darwin' ? '-dynamiclib' : '-shared';
  await run('cc', [
    linkMode,
    '-fPIC',
    '-Wall',
    '-Wextra',
    '-Werror',
    source,
    '-o',
    output,
  ]);
  return output;
}

function authenticated_environment(secrets) {
  return {
    ...process.env,
    MAKOSH_STORAGE_AUTHENTICATED_TEST: '1',
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE: secrets.pgbouncerPath,
    MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE: secrets.postgresPath,
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST: '127.0.0.1',
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT: String(secrets.pgbouncerPort),
    MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST: '127.0.0.1',
    MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT: String(secrets.postgresPort),
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE: secrets.databasesPath,
    MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE: secrets.authPath,
    MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_CONTAINER: secrets.postgresContainer,
    MAKOSH_STORAGE_AUTHENTICATED_NATS_CONTAINER: secrets.natsContainer,
  };
}

function print_test_diagnostics(error) {
  if (!(error && typeof error === 'object' && 'stdout' in error)) return;
  const output = String(error.stdout)
    .split('\n')
    .filter((line) => !/(password|secret)/i.test(line));
  process.stderr.write(`${output.join('\n')}\n`);
}

async function cleanup(secret) {
  if (keepContour) {
    process.stderr.write(`authenticated-storage-contour: ${project}\n`);
    process.stderr.write(`authenticated-storage-runtime: ${secret.pgbouncerDirectory}\n`);
    return;
  }
  await stop_contour(secret).catch(() => undefined);
  await rm(secret.directory, { force: true, recursive: true });
}

async function print_startup_diagnostics(secrets) {
  const result = await execFileAsync('docker', [...compose, 'logs', '--tail', '30', 'pgbouncer'], {
    encoding: 'utf8',
    env: compose_environment(secrets),
  }).catch(() => null);
  if (!result) return;
  const safeLines = `${result.stdout}${result.stderr}`
    .split('\n')
    .filter((line) => !/(password|secret)/i.test(line));
  process.stderr.write(`${safeLines.join('\n')}\n`);
}

const secret = await create_secret_files();
let stage = 'start';
try {
  stage = 'conformance';
  await run_conformance(secret);
  process.stdout.write('authenticated-storage-conformance: ok\n');
} catch (error) {
  process.stderr.write(`authenticated-storage-conformance: failed during ${stage}\n`);
  await print_startup_diagnostics(secret);
  if (error instanceof Error) process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
} finally {
  await cleanup(secret);
}
