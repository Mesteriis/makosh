import { access, readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];
const selfTestMode = process.argv.includes('--self-test');
const architectureContractPath = path.join(repoRoot, 'scripts', 'architecture-contract.json');
const boundaryBaselinePath = path.join(repoRoot, 'scripts', 'architecture-boundary-baseline.json');
const expectedInteractionKinds = [
	'direct_call',
	'command_port',
	'query_port',
	'event',
	'projection',
	'runtime_integration_api'
];
const forbiddenCanonicalEvidenceDirs = [
	'backend/src/domains/signals',
	'backend/src/domains/events',
	'backend/src/domains/attention',
	'backend/src/domains/evidence',
	'backend/src/vault/observations'
];
const communicationRawRecordInsertOwner = 'backend/src/domains/communications/core/raw_records.rs';
const communicationMessageInsertOwner = 'backend/src/domains/communications/messages/store/upsert.rs';
const communicationAcceptedSignalProjectionOwner =
	'backend/src/domains/communications/messages/provider_observation_projection.rs';
const communicationAcceptedSignalProjectionBootstrapOwner =
	'backend/src/application/bootstrap/core/projections.rs';
const reviewPromotionWorkflow = 'backend/src/workflows/review_promotion/mod.rs';
const communicationProviderCrudFacadeOwners = new Set([
	'backend/src/domains/communications/core/accounts.rs',
	'backend/src/domains/communications/core/secrets.rs'
]);
const telegramCommandQueueOwner = 'backend/src/integrations/telegram/client/commands.rs';
const facadeFreeFiles = [
	'backend/src/platform/secrets.rs',
	'backend/src/app/mod.rs',
	'backend/src/integrations/telegram/client/models.rs',
	'backend/src/integrations/telegram/client/models/messages.rs',
	'backend/src/integrations/telegram/client/lifecycle.rs',
	'backend/src/integrations/telegram/client/mod.rs',
	'backend/src/integrations/telegram/client/commands.rs',
	'backend/src/integrations/telegram/runtime.rs'
];
const mailSyncRunMutationOwners = new Set([
	'backend/src/domains/communications/background_sync/store/run_start.rs',
	'backend/src/domains/communications/background_sync/store/run_progress.rs',
	'backend/src/domains/communications/background_sync/store/run_finish.rs',
	'backend/src/domains/communications/background_sync/store/orphaned.rs'
]);
const aiPromptMutationOwners = new Set([
	'backend/src/ai/control_center/prompts/templates.rs',
	'backend/src/ai/control_center/prompts/versions.rs',
	'backend/src/ai/control_center/prompts/activation.rs',
	'backend/src/ai/control_center/prompts/evaluation.rs'
]);
const aiModelCatalogMutationOwners = new Set([
	'backend/src/ai/control_center/catalog.rs'
]);
const aiModelRouteMutationOwners = new Set([
	'backend/src/ai/control_center/routes.rs'
]);
const aiSemanticEmbeddingMutationOwners = new Set([
	'backend/src/ai/core/semantic/embeddings.rs'
]);
const documentProcessingJobMutationOwners = new Set([
	'backend/src/domains/documents/processing/jobs.rs'
]);
const whatsappSessionMutationOwners = new Set([
	'backend/src/integrations/whatsapp/client/store/sessions.rs'
]);
const telegramChatMutationOwners = new Set([
	'backend/src/integrations/telegram/client/chats.rs'
]);
const telegramChatParticipantMutationOwners = new Set([
	'backend/src/integrations/telegram/client/participants.rs'
]);
const telegramTopicMutationOwners = new Set([
	'backend/src/integrations/telegram/client/topics.rs'
]);
const telegramReactionMutationOwners = new Set([
	'backend/src/integrations/telegram/client/reactions.rs'
]);
const automationTemplatePolicyMutationOwners = new Set([
	'backend/src/engines/automation/store.rs'
]);
const automationOutboundMessageMutationOwners = new Set([
	'backend/src/engines/automation/dry_run.rs'
]);
const reviewManualOrchestrationOwner = 'backend/src/domains/review/service.rs';
const taskCommandServiceOwner = 'backend/src/domains/tasks/service.rs';
const calendarCommandServiceOwner = 'backend/src/domains/calendar/service.rs';
const organizationCommandServiceOwner = 'backend/src/domains/organizations/service.rs';
const personCommandServiceOwner = 'backend/src/domains/personas/service.rs';
const decisionCommandServiceOwner = 'backend/src/domains/decisions/service.rs';
const obligationCommandServiceOwner = 'backend/src/domains/obligations/service.rs';
const relationshipCommandServiceOwner = 'backend/src/domains/relationships/service.rs';
const taskCandidateReviewServiceOwner = 'backend/src/application/review_transitions.rs';
const projectLinkReviewServiceOwner = 'backend/src/domains/projects/link_reviews/service.rs';
const contradictionReviewServiceOwner = 'backend/src/engines/consistency/service.rs';
const documentProcessingCommandServiceOwner = 'backend/src/domains/documents/processing/service.rs';
const mailCommandServiceOwner = 'backend/src/domains/communications/service.rs';
const emailSyncPipelineOrganizationOwner = 'backend/src/workflows/email_sync_pipeline/organizations.rs';
const emailSyncPipelineParticipantsOwner = 'backend/src/workflows/email_sync_pipeline/participants.rs';
const emailSyncPipelineRelationshipsOwner = 'backend/src/workflows/email_sync_pipeline/relationships.rs';
const sharedBackendDomainModules = new Set();
const businessBackendDomains = new Set([
	'agents',
	'calendar',
	'communications',
	'decisions',
	'documents',
	'graph',
	'knowledge',
	'mail',
	'notes',
	'obligations',
	'organizations',
	'personas',
	'projects',
	'radar',
	'relationships',
	'signal_hub',
	'tasks',
	'timeline'
]);
const platformTechnicalTablePrefixes = [
	'ai_runtime_',
	'api_audit_',
	'application_',
	'audit_',
	'event_',
	'observation_',
	'projection_',
	'secret_',
	'settings_'
];
const platformForbiddenBusinessTablePrefixes = [
	'communication_',
	'task_',
	'calendar_',
	'review_',
	'graph_'
];

async function exists(filePath) {
	try {
		await access(filePath);
		return true;
	} catch {
		return false;
	}
}

async function gitLsFiles() {
	const { stdout } = await execFileAsync('git', ['ls-files'], { cwd: repoRoot });
	return stdout.split('\n').filter(Boolean);
}

function normalizePath(filePath) {
	return filePath.split(path.sep).join('/');
}

async function collectFiles(relativeRoot, extensions) {
	const absoluteRoot = path.join(repoRoot, relativeRoot);
	let entries;
	try {
		entries = await readdir(absoluteRoot, { withFileTypes: true });
	} catch {
		return [];
	}

	const files = [];
	for (const entry of entries) {
		const relativePath = normalizePath(path.join(relativeRoot, entry.name));
		if (entry.isDirectory()) {
			files.push(...await collectFiles(relativePath, extensions));
			continue;
		}
		if (entry.isFile() && extensions.has(path.extname(entry.name))) {
			files.push(relativePath);
		}
	}
	return files;
}

function topLevelRustUseGroupItems(groupBody) {
	const items = [];
	let depth = 0;
	let segment = '';

	for (const char of groupBody) {
		if (char === '{') {
			depth += 1;
			segment += char;
			continue;
		}
		if (char === '}') {
			depth -= 1;
			segment += char;
			continue;
		}
		if (char === ',' && depth === 0) {
			items.push(segment.trim());
			segment = '';
			continue;
		}
		segment += char;
	}

	if (segment.trim() !== '') {
		items.push(segment.trim());
	}

	return items;
}

function extractGroupedBackendDomainImports(source) {
	const imports = new Set();
	const marker = 'crate::domains::{';
	let searchFrom = 0;

	while (searchFrom < source.length) {
		const markerIndex = source.indexOf(marker, searchFrom);
		if (markerIndex === -1) break;

		const groupStart = markerIndex + marker.length;
		let depth = 1;
		let cursor = groupStart;
		while (cursor < source.length && depth > 0) {
			const char = source[cursor];
			if (char === '{') depth += 1;
			if (char === '}') depth -= 1;
			cursor += 1;
		}

		if (depth !== 0) {
			searchFrom = groupStart;
			continue;
		}

		const groupBody = source.slice(groupStart, cursor - 1);
		for (const item of topLevelRustUseGroupItems(groupBody)) {
			const match = /^([a-zA-Z_][a-zA-Z0-9_]*)/.exec(item);
			if (match !== null && match[1] !== 'self' && match[1] !== 'super') {
				imports.add(match[1]);
			}
		}

		searchFrom = cursor;
	}

	return imports;
}

function extractBackendDomainImports(source) {
	const imports = extractGroupedBackendDomainImports(source);
	const directPattern = /\bcrate::domains::([a-zA-Z_][a-zA-Z0-9_]*)\b/g;
	for (const match of source.matchAll(directPattern)) {
		imports.add(match[1]);
	}
	return imports;
}

function extractGroupedBackendRootImports(source, rootModule) {
	const imports = new Set();
	const marker = `crate::${rootModule}::{`;
	let searchFrom = 0;

	while (searchFrom < source.length) {
		const markerIndex = source.indexOf(marker, searchFrom);
		if (markerIndex === -1) break;

		const groupStart = markerIndex + marker.length;
		let depth = 1;
		let cursor = groupStart;
		while (cursor < source.length && depth > 0) {
			const char = source[cursor];
			if (char === '{') depth += 1;
			if (char === '}') depth -= 1;
			cursor += 1;
		}

		if (depth !== 0) {
			searchFrom = groupStart;
			continue;
		}

		const groupBody = source.slice(groupStart, cursor - 1);
		for (const item of topLevelRustUseGroupItems(groupBody)) {
			const match = /^([a-zA-Z_][a-zA-Z0-9_]*)/.exec(item);
			if (match !== null && match[1] !== 'self' && match[1] !== 'super') {
				imports.add(match[1]);
			}
		}

		searchFrom = cursor;
	}

	return imports;
}

function extractBackendRootImports(source, rootModule) {
	const imports = extractGroupedBackendRootImports(source, rootModule);
	const directPattern = new RegExp(`\\bcrate::${rootModule}::([a-zA-Z_][a-zA-Z0-9_]*)\\b`, 'g');
	for (const match of source.matchAll(directPattern)) {
		imports.add(match[1]);
	}
	return imports;
}

