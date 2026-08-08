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
const contactsPostgresTest = process.env.MAKOSH_CONTACTS_POSTGRES_TEST_FILTER?.trim();
const mailContactsSyncPostgresTest =
  process.env.MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_TEST_FILTER?.trim();
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
    if (contactsPostgresTest) {
      await run_contacts_postgres_conformance(secrets, contactsPostgresTest);
      return;
    }
    if (mailContactsSyncPostgresTest) {
      await run_mail_contacts_sync_postgres_conformance(secrets, mailContactsSyncPostgresTest);
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

async function run_contacts_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-contacts-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_CONTACTS_POSTGRES_URL: await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function run_mail_contacts_sync_postgres_conformance(secrets, test) {
  await start_contour(secrets);
  try {
    await run('cargo', [
      `+${toolchain}`,
      '--config',
      'build.rustc-wrapper=""',
      'test',
      '--locked',
      '-p',
      'makosh-mail-contacts-sync-testkit',
      '--test',
      'postgres_live',
      '--',
      '--ignored',
      test,
      '--test-threads=1',
    ], {
      env: {
        ...process.env,
        MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL: await postgres_test_database_url(secrets),
      },
    });
  } finally {
    await stop_contour(secrets);
  }
}

async function postgres_test_database_url(secrets) {
  const password = (await readFile(secrets.postgresPath, 'utf8')).trim();
  const databaseUrl = new URL(
    'postgres://makosh_postgres_admin@127.0.0.1/makosh_storage_authenticated',
  );
  databaseUrl.password = password;
  databaseUrl.port = String(secrets.postgresPort);
  databaseUrl.searchParams.set('sslmode', 'disable');
  return databaseUrl.toString();
}

async function run_managed_process_conformance(secrets) {
  const tdjsonFixture = await compile_tdjson_fixture(secrets);
  const tgcallsFixture = await compile_tgcalls_fixture(secrets);
  const textExtractionOcr = await prepare_attachment_text_extraction_ocr();
  const whisperStt = await prepare_whisper_stt(secrets);
  await run('cargo', [
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
    'makosh-contacts-runtime',
    '-p',
    'makosh-mail-contacts-sync-runtime',
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
  ]);
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
    'managed_contacts_command_is_atomic_replayable_and_restart_safe',
    'managed_mail_contacts_sync_reaches_contacts_through_events',
    'managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart',
    'managed_communications_ai_source_is_event_only_and_revision_fenced',
    'managed_reply_suggestion_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_summary_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_translation_reaches_ai_and_replays_through_gateway_sse',
    'managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse',
    'managed_recipient_suggestion_reaches_gateway_sse_and_replays_after_restart',
    'managed_ai_inference_routes_to_ollama_and_replays_after_restart',
    'managed_ollama_ai_runtime_replays_provider_unavailable_without_second_http_attempt',
    'managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart',
    'managed_desktop_recording_reaches_blob_event_gateway_sse_and_restart',
    'managed_call_transcription_reaches_recording_stt_gateway_blob_and_restarts',
    'managed_mail_runtime_uses_kernel_leases_and_route_specific_admission',
    'managed_mail_credential_rotation_quiesces_until_settings_successor',
    'managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event',
    'managed_mail_gmail_runtime_mutates_once_and_replays_event_without_private_payload',
    'managed_mail_gmail_oauth_rotates_credentials_once_and_fails_closed',
    'managed_mail_gmail_oauth_route_is_fenced_by_owner_revoke',
    'managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission',
    'managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing',
    'managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff',
    'managed_telegram_automation_route_is_durable_and_provider_side_effect_free',
    'managed_telegram_call_history_route_is_durable_and_replayable',
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
      MAKOSH_CONTACTS_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-contacts-runtime`,
      MAKOSH_MAIL_CONTACTS_SYNC_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-mail-contacts-sync-runtime`,
      MAKOSH_REVIEW_NOTE_CANDIDATE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-review-note-candidate-runtime`,
      MAKOSH_REVIEWED_NOTE_CANDIDATE_PROMOTION_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-reviewed-note-candidate-promotion-runtime`,
      MAKOSH_KNOWLEDGE_RUNTIME_BIN: `${process.cwd()}/target/debug/makosh-knowledge-runtime`,
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
