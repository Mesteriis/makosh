#!/usr/bin/env node
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync } from 'node:fs';
import { access, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import { runCompliance } from './makosh-lab/compliance.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const provider = valueAfter('--provider') ?? process.env.PROVIDER ?? 'zulip';
const execute = args.includes('--execute');
const useTestcontainers = args.includes('--testcontainers') || process.env.MAKOSH_LAB_TESTCONTAINERS === '1';
const backendEvidence = args.includes('--backend-evidence') || process.env.MAKOSH_LAB_BACKEND_EVIDENCE === '1';
const strictEnv = args.includes('--strict-env');
const strictTools = args.includes('--strict-tools');
const requireClosed = args.includes('--require-closed') || process.env.MAKOSH_LAB_REQUIRE_CLOSED === '1';
const action = positionalAction() ?? 'help';
const provisionPrefix = 'MAKOSH_ZULIP_PROVISION ';

if (provider !== 'zulip') {
  fail(`Unsupported Макошь Lab provider ${provider}; supported provider: zulip.`);
}

switch (action) {
  case 'readiness':
    await readiness();
    break;
  case 'prepare':
    ensureZulipCheckout();
    break;
  case 'init':
    zulipCompose(['pull']);
    zulipCompose(['run', '--rm', 'zulip', 'app:init']);
    break;
  case 'up':
    zulipCompose(['up', 'zulip', '--wait']);
    break;
  case 'down':
    zulipCompose(['down']);
    break;
  case 'logs':
    zulipCompose(['logs', '-f', 'zulip']);
    break;
  case 'realm-link':
    zulipCompose(['exec', 'zulip', './manage.py', 'generate_realm_creation_link']);
    break;
  case 'scenario':
    await runScenario();
    break;
  case 'compliance':
    try {
      await runCompliance({
        repoRoot,
        provider,
        reportDir: valueAfter('--report-dir') ?? `.local/makosh-lab/reports/${provider}`,
        backendEvidence,
        requireClosed,
        log: labLog,
      });
    } catch (error) {
      fail(safeErrorMessage(error));
    }
    break;
  case 'help':
    usage(0);
    break;
  default:
    console.error(`Unknown Макошь Lab action ${action}.`);
    usage(1);
}

function positionalAction() {
  return args.find((arg, index) => {
    if (arg.startsWith('--')) return false;
    const previous = args[index - 1];
    return !['--provider', '--scenario', '--report-dir'].includes(previous);
  });
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index === -1) return null;
  return args[index + 1] ?? null;
}

