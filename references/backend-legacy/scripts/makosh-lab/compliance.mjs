import { spawnSync } from 'node:child_process';
import { access, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';

const suiteVersion = 1;

const passStatuses = new Set([
  'backend_test_pass',
  'fixture_pass',
  'lab_pass',
  'live_pass',
  'source_check_pass',
]);

const scenarioPassStatuses = new Set(['provider_observed', 'provider_and_backend_observed']);

const scenarioPaths = [
  'testing/makosh-lab/scenarios/zulip/message-to-task-candidate.json',
  'testing/makosh-lab/scenarios/zulip/attachment-materialization.json',
  'testing/makosh-lab/scenarios/zulip/direct-message.json',
];

const backendStageRequirements = {
  'receive.message.recorded': ['communication.message.recorded'],
  'receive.direct_message.recorded': [
    'communication.message.recorded',
    'zulip.direct_conversation',
  ],
  'mutation.reaction.materialized': ['communication_message_reactions'],
  'mutation.edit.materialized': [
    'communication.message.updated',
    'communication_message_versions',
  ],
  'mutation.delete.materialized': ['communication_message_tombstones'],
  'review.task_candidate.detected': ['task_candidates', 'review_items'],
  'provider_api.send_stream_direct': [
    'provider_api.stream_message_sent',
    'provider_api.direct_message_sent',
  ],
  'provider_api.send_direct': ['provider_api.direct_message_sent'],
  'provider_api.edit_delete_reaction': [
    'provider_api.reaction_added',
    'provider_api.reaction_removed',
    'provider_api.message_updated',
    'provider_api.message_deleted',
  ],
  'provider_api.upload_file': ['provider_api.file_uploaded'],
  'provider_api.download_user_upload': ['provider_api.user_upload_downloaded'],
  'provider_command.executes_stream_send': [
    'provider_command.completed',
    'signal.accepted.zulip.message',
    'zulip.command.reconciled',
  ],
  'media.attachment.materialized': [
    'communication_attachments',
    'attachment_state.materialized',
  ],
  'event_ingest.queue_reregistration': ['zulip_event_queue.reregistered'],
  'provider_observation.reconciled': ['zulip.command.reconciled'],
  'signal_hub.source_and_raw_dispatch': [
    'signal.raw.zulip.message.observed',
    'signal.raw.zulip.reaction.observed',
    'signal.raw.zulip.message_update.observed',
    'signal.raw.zulip.message_delete.observed',
  ],
};

const requiredZulipCapabilities = [
  {
    name: 'account_setup.hostvault_secret_boundary',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_account_setup_stores_api_key_in_hostvault_without_postgres_secret_payload',
    note: 'Proves the API key stays outside PostgreSQL payload surfaces.',
  },
  {
    name: 'signal_hub.source_and_raw_dispatch',
    status: 'backend_test_required',
    evidence: 'backend/tests/signal_hub.rs and backend/tests/signal_hub_api.rs',
    note: 'Proves Zulip is registered as a Signal Hub source and emits canonical raw signal names.',
  },
  {
    name: 'event_ingest.replay_idempotency',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_event_ingest_persists_checkpoint_and_is_replay_idempotent',
    note: 'Prevents repeated queue polling from duplicating raw or accepted Signal Hub events.',
  },
  {
    name: 'event_ingest.queue_reregistration',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_event_ingest_reregisters_when_checkpoint_queue_expired',
    note: 'The live harness also covers the real provider BAD_EVENT_QUEUE_ID path.',
  },
  {
    name: 'event_ingest.scheduler_ready',
    status: 'source_check_required',
    evidence: 'backend/src/application/bootstrap.rs::start_zulip_event_ingest',
  },
  {
    name: 'provider_command.durable_retry_dead_letter',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_provider_commands_are_durable_idempotent_and_retryable',
  },
  {
    name: 'provider_command.failure_redaction',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_command_worker_sanitizes_provider_errors_before_retrying',
  },
  {
    name: 'provider_observation.reconciled',
    status: 'backend_test_required',
    evidence: 'backend/tests/zulip.rs::zulip_completed_provider_command_is_reconciled_after_matching_accepted_observation',
  },
  {
    name: 'identity.zulip_person_trace',
    status: 'backend_test_required',
    evidence: 'backend/tests/consistency_contradiction/refresh_provider_messages.rs::contradiction_refresh_detects_zulip_message_claim_against_active_person_fact_without_overwriting_memory',
  },
  {
    name: 'polygraph.structured_fact_contradiction',
    status: 'backend_test_required',
    evidence: 'backend/tests/consistency_contradiction/refresh_provider_messages.rs::contradiction_refresh_detects_zulip_message_claim_against_active_person_fact_without_overwriting_memory',
  },
  {
    name: 'review.multilingual_deterministic_candidates',
    status: 'backend_test_required',
    evidence: 'backend/tests/task_candidates/refresh.rs::task_candidate_refresh_detects_multilingual_message_actions_against_postgres',
  },
  {
    name: 'intelligence.semantic_multilingual_freeform',
    status: 'backend_test_required',
    evidence: 'backend/tests/task_candidates/refresh.rs::task_candidate_refresh_detects_freeform_multilingual_message_requests_against_postgres',
    note: 'Provider-neutral deterministic free-form request extraction for English, Russian, Spanish, French and German; not a claim of open-ended LLM extraction.',
  },
];

const backendContractRuns = [
  {
    id: 'zulip_backend_contract',
    command: [
      'cargo',
      'run',
      '--manifest-path',
      'crates/testkit/Cargo.toml',
      '--bin',
      'makosh_test_session',
      '--',
      'cargo',
      'nextest',
      'run',
      '--manifest-path',
      'backend/Cargo.toml',
      '--profile',
      'default',
      '--show-progress',
      'none',
      '--test-threads',
      '1',
      '--test',
      'zulip',
    ],
    capabilities: [
      'account_setup.hostvault_secret_boundary',
      'event_ingest.replay_idempotency',
      'provider_command.durable_lifecycle',
      'provider_command.durable_retry_dead_letter',
      'provider_command.failure_redaction',
      'provider_observation.reconciled',
    ],
  },
  {
    id: 'zulip_polygraph_contract',
    command: [
      'cargo',
      'run',
      '--manifest-path',
      'crates/testkit/Cargo.toml',
      '--bin',
      'makosh_test_session',
      '--',
      'cargo',
      'nextest',
      'run',
      '--manifest-path',
      'backend/Cargo.toml',
      '--profile',
      'default',
      '--show-progress',
      'none',
      '--test-threads',
      '1',
      '--test',
      'consistency_contradiction',
      'contradiction_refresh_detects_zulip_message_claim_against_active_person_fact_without_overwriting_memory',
    ],
    capabilities: [
      'identity.zulip_person_trace',
      'polygraph.structured_fact_contradiction',
    ],
  },
  {
    id: 'zulip_multilingual_task_candidate_contract',
    command: [
      'cargo',
      'run',
      '--manifest-path',
      'crates/testkit/Cargo.toml',
      '--bin',
      'makosh_test_session',
      '--',
      'cargo',
      'nextest',
      'run',
      '--manifest-path',
      'backend/Cargo.toml',
      '--profile',
      'default',
      '--show-progress',
      'none',
      '--test-threads',
      '1',
      '--test',
      'task_candidates',
      'task_candidate_refresh_detects_multilingual_message_actions_against_postgres',
    ],
    capabilities: ['review.multilingual_deterministic_candidates'],
  },
  {
    id: 'zulip_freeform_multilingual_task_candidate_contract',
    command: [
      'cargo',
      'run',
      '--manifest-path',
      'crates/testkit/Cargo.toml',
      '--bin',
      'makosh_test_session',
      '--',
      'cargo',
      'nextest',
      'run',
      '--manifest-path',
      'backend/Cargo.toml',
      '--profile',
      'default',
      '--show-progress',
      'none',
      '--test-threads',
      '1',
      '--test',
      'task_candidates',
      'task_candidate_refresh_detects_freeform_multilingual_message_requests_against_postgres',
    ],
    capabilities: ['intelligence.semantic_multilingual_freeform'],
  },
];

export async function runCompliance({
  repoRoot,
  provider,
  reportDir,
  backendEvidence = false,
  requireClosed = false,
  log = console.error,
}) {
  if (provider !== 'zulip') {
    throw new Error(`Unsupported compliance provider ${provider}; supported provider: zulip.`);
  }

  const absoluteReportDir = path.resolve(repoRoot, reportDir);
  const backendContractEvidence = backendEvidence
    ? await runBackendContractEvidence({ repoRoot, absoluteReportDir, log })
    : await latestBackendContractEvidence(repoRoot, path.join(absoluteReportDir, 'compliance'));
  const scenarios = await loadScenarios(repoRoot);
  const scenarioReports = await latestScenarioReports(repoRoot, absoluteReportDir);
  const backendReports = await latestBackendReports(repoRoot, path.join(absoluteReportDir, 'backend'));
  const backendStages = new Set([
    ...backendReports.flatMap((report) => report.payload.observed_stages ?? []),
    ...Array.from(scenarioReports.values()).flatMap(
      (report) => report.payload.backend_observed_stages ?? []
    ),
  ]);
  const backendContractPasses = backendContractCapabilityPasses(backendContractEvidence);

  const capabilities = new Map();
  for (const scenario of scenarios) {
    for (const capability of scenario.capability_contract ?? []) {
      addCapability(capabilities, {
        ...capability,
        scenario_id: scenario.scenario_id,
        scenario_path: scenario.path,
      });
    }
  }
  for (const capability of requiredZulipCapabilities) {
    addCapability(capabilities, capability);
  }

  const resolved = [];
  for (const capability of capabilities.values()) {
    resolved.push(
      await resolveCapability({
        repoRoot,
        capability,
        scenarioReports,
        backendReports,
        backendStages,
        backendContractPasses,
      })
    );
  }
  resolved.sort((left, right) => left.name.localeCompare(right.name));

  const generatedAt = new Date().toISOString();
  const report = {
    provider,
    suite_version: suiteVersion,
    generated_at: generatedAt,
    source: 'make makosh-lab ACTION=compliance PROVIDER=zulip',
    report_dir: path.relative(repoRoot, absoluteReportDir),
    reports_used: {
      scenarios: Array.from(scenarioReports.values()).map((item) => item.relative_path),
      backend: backendReports.map((item) => item.relative_path),
      backend_contracts: backendContractEvidence.map((item) => item.relative_path),
    },
    summary: summarize(resolved),
    capabilities: resolved,
  };

  const outputDir = path.join(absoluteReportDir, 'compliance');
  await mkdir(outputDir, { recursive: true });
  const outputPath = path.join(
    outputDir,
    `zulip-compliance-${safeTimestamp(generatedAt)}-${process.pid}.json`
  );
  await writeFile(outputPath, `${JSON.stringify(report, null, 2)}\n`);

  log(
    `compliance summary provider=zulip pass=${report.summary.pass} pending=${report.summary.pending} deferred=${report.summary.deferred} total=${report.summary.total}`
  );
  console.log(`Zulip compliance report written: ${path.relative(repoRoot, outputPath)}`);

  if (requireClosed && report.summary.pending > 0) {
    throw new Error(`Zulip compliance is not closed: ${report.summary.pending} pending capability(s).`);
  }

  return report;
}

async function loadScenarios(repoRoot) {
  const scenarios = [];
  for (const relativePath of scenarioPaths) {
    const payload = JSON.parse(await readFile(path.join(repoRoot, relativePath), 'utf8'));
    scenarios.push({ ...payload, path: relativePath });
  }
  return scenarios;
}

function addCapability(capabilities, capability) {
  const existing = capabilities.get(capability.name);
  if (!existing) {
    capabilities.set(capability.name, {
      name: capability.name,
      group: capability.group ?? groupFromName(capability.name),
      declared_statuses: [capability.status],
      evidence: [capability.evidence],
      scenarios: capability.scenario_id ? [capability.scenario_id] : [],
      scenario_paths: capability.scenario_path ? [capability.scenario_path] : [],
      notes: capability.note ? [capability.note] : [],
    });
    return;
  }

  pushUnique(existing.declared_statuses, capability.status);
  pushUnique(existing.evidence, capability.evidence);
  if (capability.scenario_id) pushUnique(existing.scenarios, capability.scenario_id);
  if (capability.scenario_path) pushUnique(existing.scenario_paths, capability.scenario_path);
  if (capability.note) pushUnique(existing.notes, capability.note);
}

async function resolveCapability({
  repoRoot,
  capability,
  scenarioReports,
  backendReports,
  backendStages,
  backendContractPasses,
}) {
  let status = mostConcreteDeclaredStatus(capability.declared_statuses);
  const evidence = [...capability.evidence];
  const trace = {};
  const missing = [];

  if (capability.scenarios.length > 0) {
    trace.scenario_ids = capability.scenarios;
  }

  if (capability.declared_statuses.includes('source_check_required')) {
    const source = await sourceCheck(repoRoot, capability.evidence);
    if (source.ok) {
      status = 'source_check_pass';
      evidence.push(...source.evidence);
    } else {
      status = 'source_check_required';
      missing.push(...source.evidence);
    }
  }

  if (capability.declared_statuses.includes('execute_required')) {
    const report = capability.scenarios
      .map((scenarioId) => scenarioReports.get(scenarioId))
      .find((candidate) => candidate && scenarioPassStatuses.has(candidate.payload.status));
    if (report) {
      status = 'lab_pass';
      evidence.push(report.relative_path);
      trace.lab_report = report.relative_path;
    } else {
      status = 'execute_required';
      missing.push('No passing Lab scenario report found for this capability.');
    }
  }

  if (capability.declared_statuses.includes('backend_test_required')) {
    const backendContract = backendContractPasses.get(capability.name);
    const requiredStages = backendStageRequirements[capability.name] ?? [];
    const absentStages = requiredStages.filter((stage) => !backendStages.has(stage));
    if (backendContract) {
      status = 'backend_test_pass';
      evidence.push(backendContract.relative_path, backendContract.command);
      trace.backend_contract_run = backendContract.id;
    } else if (requiredStages.length > 0 && absentStages.length === 0) {
      status = 'backend_test_pass';
      evidence.push(...backendReports.map((report) => report.relative_path));
      trace.backend_stages = requiredStages;
    } else if (requiredStages.length > 0) {
      status = 'backend_test_required';
      missing.push(`Missing backend observed stage(s): ${absentStages.join(', ')}`);
    } else if (!passStatuses.has(status)) {
      status = 'backend_test_required';
      missing.push('No machine-readable backend stage mapping for this capability yet.');
    }
  }

  if (status === 'deferred') {
    missing.push('Explicitly deferred; not part of current deterministic closure.');
  }

  return {
    name: capability.name,
    group: capability.group,
    status,
    confidence: passStatuses.has(status) ? 'high' : 'pending',
    evidence: [...new Set(evidence)],
    trace,
    replay_idempotency: replayNote(capability.name),
    missing,
    notes: capability.notes,
  };
}

async function runBackendContractEvidence({ repoRoot, absoluteReportDir, log }) {
  const startedAt = new Date().toISOString();
  const runs = [];
  const outputDir = path.join(absoluteReportDir, 'compliance');
  await mkdir(outputDir, { recursive: true });

  for (const contract of backendContractRuns) {
    log(`running backend compliance contract ${contract.id}: ${contract.command.join(' ')}`);
    const runStarted = Date.now();
    const result = spawnSync(contract.command[0], contract.command.slice(1), {
      cwd: repoRoot,
      env: process.env,
      stdio: 'inherit',
    });
    const run = {
      id: contract.id,
      command: contract.command.join(' '),
      capabilities: contract.capabilities,
      status: result.status === 0 ? 'passed' : 'failed',
      exit_code: result.status,
      duration_ms: Date.now() - runStarted,
      finished_at: new Date().toISOString(),
    };
    runs.push(run);
    if (result.status !== 0) {
      const failureReport = await writeBackendContractReport({
        repoRoot,
        outputDir,
        startedAt,
        status: 'failed',
        runs,
      });
      throw new Error(`Backend compliance contract ${contract.id} failed; report=${failureReport.relative_path}`);
    }
  }

  const report = await writeBackendContractReport({
    repoRoot,
    outputDir,
    startedAt,
    status: 'passed',
    runs,
  });
  return [report];
}

async function writeBackendContractReport({ repoRoot, outputDir, startedAt, status, runs }) {
  const finishedAt = new Date().toISOString();
  const outputPath = path.join(
    outputDir,
    `zulip-backend-contracts-${safeTimestamp(finishedAt)}-${process.pid}.json`
  );
  const payload = {
    provider: 'zulip',
    kind: 'backend_contract_evidence',
    status,
    started_at: startedAt,
    finished_at: finishedAt,
    runs,
  };
  await writeFile(outputPath, `${JSON.stringify(payload, null, 2)}\n`);
  return {
    absolute_path: outputPath,
    relative_path: path.relative(repoRoot, outputPath),
    payload,
    stat: await stat(outputPath),
  };
}

async function latestBackendContractEvidence(repoRoot, absoluteComplianceDir) {
  const entries = await safeReadDir(absoluteComplianceDir);
  const reports = [];
  for (const entry of entries) {
    if (!entry.startsWith('zulip-backend-contracts-') || !entry.endsWith('.json')) continue;
    const absolutePath = path.join(absoluteComplianceDir, entry);
    const payload = await readJsonOrNull(absolutePath);
    if (payload?.provider !== 'zulip' || payload.kind !== 'backend_contract_evidence') continue;
    reports.push({
      absolute_path: absolutePath,
      relative_path: path.relative(repoRoot, absolutePath),
      payload,
      stat: await stat(absolutePath),
    });
  }
  reports.sort((left, right) => right.stat.mtimeMs - left.stat.mtimeMs);
  return reports.slice(0, 1);
}

function backendContractCapabilityPasses(reports) {
  const passes = new Map();
  for (const report of reports) {
    for (const run of report.payload.runs ?? []) {
      if (run.status !== 'passed') continue;
      for (const capability of run.capabilities ?? []) {
        passes.set(capability, {
          id: run.id,
          command: run.command,
          relative_path: report.relative_path,
        });
      }
    }
  }
  return passes;
}

function mostConcreteDeclaredStatus(statuses) {
  for (const status of ['deferred', 'unsupported', 'backend_test_required', 'execute_required', 'source_check_required']) {
    if (statuses.includes(status)) return status;
  }
  return statuses[0] ?? 'backend_test_required';
}

async function sourceCheck(repoRoot, evidence) {
  const results = [];
  let ok = true;
  for (const item of evidence) {
    const [relativePath, symbol] = String(item).split('::');
    try {
      const text = await readFile(path.join(repoRoot, relativePath), 'utf8');
      if (symbol && !text.includes(symbol)) {
        ok = false;
        results.push(`${relativePath} missing ${symbol}`);
      } else {
        results.push(symbol ? `${relativePath} contains ${symbol}` : `${relativePath} exists`);
      }
    } catch {
      ok = false;
      results.push(`${relativePath} is missing`);
    }
  }
  return { ok, evidence: results };
}

async function latestScenarioReports(repoRoot, absoluteReportDir) {
  const reports = new Map();
  const entries = await safeReadDir(absoluteReportDir);
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const absolutePath = path.join(absoluteReportDir, entry);
    const payload = await readJsonOrNull(absolutePath);
    if (!payload?.scenario_id || payload.provider !== 'zulip') continue;
    const current = reports.get(payload.scenario_id);
    const item = {
      absolute_path: absolutePath,
      relative_path: path.relative(repoRoot, absolutePath),
      payload,
      stat: await stat(absolutePath),
    };
    if (!current || item.stat.mtimeMs > current.stat.mtimeMs) {
      reports.set(payload.scenario_id, item);
    }
  }
  return reports;
}

