import assert from 'node:assert/strict';

import { evaluateWorkspacePolicy } from './check-cargo-architecture.mjs';

const roles = {
	kernel: { packages: ['makosh-kernel'], forbidden_dependencies: ['sqlx'] },
	provider_api: { packages: ['makosh-provider-api'], forbidden_dependencies: ['sqlx'] },
	provider_impl: {
		packages: ['makosh-provider-zulip'],
		forbidden_dependencies: ['makosh-backend', 'sqlx', 'makosh-backend-testkit']
	},
	composition: { packages: ['makosh-backend'], forbidden_dependencies: [] },
	test_support: { packages: ['makosh-backend-testkit'], forbidden_dependencies: [] },
	test_session: {
		packages: ['makosh-test-session'],
		forbidden_dependencies: ['makosh-backend', 'makosh-backend-testkit']
	},
	domain_api: { packages: [], forbidden_dependencies: [] },
	persistence_adapter: { packages: [], forbidden_dependencies: [] },
	application: { packages: [], forbidden_dependencies: [] },
	runtime: { packages: [], forbidden_dependencies: [] }
};

const roleEdges = {
	kernel: { normal: [], build: [], dev: [] },
	provider_api: { normal: ['kernel'], build: ['kernel'], dev: ['kernel'] },
	provider_impl: { normal: ['kernel', 'provider_api'], build: ['kernel', 'provider_api'], dev: ['kernel', 'provider_api'] },
	composition: { normal: ['kernel', 'provider_api', 'provider_impl', 'domain_api', 'persistence_adapter', 'application', 'runtime'], build: ['kernel'], dev: ['test_support'] },
	test_support: { normal: ['composition', 'kernel', 'provider_api', 'provider_impl', 'test_session'], build: ['kernel'], dev: ['composition', 'kernel', 'provider_api', 'provider_impl', 'test_session'] },
	test_session: { normal: [], build: [], dev: [] },
	domain_api: { normal: ['kernel'], build: ['kernel'], dev: ['kernel'] },
	persistence_adapter: { normal: ['kernel', 'domain_api'], build: ['kernel', 'domain_api'], dev: ['kernel', 'domain_api'] },
	application: { normal: ['kernel', 'domain_api'], build: ['kernel'], dev: ['test_support'] },
	runtime: { normal: ['kernel', 'provider_api', 'domain_api', 'application'], build: ['kernel'], dev: ['test_support'] }
};

const packages = new Map([
	['kernel-id', { id: 'kernel-id', name: 'makosh-kernel', workspace: true }],
	['provider-api-id', { id: 'provider-api-id', name: 'makosh-provider-api', workspace: true }],
	['provider-id', { id: 'provider-id', name: 'makosh-provider-zulip', workspace: true }],
	['backend-id', { id: 'backend-id', name: 'makosh-backend', workspace: true }],
	['testkit-id', { id: 'testkit-id', name: 'makosh-backend-testkit', workspace: true }],
	['session-id', { id: 'session-id', name: 'makosh-test-session', workspace: true }],
	['sqlx-id', { id: 'sqlx-id', name: 'sqlx', workspace: false }]
]);

const dependencies = new Map([
	['provider-api-id', [{ packageId: 'kernel-id', kind: 'normal' }]],
	['provider-id', [{ packageId: 'provider-api-id', kind: 'normal' }]],
	['backend-id', [
		{ packageId: 'provider-id', kind: 'normal' },
		{ packageId: 'testkit-id', kind: 'dev' }
	]],
	['testkit-id', [
		{ packageId: 'backend-id', kind: 'normal' },
		{ packageId: 'session-id', kind: 'normal' }
	]]
]);

assert.deepEqual(
	evaluateWorkspacePolicy({ packages, dependencies, roles, roleEdges }),
	[]
);

const unclassifiedPackages = new Map(packages);
unclassifiedPackages.set('mail-id', { id: 'mail-id', name: 'makosh-provider-mail', workspace: true });
assert.deepEqual(
	evaluateWorkspacePolicy({ packages: unclassifiedPackages, dependencies, roles, roleEdges }),
	['workspace package "makosh-provider-mail" has no role']
);

const duplicateRoles = structuredClone(roles);
duplicateRoles.kernel.packages.push('makosh-provider-zulip');
assert.deepEqual(
	evaluateWorkspacePolicy({ packages, dependencies, roles: duplicateRoles, roleEdges }),
	['workspace package "makosh-provider-zulip" has multiple roles: kernel, provider_impl']
);

const missingRoles = structuredClone(roles);
missingRoles.runtime.packages.push('makosh-worker-runtime');
assert.deepEqual(
	evaluateWorkspacePolicy({ packages, dependencies, roles: missingRoles, roleEdges }),
	['role runtime references missing workspace package "makosh-worker-runtime"']
);

const forbiddenDependencies = new Map(dependencies);
forbiddenDependencies.set('provider-id', [
	{ packageId: 'provider-api-id', kind: 'normal' },
	{ packageId: 'sqlx-id', kind: 'dev' }
]);
assert.deepEqual(
	evaluateWorkspacePolicy({ packages, dependencies: forbiddenDependencies, roles, roleEdges }),
	['makosh-provider-zulip: forbidden dependency "sqlx" for role provider_impl']
);

const illegalDevEdge = new Map(dependencies);
illegalDevEdge.set('provider-id', [
	{ packageId: 'provider-api-id', kind: 'normal' },
	{ packageId: 'testkit-id', kind: 'dev' }
]);
assert.deepEqual(
	evaluateWorkspacePolicy({ packages, dependencies: illegalDevEdge, roles, roleEdges }),
	[
		'makosh-provider-zulip: role provider_impl cannot depend on role test_support via dev dependency',
		'makosh-provider-zulip: forbidden dependency "makosh-backend" for role provider_impl',
		'makosh-provider-zulip: forbidden dependency "makosh-backend-testkit" for role provider_impl'
	]
);

console.log('Cargo architecture guard tests passed.');