function backendBoundaryViolations(relativePath, source) {
	const violations = [];
	const domainMatch = /^backend\/src\/domains\/([^/]+)\//.exec(relativePath);
	const integrationMatch = /^backend\/src\/integrations\/([^/]+)\//.exec(relativePath);
	const workflowMatch = /^backend\/src\/workflows\//.exec(relativePath);
	const platformMatch = /^backend\/src\/platform\//.exec(relativePath);
	const aiMatch = /^backend\/src\/ai\//.exec(relativePath);
	const engineMatch = /^backend\/src\/engines\//.exec(relativePath);
	const isBackendTestFile =
		relativePath.includes('/tests/') ||
		relativePath.endsWith('/tests.rs') ||
		relativePath.endsWith('_tests.rs');
	const importedDomains = extractBackendDomainImports(source);
	const importedIntegrations = extractBackendRootImports(source, 'integrations');
	const importedAppModules = extractBackendRootImports(source, 'app');
	const importedWorkflowModules = extractBackendRootImports(source, 'workflows');
	const importedVaultModules = extractBackendRootImports(source, 'vault');

	for (const importedDomain of importedDomains) {
		if (sharedBackendDomainModules.has(importedDomain)) continue;

		if (domainMatch !== null) {
			const currentDomain = domainMatch[1];
			if (importedDomain !== currentDomain) {
				violations.push({
					file: relativePath,
					importedDomain,
					message: `${relativePath}: domain "${currentDomain}" imports domain "${importedDomain}"; publish/consume events instead`
				});
			}
			continue;
		}

		if (
			integrationMatch !== null &&
			businessBackendDomains.has(importedDomain) &&
			!isBackendTestFile
		) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: integration "${integrationMatch[1]}" imports business domain "${importedDomain}"; publish integration/communication events instead`
			});
		}
	}

	if (domainMatch !== null) {
		for (const importedIntegration of importedIntegrations) {
			violations.push({
				file: relativePath,
				importedIntegration,
				message: `${relativePath}: domain "${domainMatch[1]}" imports integration "${importedIntegration}"; depend on platform contracts or provider command ports instead`
			});
		}
		for (const importedWorkflow of importedWorkflowModules) {
			violations.push({
				file: relativePath,
				importedWorkflow,
				message: `${relativePath}: domain "${domainMatch[1]}" imports workflow "${importedWorkflow}"; publish/consume events or use domain-owned ports instead`
			});
		}
		for (const importedVaultModule of importedVaultModules) {
			violations.push({
				file: relativePath,
				importedVaultModule,
				message: `${relativePath}: domain "${domainMatch[1]}" imports vault "${importedVaultModule}"; carry secret refs and use platform/provider ports instead`
			});
		}
		for (const importedAppModule of importedAppModules) {
			violations.push({
				file: relativePath,
				importedAppModule,
				message: `${relativePath}: domain "${domainMatch[1]}" imports app module "${importedAppModule}"; app must depend on domains, not the reverse`
			});
		}
	}

	if (workflowMatch !== null) {
		for (const importedIntegration of importedIntegrations) {
			violations.push({
				file: relativePath,
				importedIntegration,
				message: `${relativePath}: workflow imports integration "${importedIntegration}"; coordinate providers through platform ports/events instead`
			});
		}
		if (/use\s+crate::domains::[^;]*\b[A-Za-z0-9_]*Store\b/.test(source)) {
			violations.push({
				file: relativePath,
				message: `${relativePath}: workflow imports concrete domain store types; depend on domain command/query ports instead`
			});
		}
	}

	if (/^backend\/src\/app\/handlers\//.test(relativePath)) {
		if (/\bRuntime[A-Za-z0-9_]*Context\b|\b[A-Za-z0-9_]*RuntimeOperationContext\b/.test(source)) {
			violations.push({
				file: relativePath,
				message: `${relativePath}: app handler constructs runtime orchestration context; route through an application service`
			});
		}
	}

	if (integrationMatch !== null) {
		for (const importedWorkflow of importedWorkflowModules) {
			violations.push({
				file: relativePath,
				importedWorkflow,
				message: `${relativePath}: integration "${integrationMatch[1]}" imports workflow "${importedWorkflow}"; publish provider events or depend on platform contracts instead`
			});
		}
	}

	if (platformMatch !== null) {
		for (const importedDomain of importedDomains) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: platform imports domain "${importedDomain}"; platform contracts must stay below domains`
			});
		}
		for (const importedIntegration of importedIntegrations) {
			violations.push({
				file: relativePath,
				importedIntegration,
				message: `${relativePath}: platform imports integration "${importedIntegration}"; integrations must depend on platform, not the reverse`
			});
		}
		for (const importedWorkflow of importedWorkflowModules) {
			violations.push({
				file: relativePath,
				importedWorkflow,
				message: `${relativePath}: platform imports workflow "${importedWorkflow}"; workflows must depend on platform, not the reverse`
			});
		}
	}

	if (aiMatch !== null) {
		for (const importedDomain of importedDomains) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: AI layer imports domain "${importedDomain}"; use AI/platform contracts or query ports instead`
			});
		}
	}

	if (engineMatch !== null) {
		for (const importedDomain of importedDomains) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: engine imports domain "${importedDomain}"; engines must return neutral candidates/projections through platform contracts`
			});
		}
		for (const importedIntegration of importedIntegrations) {
			violations.push({
				file: relativePath,
				importedIntegration,
				message: `${relativePath}: engine imports integration "${importedIntegration}"; engines must remain provider-agnostic`
			});
		}
	}

	return violations;
}

function integrationCommunicationBusinessSqlFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('backend/src/integrations/')) return [];
	const errors = [];
	const allowedIntegrationCommunicationTables = new Map([
		[
			'backend/src/integrations/whatsapp/runtime/mod.rs',
			new Set([
				'communication_provider_accounts',
				'communication_provider_commands'
			])
		]
	]);
	const sqlTablePattern =
		/\b(?:FROM|JOIN|INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+(communication_[a-zA-Z0-9_]*)\b/gi;
	for (const match of source.matchAll(sqlTablePattern)) {
		const tableName = match[1];
		if (allowedIntegrationCommunicationTables.get(relativePath)?.has(tableName)) continue;
		errors.push(
			`${relativePath}: integration code must not read or mutate business communication table "${tableName}"; use Communications query/command ports or provider-neutral platform raw records`
		);
	}
	return errors;
}

function platformBusinessSqlFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('backend/src/platform/')) return [];
	const errors = [];
	const sqlTablePattern =
		/\b(?:FROM|JOIN|INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+([a-zA-Z][a-zA-Z0-9_]*)\b/gi;
	for (const match of source.matchAll(sqlTablePattern)) {
		const tableName = match[1];
		if (platformTechnicalTablePrefixes.some((prefix) => tableName.startsWith(prefix))) continue;
		if (!platformForbiddenBusinessTablePrefixes.some((prefix) => tableName.startsWith(prefix))) continue;
		errors.push(
			`${relativePath}: platform code must not read or mutate business table "${tableName}"; move SQL ownership to the owning domain and expose neutral platform contracts only`
		);
	}
	return errors;
}

function resolveFrontendImport(relativePath, specifier) {
	if (specifier.startsWith('.')) {
		return normalizePath(path.posix.normalize(path.posix.join(path.posix.dirname(relativePath), specifier)));
	}
	if (specifier.startsWith('@/')) {
		return normalizePath(path.posix.normalize(path.posix.join('frontend/src', specifier.slice(2))));
	}
	if (specifier.startsWith('src/')) {
		return normalizePath(path.posix.normalize(path.posix.join('frontend', specifier)));
	}
	return null;
}

function frontendBoundaryViolations(relativePath, source) {
	const domainMatch = /^frontend\/src\/domains\/([^/]+)\//.exec(relativePath);
	const integrationMatch = /^frontend\/src\/integrations\/([^/]+)\//.exec(relativePath);
	if (domainMatch === null && integrationMatch === null) return [];

	const violations = [];
	const importPattern = /\bfrom\s+['"]([^'"]+)['"]|\bimport\s+['"]([^'"]+)['"]/g;

	for (const match of source.matchAll(importPattern)) {
		const specifier = match[1] ?? match[2];
		const resolved = resolveFrontendImport(relativePath, specifier);
		if (resolved === null) continue;

		const importedDomainMatch = /^frontend\/src\/domains\/([^/]+)\//.exec(resolved);
		const importedIntegrationMatch = /^frontend\/src\/integrations\/([^/]+)\//.exec(resolved);

		if (domainMatch !== null && importedDomainMatch !== null) {
			const currentDomain = domainMatch[1];
			const importedDomain = importedDomainMatch[1];
			if (importedDomain !== currentDomain) {
				violations.push({
					file: relativePath,
					importedDomain,
					message: `${relativePath}: frontend domain "${currentDomain}" imports domain "${importedDomain}"; compose domains from frontend/src/app instead`
				});
			}
		}

		if (domainMatch !== null && importedIntegrationMatch !== null) {
			violations.push({
				file: relativePath,
				importedIntegration: importedIntegrationMatch[1],
				message: `${relativePath}: frontend domain "${domainMatch[1]}" imports integration "${importedIntegrationMatch[1]}"; move shared code to frontend/src/shared or compose from app`
			});
		}

		if (integrationMatch !== null && importedDomainMatch !== null) {
			violations.push({
				file: relativePath,
				importedDomain: importedDomainMatch[1],
				message: `${relativePath}: frontend integration "${integrationMatch[1]}" imports domain "${importedDomainMatch[1]}"; move shared code to frontend/src/shared or platform`
			});
		}
	}

	return violations;
}

function providerScopedCommunicationRouteFailuresForSource(relativePath, source) {
	if (!/\.(?:rs|ts|vue|md)$/.test(relativePath)) return [];
	const errors = [];
	const forbiddenRoutePattern = /\/api\/v1\/communications\/(mail|telegram|whatsapp)(?=\/|['"`\s])/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: provider-scoped Communications business route "/api/v1/communications/${match[1]}" is forbidden; use provider-neutral /api/v1/communications/* or runtime /api/v1/integrations/${match[1]}/*`
		);
	}
	return errors;
}

