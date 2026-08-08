import { access, readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

async function exists(relativePath) {
	try {
		await access(path.join(repoRoot, relativePath));
		return true;
	} catch {
		return false;
	}
}

const contractPath = 'scripts/architecture-contract.json';
const contract = JSON.parse(await readFile(path.join(repoRoot, contractPath), 'utf8'));
const routerSource = await readFile(path.join(repoRoot, 'backend/src/app/router.rs'), 'utf8');
const relationshipErrorsSource = await readFile(
	path.join(repoRoot, 'backend/src/domains/relationships/errors.rs'),
	'utf8'
);
const taskRelationsSource = await readFile(
	path.join(repoRoot, 'backend/src/domains/tasks/core/relations.rs'),
	'utf8'
);
const taskCoreErrorsSource = await readFile(
	path.join(repoRoot, 'backend/src/domains/tasks/core/errors.rs'),
	'utf8'
);
const taskCandidateReviewSource = await readFile(
	path.join(repoRoot, 'backend/src/domains/tasks/candidates/store/review.rs'),
	'utf8'
);
const taskCandidateErrorsSource = await readFile(
	path.join(repoRoot, 'backend/src/domains/tasks/candidates/errors.rs'),
	'utf8'
);

assert.equal(
	await exists('scripts/architecture-boundary-baseline.json'),
	false,
	'architecture-boundary-baseline.json must not exist'
);

assert.equal(contract.schema_version, 1, 'architecture contract schema_version must be 1');
assert.ok(Array.isArray(contract.interaction_kinds), 'architecture contract must list interaction kinds');
assert.deepEqual(
	contract.interaction_kinds,
	['direct_call', 'command_port', 'query_port', 'event', 'projection', 'runtime_integration_api'],
	'architecture contract interaction kinds are the public communication vocabulary'
);

assert.ok(contract.backend?.layers?.domains?.deny, 'backend domain deny rules must be explicit');
assert.ok(contract.backend.layers.domains.deny.includes('other_domains'));
assert.ok(contract.backend.layers.domains.deny.includes('integrations'));
assert.ok(contract.backend.layers.domains.deny.includes('vault'));
assert.ok(
	contract.backend.layers.domains.owned.includes('signal_hub'),
	'Signal Hub must be registered as a backend business domain'
);
assert.ok(contract.backend.layers.integrations.deny.includes('domains'));
assert.ok(contract.backend.layers.workflows.allow.includes('domain_command_ports'));
assert.ok(contract.backend.layers.workflows.allow.includes('domain_query_ports'));
assert.ok(contract.backend.layers.app.deny.includes('stores'));
assert.ok(contract.backend.layers.ai.deny.includes('domain_stores'));
assert.ok(contract.backend.layers.platform.deny.includes('business_table_sql'));
assert.doesNotMatch(
	routerSource,
	/(?:start_background_services|spawn_host_vault_manifest_reconciliation)\s*\(/,
	'router construction must not launch background runtime tasks'
);
assert.doesNotMatch(
	taskRelationsSource,
	/\bdomains::relationships\b|\bRelationship(?:Store|EntityKind|ReviewState|Evidence)\b/,
	'Tasks relation store must not import or materialize Relationships state'
);
assert.doesNotMatch(
	taskCoreErrorsSource,
	/\bRelationshipStoreError\b/,
	'Tasks core errors must not expose cross-domain Relationships failures'
);
assert.doesNotMatch(
	relationshipErrorsSource,
	/\bGraphStoreError\b/,
	'Relationships errors must not expose cross-domain Graph failures'
);
assert.doesNotMatch(
	taskCandidateReviewSource,
	/\bdomains::obligations\b|\bObligation(?:Store|ReviewState|EntityKind|TaskLink)\b/,
	'Task candidate review store must not import or materialize Obligations state'
);
assert.doesNotMatch(
	taskCandidateErrorsSource,
	/\bObligationStoreError\b/,
	'Task candidate errors must not expose cross-domain Obligations failures'
);

assert.deepEqual(contract.workspace.roles.kernel.packages, ['makosh-kernel']);
assert.ok(contract.workspace.roles.provider_api.forbidden_dependencies.includes('sqlx'));
assert.deepEqual(contract.workspace.roles.contract_api, {
	packages: ['makosh-connectrpc-contracts'],
	forbidden_dependencies: ['makosh-backend', 'reqwest', 'sqlx']
});
assert.ok(contract.workspace.roles.provider_impl.forbidden_dependencies.includes('makosh-backend'));
assert.deepEqual(contract.workspace.roles.test_session, {
	packages: ['makosh-test-session'],
	forbidden_dependencies: ['makosh-backend', 'makosh-backend-testkit']
});

assert.ok(contract.frontend?.layers?.domains?.deny.includes('other_frontend_domains'));
assert.ok(contract.frontend.layers.domains.deny.includes('integrations'));
assert.ok(contract.frontend.layers.integrations.deny.includes('domains'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('telegram'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('whatsapp'));
assert.ok(contract.frontend.provider_business_cache_roots.business_query_key_root === 'communications');
assert.deepEqual(
	contract.frontend.business_route_model.forbidden_provider_business_roots,
	[
		'/api/v1/communications/mail',
		'/api/v1/communications/telegram',
		'/api/v1/communications/whatsapp'
	]
);

console.log('Architecture contract tests passed.');