function usage(exitCode) {
  console.log(`Usage: node scripts/makosh-lab.mjs [--provider zulip] [--execute] <action>

Actions:
  readiness   Run static Макошь Lab readiness checks
  prepare     Clone the official docker-zulip repository into .local/makosh-lab/docker-zulip
  init        Pull images and run docker-zulip app:init
  up          Start local Zulip with docker compose up zulip --wait
  down        Stop and remove local Zulip compose services
  logs        Tail local Zulip logs
  realm-link  Generate a local Zulip realm creation link
  scenario    Run the reference provider scenario; dry-run unless --execute is set
  compliance  Generate a Communication Compliance Suite report from local evidence

Options:
  --provider zulip        Provider to use; defaults to zulip
  --execute               Execute provider API calls for scenario
  --testcontainers        For scenario --execute, start and clean up a local Zulip fixture
  --backend-evidence      For scenario, run backend live evidence; for compliance, run backend contract suites
  --scenario <path>       Scenario JSON path
  --report-dir <path>     Report directory; defaults to .local/makosh-lab/reports/zulip
  --require-closed        Fail compliance when any capability is pending
  --strict-env            Fail readiness when ZULIP_BASE_URL, ZULIP_EMAIL or ZULIP_API_KEY is missing
  --strict-tools          Fail readiness when docker or git is missing
`);
  process.exit(exitCode);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

async function exists(relativePath) {
  try {
    await access(path.join(repoRoot, relativePath));
    return true;
  } catch {
    return false;
  }
}

function commandExists(command) {
  const result = spawnSync('bash', ['-lc', `command -v ${command}`], {
    cwd: repoRoot,
    stdio: 'ignore',
  });
  return result.status === 0;
}

async function readiness() {
  const checks = [];
  const record = (name, ok, detail = '') => checks.push({ name, ok, detail });

  record(
    'docker command',
    commandExists('docker') || !strictTools,
    'required for local Zulip stack; use --strict-tools to fail when missing'
  );
  record(
    'git command',
    commandExists('git') || !strictTools,
    'required for docker-zulip checkout; use --strict-tools to fail when missing'
  );
  record('node fetch runtime', typeof fetch === 'function', 'Node 18+ fetch is required');
  record(
    'Zulip integration module',
    await exists('backend/src/integrations/zulip/mod.rs'),
    'backend/src/integrations/zulip/mod.rs'
  );
  record(
    'Zulip Signal Hub dispatcher',
    await exists('backend/src/domains/signal_hub/zulip.rs'),
    'backend/src/domains/signal_hub/zulip.rs'
  );
  for (const scenarioPath of scenarioPaths()) {
    record(`Zulip scenario fixture ${path.basename(scenarioPath)}`, await exists(scenarioPath), scenarioPath);
  }

  for (const scenarioPath of scenarioPaths()) {
    try {
      await validateScenario(scenarioPath);
      record(`scenario JSON schema ${path.basename(scenarioPath)}`, true);
    } catch (error) {
      record(`scenario JSON schema ${path.basename(scenarioPath)}`, false, error.message);
    }
  }

  for (const key of ['ZULIP_BASE_URL', 'ZULIP_EMAIL', 'ZULIP_API_KEY']) {
    const ok = Boolean(process.env[key]);
    record(`env ${key}`, ok || !strictEnv, ok ? 'configured' : 'missing');
  }

  let failed = 0;
  for (const check of checks) {
    const marker = check.ok ? 'ok' : 'fail';
    console.log(`${marker.padEnd(4)} ${check.name}${check.detail ? ` - ${check.detail}` : ''}`);
    if (!check.ok) failed += 1;
  }

  if (failed > 0) {
    fail(`Макошь Lab Zulip readiness failed: ${failed} check(s) failed.`);
  }

  console.log('Макошь Lab Zulip readiness passed.');
}

async function validateScenario(relativePath) {
  const scenario = JSON.parse(await readFile(path.join(repoRoot, relativePath), 'utf8'));
  const required = ['scenario_id', 'provider', 'actions', 'expect'];
  const missing = required.filter((key) => !(key in scenario));
  if (missing.length > 0) {
    throw new Error(`${relativePath} is missing ${missing.join(', ')}`);
  }
  if (scenario.provider !== 'zulip') {
    throw new Error(`${relativePath} provider must be zulip`);
  }
  if (!Array.isArray(scenario.actions) || scenario.actions.length === 0) {
    throw new Error(`${relativePath} must contain at least one action`);
  }
  if (!Array.isArray(scenario.expect) || scenario.expect.length === 0) {
    throw new Error(`${relativePath} must contain at least one expected stage`);
  }
  if (scenario.backend_expected_stages && !Array.isArray(scenario.backend_expected_stages)) {
    throw new Error(`${relativePath} backend_expected_stages must be an array when present`);
  }
  if (scenario.capability_contract && !Array.isArray(scenario.capability_contract)) {
    throw new Error(`${relativePath} capability_contract must be an array when present`);
  }
  for (const [index, capability] of (scenario.capability_contract ?? []).entries()) {
    for (const key of ['name', 'status', 'evidence']) {
      if (!(key in capability)) {
        throw new Error(`${relativePath} capability_contract[${index}] is missing ${key}`);
      }
    }
  }
  for (const [index, action] of scenario.actions.entries()) {
    validateScenarioAction(relativePath, index, action);
  }
}

function validateScenarioAction(relativePath, index, action) {
  if (!action || typeof action !== 'object' || typeof action.kind !== 'string') {
    throw new Error(`${relativePath} actions[${index}] must include kind`);
  }
  const requiredByKind = {
    send_stream_message: ['stream', 'topic', 'content'],
    send_direct_message: ['recipients', 'content'],
    send_stream_message_with_upload: ['stream', 'topic', 'content', 'filename'],
    add_reaction: ['emoji_name'],
    remove_reaction: ['emoji_name'],
    update_message: ['updated_content'],
    delete_message: [],
    upload_file: ['filename'],
    download_user_upload: [],
  };
  const required = requiredByKind[action.kind];
  if (!required) {
    throw new Error(`${relativePath} actions[${index}] has unsupported kind ${action.kind}`);
  }
  const missing = required.filter((key) => !nonEmptyActionValue(action[key]));
  if (missing.length > 0) {
    throw new Error(`${relativePath} actions[${index}] is missing ${missing.join(', ')}`);
  }
  if (
    action.kind === 'send_direct_message'
    && (!Array.isArray(action.recipients) || action.recipients.length === 0)
  ) {
    throw new Error(`${relativePath} actions[${index}] recipients must be a non-empty array`);
  }
  if (action.kind === 'send_direct_message') {
    for (const [recipientIndex, recipient] of action.recipients.entries()) {
      const validRecipient =
        (typeof recipient === 'string' && recipient.trim() !== '')
        || (Number.isSafeInteger(recipient) && recipient > 0);
      if (!validRecipient) {
        throw new Error(
          `${relativePath} actions[${index}] recipients[${recipientIndex}] must be a non-empty string or positive integer user id`
        );
      }
    }
  }
  if (
    ['add_reaction', 'remove_reaction', 'update_message', 'delete_message'].includes(action.kind)
    && !action.message_id
    && action.message_id_ref !== 'last'
  ) {
    throw new Error(`${relativePath} actions[${index}] requires message_id or message_id_ref=last`);
  }
  if (
    action.kind === 'download_user_upload'
    && !action.upload_uri
    && action.upload_uri_ref !== 'last'
  ) {
    throw new Error(`${relativePath} actions[${index}] requires upload_uri or upload_uri_ref=last`);
  }
}

function nonEmptyActionValue(value) {
  return typeof value === 'string' ? value.trim() !== '' : value !== undefined && value !== null;
}

function defaultScenarioPath() {
  return 'testing/makosh-lab/scenarios/zulip/message-to-task-candidate.json';
}

function scenarioPaths() {
  return [
    defaultScenarioPath(),
    'testing/makosh-lab/scenarios/zulip/attachment-materialization.json',
    'testing/makosh-lab/scenarios/zulip/direct-message.json',
  ];
}

function zulipRepoDir() {
  return path.resolve(
    repoRoot,
    process.env.MAKOSH_LAB_ZULIP_REPO_DIR ?? '.local/makosh-lab/docker-zulip'
  );
}

function ensureZulipCheckout() {
  const repoDir = zulipRepoDir();
  if (existsSync(path.join(repoDir, '.git'))) {
    console.log(`docker-zulip checkout already exists: ${repoDir}`);
    return;
  }

  const repoUrl = process.env.MAKOSH_LAB_ZULIP_REPO_URL ?? 'https://github.com/zulip/docker-zulip.git';
  mkdirSync(path.dirname(repoDir), { recursive: true });
  run('git', ['clone', '--depth=1', repoUrl, repoDir], { cwd: repoRoot });
}

function zulipCompose(composeArgs) {
  ensureZulipCheckout();
  run('docker', ['compose', ...composeArgs], { cwd: zulipRepoDir() });
}

function run(program, programArgs, options = {}) {
  const result = spawnSync(program, programArgs, {
    cwd: options.cwd ?? repoRoot,
    stdio: 'inherit',
    env: options.env ?? process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

async function runScenario() {
  const scenarioPath = valueAfter('--scenario') ?? defaultScenarioPath();
  const reportDir = valueAfter('--report-dir') ?? `.local/makosh-lab/reports/${provider}`;
  const scenario = JSON.parse(await readFile(path.resolve(repoRoot, scenarioPath), 'utf8'));
  const startedAt = new Date().toISOString();
  const runId = safeRunId(`${scenario.scenario_id}-${startedAt}`);
  const labCorrelationId = `${scenario.correlation_id_prefix ?? scenario.scenario_id}-${runId}`;
  const report = {
    run_id: runId,
    scenario_id: scenario.scenario_id,
    provider: scenario.provider,
    lab_correlation_id: labCorrelationId,
    started_at: startedAt,
    finished_at: null,
    status: execute ? 'running' : 'dry_run',
    provider_actions: [],
    provider_events: [],
    expected_stages: scenario.expect ?? [],
    backend_expected_stages: scenario.backend_expected_stages ?? [],
    backend_observed_stages: [],
    capability_contract: scenario.capability_contract ?? [],
    backend_validation: scenario.backend_validation ?? {
      status: 'not_run_by_lab',
      command: 'cargo test --manifest-path backend/Cargo.toml --test zulip',
    },
    lab_stack: null,
    observed_stages: [],
    failures: [],
  };

  if (scenario.provider !== 'zulip') {
    throw new Error(`Unsupported provider ${scenario.provider}; expected zulip.`);
  }

  labLog(
    `scenario=${scenario.scenario_id} execute=${execute ? 'yes' : 'no'} testcontainers=${useTestcontainers ? 'yes' : 'no'} backend=${backendEvidence ? 'yes' : 'no'}`
  );

  if (!execute) {
    report.provider_actions = scenario.actions.map((scenarioAction) => ({
      ...reportableScenarioAction(renderScenarioAction(scenarioAction, labCorrelationId)),
      dry_run: true,
    }));
    report.finished_at = new Date().toISOString();
    await writeReport(reportDir, report);
    console.log(`Dry run report written: ${path.join(reportDir, `${report.run_id}.json`)}`);
    return;
  }

  try {
    if (useTestcontainers) {
      await runScenarioWithTestcontainers({
        scenario,
        report,
        reportDir,
        labCorrelationId,
      });
    } else {
      await runExecutedScenarioAgainstZulip({
        scenario,
        report,
        reportDir,
        labCorrelationId,
        providerEnv: process.env,
      });
    }
    if (backendEvidence) {
      await runBackendEvidence({
        scenario,
        report,
        reportDir,
      });
    }
  } catch (error) {
    report.status = 'failed';
    report.finished_at = new Date().toISOString();
    report.failures.push({
      kind: 'scenario_execution_error',
      message: safeErrorMessage(error),
    });
    await writeReport(reportDir, report);
    throw error;
  }
}

async function runScenarioWithTestcontainers({ scenario, report, reportDir, labCorrelationId }) {
  let stack = null;
  try {
    stack = await startZulipTestcontainers(report.run_id);
    report.lab_stack = {
      kind: 'zulip_testcontainers',
      base_url: stack.baseUrl,
      stream_name: stack.realm.stream_name,
      owner_email: stack.realm.owner_email,
      owner_user_id: stack.realm.owner_user_id,
      bot_email: stack.realm.bot_email,
      bot_user_id: stack.realm.bot_user_id,
      human_email: stack.realm.human_email,
      human_user_id: stack.realm.human_user_id,
      docker_compose_project: stack.projectName,
    };

    await runExecutedScenarioAgainstZulip({
      scenario,
      report,
      reportDir,
      labCorrelationId,
      providerEnv: {
        ...process.env,
        ZULIP_BASE_URL: stack.baseUrl,
        ZULIP_EMAIL: stack.realm.bot_email,
        ZULIP_API_KEY: stack.realm.bot_api_key,
        ZULIP_DIRECT_RECIPIENT_USER_IDS: JSON.stringify([stack.realm.human_user_id]),
      },
    });
  } finally {
    if (stack) {
      stopZulipTestcontainers(stack);
    }
  }
}

async function runExecutedScenarioAgainstZulip({
  scenario,
  report,
  reportDir,
  labCorrelationId,
  providerEnv,
}) {
  const baseUrl = providerEnv.ZULIP_BASE_URL;
  const email = providerEnv.ZULIP_EMAIL;
  const apiKey = providerEnv.ZULIP_API_KEY;
  if (!baseUrl || !email || !apiKey) {
    throw new Error('ZULIP_BASE_URL, ZULIP_EMAIL and ZULIP_API_KEY must be set for --execute.');
  }

  labLog(`registering Zulip event queue at ${baseUrl}`);

  const registered = await zulipRequest(baseUrl, email, apiKey, 'POST', '/api/v1/register', {
    form: {
      event_types: JSON.stringify(
        scenario.event_types ?? ['message', 'reaction', 'update_message', 'delete_message']
      ),
    },
  });
  let lastEventId = registered.last_event_id ?? -1;
  labLog(`registered Zulip event queue last_event_id=${lastEventId}`);

  const scenarioState = {};
  for (const [index, scenarioAction] of scenario.actions.entries()) {
    const renderedAction = renderScenarioAction(scenarioAction, labCorrelationId);
    labLog(`executing action ${index + 1}/${scenario.actions.length}: ${renderedAction.kind}`);
    const response = await executeZulipScenarioAction(
      baseUrl,
      email,
      apiKey,
      renderedAction,
      scenarioState,
      providerEnv
    );
    report.provider_actions.push({ ...reportableScenarioAction(renderedAction), response });
    labLog(`action ${index + 1}/${scenario.actions.length} completed: ${renderedAction.kind}`);
  }

  await collectZulipScenarioEvents({
    baseUrl,
    email,
    apiKey,
    queueId: registered.queue_id,
    lastEventId,
    report,
  });

  report.observed_stages = [...new Set(report.observed_stages)];
  recordMissingExpectedStages(report);
  report.status = report.failures.length === 0 ? 'provider_observed' : 'failed';
  report.finished_at = new Date().toISOString();
  await writeReport(reportDir, report);
  console.log(`Zulip scenario report written: ${path.join(reportDir, `${report.run_id}.json`)}`);
  if (report.failures.length > 0) {
    throw new Error(`Zulip scenario failed with ${report.failures.length} failure(s).`);
  }
}

async function collectZulipScenarioEvents({ baseUrl, email, apiKey, queueId, lastEventId, report }) {
  let cursor = lastEventId;
  const maxAttempts = 15;
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const eventsResponse = await zulipRequest(baseUrl, email, apiKey, 'GET', '/api/v1/events', {
      query: {
        queue_id: queueId,
        last_event_id: cursor,
        dont_block: 'true',
      },
    });
    const events = eventsResponse.events ?? [];
    for (const event of events) {
      cursor = Math.max(cursor, event.id ?? cursor);
      report.provider_events.push(event);
      report.observed_stages.push(zulipRawSignalEventType(event.type));
    }

    const observed = [...new Set(report.observed_stages)];
    labLog(
      `event poll ${attempt}/${maxAttempts}: fetched=${events.length} observed=${observed.length > 0 ? observed.join(',') : 'none'}`
    );
    if (expectedStagesObserved(report)) {
      return;
    }
    await delay(1000);
  }
}

async function runBackendEvidence({ scenario, report, reportDir }) {
  const backendReportDir = path.resolve(repoRoot, reportDir, 'backend');
  await mkdir(backendReportDir, { recursive: true });

  const commandArgs = [
    'test',
    '--manifest-path',
    'backend/Cargo.toml',
    '--test',
    'zulip_live',
    '--',
    '--ignored',
    '--nocapture',
  ];
  const command = `cargo ${commandArgs.join(' ')}`;
  report.backend_validation = {
    status: 'running',
    command,
    evidence_report_dir: path.relative(repoRoot, backendReportDir),
  };
  await writeReport(reportDir, report);

  labLog(`running backend Zulip live evidence harness: ${command}`);
  await checkedRunWithHeartbeat('cargo', commandArgs, {
    cwd: repoRoot,
    env: {
      ...process.env,
      MAKOSH_ZULIP_TESTCONTAINERS: '1',
      MAKOSH_ZULIP_START_TIMEOUT_SECS: process.env.MAKOSH_ZULIP_START_TIMEOUT_SECS ?? '900',
      MAKOSH_ZULIP_LIVE_REPORT_DIR: backendReportDir,
      MAKOSH_ZULIP_LIVE_SCENARIO_ID: scenario.scenario_id,
    },
    label: 'backend Zulip live evidence harness',
    intervalMs: 30_000,
  });

  const backendReport = await latestBackendEvidenceReport(backendReportDir);
  report.backend_observed_stages = backendReport.payload.observed_stages ?? [];
  report.backend_validation = {
    status: 'passed',
    command,
    evidence_report: path.relative(repoRoot, backendReport.path),
    harness: backendReport.payload.harness ?? 'zulip_live',
    finished_at: new Date().toISOString(),
  };
  recordMissingBackendExpectedStages(report);
  report.status = report.failures.length === 0 ? 'provider_and_backend_observed' : 'failed';
  report.finished_at = new Date().toISOString();
  await writeReport(reportDir, report);
  if (report.failures.length > 0) {
    throw new Error(`Zulip backend evidence failed with ${report.failures.length} failure(s).`);
  }
  labLog(`backend Zulip live evidence report: ${report.backend_validation.evidence_report}`);
}

async function latestBackendEvidenceReport(absoluteDir) {
  const entries = await readdir(absoluteDir);
  const candidates = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const absolutePath = path.join(absoluteDir, entry);
    candidates.push({
      path: absolutePath,
      stat: await stat(absolutePath),
    });
  }
  candidates.sort((left, right) => right.stat.mtimeMs - left.stat.mtimeMs);
  const latest = candidates[0];
  if (!latest) {
    throw new Error(`Backend Zulip live evidence did not write a JSON report in ${absoluteDir}`);
  }
  return {
    path: latest.path,
    payload: JSON.parse(await readFile(latest.path, 'utf8')),
  };
}

function recordMissingExpectedStages(report) {
  const observed = new Set(report.observed_stages);
  for (const stage of report.expected_stages) {
    if (!observed.has(stage)) {
      report.failures.push({
        kind: 'missing_provider_stage',
        stage,
      });
    }
  }
}

function recordMissingBackendExpectedStages(report) {
  const observed = new Set(report.backend_observed_stages);
  for (const stage of report.backend_expected_stages) {
    if (!observed.has(stage)) {
      report.failures.push({
        kind: 'missing_backend_stage',
        stage,
      });
    }
  }
}

function expectedStagesObserved(report) {
  const observed = new Set(report.observed_stages);
  return report.expected_stages.every((stage) => observed.has(stage));
}

function renderScenarioAction(scenarioAction, labCorrelationId) {
  const rendered = { ...scenarioAction };
  for (const key of ['content', 'updated_content', 'file_content']) {
    if (typeof rendered[key] === 'string') {
      rendered[key] = renderLabText(rendered[key], labCorrelationId);
    }
  }
  return rendered;
}

function renderLabText(value, labCorrelationId) {
  return typeof value === 'string'
    ? value.replaceAll('{{correlation_id}}', labCorrelationId)
    : value;
}

function reportableScenarioAction(action) {
  const reportable = { ...action };
  if (typeof reportable.file_content === 'string') {
    reportable.file_content_size_bytes = Buffer.byteLength(reportable.file_content);
    reportable.file_content_sha256 = createHash('sha256')
      .update(reportable.file_content)
      .digest('hex');
    delete reportable.file_content;
  }
  return reportable;
}

async function executeZulipScenarioAction(baseUrl, email, apiKey, action, state, providerEnv) {
  switch (action.kind) {
    case 'send_stream_message': {
      const response = await zulipRequest(baseUrl, email, apiKey, 'POST', '/api/v1/messages', {
        form: {
          type: 'stream',
          to: action.stream,
          topic: action.topic,
          content: action.content,
        },
      });
      if (response.id) {
        state.last_message_id = response.id;
      }
      return response;
    }
    case 'send_direct_message': {
      const response = await zulipRequest(baseUrl, email, apiKey, 'POST', '/api/v1/messages', {
        form: {
          type: 'direct',
          to: JSON.stringify(zulipDirectRecipients(action, providerEnv)),
          content: action.content,
        },
      });
      if (response.id) {
        state.last_message_id = response.id;
      }
      return response;
    }
    case 'send_stream_message_with_upload': {
      const upload = await zulipUploadFile(baseUrl, email, apiKey, action);
      if (upload.uri) {
        state.last_upload_uri = upload.uri;
      }
      const content = contentWithUploadUri(action.content, upload.uri);
      const message = await zulipRequest(baseUrl, email, apiKey, 'POST', '/api/v1/messages', {
        form: {
          type: 'stream',
          to: action.stream,
          topic: action.topic,
          content,
        },
      });
      if (message.id) {
        state.last_message_id = message.id;
      }
      return {
        result: message.result ?? upload.result,
        msg: message.msg ?? upload.msg ?? '',
        upload_uri: upload.uri,
        provider_message_id: message.id,
      };
    }
    case 'add_reaction':
      return zulipRequest(
        baseUrl,
        email,
        apiKey,
        'POST',
        `/api/v1/messages/${resolveScenarioMessageId(action, state)}/reactions`,
        { form: zulipReactionForm(action) }
      );
    case 'remove_reaction':
      return zulipRequest(
        baseUrl,
        email,
        apiKey,
        'DELETE',
        `/api/v1/messages/${resolveScenarioMessageId(action, state)}/reactions`,
        { form: zulipReactionForm(action) }
      );
    case 'update_message':
      return zulipRequest(
        baseUrl,
        email,
        apiKey,
        'PATCH',
        `/api/v1/messages/${resolveScenarioMessageId(action, state)}`,
        {
          form: compactForm({
            content: action.updated_content,
            topic: action.topic,
            stream_id: action.stream_id,
            propagate_mode: action.propagate_mode,
          }),
        }
      );
    case 'delete_message':
      return zulipRequest(
        baseUrl,
        email,
        apiKey,
        'DELETE',
        `/api/v1/messages/${resolveScenarioMessageId(action, state)}`
      );
    case 'upload_file': {
      const response = await zulipUploadFile(baseUrl, email, apiKey, action);
      if (response.uri) {
        state.last_upload_uri = response.uri;
      }
      return response;
    }
    case 'download_user_upload':
      return zulipDownloadUserUpload(baseUrl, email, apiKey, resolveScenarioUploadUri(action, state));
    default:
      throw new Error(`Unsupported action kind ${action.kind}`);
  }
}

function zulipDirectRecipients(action, providerEnv) {
  if (!action.recipient_user_ids_env) {
    return action.recipients;
  }

  const raw = providerEnv[action.recipient_user_ids_env];
  if (!raw || raw.trim() === '') {
    throw new Error(`${action.recipient_user_ids_env} must be set for send_direct_message`);
  }
  const values = raw.trim().startsWith('[')
    ? JSON.parse(raw)
    : raw.split(',').map((value) => value.trim()).filter(Boolean);
  if (!Array.isArray(values) || values.length === 0) {
    throw new Error(`${action.recipient_user_ids_env} must contain a non-empty user id array`);
  }
  return values.map((value) => {
    const id = typeof value === 'number' ? value : Number.parseInt(String(value), 10);
    if (!Number.isSafeInteger(id) || id <= 0) {
      throw new Error(`${action.recipient_user_ids_env} contains invalid Zulip user id ${value}`);
    }
    return id;
  });
}

function resolveScenarioMessageId(action, state) {
  if (action.message_id) return action.message_id;
  if (action.message_id_ref === 'last' && state.last_message_id) return state.last_message_id;
  throw new Error(`${action.kind} requires message_id or message_id_ref=last after send_stream_message`);
}

function resolveScenarioUploadUri(action, state) {
  if (action.upload_uri) return action.upload_uri;
  if (action.upload_uri_ref === 'last' && state.last_upload_uri) return state.last_upload_uri;
  throw new Error(`${action.kind} requires upload_uri or upload_uri_ref=last after an upload action`);
}

function zulipReactionForm(action) {
  return compactForm({
    emoji_name: action.emoji_name,
    emoji_code: action.emoji_code,
    reaction_type: action.reaction_type,
  });
}

function compactForm(values) {
  return Object.fromEntries(
    Object.entries(values).filter(([, value]) => value !== undefined && value !== null && value !== '')
  );
}

async function startZulipTestcontainers(runId) {
  if (!commandExists('docker')) {
    throw new Error('docker is required for Макошь Lab --testcontainers.');
  }

  const projectName = `makosh-lab-zulip-${createHash('sha256').update(runId).digest('hex').slice(0, 12)}`;
  const composeDir = path.join(repoRoot, 'testing/zulip');
  const env = zulipLabComposeEnv(projectName);
  const partialStack = { projectName, composeDir, env };

  try {
    labLog(`starting local Zulip fixture project=${projectName}`);
    await checkedRunWithHeartbeat('docker', dockerComposeArgs(projectName, ['up', '-d']), {
      cwd: composeDir,
      env,
      label: `docker compose up for local Zulip fixture ${projectName}`,
      intervalMs: 15_000,
    });

    const port = captureChecked('docker', dockerComposeArgs(projectName, ['port', 'proxy', '8080']), {
      cwd: composeDir,
      env,
    }).stdout;
    const baseUrl = zulipBaseUrlFromPortOutput(port);
    labLog(`local Zulip proxy mapped to ${baseUrl}`);

    await waitForZulipLabHttp(baseUrl, zulipStartTimeoutMs());
    const realm = await provisionZulipLabRealm({ projectName, composeDir, env, baseUrl });
    return { ...partialStack, baseUrl, realm };
  } catch (error) {
    stopZulipTestcontainers(partialStack);
    throw error;
  }
}

function stopZulipTestcontainers(stack) {
  labLog(`cleaning local Zulip fixture project=${stack.projectName}`);
  const result = spawnSync(
    'docker',
    dockerComposeArgs(stack.projectName, ['down', '-v', '--remove-orphans']),
    {
      cwd: stack.composeDir,
      env: stack.env,
      stdio: 'inherit',
    }
  );
  if (result.status !== 0) {
    console.error(`[makosh-lab] cleanup failed for project=${stack.projectName}`);
  } else {
    labLog(`local Zulip fixture cleaned project=${stack.projectName}`);
  }
}

function dockerComposeArgs(projectName, composeArgs) {
  return ['compose', '-p', projectName, '-f', 'compose.testcontainers.yml', ...composeArgs];
}

function zulipLabComposeEnv(sessionId) {
  return {
    ...process.env,
    MAKOSH_TEST_SESSION_ID: sessionId,
    ZULIP__POSTGRES_PASSWORD: labCredential('postgres'),
    ZULIP__MEMCACHED_PASSWORD: labCredential('memcached'),
    ZULIP__RABBITMQ_PASSWORD: labCredential('rabbitmq'),
    ZULIP__REDIS_PASSWORD: labCredential('redis'),
    ZULIP__SECRET_KEY: labCredential('zulip-secret-key', 64),
    ZULIP__EMAIL_PASSWORD: labCredential('email'),
  };
}

function checkedRun(program, programArgs, options = {}) {
  const result = spawnSync(program, programArgs, {
    cwd: options.cwd ?? repoRoot,
    stdio: 'inherit',
    env: options.env ?? process.env,
  });
  if (result.status !== 0) {
    throw new Error(`${program} ${programArgs.join(' ')} failed with status ${result.status}`);
  }
}

async function checkedRunWithHeartbeat(program, programArgs, options = {}) {
  const startedAt = Date.now();
  const label = options.label ?? `${program} ${programArgs.join(' ')}`;
  const intervalMs = options.intervalMs ?? 30_000;
  labLog(`starting ${label}`);

  const child = spawn(program, programArgs, {
    cwd: options.cwd ?? repoRoot,
    stdio: 'inherit',
    env: options.env ?? process.env,
  });
  const heartbeat = setInterval(() => {
    labLog(`${label} still running; elapsed=${Math.round((Date.now() - startedAt) / 1000)}s`);
  }, intervalMs);

  try {
    const result = await new Promise((resolve, reject) => {
      child.on('error', reject);
      child.on('close', (code, signal) => resolve({ code, signal }));
    });
    if (result.code !== 0) {
      const suffix = result.signal ? `signal ${result.signal}` : `status ${result.code}`;
      throw new Error(`${program} ${programArgs.join(' ')} failed with ${suffix}`);
    }
    labLog(`${label} finished in ${Math.round((Date.now() - startedAt) / 1000)}s`);
  } finally {
    clearInterval(heartbeat);
  }
}

function captureChecked(program, programArgs, options = {}) {
  const result = spawnSync(program, programArgs, {
    cwd: options.cwd ?? repoRoot,
    env: options.env ?? process.env,
    encoding: 'utf8',
    maxBuffer: 20 * 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(
      `${program} ${programArgs.join(' ')} failed with status ${result.status}: ${redactProvisioningOutput(
        `${result.stdout ?? ''}\n${result.stderr ?? ''}`
      )}`
    );
  }
  return {
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

function zulipBaseUrlFromPortOutput(output) {
  const line = output.split(/\r?\n/).map((value) => value.trim()).filter(Boolean).at(-1);
  const match = line?.match(/:(\d+)$/);
  if (!match) {
    throw new Error(`Unable to parse docker compose proxy port from output: ${output}`);
  }
  return `http://127.0.0.1:${match[1]}`;
}

function zulipStartTimeoutMs() {
  const seconds = Number.parseInt(process.env.MAKOSH_ZULIP_START_TIMEOUT_SECS ?? '600', 10);
  return (Number.isSafeInteger(seconds) && seconds > 0 ? seconds : 600) * 1000;
}

async function waitForZulipLabHttp(baseUrl, timeoutMs) {
  const readinessUrl = new URL('/api/v1/server_settings', baseUrl).toString();
  const deadline = Date.now() + timeoutMs;
  let nextLogAt = 0;
  let lastError = 'not attempted yet';

  while (Date.now() < deadline) {
    try {
      const response = await fetch(readinessUrl, { redirect: 'follow' });
      if (response.ok || (response.status >= 300 && response.status < 400)) {
        labLog(`Zulip API readiness passed at ${readinessUrl}`);
        return;
      }
      lastError = `HTTP ${response.status}`;
    } catch (error) {
      lastError = safeErrorMessage(error);
    }

    if (Date.now() >= nextLogAt) {
      labLog(`waiting for Zulip API readiness at ${readinessUrl}; last=${lastError}`);
      nextLogAt = Date.now() + 15_000;
    }
    await delay(2000);
  }

  throw new Error(`Zulip did not become ready within ${Math.round(timeoutMs / 1000)}s: ${lastError}`);
}

async function provisionZulipLabRealm({ projectName, composeDir, env, baseUrl }) {
  labLog('provisioning Zulip realm, bot, human user and stream');
  const script = await readFile(path.join(repoRoot, 'testing/zulip/provision-test-realm.sh'), 'utf8');
  const provisionEnv = {
    MAKOSH_OWNER_PASSWORD: labCredential('owner'),
    MAKOSH_REALM_NAME: 'Макошь Test',
    MAKOSH_STREAM_NAME: process.env.MAKOSH_LAB_ZULIP_STREAM ?? 'Макошь Lab',
    MAKOSH_OWNER_EMAIL: 'owner@example.com',
    MAKOSH_OWNER_NAME: 'Макошь Owner',
    MAKOSH_HUMAN_EMAIL: 'alice@example.com',
    MAKOSH_HUMAN_NAME: 'Alice Example',
    MAKOSH_BOT_EMAIL: 'makosh-bot@example.com',
    MAKOSH_BOT_NAME: 'Макошь Bot',
  };
  const args = dockerComposeArgs(projectName, [
    'exec',
    '-T',
    ...Object.entries(provisionEnv).flatMap(([key, value]) => ['-e', `${key}=${value}`]),
    'zulip',
    'sh',
    '-lc',
    script,
  ]);
  const result = spawnSync('docker', args, {
    cwd: composeDir,
    env,
    encoding: 'utf8',
    maxBuffer: 20 * 1024 * 1024,
  });
  const stdout = result.stdout ?? '';
  const stderr = result.stderr ?? '';
  if (result.status !== 0) {
    throw new Error(
      `Zulip provisioning failed with status ${result.status}: ${redactProvisioningOutput(stderr)}`
    );
  }

  const payloadLine = stdout
    .split(/\r?\n/)
    .find((line) => line.startsWith(provisionPrefix));
  if (!payloadLine) {
    throw new Error('Zulip provisioning did not return credentials payload');
  }
  const realm = JSON.parse(payloadLine.slice(provisionPrefix.length));
  labLog(
    `provisioned Zulip realm stream=${realm.stream_name} owner=${realm.owner_email} bot=${realm.bot_email} human=${realm.human_email}`
  );
  return { ...realm, base_url: baseUrl };
}

function labCredential(seed, length = 40) {
  return createHash('sha256')
    .update(`makosh-lab-zulip:${seed}`)
    .digest('hex')
    .slice(0, length);
}

function labLog(message) {
  console.error(`[makosh-lab] ${new Date().toISOString()} ${message}`);
}

function redactProvisioningOutput(output) {
  return String(output)
    .split(/\r?\n/)
    .filter((line) => !line.includes(provisionPrefix))
    .join('\n')
    .replace(/Basic [A-Za-z0-9+/=]+/g, 'Basic [redacted]');
}

function safeErrorMessage(error) {
  return redactProvisioningOutput(error?.message ?? error);
}

function safeRunId(value) {
  return value.replace(/[^a-zA-Z0-9_.-]+/g, '-').replace(/^-+|-+$/g, '');
}

function authHeader(email, apiKey) {
  return `Basic ${Buffer.from(`${email}:${apiKey}`).toString('base64')}`;
}

function contentWithUploadUri(content, uploadUri) {
  if (!uploadUri) {
    throw new Error('send_stream_message_with_upload requires Zulip upload response uri');
  }
  return `${String(content ?? '').trim()}\n${String(uploadUri).trim()}`;
}

function zulipRawSignalEventType(providerEventType) {
  switch (providerEventType) {
    case 'message':
      return 'signal.raw.zulip.message.observed';
    case 'reaction':
      return 'signal.raw.zulip.reaction.observed';
    case 'update_message':
      return 'signal.raw.zulip.message_update.observed';
    case 'delete_message':
      return 'signal.raw.zulip.message_delete.observed';
    default:
      return 'signal.raw.zulip.unknown.observed';
  }
}

async function zulipRequest(baseUrl, email, apiKey, method, apiPath, { form, query } = {}) {
  const url = new URL(apiPath, baseUrl);
  if (query) {
    for (const [key, value] of Object.entries(query)) {
      url.searchParams.set(key, String(value));
    }
  }
  const headers = { Authorization: authHeader(email, apiKey) };
  let body;
  if (form) {
    body = new URLSearchParams(form);
    headers['Content-Type'] = 'application/x-www-form-urlencoded';
  }
  const response = await fetch(url, { method, headers, body });
  const text = await response.text();
  let payload;
  try {
    payload = JSON.parse(text);
  } catch {
    payload = { raw: text };
  }
  if (!response.ok) {
    throw new Error(`Zulip ${method} ${apiPath} failed with ${response.status}: ${text}`);
  }
  return payload;
}

async function zulipUploadFile(baseUrl, email, apiKey, action) {
  const filename = action.filename;
  if (!filename) {
    throw new Error('upload_file requires filename');
  }
  const content = action.file_content ?? '';
  const url = new URL('/api/v1/user_uploads', baseUrl);
  const body = new FormData();
  body.append('file', new Blob([content]), filename);
  const response = await fetch(url, {
    method: 'POST',
    headers: { Authorization: authHeader(email, apiKey) },
    body,
  });
  const text = await response.text();
  let payload;
  try {
    payload = JSON.parse(text);
  } catch {
    payload = { raw: text };
  }
  if (!response.ok) {
    throw new Error(`Zulip POST /api/v1/user_uploads failed with ${response.status}: ${text}`);
  }
  return payload;
}

async function zulipDownloadUserUpload(baseUrl, email, apiKey, uploadUri) {
  const url = zulipUserUploadUrl(baseUrl, uploadUri);
  const response = await fetch(url, {
    method: 'GET',
    headers: { Authorization: authHeader(email, apiKey) },
  });
  const bytes = Buffer.from(await response.arrayBuffer());
  if (!response.ok) {
    throw new Error(`Zulip GET ${url.pathname} failed with ${response.status}: ${bytes.toString('utf8')}`);
  }
  return {
    result: 'success',
    upload_uri: uploadUri,
    content_type: response.headers.get('content-type') ?? null,
    size_bytes: bytes.length,
    sha256: createHash('sha256').update(bytes).digest('hex'),
  };
}

function zulipUserUploadUrl(baseUrl, uploadUri) {
  const base = new URL(baseUrl);
  const url = new URL(uploadUri, base);
  if (
    url.protocol !== base.protocol ||
    url.hostname !== base.hostname ||
    effectivePort(url) !== effectivePort(base) ||
    !(url.pathname === '/user_uploads' || url.pathname.startsWith('/user_uploads/'))
  ) {
    throw new Error('Zulip upload URL must be same-realm /user_uploads path');
  }
  return url;
}

function effectivePort(url) {
  if (url.port) return url.port;
  if (url.protocol === 'http:') return '80';
  if (url.protocol === 'https:') return '443';
  return '';
}

async function writeReport(relativeDir, report) {
  const absoluteDir = path.resolve(repoRoot, relativeDir);
  await mkdir(absoluteDir, { recursive: true });
  await writeFile(path.join(absoluteDir, `${report.run_id}.json`), `${JSON.stringify(report, null, 2)}\n`);
}