async function latestBackendReports(repoRoot, absoluteBackendDir) {
  const entries = await safeReadDir(absoluteBackendDir);
  const reports = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const absolutePath = path.join(absoluteBackendDir, entry);
    const payload = await readJsonOrNull(absolutePath);
    if (payload?.provider !== 'zulip' || !Array.isArray(payload.observed_stages)) continue;
    reports.push({
      absolute_path: absolutePath,
      relative_path: path.relative(repoRoot, absolutePath),
      payload,
      stat: await stat(absolutePath),
    });
  }
  reports.sort((left, right) => right.stat.mtimeMs - left.stat.mtimeMs);
  return reports.slice(0, 3);
}

async function safeReadDir(absoluteDir) {
  try {
    await access(absoluteDir);
    return await readdir(absoluteDir);
  } catch {
    return [];
  }
}

async function readJsonOrNull(absolutePath) {
  try {
    return JSON.parse(await readFile(absolutePath, 'utf8'));
  } catch {
    return null;
  }
}

function summarize(capabilities) {
  const summary = {
    total: capabilities.length,
    pass: 0,
    pending: 0,
    deferred: 0,
    unsupported: 0,
    by_status: {},
  };
  for (const capability of capabilities) {
    summary.by_status[capability.status] = (summary.by_status[capability.status] ?? 0) + 1;
    if (passStatuses.has(capability.status)) {
      summary.pass += 1;
    } else if (capability.status === 'deferred') {
      summary.deferred += 1;
      summary.pending += 1;
    } else if (capability.status === 'unsupported') {
      summary.unsupported += 1;
      summary.pending += 1;
    } else {
      summary.pending += 1;
    }
  }
  return summary;
}

function groupFromName(name) {
  const prefix = name.split('.')[0];
  return {
    account_setup: 'Identity',
    event_ingest: 'Receive',
    identity: 'Identity',
    intelligence: 'Intelligence',
    lab: 'Trace',
    media: 'Media',
    mutation: 'Mutation',
    polygraph: 'Intelligence',
    provider_api: 'Send',
    provider_command: 'Send',
    provider_observation: 'Trace',
    receive: 'Receive',
    review: 'Intelligence',
    signal_hub: 'Trace',
  }[prefix] ?? 'Trace';
}

function replayNote(name) {
  if (name.includes('idempotency') || name.includes('durable')) {
    return 'Capability directly requires replay/idempotency evidence.';
  }
  if (name.startsWith('mutation.') || name.startsWith('media.') || name.startsWith('receive.')) {
    return 'Replay must not duplicate canonical Communications state.';
  }
  return 'No additional replay note for this capability.';
}

function pushUnique(values, value) {
  if (!values.includes(value)) values.push(value);
}

function safeTimestamp(value) {
  return value.replace(/[^a-zA-Z0-9_.-]+/g, '-').replace(/^-+|-+$/g, '');
}