function integrationBusinessCommunicationRouteFailuresForSource(relativePath, source) {
	if (!/\.(?:rs|ts|vue|md)$/.test(relativePath)) return [];
	const errors = [];
	const forbiddenRoutePattern =
		/\/api\/v1\/integrations\/(telegram|whatsapp)\/(conversations|messages|topics|search|sync)(?=\/|\?|['"`\s])/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: business Communication route "/api/v1/integrations/${match[1]}/${match[2]}" is forbidden; use provider-neutral /api/v1/communications/* or explicit provider-control /api/v1/integrations/${match[1]}/provider-*`
		);
	}
	const forbiddenProviderShapedBusinessRoutePattern =
		/\/api\/v1\/integrations\/(telegram|whatsapp|mail)\/provider-(conversations|messages|topics|reactions|reply-chain|forward-chain|raw-evidence|pinned-messages)(?=\/|\?|['"`\s])/g;
	for (const match of source.matchAll(forbiddenProviderShapedBusinessRoutePattern)) {
		errors.push(
			`${relativePath}: provider-shaped business Communication route "/api/v1/integrations/${match[1]}/provider-${match[2]}" is forbidden; use provider-neutral /api/v1/communications/* or explicit provider-control /api/v1/integrations/${match[1]}/provider-commands|provider-search|provider-sync|provider-media/*`
		);
	}
	return errors;
}

function providerSearchBusinessReadRouteFailuresForSource(relativePath, source) {
	if (!/\.(?:rs|ts|vue|md)$/.test(relativePath)) return [];
	const errors = [];
	const forbiddenRoutePattern =
		/\/api\/v1\/integrations\/(telegram|whatsapp|mail)\/provider-search\/(messages|media|provider)(?=\/|\?|['"`\s])/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: provider-search route "/api/v1/integrations/${match[1]}/provider-search/${match[2]}" is forbidden for business reads; provider search may only be the status/trigger route /api/v1/integrations/${match[1]}/provider-search and must not return projected Communication items`
		);
	}
	return errors;
}

function frontendTemporaryBusinessMovedStubFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/')) return [];
	const errors = [];
	if (/\bcommunicationBusinessApiMoved\b|moved to frontend\/src\/domains\/communications/.test(source)) {
		errors.push(
			`${relativePath}: temporary Communication business API moved stubs are forbidden; move the client/hook to Communications/shared and call the provider-neutral implementation`
		);
	}
	return errors;
}

function frontendDomainProviderControlRouteFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/domains/')) return [];
	const errors = [];
	const forbiddenRoutePattern =
		/\/api\/v1\/integrations\/(?:mail|telegram|whatsapp)\/provider-(?:commands|search|media|sync)\b/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: frontend domain code must not call provider-control route "${match[0]}"; normal business UI must use /api/v1/communications/*`
		);
	}
	return errors;
}

function frontendCommunicationsDomainIntegrationRouteFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/domains/communications/')) return [];
	const errors = [];
	const forbiddenRoutePattern = /\/api\/v1\/integrations\/(?:mail|telegram|whatsapp)\b/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: Communications domain must not call provider integration route "${match[0]}"; compose provider-control actions outside the domain`
		);
	}
	return errors;
}

function frontendIntegrationCommunicationBusinessRouteFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/integrations/')) return [];
	const errors = [];
	const forbiddenRoutePattern =
		/\/api\/v1\/communications\/(?:conversations|messages|search|topics)(?=\/|\?|['"`\s])/g;
	for (const match of source.matchAll(forbiddenRoutePattern)) {
		errors.push(
			`${relativePath}: frontend integration code must not read Communication business route "${match[0]}"; pass business data through app/shared composition`
		);
	}
	return errors;
}

function frontendSharedCommunicationBusinessLayerFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/shared/communications/')) return [];
	const errors = [];
	const forbiddenPattern = /\/api\/v1\/communications|\buseQuery\b|\bqueryKey\b|\[\s*['"]communications['"]|\bfetch\(/g;
	for (const match of source.matchAll(forbiddenPattern)) {
		errors.push(
			`${relativePath}: shared communication modules must stay DTO/helper-only; business API/query/cache token "${match[0]}" belongs in frontend/src/domains/communications`
		);
	}
	return errors;
}

function frontendIntegrationBusinessOwnershipFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('frontend/src/integrations/')) return [];
	const errors = [];
	const forbiddenSharedBusinessPattern = /shared\/communications\/.*Business/g;
	const forbiddenBusinessComponentPattern = /MessageThread|ChatList|MediaGallery|RawEvidence|ReplyChain|ForwardChain|Reactions|Topics/g;
	for (const match of source.matchAll(forbiddenSharedBusinessPattern)) {
		errors.push(
			`${relativePath}: integration UI must not import shared Communication business module "${match[0]}"; move business hooks/components to Communications domain`
		);
	}
	for (const match of source.matchAll(forbiddenBusinessComponentPattern)) {
		errors.push(
			`${relativePath}: integration UI must not own Communication business component/token "${match[0]}"; provider panels are runtime/setup/control/debug only`
		);
	}
	return errors;
}

function applicationAppImportFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('backend/src/application/')) return [];
	const errors = [];
	if (/\buse\s+crate::app::|\bcrate::app::/.test(source)) {
		errors.push(
			`${relativePath}: application services must not import crate::app; pass explicit dependencies and map application errors in app handlers`
		);
	}
	return errors;
}

function providerClientLeakFailuresForSource(relativePath, source) {
	if (
		!relativePath.startsWith('backend/src/app/') &&
		!relativePath.startsWith('backend/src/application/') &&
		!relativePath.startsWith('backend/src/domains/')
	) {
		return [];
	}
	const whatsappRuntimeApplicationContractOwners = new Set([
		'backend/src/application/provider_runtime_contracts.rs',
		'backend/src/application/provider_runtime_services.rs'
	]);
	const errors = [];
	const forbiddenProviderRuntimePattern =
		/\b(?:SmtpClient|GmailApiClient|TdJson|TelegramRuntimeOperationContext|WhatsApp[A-Za-z0-9_]*Runtime|LiveSmtpTransport|LiveGmailOutboxTransport|ProviderOutboxEmailSender)\b/g;
	for (const match of source.matchAll(forbiddenProviderRuntimePattern)) {
		if (
			match[0] === 'WhatsAppProviderRuntime' &&
			whatsappRuntimeApplicationContractOwners.has(relativePath)
		) {
			continue;
		}
		errors.push(
			`${relativePath}: app/application/domain code must not construct or depend on provider runtime/client symbol "${match[0]}"; use application ports, provider commands, or integration workers`
		);
	}
	return errors;
}

function appMessagingHandlerIntegrationImportFailuresForSource(relativePath, source) {
	const isTelegramHandler = relativePath.startsWith('backend/src/app/handlers/telegram/');
	const isWhatsappHandler = relativePath === 'backend/src/app/handlers/whatsapp.rs'
		|| relativePath.startsWith('backend/src/app/handlers/whatsapp/');
	if (!isTelegramHandler && !isWhatsappHandler) return [];
	const errors = [];
	const forbiddenPattern = /\b(?:use\s+)?crate::integrations::/g;
	for (const match of source.matchAll(forbiddenPattern)) {
		errors.push(
			`${relativePath}: app messaging handlers must not import integration runtime/store code "${match[0]}"; call one application service and map the response`
		);
	}
	return errors;
}

function integrationBusinessMutationBridgeFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('backend/src/integrations/')) return [];
	const errors = [];
	const forbiddenMutationBridgePattern =
		/\b(?:provider_message_state|ProviderChannelMessageCommandPort|ProviderMessageObservationProjectionPort|provider_observation_projection|apply_metadata|set_delivery_state|apply_content_update|apply_pinned_state|update_attachment_download_state|append_idempotent)\b/g;
	for (const match of source.matchAll(forbiddenMutationBridgePattern)) {
		errors.push(
			`${relativePath}: integrations must publish immutable provider observations, not call Communication business mutation bridge "${match[0]}"`
		);
	}
	return errors;
}

function applicationProviderMessageStateFailuresForSource(relativePath, source) {
	if (!relativePath.startsWith('backend/src/application/')) return [];
	const errors = [];
	const forbiddenProviderStatePattern =
		/\b(?:provider_message_state|ProviderChannelMessageCommandPort|apply_metadata|set_delivery_state|apply_content_update|apply_pinned_state|update_attachment_download_state)\b/g;
	for (const match of source.matchAll(forbiddenProviderStatePattern)) {
		errors.push(
			`${relativePath}: provider message state mutation bridge "${match[0]}" is forbidden; project provider observation events from application/workflow side`
		);
	}
	return errors;
}

function appRouterBootstrapLeakFailuresForSource(relativePath, source) {
	if (relativePath !== 'backend/src/app/router.rs') return [];
	const errors = [];
	const forbiddenBootstrapPattern =
		/\b(?:spawn_mail_background_sync_scheduler|spawn_mail_outbox_delivery_scheduler|spawn_telegram_command_executor|ProviderOutboxEmailSender|LiveSmtpTransport|LiveGmailOutboxTransport|execute_queued_commands)\b/g;
	for (const match of source.matchAll(forbiddenBootstrapPattern)) {
		errors.push(
			`${relativePath}: app router must not own provider/background orchestration symbol "${match[0]}"; move construction to application bootstrap`
		);
	}
	return errors;
}

function fixtureRouteFailuresForSource(relativePath, source) {
	if (!/^backend\/src\/app\/(?:router\/routes\/messaging\.rs|handlers\/(?:telegram\/(?:accounts|messages)\.rs|whatsapp\.rs))$/.test(relativePath)) {
		return [];
	}
	const errors = [];
	const legacyFixtureRoutePattern =
		/\/api\/v1\/integrations\/(telegram|whatsapp)\/(?:accounts\/fixture|messages)(?=\/|\?|['"`\s])/g;
	for (const match of source.matchAll(legacyFixtureRoutePattern)) {
		errors.push(
			`${relativePath}: fixture routes for ${match[1]} must live under /api/v1/integrations/${match[1]}/fixtures/* and be gated to dev/test/local mode`
		);
	}
	if (
		relativePath.startsWith('backend/src/app/handlers/') &&
		/\bpost_(?:telegram|whatsapp)_fixture_/.test(source) &&
		!/\bensure_fixture_routes_enabled\s*\(/.test(source)
	) {
		errors.push(
			`${relativePath}: fixture handlers must call ensure_fixture_routes_enabled before writing fixture provider state`
		);
	}
	return errors;
}

function canonicalEvidenceBoundaryFailures(trackedFiles, existingDirs = new Set()) {
	const errors = [];
	const rejectedFiles = new Set();

	for (const forbiddenDir of forbiddenCanonicalEvidenceDirs) {
		if (
			existingDirs.has(forbiddenDir) ||
			trackedFiles.some((file) => file === forbiddenDir || file.startsWith(`${forbiddenDir}/`))
		) {
			errors.push(`${forbiddenDir}: forbidden boundary; observations belong to platform/observations, not Vault or a domain`);
			for (const file of trackedFiles) {
				if (file === forbiddenDir || file.startsWith(`${forbiddenDir}/`)) {
					rejectedFiles.add(file);
				}
			}
		}
	}

	for (const file of trackedFiles) {
		if (!rejectedFiles.has(file) && /^backend\/src\/vault\/.*observations?/i.test(file)) {
			errors.push(`${file}: Vault must not own observations; use backend/src/platform/observations`);
		}
	}

	return errors;
}

function canonicalCommunicationRawRecordWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		if (file === communicationRawRecordInsertOwner) continue;
		if (/\bINSERT\s+INTO\s+communication_raw_records\b/i.test(content)) {
			errors.push(`${file}: communication_raw_records writes must go through ${communicationRawRecordInsertOwner} so every raw provider record has a canonical observation`);
		}
	}
	return errors;
}

function canonicalCommunicationMessageWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		if (file === communicationMessageInsertOwner) continue;
		if (/\bINSERT\s+INTO\s+communication_messages\b/i.test(content)) {
			errors.push(`${file}: communication_messages writes must go through ${communicationMessageInsertOwner} so every message projection keeps its canonical observation reference`);
		}
	}
	return errors;
}

function communicationAcceptedSignalBoundaryFailures(fileContents) {
	const errors = [];
	const projectionOwnerSource = fileContents.get(communicationAcceptedSignalProjectionOwner);
	if (projectionOwnerSource !== undefined) {
		if (/\bintegration\.telegram\./.test(projectionOwnerSource)) {
			errors.push(
				`${communicationAcceptedSignalProjectionOwner}: Communications accepted-signal projection must not consume legacy integration.telegram.* events; consume only signal.accepted.* families`
			);
		}
		if (/\bsignal\.raw\./.test(projectionOwnerSource)) {
			errors.push(
				`${communicationAcceptedSignalProjectionOwner}: Communications projection must not consume signal.raw.* events directly; Signal Hub accepted events are the only supported input`
			);
		}
	}

	for (const [file, content] of fileContents.entries()) {
		if (/\bproject_mail_signal_event\s*\(/.test(content) && file !== communicationAcceptedSignalProjectionOwner) {
			errors.push(
				`${file}: direct project_mail_signal_event() usage is forbidden outside ${communicationAcceptedSignalProjectionOwner}; consume accepted signals through the owner entry point`
			);
		}

		if (
			/\bproject_provider_observation_event\s*\(/.test(content) &&
			file !== communicationAcceptedSignalProjectionOwner &&
			file !== communicationAcceptedSignalProjectionBootstrapOwner &&
			!file.startsWith('backend/tests/')
		) {
			errors.push(
				`${file}: project_provider_observation_event() may only be called by the accepted-signal consumer bootstrap or tests`
			);
		}

		if (
			/\bproject_accepted_signal_event\s*\(/.test(content) &&
			file !== communicationAcceptedSignalProjectionOwner &&
			!file.startsWith('backend/tests/')
		) {
			errors.push(
				`${file}: direct project_accepted_signal_event() usage is forbidden outside ${communicationAcceptedSignalProjectionOwner}; route callers through consume_accepted_signal_event() or the event consumer`
			);
		}
	}

	return errors;
}

function canonicalTaskCandidateWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+task_candidates\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: task_candidates writes must include observation_id so message candidates stay linked to canonical evidence`);
			}
		}
	}
	return errors;
}

function canonicalGraphEvidenceWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+graph_evidence\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: graph_evidence writes must include observation_id so knowledge graph evidence can point at canonical observations`);
			}
		}
	}
	return errors;
}

function canonicalSemanticEmbeddingWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+semantic_embeddings\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: semantic_embeddings writes must include observation_id so AI retrieval can cite canonical evidence`);
			}
		}
	}
	return errors;
}

function canonicalReviewPromotionEvidenceFailures(fileContents) {
	const content = fileContents.get(reviewPromotionWorkflow);
	if (content === undefined) return [];
	if (/\b[A-Za-z]*EvidenceSourceKind::RawRecord\b/.test(content)) {
		return [
			`${reviewPromotionWorkflow}: review promotion evidence must use observation source kind and observation_id, not raw_record compatibility evidence`
		];
	}
	return [];
}

function canonicalReviewPromotionOwnerFailures(fileContents) {
	const content = fileContents.get(reviewPromotionWorkflow);
	if (content === undefined) return [];
	if (/\bINSERT\s+INTO\s+(personas|persons|person_personas|organizations|projects|project_keywords|obligation_task_links)\b/i.test(content)) {
		return [
			`${reviewPromotionWorkflow}: review promotion must materialize personas/organizations/projects/task-links through their domain stores, not direct SQL owners`
		];
	}
	return [];
}

function emailSyncPipelineOrganizationOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineOrganizationOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+(organizations|organization_domains|organization_identities|organization_persona_links)\b/i.test(content)) {
		return [
			`${emailSyncPipelineOrganizationOwner}: email sync organization projection must use organization domain owner stores, not direct SQL writes`
		];
	}
	return [];
}

function emailSyncPipelineParticipantsOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineParticipantsOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+communication_message_participants\b/i.test(content)) {
		return [
			`${emailSyncPipelineParticipantsOwner}: email sync message participant projection must use message domain owner stores, not direct SQL writes`
		];
	}
	return [];
}

function emailSyncPipelineRelationshipsOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineRelationshipsOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+relationship_events\b/i.test(content)) {
		return [
			`${emailSyncPipelineRelationshipsOwner}: email sync relationship timeline projection must use relationship event owner stores, not direct SQL writes`
		];
	}
	return [];
}

function legacyContextPackTableFailures(fileContents) {
	const errors = [];
	const legacyContextPackPattern = /\b(event_context_packs|task_context_packs)\b/;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/')) continue;
		if (legacyContextPackPattern.test(content)) {
			errors.push(
				`${file}: legacy event_context_packs/task_context_packs access is forbidden; use backend/src/engines/context_packs through domain owner stores`
			);
		}
	}
	return errors;
}

function automationTemplatePolicyMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+(automation_templates|automation_policies)\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/engines/automation')) continue;
		if (automationTemplatePolicyMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: automation template/policy mutations must stay in backend/src/engines/automation/store.rs so policy configuration writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function automationOutboundMessageMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+telegram_outbound_messages\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/engines/automation')) continue;
		if (automationOutboundMessageMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_outbound_messages mutations for automation dry-run/live send must stay in backend/src/engines/automation/dry_run.rs so outbound materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function communicationProviderCrudFacadeFailures(fileContents) {
	const errors = [];
	const forbiddenUsagePattern =
		/\.(?:upsert_provider_account|provider_account|list_provider_accounts|bind_provider_account_secret|provider_account_secret_bindings|provider_account_secret_binding)\s*\(/g;
	for (const [file, content] of fileContents.entries()) {
		if (communicationProviderCrudFacadeOwners.has(file)) continue;
		if (forbiddenUsagePattern.test(content)) {
			errors.push(
				`${file}: provider account and secret binding ownership must use crate::vault::*Store directly, not CommunicationIngestionStore compatibility CRUD`
			);
		}
	}
	return errors;
}

function providerAccountOwnerMutationFailures(fileContents) {
	const errors = [];
	const ownedTablePatterns = [
		{
			ownerFile: 'backend/src/domains/communications/core/provider_store.rs',
			pattern:
				/\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+(?:communication_provider_accounts|communication_provider_account_secret_refs)\b/gi,
			description: 'communications provider account durable mutations'
		},
		{
			ownerFile: 'backend/src/domains/tasks/core/provider_store.rs',
			pattern: /\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+task_provider_accounts\b/gi,
			description: 'task provider durable mutations'
		},
		{
			ownerFile: 'backend/src/domains/calendar/events/account_store.rs',
			pattern: /\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+calendar_accounts\b/gi,
			description: 'calendar account durable mutations'
		},
		{
			ownerFile: 'backend/src/domains/calendar/events/source_store.rs',
			pattern: /\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+calendar_sources\b/gi,
			description: 'calendar source durable mutations'
		}
	];
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/')) continue;
		for (const { ownerFile, pattern, description } of ownedTablePatterns) {
			if (file === ownerFile) continue;
			if (pattern.test(content)) {
				errors.push(`${file}: ${description} must stay in ${ownerFile}`);
			}
		}
	}
	return errors;
}

function telegramProviderOwnershipFailures(fileContents) {
	const errors = [];
	const forbiddenUsagePattern =
		/\bCommunicationIngestionStore\b|\bcommunication_ingestion_store\s*\(/g;
	for (const [file, content] of fileContents.entries()) {
		const isTelegramRuntime = file.startsWith('backend/src/integrations/telegram/runtime/');
		const isTelegramApi = file.startsWith('backend/src/integrations/telegram/api/');
		if (!isTelegramRuntime && !isTelegramApi) continue;
		if (forbiddenUsagePattern.test(content)) {
			errors.push(
				`${file}: Telegram runtime/API provider ownership must use crate::vault::CommunicationProvider*Store, not CommunicationIngestionStore`
			);
		}
	}
	return errors;
}

function telegramCommandQueueMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT|UPDATE)\s+telegram_provider_write_commands\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (file === telegramCommandQueueOwner) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram provider command queue mutations must go through ${telegramCommandQueueOwner} so command lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function mailSyncRunMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+communication_mail_sync_runs\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/domains/communications/background_sync/')) continue;
		if (mailSyncRunMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: communication_mail_sync_runs mutations must stay in the dedicated owner store files so run lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiPromptMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+(ai_prompt_templates|ai_prompt_template_versions|ai_prompt_eval_runs)\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiPromptMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: AI prompt template/version/eval mutations must stay in dedicated prompt owner files so prompt studio writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiModelCatalogMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+ai_model_catalog\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiModelCatalogMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: ai_model_catalog mutations must stay in backend/src/ai/control_center/catalog.rs so curated model materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function aiModelRouteMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+ai_model_routes\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiModelRouteMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: ai_model_routes mutations must stay in backend/src/ai/control_center/routes.rs so model route writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiHubBoundaryFailures(fileContents) {
	const errors = [];
	const forbiddenImportPatterns = [
		/\buse\s+crate::integrations::ai_runtime::AiRuntimeClient\b/g,
		/\buse\s+crate::platform::ai_runtime::SharedAiRuntimePort\b/g
	];
	const forbiddenInferencePatterns = [/\.\s*chat_with_model\s*\(/g, /\.\s*embed_with_model\s*\(/g];
	for (const [file, content] of fileContents.entries()) {
		if (
			!file.startsWith('backend/src/domains/') &&
			!file.startsWith('backend/src/workflows/') &&
			!file.startsWith('backend/src/app/handlers/')
		) {
			continue;
		}
		for (const pattern of forbiddenImportPatterns) {
			if (pattern.test(content)) {
				errors.push(
					`${file}: direct AI runtime imports are forbidden in domains, workflows and app handlers; route inference through crate::ai::hub::SharedAiHub`
				);
				break;
			}
		}
		for (const pattern of forbiddenInferencePatterns) {
			if (pattern.test(content)) {
				errors.push(
					`${file}: direct runtime inference calls are forbidden in domains, workflows and app handlers; use crate::ai::hub::AiHub instead`
				);
				break;
			}
		}
	}
	return errors;
}

function aiSemanticEmbeddingMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+semantic_embeddings\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/core/semantic/')) continue;
		if (aiSemanticEmbeddingMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: semantic_embeddings mutations must stay in backend/src/ai/core/semantic/embeddings.rs so derived embedding materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function documentProcessingJobMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+document_processing_jobs\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/domains/documents/processing/')) continue;
		if (documentProcessingJobMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: document_processing_jobs mutations must stay in backend/src/domains/documents/processing/jobs.rs so job lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function whatsappSessionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+whatsapp_web_sessions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/whatsapp/')) continue;
		if (whatsappSessionMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: whatsapp_web_sessions mutations must stay in backend/src/integrations/whatsapp/client/store/sessions.rs so session lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramChatMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_chats\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramChatMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_chats mutations must stay in backend/src/integrations/telegram/client/chats.rs so chat lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramChatParticipantMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_chat_participants\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramChatParticipantMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_chat_participants mutations must stay in backend/src/integrations/telegram/client/participants.rs so roster writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramTopicMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_topics\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramTopicMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_topics mutations must stay in backend/src/integrations/telegram/client/topics.rs so topic projection writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramReactionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_message_reactions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramReactionMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_reactions mutations must stay in backend/src/integrations/telegram/client/reactions.rs so reaction writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramMessageVersionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\bINSERT\s+INTO\s+telegram_message_versions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (isTelegramLifecycleMutationOwner(file, 'message_versions')) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_versions mutations must stay in backend/src/integrations/telegram/client/lifecycle/message_versions.rs so edit-version writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramMessageTombstoneMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\bINSERT\s+INTO\s+telegram_message_tombstones\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (isTelegramLifecycleMutationOwner(file, 'tombstones')) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_tombstones mutations must stay in backend/src/integrations/telegram/client/lifecycle/tombstones.rs so tombstone writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function isTelegramLifecycleMutationOwner(file, ownerModule) {
	return file === `backend/src/integrations/telegram/client/lifecycle/${ownerModule}.rs`;
}

function reviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/review/api.rs');
	if (content === undefined) return [];

	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bReviewPromotionService\b/,
		/\.promote_with_observation\s*\(/,
		/\.set_status_with_observation\s*\(/
	];

	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/review/api.rs: manual review transition/promotion orchestration must stay in ${reviewManualOrchestrationOwner}, not the API layer`
		];
	}

	return [];
}

function tasksHandlerManualOrchestrationFailures(fileContents) {
	const taskHandlerFiles = [
		'backend/src/domains/tasks/handlers/tasks.rs',
		'backend/src/domains/tasks/handlers/core_records.rs',
		'backend/src/domains/tasks/handlers/intelligence.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bTaskStore::new\s*\([^)]+\)\.create\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.set_status_with_observation\s*\(/,
		/\.archive_with_observation\s*\(/,
		/\.add_with_source\s*\(/
	];

	const errors = [];
	for (const file of taskHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual task mutation orchestration must stay in ${taskCommandServiceOwner}, not task handlers`
			);
		}
	}
	return errors;
}

function calendarHandlerManualOrchestrationFailures(fileContents) {
	const calendarHandlerFiles = [
		'backend/src/domains/calendar/handlers/accounts.rs',
		'backend/src/domains/calendar/handlers/events/agenda.rs',
		'backend/src/domains/calendar/handlers/events/checklist.rs',
		'backend/src/domains/calendar/handlers/events/participants.rs',
		'backend/src/domains/calendar/handlers/events/relations.rs',
		'backend/src/domains/calendar/handlers/meetings.rs',
		'backend/src/domains/calendar/handlers/reminders.rs',
		'backend/src/domains/calendar/handlers/rules.rs',
		'backend/src/domains/calendar/handlers/scheduling.rs',
		'backend/src/domains/calendar/handlers/sync.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.set_with_observation\s*\(/,
		/\.add_with_observation\s*\(/,
		/\.link_with_observation\s*\(/,
		/\.set_active_with_observation\s*\(/
	];

	const errors = [];
	for (const file of calendarHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual calendar mutation orchestration must stay in ${calendarCommandServiceOwner}, not calendar handlers`
			);
		}
	}
	return errors;
}

function organizationHandlerManualOrchestrationFailures(fileContents) {
	const organizationHandlerFiles = [
		'backend/src/app/handlers/organizations/directory.rs',
		'backend/src/app/handlers/organizations/core_records.rs',
		'backend/src/app/handlers/organizations/enrichment.rs',
		'backend/src/app/handlers/organizations/health.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.archive_with_observation\s*\(/,
		/\.upsert_with_observation\s*\(/,
		/\.add_with_observation\s*\(/,
		/\.link_with_observation\s*\(/,
		/\.apply_with_observation\s*\(/,
		/\.toggle_watchlist_with_observation\s*\(/
	];

	const errors = [];
	for (const file of organizationHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual organization mutation orchestration must stay in ${organizationCommandServiceOwner}, not organization handlers`
			);
		}
	}
	return errors;
}

function personHandlerManualOrchestrationFailures(fileContents) {
	const personHandlerFiles = [
		'backend/src/app/handlers/personas/compatibility.rs',
		'backend/src/app/handlers/personas/health.rs',
		'backend/src/app/handlers/personas/history.rs',
		'backend/src/app/handlers/personas/identity.rs',
		'backend/src/app/handlers/personas/intelligence.rs',
		'backend/src/app/handlers/personas/investigator.rs',
		'backend/src/app/handlers/personas/memory.rs',
		'backend/src/app/handlers/personas/profile/actions.rs',
		'backend/src/app/handlers/personas/profile/owner.rs',
		'backend/src/app/handlers/personas/profile/personas.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_unattached_with_observation\s*\(/,
		/\.attach_to_persona_with_observation\s*\(/,
		/\.upsert_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.assign_with_observation\s*\(/,
		/\.remove_with_observation\s*\(/,
		/\.apply_with_observation\s*\(/,
		/\.reject_with_observation\s*\(/,
		/\.toggle_watchlist_with_observation\s*\(/,
		/\.enrich_persona_with_observation\s*\(/,
		/\.toggle_favorite_with_observation\s*\(/,
		/\.set_notes_with_observation\s*\(/,
		/\.set_owner_persona_with_observation\s*\(/,
		/\.update_persona_with_observation\s*\(/,
		/\.set_review_state_with_observation\s*\(/,
		/\.review_dossier_snapshot_with_observation\s*\(/,
		/\.add_with_observation\s*\(/
	];

	const errors = [];
	for (const file of personHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual person mutation/review orchestration must stay in ${personCommandServiceOwner}, not person handlers`
			);
		}
	}
	return errors;
}

function mailCommunicationQueryManualOrchestrationFailures(fileContents) {
	const mailHandlerFiles = [
		'backend/src/app/handlers/communications/communication_queries/drafts.rs',
		'backend/src/app/handlers/communications/communication_queries/folders.rs',
		'backend/src/app/handlers/communications/communication_queries/saved_searches.rs',
		'backend/src/app/handlers/communications/communication_queries/outbox.rs',
		'backend/src/app/handlers/communications/communication_queries/imports.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.upsert_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.copy_message_with_observation\s*\(/,
		/\.move_message_with_observation\s*\(/,
		/\.undo_with_observation\s*\(/,
		/\.upsert_imported_attachment_with_observation\s*\(/
	];

	const errors = [];
	for (const file of mailHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual mail communication mutation orchestration must stay in ${mailCommandServiceOwner}, not communication query handlers`
			);
		}
	}
	return errors;
}

function mailProviderSendManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/app/handlers/communications/sending/provider_send.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.record_sent_with_observation\s*\(/,
		/\.enqueue_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/app/handlers/communications/sending/provider_send.rs: manual provider send evidence orchestration must stay in ${mailCommandServiceOwner}, not the sending handler`
		];
	}
	return [];
}

function mailFinalHandlerManualOrchestrationFailures(fileContents) {
	const files = [
		'backend/src/app/handlers/communications/sending/forwarding.rs',
		'backend/src/app/handlers/communications/workflow_state.rs',
		'backend/src/app/handlers/communications/sending/local_state.rs',
		'backend/src/app/handlers/communications/message_ai_state.rs',
		'backend/src/app/handlers/communications/message_actions.rs',
		'backend/src/app/handlers/communications/workflow_actions/actions/personas.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bObservationStore::capture_in_transaction\b/,
		/\.transition_workflow_state_with_observation\s*\(/,
		/\.move_to_local_trash_with_observation\s*\(/,
		/\.restore_from_local_trash_with_observation\s*\(/,
		/\.transition_with_observation\s*\(/,
		/\.toggle_pin_with_observation\s*\(/,
		/\.toggle_important_with_observation\s*\(/,
		/\.snooze_with_observation\s*\(/,
		/\.toggle_mute_with_observation\s*\(/,
		/\.add_label_with_observation\s*\(/,
		/\.remove_label_with_observation\s*\(/,
		/\.enqueue_with_observation\s*\(/,
		/\.link_email_person_projection_in_transaction\s*\(/
	];

	const errors = [];
	for (const file of files) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: final mail handler observation/projection orchestration must stay in ${mailCommandServiceOwner}, not the handler layer`
			);
		}
	}
	return errors;
}

function mailAccountManagementManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/app/handlers/communications/account_management.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bObservationOriginKind\b/,
		/\.update_config_with_origin\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			'backend/src/app/handlers/communications/account_management.rs: email account logout/config mutation orchestration must stay in backend/src/domains/communications/core/provider_store.rs owner methods, not the handler'
		];
	}
	return [];
}

function decisionHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/decisions/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/decisions/api/handlers.rs: manual decision review orchestration must stay in ${decisionCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function obligationHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/obligations/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/obligations/api/handlers.rs: manual obligation review orchestration must stay in ${obligationCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function relationshipHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/relationships/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/relationships/api/handlers.rs: manual relationship review orchestration must stay in ${relationshipCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function taskCandidateHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/tasks/handlers/candidates.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\.capture\s*\(/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/tasks/handlers/candidates.rs: manual task candidate review orchestration must stay in ${taskCandidateReviewServiceOwner}, not the handler`
		];
	}
	return [];
}

function projectLinkReviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/projects/api/mod.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/projects/api/mod.rs: manual project link review orchestration must stay in ${projectLinkReviewServiceOwner}, not the API layer`
		];
	}
	return [];
}

function contradictionReviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/app/handlers/consistency.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/app/handlers/consistency.rs: manual contradiction review orchestration must stay in ${contradictionReviewServiceOwner}, not the API layer`
		];
	}
	return [];
}

function documentProcessingApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/documents/api/mod.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.retry_failed_job_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/documents/api/mod.rs: manual document-processing retry orchestration must stay in ${documentProcessingCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

async function checkAdrFiles() {
	const adrDir = path.join(repoRoot, 'docs', 'adr');
	const entries = await readdir(adrDir);
	const adrFiles = entries.filter((entry) => entry.startsWith('ADR-') && entry.endsWith('.md'));
	const adrNumbers = new Map();

	for (const file of adrFiles) {
		if (file === 'ADR-architecture-communication-contract.md') {
			continue;
		}
		const match = /^ADR-(\d{4})-[a-z0-9-]+\.md$/.exec(file);
		if (match === null) {
			failures.push(`docs/archive/adr/${file}: ADR filename must be ADR-NNNN-kebab-case.md`);
			continue;
		}

		const number = match[1];
		const existing = adrNumbers.get(number);
		if (existing !== undefined) {
			failures.push(`docs/archive/adr/${file}: duplicates ADR-${number} already used by ${existing}`);
		}
		adrNumbers.set(number, file);
	}

	for (const file of adrFiles) {
		const content = await readFile(path.join(adrDir, file), 'utf8');
		const statusMatch = /^Status:\s*(.+)$/m.exec(content);
		if (statusMatch === null) {
			failures.push(`docs/archive/adr/${file}: missing Status line`);
		} else if (!/^(Proposed|Accepted|Temporary|Superseded|Superseded by ADR-\d{4})$/.test(statusMatch[1].trim())) {
			failures.push(`docs/archive/adr/${file}: unsupported status "${statusMatch[1].trim()}"`);
		}

		for (const reference of content.matchAll(/\bADR-(\d{4})\b/g)) {
			if (!adrNumbers.has(reference[1])) {
				failures.push(`docs/archive/adr/${file}: references missing ADR-${reference[1]}`);
			}
		}
	}
}

async function checkArchitectureContract() {
	if (await exists(boundaryBaselinePath)) {
		failures.push(
			'scripts/architecture-boundary-baseline.json: forbidden by the architecture communication contract; fix the boundary instead of baselining it'
		);
	}

	let contract;
	try {
		contract = JSON.parse(await readFile(architectureContractPath, 'utf8'));
	} catch (error) {
		failures.push(`scripts/architecture-contract.json: cannot read or parse contract: ${error.message}`);
		return;
	}

	if (contract.schema_version !== 1) {
		failures.push('scripts/architecture-contract.json: schema_version must be 1');
	}
	if (JSON.stringify(contract.interaction_kinds) !== JSON.stringify(expectedInteractionKinds)) {
		failures.push(
			`scripts/architecture-contract.json: interaction_kinds must be ${expectedInteractionKinds.join(', ')}`
		);
	}

	const requiredBackendLayers = ['app', 'domains', 'integrations', 'workflows', 'engines', 'ai', 'platform', 'vault'];
	for (const layer of requiredBackendLayers) {
		if (contract.backend?.layers?.[layer] === undefined) {
			failures.push(`scripts/architecture-contract.json: missing backend layer ${layer}`);
		}
	}
	const requiredFrontendLayers = ['app', 'domains', 'integrations'];
	for (const layer of requiredFrontendLayers) {
		if (contract.frontend?.layers?.[layer] === undefined) {
			failures.push(`scripts/architecture-contract.json: missing frontend layer ${layer}`);
		}
	}

	if (!contract.backend?.layers?.domains?.deny?.includes('other_domains')) {
		failures.push('scripts/architecture-contract.json: backend domains must deny other_domains');
	}
	if (!contract.backend?.layers?.integrations?.deny?.includes('domains')) {
		failures.push('scripts/architecture-contract.json: backend integrations must deny domains');
	}
	if (contract.frontend?.provider_business_cache_roots?.business_query_key_root !== 'communications') {
		failures.push(
			'scripts/architecture-contract.json: frontend provider business cache root must be communications'
		);
	}
}

async function checkMigrations() {
	const migrationsDir = path.join(repoRoot, 'backend', 'migrations');
	const entries = await readdir(migrationsDir);
	const migrationFiles = entries.filter((entry) => entry.endsWith('.sql'));
	const seenNumbers = new Map();

	for (const file of migrationFiles) {
		const match = /^(\d{4})_[a-z0-9_]+\.sql$/.exec(file);
		if (match === null) {
			failures.push(`backend/migrations/${file}: migration filename must be NNNN_snake_case.sql`);
			continue;
		}

		const number = match[1];
		const existing = seenNumbers.get(number);
		if (existing !== undefined) {
			failures.push(`backend/migrations/${file}: duplicates migration number ${number} used by ${existing}`);
		}
		seenNumbers.set(number, file);
	}
}

async function checkDockerBoundary() {
	const trackedFiles = await gitLsFiles();
	for (const file of trackedFiles) {
		if ((file.endsWith('/Dockerfile') || file === 'Dockerfile') && !file.startsWith('docker/')) {
			failures.push(`${file}: Dockerfiles must stay under docker/`);
		}

		if (/docker-compose\.ya?ml$/.test(file) && !file.startsWith('docker/')) {
			failures.push(`${file}: Compose files must stay under docker/`);
		}

		if (file.startsWith('docker/data/') && file !== 'docker/data/.gitkeep') {
			failures.push(`${file}: docker/data contents are local state and must not be committed`);
		}
	}
}

async function checkCanonicalEvidenceBoundaries() {
	const trackedFiles = await gitLsFiles();
	const vaultFiles = await collectFiles('backend/src/vault', new Set(['.rs']));
	const files = [...new Set([...trackedFiles, ...vaultFiles])];
	const existingDirs = new Set();
	for (const forbiddenDir of forbiddenCanonicalEvidenceDirs) {
		if (await exists(path.join(repoRoot, forbiddenDir))) {
			existingDirs.add(forbiddenDir);
		}
	}
	failures.push(...canonicalEvidenceBoundaryFailures(files, existingDirs));

	const backendFiles = await collectFiles('backend/src', new Set(['.rs']));
	const fileContents = new Map();
	for (const file of backendFiles) {
		fileContents.set(file, await readFile(path.join(repoRoot, file), 'utf8'));
	}
	failures.push(...canonicalCommunicationRawRecordWriteFailures(fileContents));
	failures.push(...canonicalCommunicationMessageWriteFailures(fileContents));
	failures.push(...communicationAcceptedSignalBoundaryFailures(fileContents));
	failures.push(...canonicalTaskCandidateWriteFailures(fileContents));
	failures.push(...canonicalGraphEvidenceWriteFailures(fileContents));
	failures.push(...canonicalSemanticEmbeddingWriteFailures(fileContents));
	failures.push(...canonicalReviewPromotionEvidenceFailures(fileContents));
	failures.push(...canonicalReviewPromotionOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineOrganizationOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineParticipantsOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineRelationshipsOwnerFailures(fileContents));
	failures.push(...legacyContextPackTableFailures(fileContents));
	failures.push(...automationTemplatePolicyMutationFailures(fileContents));
	failures.push(...automationOutboundMessageMutationFailures(fileContents));
	failures.push(...communicationProviderCrudFacadeFailures(fileContents));
	failures.push(...telegramProviderOwnershipFailures(fileContents));
	failures.push(...telegramCommandQueueMutationFailures(fileContents));
	failures.push(...mailSyncRunMutationFailures(fileContents));
	failures.push(...aiPromptMutationFailures(fileContents));
	failures.push(...aiModelCatalogMutationFailures(fileContents));
	failures.push(...aiModelRouteMutationFailures(fileContents));
	failures.push(...aiHubBoundaryFailures(fileContents));
	failures.push(...aiSemanticEmbeddingMutationFailures(fileContents));
	failures.push(...documentProcessingJobMutationFailures(fileContents));
	failures.push(...whatsappSessionMutationFailures(fileContents));
	failures.push(...telegramChatMutationFailures(fileContents));
	failures.push(...telegramChatParticipantMutationFailures(fileContents));
	failures.push(...telegramTopicMutationFailures(fileContents));
	failures.push(...telegramReactionMutationFailures(fileContents));
	failures.push(...telegramMessageVersionMutationFailures(fileContents));
	failures.push(...telegramMessageTombstoneMutationFailures(fileContents));
	failures.push(...providerAccountOwnerMutationFailures(fileContents));
	failures.push(...reviewApiManualOrchestrationFailures(fileContents));
	failures.push(...tasksHandlerManualOrchestrationFailures(fileContents));
	failures.push(...calendarHandlerManualOrchestrationFailures(fileContents));
	failures.push(...organizationHandlerManualOrchestrationFailures(fileContents));
	failures.push(...personHandlerManualOrchestrationFailures(fileContents));
	failures.push(...mailCommunicationQueryManualOrchestrationFailures(fileContents));
	failures.push(...mailProviderSendManualOrchestrationFailures(fileContents));
	failures.push(...mailFinalHandlerManualOrchestrationFailures(fileContents));
	failures.push(...mailAccountManagementManualOrchestrationFailures(fileContents));
	failures.push(...decisionHandlerManualOrchestrationFailures(fileContents));
	failures.push(...obligationHandlerManualOrchestrationFailures(fileContents));
	failures.push(...relationshipHandlerManualOrchestrationFailures(fileContents));
	failures.push(...taskCandidateHandlerManualOrchestrationFailures(fileContents));
	failures.push(...projectLinkReviewApiManualOrchestrationFailures(fileContents));
	failures.push(...contradictionReviewApiManualOrchestrationFailures(fileContents));
	failures.push(...documentProcessingApiManualOrchestrationFailures(fileContents));
}

async function checkLayerBoundaries() {
	const backendFiles = await collectFiles('backend/src', new Set(['.rs']));
	const frontendFiles = [
		...await collectFiles('frontend/src/domains', new Set(['.ts', '.vue'])),
		...await collectFiles('frontend/src/integrations', new Set(['.ts', '.vue'])),
		...await collectFiles('frontend/src/shared/communications', new Set(['.ts', '.vue']))
	];
	const backendViolations = [];
	const frontendViolations = [];

	for (const file of backendFiles) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		backendViolations.push(...backendBoundaryViolations(file, source));
		backendViolations.push(
			...integrationCommunicationBusinessSqlFailuresForSource(file, source).map((message) => ({
				file,
				message
			}))
		);
		backendViolations.push(
			...platformBusinessSqlFailuresForSource(file, source).map((message) => ({
				file,
				message
			}))
		);
		backendViolations.push(...applicationAppImportFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...providerClientLeakFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...appMessagingHandlerIntegrationImportFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...integrationBusinessMutationBridgeFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...applicationProviderMessageStateFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...appRouterBootstrapLeakFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		backendViolations.push(...fixtureRouteFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
	}

	for (const file of frontendFiles) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		frontendViolations.push(...frontendBoundaryViolations(file, source));
		frontendViolations.push(...frontendTemporaryBusinessMovedStubFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		frontendViolations.push(...frontendSharedCommunicationBusinessLayerFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		frontendViolations.push(...frontendIntegrationBusinessOwnershipFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		frontendViolations.push(...frontendDomainProviderControlRouteFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		frontendViolations.push(...frontendCommunicationsDomainIntegrationRouteFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
		frontendViolations.push(...frontendIntegrationCommunicationBusinessRouteFailuresForSource(file, source).map((message) => ({
			file,
			message
		})));
	}

	failures.push(...backendViolations.map((violation) => violation.message));
	failures.push(...frontendViolations.map((violation) => violation.message));
	failures.push(...await frontendProviderBusinessCacheRootFailures());
	failures.push(...await routeOwnershipFailures());
}

async function checkFacadeFreeFiles() {
	for (const relativePath of facadeFreeFiles) {
		const absolutePath = path.join(repoRoot, relativePath);
		if (!(await exists(absolutePath))) continue;
		const source = await readFile(absolutePath, 'utf8');
		if (/^\s*pub(?:\([^)]*\))?\s+use\b/m.test(source)) {
			failures.push(`${relativePath}: compatibility re-export facade is forbidden`);
		}
	}
}

async function frontendProviderBusinessCacheRootFailures() {
	const frontendFiles = await collectFiles('frontend/src', new Set(['.ts', '.vue']));
	const errors = [];
	for (const file of frontendFiles) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		errors.push(...frontendProviderBusinessCacheRootFailuresForSource(file, source));
	}
	return errors;
}

function frontendProviderBusinessCacheRootFailuresForSource(relativePath, source) {
	const errors = [];
	const forbiddenRootPattern =
		/\b(?:queryKey|invalidateQueries|setQueryData|getQueryData|removeQueries|refetchQueries|cancelQueries)\b[\s\S]{0,180}?\[\s*['"](telegram|whatsapp|mail)['"]/g;
	const forbiddenIntegrationBusinessRootPattern =
		/\[\s*['"]integrations['"]\s*,\s*['"](telegram|whatsapp|mail)['"]\s*,\s*['"]([^'"]+)['"]/g;
	for (const match of source.matchAll(forbiddenRootPattern)) {
		errors.push(
			`${relativePath}: provider business query/cache root "${match[1]}" is forbidden; use ["communications", ...] for business data or ["integrations", "${match[1]}", "runtime", ...] for provider runtime state`
		);
	}
	for (const match of source.matchAll(forbiddenIntegrationBusinessRootPattern)) {
		const cacheKey = match[2];
		if (isFrontendIntegrationRuntimeCacheRoot(cacheKey)) continue;
		errors.push(
			`${relativePath}: provider query/cache key ["integrations", "${match[1]}", "${cacheKey}"] is not a runtime/setup/control cache root; use ["communications", ...] for business/read-model data`
		);
	}
	return errors;
}

function isFrontendIntegrationRuntimeCacheRoot(cacheKey) {
	return /^(?:capabilities|account-capabilities|accounts|runtime|commands|qr-login-status|automation|sessions|conversation-folders|provider-(?:sync|search|media|commands|conversations|conversation-detail|conversation-members|folders|calls|call-transcript))$/.test(cacheKey);
}

async function routeOwnershipFailures() {
	const files = [
		...await collectFiles('backend/src', new Set(['.rs'])),
		...await collectFiles('frontend/src', new Set(['.ts', '.vue'])),
		...await collectFiles('docs', new Set(['.md'])),
		...await collectFiles('backend/tests', new Set(['.rs']))
	];
	const errors = [];
	for (const file of files) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		errors.push(...providerScopedCommunicationRouteFailuresForSource(file, source));
		errors.push(...integrationBusinessCommunicationRouteFailuresForSource(file, source));
		errors.push(...providerSearchBusinessReadRouteFailuresForSource(file, source));
	}
	return errors;
}

function assertSelfTest(name, condition) {
	if (!condition) {
		throw new Error(`architecture guard self-test failed: ${name}`);
	}
}

function runSelfTests() {
	assertSelfTest(
		'domain-to-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/radar/example.rs',
			'use crate::domains::personas::core::Person;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-business-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/integrations/slack/client.rs',
			'use crate::domains::tasks::api::TaskStore;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-business-domain grouped backend import fails',
		backendBoundaryViolations(
			'backend/src/integrations/slack/client.rs',
			'use crate::domains::{tasks::api::TaskStore, personas::core::Persona};'
		).length === 2
	);
	assertSelfTest(
		'handler-to-handler backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/radar/api.rs',
			'use crate::domains::tasks::api::handlers::create_task;'
		).length === 1
	);
	assertSelfTest(
		'backend event platform import passes',
		backendBoundaryViolations(
			'backend/src/domains/tasks/example.rs',
			'use crate::platform::events::NewEventEnvelope;'
		).length === 0
	);
	assertSelfTest(
		'domain-to-integration backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/communications/outbox.rs',
			'use crate::integrations::mail::send::SmtpClient;'
		).length === 1
	);
	assertSelfTest(
		'domain-to-workflow backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/communications/ingestion.rs',
			'use crate::workflows::email_intelligence::EmailIntelligenceService;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-workflow backend import fails',
		backendBoundaryViolations(
			'backend/src/integrations/telegram/client/messages/ingestion.rs',
			'use crate::workflows::provider_communication_projection::record_and_project_telegram_message;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-communications backend import fails without bridge exceptions',
		backendBoundaryViolations(
			'backend/src/integrations/telegram/client/store.rs',
			'use crate::domains::communications::core::CommunicationProviderAccountStore;'
		).length === 1
	);
	assertSelfTest(
		'domain-to-vault backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/communications/outbox/provider_sender.rs',
			'use crate::vault::HostVault;'
		).length === 1
	);
	assertSelfTest(
		'workflow-to-integration backend import fails',
		backendBoundaryViolations(
			'backend/src/workflows/mail_background_sync/provider.rs',
			'use crate::integrations::mail::gmail::client::GmailApiClient;'
		).length === 1
	);
	assertSelfTest(
		'workflow concrete domain store import fails',
		backendBoundaryViolations(
			'backend/src/workflows/review_inbox.rs',
			'use crate::domains::tasks::candidates::TaskCandidateStore;'
		).length === 1
	);
	assertSelfTest(
		'workflow may use a persistence adapter without being treated as a domain store import',
		backendBoundaryViolations(
			'backend/src/workflows/zoom_signal_detection.rs',
			'use crate::domains::signal_hub::SignalHubPort;\nuse makosh_events_postgres::store::EventStore;'
		).length === 0
	);
	assertSelfTest(
		'app handler runtime context construction fails',
		backendBoundaryViolations(
			'backend/src/app/handlers/telegram/messages.rs',
			'use crate::integrations::telegram::runtime::TelegramRuntimeOperationContext;'
		).length === 1
	);
	assertSelfTest(
		'app handler may import a workflow public API',
		backendBoundaryViolations(
			'backend/src/app/handlers/telegram/messages.rs',
			'use crate::workflows::provider_communication_projection::record_and_project_telegram_message;'
		).length === 0
	);
	assertSelfTest(
		'platform-to-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/platform/communications.rs',
			'use crate::domains::communications::messages::ProjectedMessage;'
		).length === 1
	);
	assertSelfTest(
		'ai-to-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/ai/core/service/attribution.rs',
			'use crate::domains::personas::api::PersonaProjectionStore;'
		).length === 1
	);
	assertSelfTest(
		'engine-to-integration backend import fails',
		backendBoundaryViolations(
			'backend/src/engines/search/index.rs',
			'use crate::integrations::telegram::client::TelegramStore;'
		).length === 1
	);
	assertSelfTest(
		'engine-to-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/engines/relationships/mod.rs',
			'use crate::domains::decisions::DecisionReviewState;'
		).length === 1
	);
	assertSelfTest(
		'integration communication business SQL fails',
		integrationCommunicationBusinessSqlFailuresForSource(
			'backend/src/integrations/telegram/client/messages/state_updates.rs',
			'UPDATE communication_messages SET delivery_state = $1 WHERE message_id = $2'
		).length === 1
	);
	assertSelfTest(
		'integration raw communication SQL fails on business table family',
		integrationCommunicationBusinessSqlFailuresForSource(
			'backend/src/integrations/telegram/client/messages/raw_records.rs',
			'INSERT INTO communication_raw_records (raw_record_id) VALUES ($1)'
		).length === 1
	);
	assertSelfTest(
		'platform business SQL fails',
		platformBusinessSqlFailuresForSource(
			'backend/src/platform/communications/channel_messages.rs',
			'UPDATE communication_messages SET delivery_state = $1 WHERE message_id = $2'
		).length === 1
	);
	assertSelfTest(
		'platform technical event SQL passes',
		platformBusinessSqlFailuresForSource(
			'backend/src/platform/events/store.rs',
			'INSERT INTO event_log (event_id) VALUES ($1)'
		).length === 0
	);
	assertSelfTest(
		'frontend integration business read-model query key fails',
		frontendProviderBusinessCacheRootFailuresForSource(
			'frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts',
			"queryKey: ['integrations', 'telegram', 'message-versions', messageId]"
		).length === 1
	);
	assertSelfTest(
		'frontend integration commands query key passes',
		frontendProviderBusinessCacheRootFailuresForSource(
			'frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts',
			"queryKey: ['integrations', 'telegram', 'commands', accountId]"
		).length === 0
	);
	assertSelfTest(
		'frontend integration communications query key fails with double quotes',
		frontendProviderBusinessCacheRootFailuresForSource(
			'frontend/src/integrations/telegram/queries/useTelegramBusinessQuery.ts',
			'queryKey: ["integrations", "telegram", "message-versions", messageId]'
		).length === 1
	);
	assertSelfTest(
		'frontend shared communications business query token fails',
		frontendSharedCommunicationBusinessLayerFailuresForSource(
			'frontend/src/shared/communications/hiddenBusinessQueries.ts',
			"import { useQuery } from '@tanstack/vue-query'; const queryKey = ['communications', 'messages'];"
		).length >= 1
	);
	assertSelfTest(
		'frontend integration shared business import fails',
		frontendIntegrationBusinessOwnershipFailuresForSource(
			'frontend/src/integrations/telegram/components/Panel.vue',
			"import { useTelegramBusinessMessages } from '../../../shared/communications/telegramBusinessQueries'"
		).length === 1
	);
	assertSelfTest(
		'frontend integration business component token fails',
		frontendIntegrationBusinessOwnershipFailuresForSource(
			'frontend/src/integrations/telegram/components/TelegramRuntimePanel.vue',
			'import TelegramMessageThread from "./TelegramMessageThread.vue"'
		).length === 2
	);
	assertSelfTest(
		'frontend communications domain integration route fails',
		frontendCommunicationsDomainIntegrationRouteFailuresForSource(
			'frontend/src/domains/communications/api/providerControl.ts',
			'"/api/v1/integrations/telegram/provider-commands/conversations/123/pin"'
		).length === 1
	);
	assertSelfTest(
		'app messaging handler integration import fails',
		appMessagingHandlerIntegrationImportFailuresForSource(
			'backend/src/app/handlers/telegram/messages.rs',
			'use crate::integrations::telegram::client::TelegramStore;'
		).length === 1
	);
	assertSelfTest(
		'same backend domain import passes',
		backendBoundaryViolations(
			'backend/src/domains/radar/example.rs',
			'use crate::domains::radar::core::SignalStore;'
		).length === 0
	);
	assertSelfTest(
		'frontend domain-to-domain relative import fails',
		frontendBoundaryViolations(
			'frontend/src/domains/radar/views/RadarPage.vue',
			"import TasksPage from '../../tasks/views/TasksPage.vue'"
		).length === 1
	);
	assertSelfTest(
		'frontend domain-to-domain alias import fails',
		frontendBoundaryViolations(
			'frontend/src/domains/radar/views/RadarPage.vue',
			"import TasksPage from '@/domains/tasks/views/TasksPage.vue'"
		).length === 1
	);
	assertSelfTest(
		'frontend app composition is outside domain lint',
		frontendBoundaryViolations(
			'frontend/src/app/views/HomeView.vue',
			"import TasksPage from '../../domains/tasks/views/TasksPage.vue'"
		).length === 0
	);
	assertSelfTest(
		'frontend domain-to-integration import fails',
		frontendBoundaryViolations(
			'frontend/src/domains/communications/views/CommunicationsPage.vue',
			"import AccountSetupModal from '../../../integrations/mail/components/AccountSetupModal.vue'"
		).length === 1
	);
	assertSelfTest(
		'frontend integration-to-domain import fails',
		frontendBoundaryViolations(
			'frontend/src/integrations/telegram/components/TelegramSavedSearchStrip.vue',
			"import SavedSearchStrip from '../../../domains/communications/components/SavedSearchStrip.vue'"
		).length === 1
	);
	assertSelfTest(
		'provider-scoped communications business route fails',
		providerScopedCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/communications/telegram/messages"'
		).length === 1
	);
	assertSelfTest(
		'provider runtime integration route passes',
		providerScopedCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/runtime/status"'
		).length === 0
	);
	assertSelfTest(
		'integration business conversations route fails',
		integrationBusinessCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/conversations/{telegram_chat_id}"'
		).length === 1
	);
	assertSelfTest(
		'integration provider-shaped business route fails',
		integrationBusinessCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/provider-conversations/{telegram_chat_id}/pin"'
		).length === 1
	);
	assertSelfTest(
		'integration provider-command route passes',
		integrationBusinessCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/pin"'
		).length === 0
	);
	assertSelfTest(
		'integration provider-shaped message subresource route fails',
		integrationBusinessCommunicationRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/provider-messages/{message_id}/raw-evidence"'
		).length === 1
	);
	assertSelfTest(
		'provider-search message read route fails',
		providerSearchBusinessReadRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/provider-search/messages"'
		).length === 1
	);
	assertSelfTest(
		'provider-search media read route fails',
		providerSearchBusinessReadRouteFailuresForSource(
			'frontend/src/integrations/telegram/api/telegramSearch.ts',
			"api.get('/api/v1/integrations/telegram/provider-search/media?q=alpha')"
		).length === 1
	);
	assertSelfTest(
		'provider-search trigger route passes',
		providerSearchBusinessReadRouteFailuresForSource(
			'frontend/src/integrations/telegram/api/telegramSearch.ts',
			"api.post('/api/v1/integrations/telegram/provider-search')"
		).length === 0
	);
	assertSelfTest(
		'temporary Communication business moved stub fails',
		frontendTemporaryBusinessMovedStubFailuresForSource(
			'frontend/src/integrations/telegram/api/telegramSearch.ts',
			'function communicationBusinessApiMoved() { throw new Error("moved to frontend/src/domains/communications") }'
		).length === 1
	);
	assertSelfTest(
		'frontend domain provider-control route fails',
		frontendDomainProviderControlRouteFailuresForSource(
			'frontend/src/domains/communications/api/providerCommands.ts',
			"apiFetch('/api/v1/integrations/telegram/provider-commands/send')"
		).length === 1
	);
	assertSelfTest(
		'frontend integration business route fails',
		frontendIntegrationCommunicationBusinessRouteFailuresForSource(
			'frontend/src/integrations/telegram/api/messages.ts',
			"apiFetch('/api/v1/communications/messages/msg-1/versions')"
		).length === 1
	);
	assertSelfTest(
		'application importing app fails',
		applicationAppImportFailuresForSource(
			'backend/src/application/telegram_runtime.rs',
			'use crate::app::{ApiError, AppState};'
		).length === 1
	);
	assertSelfTest(
		'integration projection port leakage fails',
		integrationBusinessMutationBridgeFailuresForSource(
			'backend/src/integrations/telegram/client/store.rs',
			'use crate::platform::communications::ProviderMessageObservationProjectionPort;'
		).length === 1
	);
	assertSelfTest(
		'integration idempotent event append leakage fails',
		integrationBusinessMutationBridgeFailuresForSource(
			'backend/src/integrations/telegram/runtime/manager/message_events/projection.rs',
			'EventStore::new(pool).append_idempotent(&event).await?;'
		).length === 1
	);
	assertSelfTest(
		'provider client leaks fail in app/application/domain',
		providerClientLeakFailuresForSource(
			'backend/src/application/communication_send.rs',
			'let client = SmtpClient::new();'
		).length === 1
	);
	assertSelfTest(
		'provider client symbols pass inside integrations',
		providerClientLeakFailuresForSource(
			'backend/src/integrations/mail/send.rs',
			'let client = SmtpClient::new();'
		).length === 0
	);
	assertSelfTest(
		'integration provider message state bridge fails',
		integrationBusinessMutationBridgeFailuresForSource(
			'backend/src/integrations/telegram/runtime/manager/message_events.rs',
			'use crate::application::provider_message_state::observe_telegram_message_pin_state;'
		).length === 1
	);
	assertSelfTest(
		'application provider message state bridge fails',
		applicationProviderMessageStateFailuresForSource(
			'backend/src/application/provider_message_state.rs',
			'ProviderChannelMessageStore::new(pool).apply_metadata(message_id, metadata, context).await'
		).length === 1
	);
	assertSelfTest(
		'app router bootstrap orchestration fails',
		appRouterBootstrapLeakFailuresForSource(
			'backend/src/app/router.rs',
			'spawn_mail_outbox_delivery_scheduler(&state);'
		).length === 1
	);
	assertSelfTest(
		'legacy fixture route fails',
		fixtureRouteFailuresForSource(
			'backend/src/app/router/routes/messaging.rs',
			'"/api/v1/integrations/telegram/accounts/fixture"'
		).length === 1
	);
	assertSelfTest(
		'ungated fixture handler fails',
		fixtureRouteFailuresForSource(
			'backend/src/app/handlers/telegram/messages.rs',
			'pub(crate) async fn post_telegram_fixture_message() { store.ingest_fixture_message(&request).await?; }'
		).length === 1
	);
	assertSelfTest(
		'gated fixture handler passes',
		fixtureRouteFailuresForSource(
			'backend/src/app/handlers/telegram/messages.rs',
			'pub(crate) async fn post_telegram_fixture_message() { ensure_fixture_routes_enabled(&state)?; store.ingest_fixture_message(&request).await?; }'
		).length === 0
	);
	assertSelfTest(
		'canonical evidence guard rejects forbidden evidence domain',
		canonicalEvidenceBoundaryFailures(['backend/src/domains/evidence/mod.rs']).length === 1
	);
	assertSelfTest(
		'canonical evidence guard rejects vault observations owner',
		canonicalEvidenceBoundaryFailures(['backend/src/vault/observations/mod.rs']).length === 1
	);
	assertSelfTest(
		'canonical evidence guard allows platform observations',
		canonicalEvidenceBoundaryFailures(['backend/src/platform/observations/mod.rs']).length === 0
	);
	assertSelfTest(
		'canonical communication raw record guard rejects direct writes',
		canonicalCommunicationRawRecordWriteFailures(new Map([
			['backend/src/integrations/telegram/client/messages.rs', 'INSERT INTO communication_raw_records (raw_record_id) VALUES ($1)']
		])).length === 1
	);
	assertSelfTest(
		'canonical communication raw record guard allows owner writes',
		canonicalCommunicationRawRecordWriteFailures(new Map([
			[communicationRawRecordInsertOwner, 'INSERT INTO communication_raw_records (raw_record_id) VALUES ($1)']
		])).length === 0
	);
	assertSelfTest(
		'canonical communication message guard rejects direct writes',
		canonicalCommunicationMessageWriteFailures(new Map([
			['backend/src/integrations/slack/messages.rs', 'INSERT INTO communication_messages (message_id) VALUES ($1)']
		])).length === 1
	);
	assertSelfTest(
		'canonical communication message guard allows projection owner writes',
		canonicalCommunicationMessageWriteFailures(new Map([
			[communicationMessageInsertOwner, 'INSERT INTO communication_messages (message_id) VALUES ($1)']
		])).length === 0
	);
	assertSelfTest(
		'canonical task candidate guard rejects writes without observation_id',
		canonicalTaskCandidateWriteFailures(new Map([
			['backend/src/ai/core/service/task_candidate_persistence.rs', 'INSERT INTO task_candidates (task_candidate_id, source_kind, source_id) VALUES ($1, $2, $3)']
		])).length === 1
	);
	assertSelfTest(
		'canonical task candidate guard allows writes with observation_id',
		canonicalTaskCandidateWriteFailures(new Map([
			['backend/src/domains/tasks/candidates/persistence.rs', 'INSERT INTO task_candidates (task_candidate_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4)']
		])).length === 0
	);
	assertSelfTest(
		'canonical graph evidence guard rejects writes without observation_id',
		canonicalGraphEvidenceWriteFailures(new Map([
			['backend/src/domains/graph/core/store.rs', 'INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id) VALUES ($1, $2, $3, $4)']
		])).length === 1
	);
	assertSelfTest(
		'canonical graph evidence guard allows writes with observation_id',
		canonicalGraphEvidenceWriteFailures(new Map([
			['backend/src/domains/graph/core/store.rs', 'INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4, $5)']
		])).length === 0
	);
	assertSelfTest(
		'canonical semantic embedding guard rejects writes without observation_id',
		canonicalSemanticEmbeddingWriteFailures(new Map([
			['backend/src/ai/core/semantic/embeddings.rs', 'INSERT INTO semantic_embeddings (semantic_embedding_id, source_kind, source_id) VALUES ($1, $2, $3)']
		])).length === 1
	);
	assertSelfTest(
		'canonical semantic embedding guard allows writes with observation_id',
		canonicalSemanticEmbeddingWriteFailures(new Map([
			['backend/src/ai/core/semantic/embeddings.rs', 'INSERT INTO semantic_embeddings (semantic_embedding_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4)']
		])).length === 0
	);
	assertSelfTest(
		'canonical review promotion guard rejects raw_record evidence',
		canonicalReviewPromotionEvidenceFailures(new Map([
			[reviewPromotionWorkflow, 'NewDecisionEvidence::new(DecisionEvidenceSourceKind::RawRecord, observation_id)']
		])).length === 1
	);
	assertSelfTest(
		'canonical review promotion guard allows observation evidence',
		canonicalReviewPromotionEvidenceFailures(new Map([
			[reviewPromotionWorkflow, 'NewDecisionEvidence::observation(observation_id)']
		])).length === 0
	);
	assertSelfTest(
		'canonical review promotion owner guard rejects direct persona writes',
		canonicalReviewPromotionOwnerFailures(new Map([
			[reviewPromotionWorkflow, 'INSERT INTO personas (person_id, display_name) VALUES ($1, $2)']
		])).length === 1
	);
	assertSelfTest(
		'canonical review promotion owner guard allows non-owner text',
		canonicalReviewPromotionOwnerFailures(new Map([
			[reviewPromotionWorkflow, 'PersonaCommandService::new(pool).create(command).await?;']
		])).length === 0
	);
	assertSelfTest(
		'provider CRUD facade usage outside communications owner layer fails',
		communicationProviderCrudFacadeFailures(new Map([
			['backend/src/domains/communications/outbox/provider_sender.rs', 'store.upsert_provider_account(&account);']
		])).length === 1
	);
	assertSelfTest(
		'communications projection rejects legacy integration telegram events',
		communicationAcceptedSignalBoundaryFailures(new Map([
			[
				communicationAcceptedSignalProjectionOwner,
				'matches!(event_type, "integration.telegram.message.content_observed")'
			]
		])).length === 1
	);
	assertSelfTest(
		'communications projection rejects raw signal consumption',
		communicationAcceptedSignalBoundaryFailures(new Map([
			[
				communicationAcceptedSignalProjectionOwner,
				'if event_type == "signal.raw.telegram.message.content.observed" { return Ok(()); }'
			]
		])).length === 1
	);
	assertSelfTest(
		'direct mail accepted-signal projection call outside owner fails',
		communicationAcceptedSignalBoundaryFailures(new Map([
			[
				'backend/src/workflows/example.rs',
				'let _ = project_mail_signal_event(pool.clone(), &event).await?;'
			]
		])).length === 1
	);
	assertSelfTest(
		'direct accepted-signal projection call outside owner fails',
		communicationAcceptedSignalBoundaryFailures(new Map([
			[
				'backend/src/workflows/email_sync_pipeline/raw_records.rs',
				'let _ = project_accepted_signal_event(pool.clone(), &event).await?;'
			]
		])).length === 1
	);
	assertSelfTest(
		'provider observation consumer call outside bootstrap or tests fails',
		communicationAcceptedSignalBoundaryFailures(new Map([
			[
				'backend/src/workflows/example.rs',
				'project_provider_observation_event(pool.clone(), event)'
			]
		])).length === 1
	);
	assertSelfTest(
		'provider account owner mutation guard rejects non-owner durable writes',
		providerAccountOwnerMutationFailures(new Map([
			[
				'backend/src/integrations/telegram/runtime/manager/message_events.rs',
				'INSERT INTO communication_provider_accounts (account_id) VALUES ($1)'
			]
		])).length === 1
	);
	assertSelfTest(
		'provider account owner mutation guard allows communications owner file',
		providerAccountOwnerMutationFailures(new Map([
			[
				'backend/src/domains/communications/core/provider_store.rs',
				'INSERT INTO communication_provider_accounts (account_id) VALUES ($1)'
			]
		])).length === 0
	);
	assertSelfTest(
		'provider CRUD facade usage in communications owner layer passes',
		communicationProviderCrudFacadeFailures(new Map([
			['backend/src/domains/communications/core/accounts.rs', 'store.upsert_provider_account(&account);']
		])).length === 0
	);
	assertSelfTest(
		'telegram provider ownership guard rejects communication store in runtime',
		telegramProviderOwnershipFailures(new Map([
			[
				'backend/src/integrations/telegram/runtime/manager.rs',
				'use crate::domains::communications::core::CommunicationIngestionStore;'
			]
		])).length === 1
	);
	assertSelfTest(
		'telegram provider ownership guard rejects communication store in raw API',
		telegramProviderOwnershipFailures(new Map([
			[
				'backend/src/integrations/telegram/api/raw.rs',
				'let store = communication_ingestion_store(&state)?;'
			]
		])).length === 1
	);
	assertSelfTest(
		'mail account management guard rejects handler-owned logout mutation orchestration',
		mailAccountManagementManualOrchestrationFailures(new Map([
			[
				'backend/src/app/handlers/communications/account_management.rs',
				'CommunicationProviderAccountStore::new(pool).update_config_with_origin(&account_id, &config, ObservationOriginKind::LocalRuntime, "actor", "logout");'
			]
		])).length === 1
	);
	assertSelfTest(
		'mail account management guard allows owner method call',
		mailAccountManagementManualOrchestrationFailures(new Map([
			[
				'backend/src/app/handlers/communications/account_management.rs',
				'CommunicationProviderAccountStore::new(pool).mark_logged_out(&account_id);'
			]
		])).length === 0
	);
	console.log('Architecture guard self-tests passed.');
}

async function main() {
	if (selfTestMode) {
		runSelfTests();
		return;
	}

	if (!(await exists(path.join(repoRoot, 'AGENTS.md')))) {
		failures.push('AGENTS.md is required at repository root');
	}

	await checkArchitectureContract();
	await checkAdrFiles();
	await checkMigrations();
	await checkDockerBoundary();
	await checkCanonicalEvidenceBoundaries();
	await checkLayerBoundaries();
	await checkFacadeFreeFiles();

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Architecture guard passed.');
}

await main();
