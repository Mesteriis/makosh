# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `162-source-scripts`
- Group / Группа: `scripts`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/scripts.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `scripts/build.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/build.sh`
- Size bytes / Размер в байтах: `1033`
- Included characters / Включено символов: `1033`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/resources.sh
source "$SCRIPT_DIR/lib/resources.sh"

load_makosh_env
ensure_frontend_dependencies
ensure_command cargo
ensure_command node
ensure_command pnpm

backend_target_dir="${CARGO_TARGET_DIR:-$CARGO_BUILD_TARGET_DIR}"

info "Building backend release binary"
CARGO_TARGET_DIR="$backend_target_dir" \
	cargo build --manifest-path "$REPO_ROOT/backend/Cargo.toml" --bin makosh-backend --release

info "Building frontend release assets"
(
	cd "$REPO_ROOT/frontend"
	pnpm build
)

info "Preparing bundled desktop resources"
prepare_google_oauth_resource
prepare_tdlib_macos
CARGO_TARGET_DIR="$backend_target_dir" prepare_backend_sidecar_macos

info "Building Tauri release artifacts"
(
	cd "$REPO_ROOT/frontend"
	pnpm tauri build
)

success "Release build completed"
```

### `scripts/check-architecture-contract.test.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/check-architecture-contract.test.mjs`
- Size bytes / Размер в байтах: `2813`
- Included characters / Включено символов: `2813`
- Truncated / Обрезано: `no`

```javascript
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

assert.ok(contract.frontend?.layers?.domains?.deny.includes('other_frontend_domains'));
assert.ok(contract.frontend.layers.domains.deny.includes('integrations'));
assert.ok(contract.frontend.layers.integrations.deny.includes('domains'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('telegram'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('whatsapp'));
assert.ok(contract.frontend.provider_business_cache_roots.business_query_key_root === 'communications');
assert.deepEqual(
	contract.frontend.business_route_model.forbidden_provider_business_roots,
	[
		'/api/v1/integrations/mail/*',
		'/api/v1/integrations/telegram/*',
		'/api/v1/integrations/whatsapp/*'
	]
);

console.log('Architecture contract tests passed.');
```

### `scripts/check-architecture.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/check-architecture.mjs`
- Size bytes / Размер в байтах: `106734`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```javascript
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
	'backend/src/application/bootstrap.rs';
const reviewPromotionWorkflow = 'backend/src/workflows/review_promotion/mod.rs';
const communicationProviderCrudFacadeOwners = new Set([
	'backend/src/domains/communications/core/accounts.rs',
	'backend/src/domains/communications/core/secrets.rs'
]);
const telegramCommandQueueOwner = 'backend/src/integrations/telegram/client/commands.rs';
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
const personCommandServiceOwner = 'backend/src/domains/persons/service.rs';
const decisionCommandServiceOwner = 'backend/src/domains/decisions/service.rs';
const obligationCommandServiceOwner = 'backend/src/domains/obligations/service.rs';
const relationshipCommandServiceOwner = 'backend/src/domains/relationships/service.rs';
const taskCandidateReviewServiceOwner = 'backend/src/domains/tasks/candidates/service.rs';
const projectLinkReviewServiceOwner = 'backend/src/domains/projects/link_reviews/service.rs';
const contradictionReviewServiceOwner = 'backend/src/engines/consistency/service.rs';
const documentProcessingCommandServiceOwner = 'backend/src/domains/documents/processing/service.rs';
const mailCommandServiceOwner = 'backend/src/domains/communications/service.rs';
const emailSyncPipelineOrganizationOwner = 'backend/src/workflows/email_sync_pipeline/organizations.rs';
const emailSyncPipelineParticipantsOwner = 'backend/src/workflows/email_sync_pipeline/participants.rs';
const emailSyncPipelineRelationshipsOwner = 'backend/src/workflows/email_sync_pipeline/relationships.rs';
const backendDomainProjectionBridgeOwners = new Map([
	[
		'backend/src/domains/relationships/errors.rs',
		new Set(['graph'])
	],
	[
		'backend/src/domains/relationships/store.rs',
		new Set(['graph'])
	],
	[
		'backend/src/domains/tasks/candidates/errors.rs',
		new Set(['obligations'])
	],
	[
		'backend/src/domains/tasks/candidates/store/review.rs',
		new Set(['obligations'])
	],
	[
		'backend/src/domains/tasks/core/errors.rs',
		new Set(['relationships'])
	],
	[
		'backend/src/domains/tasks/core/relations.rs',
		new Set(['relationships'])
	]
]);

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
	'persons',
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
				const allowedBridgeImports = backendDomainProjectionBridgeOwners.get(relativePath);
				if (allowedBridgeImports?.has(importedDomain)) {
					continue;
				}
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
				message: `${relativePath}: integration "${integrationMatch[1]}" imports business domain "${importedDomain}"; publish integration/c
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `scripts/check-code-boundaries.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/check-code-boundaries.mjs`
- Size bytes / Размер в байтах: `5619`
- Included characters / Включено символов: `5619`
- Truncated / Обрезано: `no`

```javascript
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

const scanRoots = [
	'AGENTS.md',
	'Makefile',
	'.pre-commit-config.yaml',
	'backend',
	'docs',
	'frontend',
	'scripts'
];
const ignoredSegments = new Set([
	'.git',
	'backend/target',
	'docker/data',
	'frontend/.svelte-kit',
	'frontend/build',
	'frontend/coverage',
	'frontend/dist',
	'frontend/src/gen',
	'frontend/node_modules'
]);
const checkedExtensions = new Set([
	'.css',
	'.html',
	'.js',
	'.json',
	'.md',
	'.mjs',
	'.rs',
	'.sql',
	'.svelte',
	'.toml',
	'.ts',
	'.vue',
	'.yaml',
	'.yml'
]);
const generatedPrefixes = [
	'backend/target/',
	'docker/data/',
	'frontend/.svelte-kit/',
	'frontend/build/',
	'frontend/coverage/',
	'frontend/dist/',
	'frontend/node_modules/'
];
const secretPattern =
	/(password|passwd|secret|token|api[_-]?key|oauth|bearer)\s*[:=]\s*['"][^'"]+['"]|BEGIN (RSA|OPENSSH|PRIVATE)|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9_]+/i;
const blanketSuppressions = [
	{ pattern: /#\s*\[\s*allow\s*\(\s*warnings\s*\)\s*\]/, message: 'blanket Rust warning suppression is forbidden' },
	{ pattern: /#\s*\[\s*allow\s*\(\s*clippy::all\s*\)\s*\]/, message: 'blanket clippy suppression is forbidden' },
	{
		pattern: new RegExp('@ts-' + 'ignore'),
		message: '@ts-' + 'ignore is forbidden; use explicit typing or a documented @ts-expect-error boundary'
	},
	{
		pattern: new RegExp('eslint-' + 'disable'),
		message: 'eslint-' + 'disable is forbidden in source; fix or narrow the lint rule centrally'
	}
];
const forbiddenBackendTestEnvKeys = [
	'MAKOSH_TEST_DATABASE_URL',
	'MAKOSH_LOCAL_API_SECRET',
	'DATABASE_URL'
];
const backendTestEnvKeyAllowlist = new Set(['backend/tests/config.rs']);

function normalizePath(filePath) {
	return filePath.split(path.sep).join('/');
}

function isIgnored(relativePath) {
	return [...ignoredSegments].some(
		(segment) => relativePath === segment || relativePath.startsWith(`${segment}/`)
	);
}

function isDocFile(relativePath) {
	return relativePath.endsWith('.md');
}

function isTestFile(relativePath) {
	return (
		relativePath.includes('/__tests__/') ||
		/(\.|-)(test|spec)\.[cm]?[jt]s$/i.test(relativePath) ||
		/\.boundary\.test\.[cm]?[jt]s$/i.test(relativePath)
	);
}

function isFrontendTemplateFile(relativePath) {
	return (
		relativePath.startsWith('frontend/src/') &&
		(relativePath.endsWith('.vue') || relativePath.endsWith('.html'))
	);
}

function isVueSfcFile(relativePath) {
	return relativePath.startsWith('frontend/src/') && relativePath.endsWith('.vue');
}

async function collectFiles(relativeRoot) {
	if (isIgnored(relativeRoot)) return [];

	const absoluteRoot = path.join(repoRoot, relativeRoot);
	let entries;
	try {
		entries = await readdir(absoluteRoot, { withFileTypes: true });
	} catch {
		return checkedExtensions.has(path.extname(relativeRoot)) ? [relativeRoot] : [];
	}

	const files = [];
	for (const entry of entries) {
		const relativePath = normalizePath(path.join(relativeRoot, entry.name));
		if (isIgnored(relativePath)) continue;

		if (entry.isDirectory()) {
			files.push(...(await collectFiles(relativePath)));
			continue;
		}

		if (entry.isFile() && checkedExtensions.has(path.extname(entry.name))) {
			files.push(relativePath);
		}
	}
	return files;
}

async function gitLsFiles() {
	const { stdout } = await execFileAsync('git', ['ls-files'], { cwd: repoRoot });
	return stdout.split('\n').filter(Boolean);
}

async function checkTrackedGeneratedFiles() {
	const trackedFiles = await gitLsFiles();
	for (const file of trackedFiles) {
		if (generatedPrefixes.some((prefix) => file.startsWith(prefix)) && file !== 'docker/data/.gitkeep') {
			failures.push(`${file}: generated/local-state file is tracked`);
		}
	}
}

async function checkSourceFiles() {
	const files = (await Promise.all(scanRoots.map(collectFiles))).flat();

	for (const file of files) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		const lines = source.split('\n');

		for (const [index, line] of lines.entries()) {
			const location = `${file}:${index + 1}`;

			if (!isDocFile(file) && !isTestFile(file) && secretPattern.test(line)) {
				failures.push(`${location}: possible hardcoded secret-like value`);
			}

			if (
				(file.startsWith('backend/tests/') || file.startsWith('crates/testkit/src/')) &&
				!backendTestEnvKeyAllowlist.has(file)
			) {
				for (const forbiddenKey of forbiddenBackendTestEnvKeys) {
					if (line.includes(forbiddenKey)) {
						failures.push(
							`${location}: backend tests must use typed test fixtures, not ${forbiddenKey}`
						);
					}
				}
			}

			for (const { pattern, message } of blanketSuppressions) {
				if (pattern.test(line)) {
					failures.push(`${location}: ${message}`);
				}
			}

			if (isFrontendTemplateFile(file) && /\sstyle\s*=/.test(line)) {
				failures.push(`${location}: inline style attributes are forbidden; move styles to CSS files`);
			}

			if (isFrontendTemplateFile(file) && !isVueSfcFile(file) && /<style(\s|>)/i.test(line)) {
				failures.push(`${location}: embedded style blocks are forbidden; move styles to CSS files`);
			}
		}
	}
}

async function main() {
	await checkTrackedGeneratedFiles();
	await checkSourceFiles();

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Code boundary guard passed.');
}

await main();
```

### `scripts/clean-data.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/clean-data.sh`
- Size bytes / Размер в байтах: `587`
- Included characters / Включено символов: `587`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
confirm_or_exit "This will delete local PostgreSQL data under $(postgres_data_dir)." "DELETE"
compose_cmd down --remove-orphans >/dev/null 2>&1 || true
rm -rf "$(postgres_data_dir)"
mkdir -p "$(postgres_data_dir)"
success "Deleted PostgreSQL development data"
```

### `scripts/clean-vault.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/clean-vault.sh`
- Size bytes / Размер в байтах: `439`
- Included characters / Включено символов: `439`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"

load_makosh_env
confirm_or_exit "This will delete local vault data under $MAKOSH_HOST_VAULT_HOME." "DELETE"
rm -rf "$MAKOSH_HOST_VAULT_HOME"
success "Deleted local vault data at $MAKOSH_HOST_VAULT_HOME"
```

### `scripts/clean.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/clean.sh`
- Size bytes / Размер в байтах: `709`
- Included characters / Включено символов: `709`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

info "Removing build artifacts, temporary files, and logs"
rm -rf "$CARGO_TARGET_ROOT"
rm -rf "$REPO_ROOT/frontend/src-tauri/target"
rm -rf "$REPO_ROOT/frontend/node_modules/.vite"
rm -rf "$REPO_ROOT/frontend/node_modules/.vite-temp"
rm -rf "$REPO_ROOT/frontend/dist"
rm -rf "$REPO_ROOT/frontend/build"
rm -f "$REPO_ROOT"/frontend/src-tauri/binaries/makosh-backend-*
rm -rf "$LOG_ROOT"
rm -rf "$REPO_ROOT/tmp/makosh"
find "$REPO_ROOT" -maxdepth 1 -type f -name '*.log' -delete
success "Clean completed without deleting database data"
```

### `scripts/dev.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/dev.sh`
- Size bytes / Размер в байтах: `3995`
- Included characters / Включено символов: `3995`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
ensure_frontend_dependencies
ensure_bacon_available
ensure_command cargo
ensure_command curl
postgres_up

require_port_free "$MAKOSH_BACKEND_PORT" "Backend"
require_port_free "$MAKOSH_FRONTEND_PORT" "Frontend"

ensure_dir "$LOG_ROOT"
flow_id="dev-$(timestamp_compact_utc)-$$"
session_log="$LOG_ROOT/$flow_id"
ensure_dir "$session_log"
live_log="$session_log/live.log"
current_log_link="$LOG_ROOT/current"
rm -f "$current_log_link"
ln -s "$session_log" "$current_log_link"

child_pids=()
pipe_paths=()
logger_pids=()
RUN_SERVICE_PID=""

cleanup() {
	local status="$?"
	local pid
	trap - EXIT INT TERM
	for pid in "${child_pids[@]:-}"; do
		kill "$pid" 2>/dev/null || true
	done
	for pid in "${child_pids[@]:-}"; do
		wait "$pid" 2>/dev/null || true
	done
	for pid in "${logger_pids[@]:-}"; do
		kill "$pid" 2>/dev/null || true
	done
	for pid in "${logger_pids[@]:-}"; do
		wait "$pid" 2>/dev/null || true
	done
	local pipe_path
	for pipe_path in "${pipe_paths[@]:-}"; do
		rm -f "$pipe_path"
	done
	exit "$status"
}
trap cleanup EXIT INT TERM

run_service() {
	local service="$1"
	local color="$2"
	shift 2
	local stdout_pipe="$session_log/$service.stdout.pipe"
	local stderr_pipe="$session_log/$service.stderr.pipe"
	local log_file="$session_log/$service.jsonl"
	mkfifo "$stdout_pipe" "$stderr_pipe"
	pipe_paths+=("$stdout_pipe" "$stderr_pipe")

	"$@" >"$stdout_pipe" 2>"$stderr_pipe" &
	local service_pid="$!"
	child_pids+=("$service_pid")

	stream_service_pipe "$stdout_pipe" "$service" "$service_pid" "info" "$flow_id" "$color" "$log_file" "$live_log" &
	logger_pids+=("$!")
	stream_service_pipe "$stderr_pipe" "$service" "$service_pid" "warn" "$flow_id" "$color" "$log_file" "$live_log" &
	logger_pids+=("$!")

	RUN_SERVICE_PID="$service_pid"
}

export DATABASE_URL
export MAKOSH_LOCAL_API_SECRET
export MAKOSH_DEV_MODE
export MAKOSH_VAULT_HOME
export MAKOSH_DEV_KEY_PATH
export MAKOSH_SECRET_VAULT_KEY
export MAKOSH_HTTP_ADDR="$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT"
export MAKOSH_FLOW_ID="$flow_id"
export MAKOSH_LOG_FORMAT="json"
export RUST_LOG="${RUST_LOG:-info}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CARGO_DEV_TARGET_DIR}"
export VITE_MAKOSH_API_BASE_URL="http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT"
export VITE_MAKOSH_LOCAL_API_SECRET="$MAKOSH_LOCAL_API_SECRET"

run_service backend "$color_cyan" bash -lc "cd '$REPO_ROOT' && exec bacon --headless backend-dev"
backend_pid="$RUN_SERVICE_PID"
info "Waiting for backend health check"
wait_for_service_http "$backend_pid" "http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT/healthz" "Backend healthz" "$MAKOSH_BACKEND_STARTUP_ATTEMPTS" "$MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS"
info "Waiting for backend readiness check"
wait_for_service_http "$backend_pid" "http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT/readyz" "Backend readyz" "$MAKOSH_BACKEND_STARTUP_ATTEMPTS" "$MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS"

run_service frontend "$color_green" bash -lc "cd '$REPO_ROOT/frontend' && exec pnpm dev --host '$MAKOSH_FRONTEND_BIND' --port '$MAKOSH_FRONTEND_PORT' --strictPort"
frontend_pid="$RUN_SERVICE_PID"
info "Waiting for frontend dev server"
wait_for_service_http "$frontend_pid" "http://$MAKOSH_FRONTEND_BIND:$MAKOSH_FRONTEND_PORT" "Frontend Vite" "$MAKOSH_FRONTEND_STARTUP_ATTEMPTS" "$MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS"

info "Flow ID: $flow_id"
info "Logs: $session_log"
info "Live log: $current_log_link/live.log"
printf '%s\n' "PostgreSQL:"
postgres_status
printf '%s\n' "Backend:  http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT (pid $backend_pid)"
printf '%s\n' "Frontend: http://$MAKOSH_FRONTEND_BIND:$MAKOSH_FRONTEND_PORT (pid $frontend_pid)"

wait "$backend_pid" "$frontend_pid"
```

### `scripts/lib/common.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/lib/common.sh`
- Size bytes / Размер в байтах: `4678`
- Included characters / Включено символов: `4678`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="$(cd "$COMMON_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"
LOG_ROOT="$REPO_ROOT/.local/dev-logs"
BACKUPS_ROOT="$REPO_ROOT/backups"
TOOLS_ROOT="$REPO_ROOT/.local/tools"
TOOLS_BIN="$TOOLS_ROOT/bin"
CARGO_TARGET_ROOT="${CARGO_TARGET_ROOT:-$REPO_ROOT/target}"
CARGO_DEV_TARGET_DIR="${CARGO_DEV_TARGET_DIR:-$CARGO_TARGET_ROOT/dev}"
CARGO_VALIDATE_TARGET_DIR="${CARGO_VALIDATE_TARGET_DIR:-$CARGO_TARGET_ROOT/validate}"
CARGO_BUILD_TARGET_DIR="${CARGO_BUILD_TARGET_DIR:-$CARGO_TARGET_ROOT/build}"

color_reset=$'\033[0m'
color_blue=$'\033[34m'
color_green=$'\033[32m'
color_yellow=$'\033[33m'
color_red=$'\033[31m'
color_cyan=$'\033[36m'
color_dim=$'\033[2m'

now_utc() {
	date -u +"%Y-%m-%dT%H:%M:%SZ"
}

today_utc() {
	date -u +"%Y-%m-%d"
}

timestamp_compact_utc() {
	date -u +"%Y%m%dT%H%M%SZ"
}

status_line() {
	local color="$1"
	local label="$2"
	local message="$3"
	printf '%b[%s]%b %s\n' "$color" "$label" "$color_reset" "$message"
}

info() {
	status_line "$color_blue" "info" "$1"
}

success() {
	status_line "$color_green" "ok" "$1"
}

warn() {
	status_line "$color_yellow" "warn" "$1"
}

error() {
	status_line "$color_red" "error" "$1" >&2
}

dim() {
	printf '%b%s%b\n' "$color_dim" "$1" "$color_reset"
}

ensure_dir() {
	mkdir -p "$1"
}

prepend_tools_bin_to_path() {
	case ":$PATH:" in
		*":$TOOLS_BIN:"*) ;;
		*) export PATH="$TOOLS_BIN:$PATH" ;;
	esac
}

ensure_command() {
	local command_name="$1"
	if ! command -v "$command_name" >/dev/null 2>&1; then
		error "Required command not found: $command_name"
		exit 1
	fi
}

ensure_one_of() {
	local first="$1"
	local second="$2"
	if command -v "$first" >/dev/null 2>&1; then
		printf '%s\n' "$first"
		return 0
	fi
	if command -v "$second" >/dev/null 2>&1; then
		printf '%s\n' "$second"
		return 0
	fi
	error "Required command not found: need $first or $second"
	exit 1
}

require_port_free() {
	local port="$1"
	local label="$2"
	if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
		error "$label port $port is already in use."
		exit 1
	fi
}

confirm_or_exit() {
	local prompt="$1"
	local expected="${2:-DELETE}"
	local answer
	printf '%s Type %s to continue: ' "$prompt" "$expected"
	read -r answer
	if [ "$answer" != "$expected" ]; then
		error "Confirmation did not match. Aborting."
		exit 1
	fi
}

json_escape() {
	local value="$1"
	value=${value//\\/\\\\}
	value=${value//\"/\\\"}
	value=${value//$'\t'/\\t}
	value=${value//$'\r'/\\r}
	printf '%s' "$value"
}

emit_json_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '{"timestamp":"%s","service":"%s","pid":%s,"level":"%s","flow_id":"%s","message":"%s"}\n' \
		"$(now_utc)" \
		"$(json_escape "$service")" \
		"$pid" \
		"$(json_escape "$level")" \
		"$(json_escape "$flow_id")" \
		"$(json_escape "$message")" >>"$file_path"
}

emit_live_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '[%s] service=%s pid=%s level=%s flow_id=%s %s\n' \
		"$(now_utc)" \
		"$service" \
		"$pid" \
		"$level" \
		"$flow_id" \
		"$message" >>"$file_path"
}

stream_service_pipe() {
	local pipe_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local color="$6"
	local log_file="$7"
	local live_log_file="$8"
	local line
	while IFS= read -r line || [ -n "$line" ]; do
		emit_json_log "$log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		emit_live_log "$live_log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		printf '%b[%s]%b %s\n' "$color" "$service" "$color_reset" "$line"
	done <"$pipe_path"
}

wait_for_http() {
	local url="$1"
	local label="$2"
	local attempts="${3:-60}"
	local sleep_seconds="${4:-1}"
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if curl --silent --show-error --fail "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep "$sleep_seconds"
		index=$((index + 1))
	done
	error "$label did not become ready: $url"
	return 1
}

wait_for_service_http() {
	local pid="$1"
	local url="$2"
	local label="$3"
	local attempts="${4:-60}"
	local sleep_seconds="${5:-1}"
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if ! kill -0 "$pid" >/dev/null 2>&1; then
			error "$label failed because the service process exited before readiness: pid=$pid"
			return 1
		fi
		if curl --silent --show-error --fail "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep "$sleep_seconds"
		index=$((index + 1))
	done
	error "$label did not become ready: $url"
	return 1
}
```

### `scripts/lib/env.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/lib/env.sh`
- Size bytes / Размер в байтах: `2697`
- Included characters / Включено символов: `2697`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

DOCKER_ENV_FILE="$REPO_ROOT/docker/.env"
DOCKER_ENV_TEMPLATE="$REPO_ROOT/docker/.env.example"

ensure_docker_env_file() {
	if [ ! -f "$DOCKER_ENV_FILE" ]; then
		cp "$DOCKER_ENV_TEMPLATE" "$DOCKER_ENV_FILE"
		warn "Created docker/.env from docker/.env.example. Review local secrets before continuing."
	fi
}

load_makosh_env() {
	prepend_tools_bin_to_path
	ensure_docker_env_file
	set -a
	# shellcheck disable=SC1090
	. "$DOCKER_ENV_FILE"
	set +a

	: "${MAKOSH_POSTGRES_DB:=makosh_hub}"
	: "${MAKOSH_POSTGRES_USER:=makosh}"
	: "${MAKOSH_POSTGRES_PASSWORD:=change-me-local-dev-only}"
	: "${MAKOSH_POSTGRES_BIND:=127.0.0.1}"
	: "${MAKOSH_POSTGRES_PORT:=30432}"
	: "${MAKOSH_BACKEND_BIND:=127.0.0.1}"
	: "${MAKOSH_BACKEND_PORT:=8080}"
	: "${MAKOSH_BACKEND_STARTUP_ATTEMPTS:=300}"
	: "${MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${MAKOSH_FRONTEND_BIND:=127.0.0.1}"
	: "${MAKOSH_FRONTEND_PORT:=5174}"
	: "${MAKOSH_FRONTEND_STARTUP_ATTEMPTS:=120}"
	: "${MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${MAKOSH_LOCAL_API_SECRET:=change-me-local-api-secret}"
	: "${MAKOSH_DEV_MODE:=true}"
	: "${MAKOSH_HOST_VAULT_HOME:=$HOME/.makosh/vault}"
	: "${MAKOSH_SECRET_VAULT_KEY:=change-me-local-secret-vault-key}"
	: "${MAKOSH_OLLAMA_BASE_URL:=http://127.0.0.1:11434}"
	: "${MAKOSH_OLLAMA_CHAT_MODEL:=qwen3:4b}"
	: "${MAKOSH_OLLAMA_EMBED_MODEL:=qwen3-embedding:4b}"
	: "${MAKOSH_OLLAMA_TIMEOUT_SECONDS:=120}"

	MAKOSH_VAULT_HOME="$MAKOSH_HOST_VAULT_HOME"
	MAKOSH_DEV_KEY_PATH="$MAKOSH_HOST_VAULT_HOME/dev/master.key"
	DATABASE_URL="postgres://${MAKOSH_POSTGRES_USER}:${MAKOSH_POSTGRES_PASSWORD}@127.0.0.1:${MAKOSH_POSTGRES_PORT}/${MAKOSH_POSTGRES_DB}"
	MAKOSH_NATS_SERVER_URL="${MAKOSH_NATS_SERVER_URL:-nats://127.0.0.1:${MAKOSH_NATS_PORT:-34222}}"

	export MAKOSH_VAULT_HOME
	export MAKOSH_DEV_KEY_PATH
	export DATABASE_URL
	export MAKOSH_NATS_SERVER_URL
}

ensure_bacon_available() {
	prepend_tools_bin_to_path
	if command -v bacon >/dev/null 2>&1; then
		return 0
	fi

	ensure_command cargo
	ensure_dir "$TOOLS_ROOT"
	info "Installing local bacon into $TOOLS_ROOT"
	cargo install --locked --root "$TOOLS_ROOT" bacon
	prepend_tools_bin_to_path

	if ! command -v bacon >/dev/null 2>&1; then
		error "bacon installation completed but binary was not found in $TOOLS_BIN"
		exit 1
	fi
}

ensure_frontend_dependencies() {
	ensure_command pnpm
	if [ ! -d "$REPO_ROOT/frontend/node_modules" ] || [ ! -x "$REPO_ROOT/frontend/node_modules/.bin/tauri" ]; then
		info "Installing frontend dependencies"
		(
			cd "$REPO_ROOT/frontend"
			pnpm install --frozen-lockfile
		)
	fi
}
```

### `scripts/lib/postgres.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/lib/postgres.sh`
- Size bytes / Размер в байтах: `1259`
- Included characters / Включено символов: `1259`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"
# shellcheck source=./env.sh
source "$COMMON_DIR/env.sh"

compose_cmd() {
	docker compose \
		--env-file "$DOCKER_ENV_FILE" \
		--project-directory "$REPO_ROOT/docker" \
		-f "$REPO_ROOT/docker/docker-compose.yml" \
		"$@"
}

ensure_postgres_runtime_dependencies() {
	ensure_command docker
}

ensure_postgres_client_dependencies() {
	ensure_postgres_runtime_dependencies
	ensure_command psql
}

postgres_up() {
	load_makosh_env
	ensure_postgres_runtime_dependencies
	info "Starting PostgreSQL container"
	compose_cmd up -d --wait postgres
	wait_for_postgres
}

wait_for_postgres() {
	local attempts=60
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if compose_cmd exec -T postgres sh -lc \
			'pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"' >/dev/null 2>&1; then
			return 0
		fi
		sleep 1
		index=$((index + 1))
	done
	error "PostgreSQL did not become ready on 127.0.0.1:$MAKOSH_POSTGRES_PORT"
	exit 1
}

postgres_status() {
	load_makosh_env
	compose_cmd ps postgres
}

postgres_stop() {
	load_makosh_env
	compose_cmd stop postgres
}

postgres_data_dir() {
	printf '%s\n' "$REPO_ROOT/docker/data/postgres"
}
```

### `scripts/lib/resources.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/lib/resources.sh`
- Size bytes / Размер в байтах: `5465`
- Included characters / Включено символов: `5465`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

prepare_backend_sidecar_macos() {
	local binary_root="$REPO_ROOT/frontend/src-tauri/binaries"
	local backend_manifest="$REPO_ROOT/backend/Cargo.toml"
	local backend_bin="makosh-backend"
	local target_root target_triple source_bin target_bin

	if [ "$(uname -s)" != "Darwin" ]; then
		info "Skipping macOS backend sidecar preparation on non-macOS host"
		return 0
	fi

	case "$(uname -m)" in
		arm64)
			target_triple="${MAKOSH_MACOS_TARGET_TRIPLE:-aarch64-apple-darwin}"
			;;
		x86_64)
			target_triple="${MAKOSH_MACOS_TARGET_TRIPLE:-x86_64-apple-darwin}"
			;;
		*)
			error "Unsupported macOS architecture: $(uname -m)"
			exit 1
			;;
	esac

	target_root="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
	source_bin="$target_root/$target_triple/release/$backend_bin"
	target_bin="$binary_root/$backend_bin-$target_triple"

	cargo build \
		--manifest-path "$backend_manifest" \
		--bin "$backend_bin" \
		--release \
		--target "$target_triple"

	if [ ! -f "$source_bin" ]; then
		error "Backend sidecar build completed, but $source_bin was not found."
		exit 1
	fi

	mkdir -p "$binary_root"
	cp "$source_bin" "$target_bin"
	chmod 0755 "$target_bin"
	success "Prepared bundled backend sidecar: $target_bin"
}

prepare_google_oauth_resource() {
	local resource_root="$REPO_ROOT/frontend/src-tauri/resources/google-oauth"
	local target_json="$resource_root/client_secret.json"
	local source_json
	source_json="${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE:-${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}}"

	if [ -z "$source_json" ]; then
		error "Unable to prepare bundled Google OAuth Desktop client resource. Set MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH."
		exit 1
	fi
	if [ ! -f "$source_json" ]; then
		error "Google OAuth client config file was not found: $source_json"
		exit 1
	fi

	node - "$source_json" <<'NODE'
const fs = require('fs');
const sourcePath = process.argv[2];
const raw = fs.readFileSync(sourcePath, 'utf8');
const parsed = JSON.parse(raw);
if (!parsed || typeof parsed !== 'object' || !parsed.installed || typeof parsed.installed !== 'object') {
	throw new Error('Google OAuth bundle config must be a Desktop app JSON with top-level "installed".');
}
for (const field of ['client_id', 'auth_uri', 'token_uri']) {
	if (typeof parsed.installed[field] !== 'string' || parsed.installed[field].trim() === '') {
		throw new Error(`Google OAuth Desktop client JSON is missing required field: installed.${field}`);
	}
}
NODE

	mkdir -p "$resource_root"
	cp "$source_json" "$target_json"
	chmod 0644 "$target_json"
	success "Prepared bundled Google OAuth Desktop client resource: $target_json"
}

prepare_tdlib_macos() {
	local resource_root="$REPO_ROOT/frontend/src-tauri/resources/tdlib"
	local platform_dir target_dir target_lib source_lib

	if [ "$(uname -s)" != "Darwin" ]; then
		info "Skipping macOS TDLib resource preparation on non-macOS host"
		return 0
	fi

	case "$(uname -m)" in
		arm64)
			platform_dir="${MAKOSH_TDLIB_MACOS_PLATFORM_DIR:-macos-arm64}"
			;;
		x86_64)
			platform_dir="${MAKOSH_TDLIB_MACOS_PLATFORM_DIR:-macos-x64}"
			;;
		*)
			error "Unsupported macOS architecture: $(uname -m)"
			exit 1
			;;
	esac

	target_dir="$resource_root/$platform_dir"
	target_lib="$target_dir/libtdjson.dylib"
	source_lib="$(find_tdjson_source_lib || true)"

	if [ -z "$source_lib" ] && [ "${MAKOSH_TDLIB_BUILD_FROM_SOURCE:-0}" = "1" ]; then
		source_lib="$(build_tdlib_from_source || true)"
	fi
	if [ -z "$source_lib" ] || [ ! -f "$source_lib" ]; then
		error "Unable to find libtdjson.dylib. Set MAKOSH_TDJSON_SOURCE/MAKOSH_TDJSON_PATH or install tdlib."
		exit 1
	fi

	mkdir -p "$target_dir"
	cp "$source_lib" "$target_lib"
	chmod 0644 "$target_lib"
	success "Prepared bundled TDLib runtime: $target_lib"
}

find_tdjson_source_lib() {
	if [ -n "${MAKOSH_TDJSON_SOURCE:-}" ]; then
		printf '%s\n' "$MAKOSH_TDJSON_SOURCE"
		return 0
	fi
	if [ -n "${MAKOSH_TDJSON_PATH:-}" ]; then
		printf '%s\n' "$MAKOSH_TDJSON_PATH"
		return 0
	fi
	if command -v brew >/dev/null 2>&1; then
		local brew_prefix
		if brew_prefix="$(brew --prefix tdlib 2>/dev/null)"; then
			printf '%s\n' "$brew_prefix/lib/libtdjson.dylib"
			return 0
		fi
	fi
	if [ -f /opt/homebrew/lib/libtdjson.dylib ]; then
		printf '%s\n' /opt/homebrew/lib/libtdjson.dylib
		return 0
	fi
	if [ -f /usr/local/lib/libtdjson.dylib ]; then
		printf '%s\n' /usr/local/lib/libtdjson.dylib
		return 0
	fi
	return 1
}

build_tdlib_from_source() {
	local build_root source_dir build_dir tdlib_ref built_lib
	build_root="${MAKOSH_TDLIB_BUILD_ROOT:-$REPO_ROOT/.local/tdlib-build}"
	source_dir="$build_root/td"
	build_dir="$source_dir/build"
	tdlib_ref="${MAKOSH_TDLIB_REF:-master}"

	ensure_command git
	ensure_command cmake

	mkdir -p "$build_root"
	if [ ! -d "$source_dir/.git" ]; then
		git clone https://github.com/tdlib/td.git "$source_dir"
	fi

	git -C "$source_dir" fetch --tags origin
	git -C "$source_dir" checkout "$tdlib_ref"
	cmake -S "$source_dir" -B "$build_dir" -DCMAKE_BUILD_TYPE=Release -DTD_ENABLE_JNI=OFF
	cmake --build "$build_dir" --target tdjson --config Release --parallel "$(sysctl -n hw.ncpu)"

	built_lib="$(find "$build_dir" -type f -name libtdjson.dylib -print -quit)"
	if [ -z "$built_lib" ]; then
		error "TDLib source build completed, but libtdjson.dylib was not found under $build_dir."
		exit 1
	fi

	printf '%s\n' "$built_lib"
}
```

### `scripts/lib/rust-tooling.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/lib/rust-tooling.sh`
- Size bytes / Размер в байтах: `524`
- Included characters / Включено символов: `524`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

require_cargo_subcommand() {
	local subcommand="$1"
	local install_hint="$2"

	if cargo "$subcommand" --version >/dev/null 2>&1; then
		return 0
	fi

	echo "Missing cargo subcommand: cargo ${subcommand}" >&2
	echo "Install it with: ${install_hint}" >&2
	exit 1
}

require_binary() {
	local binary="$1"
	local install_hint="$2"

	if command -v "$binary" >/dev/null 2>&1; then
		return 0
	fi

	echo "Missing binary: ${binary}" >&2
	echo "Install it with: ${install_hint}" >&2
	exit 1
}
```

### `scripts/logs.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/logs.sh`
- Size bytes / Размер в байтах: `652`
- Included characters / Включено символов: `652`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

current_log_dir="$LOG_ROOT/current"
live_log="$current_log_dir/live.log"
follow_logs="${MAKOSH_LOGS_FOLLOW:-1}"

if [ ! -L "$current_log_dir" ] && [ ! -d "$current_log_dir" ]; then
	error "No active dev log session found at $current_log_dir. Run make dev first."
	exit 1
fi

if [ ! -f "$live_log" ]; then
	error "Live log file not found: $live_log"
	exit 1
fi

info "Streaming $live_log"
if [ "$follow_logs" = "0" ]; then
	tail -n 50 "$live_log"
else
	tail -n 50 -f "$live_log"
fi
```

### `scripts/migrate.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/migrate.sh`
- Size bytes / Размер в байтах: `621`
- Included characters / Включено символов: `621`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
ensure_command cargo
postgres_up

info "Running backend-managed SQLx migrations"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CARGO_DEV_TARGET_DIR}" \
	MAKOSH_LOG_FORMAT=plain \
	cargo run --manifest-path "$REPO_ROOT/backend/Cargo.toml" --bin makosh_migrate
success "Migrations applied successfully"
```

### `scripts/vault-backup.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/vault-backup.sh`
- Size bytes / Размер в байтах: `2199`
- Included characters / Включено символов: `2199`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
ensure_postgres_client_dependencies
ensure_command pg_dump
postgres_up

backup_day="$(today_utc)"
backup_stamp="$(timestamp_compact_utc)"
backup_dir="$BACKUPS_ROOT/$backup_day/$backup_stamp"
vault_target="$backup_dir/vault"
postgres_dump="$backup_dir/postgres.sql"
manifest_path="$backup_dir/manifest.json"
notes_path="$backup_dir/RESTORE.txt"

ensure_dir "$vault_target"

info "Creating PostgreSQL dump"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" pg_dump \
	--host 127.0.0.1 \
	--port "$MAKOSH_POSTGRES_PORT" \
	--username "$MAKOSH_POSTGRES_USER" \
	--dbname "$MAKOSH_POSTGRES_DB" \
	--no-owner \
	--no-privileges \
	--file "$postgres_dump"

vault_present=false
if [ -d "$MAKOSH_HOST_VAULT_HOME" ]; then
	vault_present=true
	info "Copying vault data from $MAKOSH_HOST_VAULT_HOME"
	cp -R "$MAKOSH_HOST_VAULT_HOME"/. "$vault_target"/
else
	warn "Vault directory does not exist yet: $MAKOSH_HOST_VAULT_HOME"
fi

git_revision="unknown"
if git -C "$REPO_ROOT" rev-parse --short HEAD >/dev/null 2>&1; then
	git_revision="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
fi

cat >"$manifest_path" <<EOF
{
  "created_at": "$(now_utc)",
  "backup_dir": "$(json_escape "$backup_dir")",
  "git_revision": "$(json_escape "$git_revision")",
  "database": {
    "name": "$(json_escape "$MAKOSH_POSTGRES_DB")",
    "user": "$(json_escape "$MAKOSH_POSTGRES_USER")",
    "host": "127.0.0.1",
    "port": $MAKOSH_POSTGRES_PORT,
    "dump_file": "postgres.sql"
  },
  "vault": {
    "source_path": "$(json_escape "$MAKOSH_HOST_VAULT_HOME")",
    "relative_path": "vault",
    "present": $vault_present
  }
}
EOF

cat >"$notes_path" <<EOF
Макошь backup created at $(now_utc)

Contents:
- postgres.sql: logical PostgreSQL dump for $MAKOSH_POSTGRES_DB
- vault/: host vault data snapshot
- manifest.json: backup metadata

Restore with:
  make vault-restore
EOF

success "Backup created: $backup_dir"
```

### `scripts/vault-restore.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/vault-restore.sh`
- Size bytes / Размер в байтах: `2488`
- Included characters / Включено символов: `2488`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
ensure_postgres_client_dependencies
ensure_command dropdb
ensure_command createdb
postgres_up

if [ ! -d "$BACKUPS_ROOT" ]; then
	error "No backups directory found at $BACKUPS_ROOT"
	exit 1
fi

backup_dirs=()
while IFS= read -r backup_dir; do
	backup_dirs+=("$backup_dir")
done <<EOF
$(find "$BACKUPS_ROOT" -mindepth 2 -maxdepth 2 -type d | sort)
EOF

if [ "${#backup_dirs[@]}" -eq 0 ]; then
	error "No backups available under $BACKUPS_ROOT"
	exit 1
fi

printf '%s\n' "Available backups:"
select selected_backup in "${backup_dirs[@]}"; do
	if [ -n "${selected_backup:-}" ]; then
		break
	fi
	warn "Invalid selection."
done

postgres_dump="$selected_backup/postgres.sql"
vault_source="$selected_backup/vault"
manifest_path="$selected_backup/manifest.json"

if [ ! -f "$postgres_dump" ] || [ ! -f "$manifest_path" ] || [ ! -d "$vault_source" ]; then
	error "Backup is incomplete: required files are missing in $selected_backup"
	exit 1
fi

confirm_or_exit "Restore will replace database $MAKOSH_POSTGRES_DB and vault path $MAKOSH_HOST_VAULT_HOME." "RESTORE"

info "Recreating PostgreSQL database $MAKOSH_POSTGRES_DB"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	-d postgres \
	-v ON_ERROR_STOP=1 \
	-c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$MAKOSH_POSTGRES_DB' AND pid <> pg_backend_pid();" >/dev/null
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" dropdb \
	--if-exists \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	"$MAKOSH_POSTGRES_DB"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" createdb \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	"$MAKOSH_POSTGRES_DB"

info "Restoring PostgreSQL dump"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	-d "$MAKOSH_POSTGRES_DB" \
	-v ON_ERROR_STOP=1 \
	-f "$postgres_dump" >/dev/null

info "Restoring vault data"
rm -rf "$MAKOSH_HOST_VAULT_HOME"
mkdir -p "$MAKOSH_HOST_VAULT_HOME"
cp -R "$vault_source"/. "$MAKOSH_HOST_VAULT_HOME"/

success "Restore completed from $selected_backup"
```

### `scripts/whatsapp-business-cloud-edge-readiness.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-business-cloud-edge-readiness.mjs`
- Size bytes / Размер в байтах: `10353`
- Included characters / Включено символов: `10353`
- Truncated / Обрезано: `no`

```javascript
#!/usr/bin/env node

import fs from 'node:fs'
import http from 'node:http'
import https from 'node:https'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const probeEdge = process.env.MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE === '1'
const probeReadyz = process.env.MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE === '1'
const edgeBaseUrl =
  process.env.MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_URL ?? 'http://127.0.0.1:8787'

const checks = []

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function pass(id, evidence) {
  checks.push({ id, status: 'pass', evidence })
}

function fail(id, evidence) {
  checks.push({ id, status: 'fail', evidence })
}

function requireContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const missing = needles.filter((needle) => !text.includes(needle))
  if (missing.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} contains ${needle}`))
  } else {
    fail(id, missing.map((needle) => `${relativePath} missing ${needle}`))
  }
}

function requireNotContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const present = needles.filter((needle) => text.includes(needle))
  if (present.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} does not contain ${needle}`))
  } else {
    fail(id, present.map((needle) => `${relativePath} still contains ${needle}`))
  }
}

function parseJson(value) {
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

function request(method, pathname, headers = {}, body = '') {
  return new Promise((resolve, reject) => {
    const url = new URL(pathname, edgeBaseUrl)
    const transport = url.protocol === 'https:' ? https : http
    const requestBody = Buffer.from(body)
    const req = transport.request(
      url,
      {
        method,
        headers: {
          ...headers,
          ...(requestBody.length > 0 ? { 'Content-Length': String(requestBody.length) } : {}),
        },
        timeout: 5_000,
      },
      (res) => {
        const chunks = []
        res.on('data', (chunk) => chunks.push(chunk))
        res.on('end', () => {
          resolve({
            statusCode: res.statusCode ?? 0,
            headers: res.headers,
            body: Buffer.concat(chunks).toString('utf8'),
          })
        })
      }
    )
    req.on('timeout', () => {
      req.destroy(new Error(`request timed out: ${method} ${url.href}`))
    })
    req.on('error', reject)
    if (requestBody.length > 0) {
      req.write(requestBody)
    }
    req.end()
  })
}

async function probeLocalEdgeProxy() {
  if (!probeEdge) {
    pass('local_edge_proxy_probe', [
      'local edge probe disabled; set MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1 to probe a running proxy',
    ])
    return
  }

  const health = await request('GET', '/healthz')
  const healthBody = parseJson(health.body)
  if (
    health.statusCode === 200
    && healthBody?.status === 'ok'
    && healthBody?.service === 'makosh-whatsapp-business-cloud-edge-proxy'
  ) {
    pass('local_edge_proxy_healthz', [`${edgeBaseUrl}/healthz returned ok`])
  } else {
    fail('local_edge_proxy_healthz', [
      `${edgeBaseUrl}/healthz returned ${health.statusCode}: ${health.body}`,
    ])
  }

  const manifest = await request('GET', '/manifest')
  const manifestBody = parseJson(manifest.body)
  if (
    manifest.statusCode === 200
    && manifestBody?.public_webhook_path === '/webhooks/whatsapp/business-cloud'
    && manifestBody?.protected_makosh_webhook_path
      === '/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks'
    && manifestBody?.protected_makosh_manifest_path
      === '/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest'
    && manifestBody?.local_auth_header === 'X-Макошь-Secret'
    && manifestBody?.signature_header === 'X-Hub-Signature-256'
    && manifestBody?.payload_policy === 'post_body_is_not_parsed_or_rewritten_by_edge_proxy'
  ) {
    pass('local_edge_proxy_manifest', [`${edgeBaseUrl}/manifest returned the protected forwarding contract`])
  } else {
    fail('local_edge_proxy_manifest', [
      `${edgeBaseUrl}/manifest returned ${manifest.statusCode}: ${manifest.body}`,
    ])
  }

  const unsignedPost = await request(
    'POST',
    '/webhooks/whatsapp/business-cloud',
    { 'Content-Type': 'application/json' },
    '{"entry":[]}'
  )
  const unsignedPostBody = parseJson(unsignedPost.body)
  if (
    unsignedPost.statusCode === 400
    && unsignedPostBody?.error === 'missing_x_hub_signature_256'
  ) {
    pass('local_edge_proxy_rejects_unsigned_post', [
      'unsigned Business Cloud POST is rejected before Макошь forwarding',
    ])
  } else {
    fail('local_edge_proxy_rejects_unsigned_post', [
      `unsigned POST returned ${unsignedPost.statusCode}: ${unsignedPost.body}`,
    ])
  }

  if (!probeReadyz) {
    pass('local_edge_proxy_readyz_probe', [
      'readyz probe disabled; set MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1 when Макошь is running',
    ])
    return
  }

  const readyz = await request('GET', '/readyz')
  if (readyz.statusCode >= 200 && readyz.statusCode < 300) {
    pass('local_edge_proxy_readyz', [`${edgeBaseUrl}/readyz reached protected Макошь manifest`])
  } else {
    fail('local_edge_proxy_readyz', [
      `${edgeBaseUrl}/readyz returned ${readyz.statusCode}: ${readyz.body}`,
    ])
  }
}

requireContains('edge_proxy_public_surface', 'backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs', [
  'PUBLIC_WEBHOOK_PATH: &str = "/webhooks/whatsapp/business-cloud"',
  'PROTECTED_MAKOSH_WEBHOOK_PATH',
  'PROTECTED_MAKOSH_MANIFEST_PATH',
  'MAKOSH_SECRET_HEADER: &str = "X-Макошь-Secret"',
  'BUSINESS_CLOUD_SIGNATURE_HEADER: &str = "X-Hub-Signature-256"',
  '.route("/healthz", get(healthz))',
  '.route("/readyz", get(readyz))',
  '.route("/manifest", get(edge_manifest))',
  'PUBLIC_WEBHOOK_PATH,',
])

requireContains('edge_proxy_forwarding_contract', 'backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs', [
  'forward_hub_query_params_and_optional_account_id_to_protected_makosh',
  'forward_exact_raw_body_and_x_hub_signature_256_to_protected_makosh',
  'post_body_is_not_parsed_or_rewritten_by_edge_proxy',
  'missing_x_hub_signature_256',
  'makosh_url(PROTECTED_MAKOSH_WEBHOOK_PATH, None, false)',
  '.header(MAKOSH_SECRET_HEADER',
  '.header(BUSINESS_CLOUD_SIGNATURE_HEADER',
])

requireContains('edge_proxy_env_boundary', 'backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs', [
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_SECRET',
  'MAKOSH_LOCAL_API_SECRET',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID',
])

requireContains('edge_proxy_behavioral_tests_cover_contract', 'backend/src/bin/makosh_whatsapp_business_cloud_edge_proxy.rs', [
  'readyz_checks_manifest_without_account_scoping',
  'get_webhook_forwards_challenge_query_account_scope_and_local_secret',
  'post_webhook_forwards_raw_body_signature_and_no_account_query',
  'post_webhook_requires_meta_signature_before_forwarding',
  'missing_x_hub_signature_256',
])

requireContains('edge_proxy_signal_hub_static_guard', 'backend/tests/whatsapp_signal_hub.rs', [
  'whatsapp_business_cloud_proxy_manifest_keeps_makosh_protected',
  '/webhooks/whatsapp/business-cloud',
  'readyz_checks_manifest_without_account_scoping',
  'post_webhook_forwards_raw_body_signature_and_no_account_query',
])

requireContains('edge_proxy_compose_profile', 'docker/docker-compose.yml', [
  'whatsapp-business-cloud-edge-proxy:',
  'profiles:',
  'whatsapp-business-cloud-edge',
  'target: whatsapp-business-cloud-edge-proxy',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL',
  'MAKOSH_LOCAL_API_SECRET',
  '127.0.0.1}:${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT:-8787}:8787',
  'curl -fsS http://127.0.0.1:8787/healthz',
])

requireContains('edge_proxy_dockerfile_target', 'docker/Dockerfile', [
  '--bin makosh-whatsapp-business-cloud-edge-proxy',
  'FROM debian:bookworm-slim AS whatsapp-business-cloud-edge-proxy',
  '/usr/local/bin/makosh-whatsapp-business-cloud-edge-proxy',
  'EXPOSE 8787',
])

requireContains('edge_proxy_env_example_is_loopback_and_non_secret', 'docker/.env.example', [
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND=127.0.0.1',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT=8787',
  'MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL=http://host.docker.internal:8080',
  '# MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID=optional-account-scope',
])

requireNotContains('edge_proxy_env_example_has_no_business_cloud_secret_values', 'docker/.env.example', [
  'whatsapp_business_cloud_access_token=',
  'whatsapp_business_cloud_app_secret=',
  'whatsapp_business_cloud_webhook_verify_token=',
])

requireContains('edge_proxy_makefile_targets', 'Makefile', [
  'whatsapp-business-cloud-edge-config:',
  'whatsapp-business-cloud-edge-up:',
  'whatsapp-business-cloud-edge-stop:',
  'whatsapp-business-cloud-edge-logs:',
])

requireContains('edge_proxy_docs_smoke_contract', 'docs/integrations/whatsapp/live-smoke-checklist.md', [
  'Business Cloud edge proxy checks',
  'make whatsapp-business-cloud-edge-config',
  'make whatsapp-business-cloud-edge-up',
  'GET /readyz',
  'Expose only the proxy path `/webhooks/whatsapp/business-cloud`',
  'do not expose Макошь `/api/v1` directly',
  'X-Hub-Signature-256',
])

requireContains('edge_proxy_status_tracks_remaining_public_gate', 'docs/integrations/whatsapp/status.md', [
  'DOMAIN CLOSURE          = not achieved',
  'Business Cloud public exposure/smoke',
  'business cloud edge proxy binary',
  'business cloud edge proxy behavioral contract',
  'business cloud edge proxy compose profile',
])

await probeLocalEdgeProxy().catch((error) => {
  fail('local_edge_proxy_probe_error', [error instanceof Error ? error.message : String(error)])
})

const failed = checks.filter((check) => check.status === 'fail')
const result = {
  ok: failed.length === 0,
  edge_probe: probeEdge,
  readyz_probe: probeReadyz,
  edge_base_url: edgeBaseUrl,
  generated_at: new Date().toISOString(),
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (failed.length > 0) {
  process.exitCode = 1
}
```

### `scripts/whatsapp-domain-closure-audit.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-domain-closure-audit.mjs`
- Size bytes / Размер в байтах: `10123`
- Included characters / Включено символов: `10123`
- Truncated / Обрезано: `no`

```javascript
#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const args = new Set(process.argv.slice(2))
const requireClosed =
  args.has('--require-closed') || process.env.MAKOSH_WHATSAPP_REQUIRE_DOMAIN_CLOSED === '1'
const evidenceDir =
  process.env.MAKOSH_WHATSAPP_DOMAIN_CLOSURE_EVIDENCE_DIR ?? '.local/whatsapp'

const requiredEvidenceShapes = [
  'whatsapp_native_md',
  'whatsapp_web_companion',
  'whatsapp_business_cloud',
]

const checks = []
const blockers = []

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function check(id, status, evidence) {
  checks.push({ id, status, evidence })
}

function pass(id, evidence) {
  check(id, 'pass', evidence)
}

function fail(id, evidence) {
  check(id, 'fail', evidence)
}

function block(id, evidence) {
  blockers.push({ id, evidence })
  check(id, 'blocked', evidence)
}

function requireContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const missing = needles.filter((needle) => !text.includes(needle))
  if (missing.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} contains ${needle}`))
  } else {
    fail(id, missing.map((needle) => `${relativePath} missing ${needle}`))
  }
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function evidenceFiles() {
  const absoluteEvidenceDir = path.isAbsolute(evidenceDir)
    ? evidenceDir
    : path.join(repoRoot, evidenceDir)
  if (!fs.existsSync(absoluteEvidenceDir)) {
    return []
  }
  return fs
    .readdirSync(absoluteEvidenceDir)
    .filter((name) => /^live-smoke-evidence.*\.json$/u.test(name))
    .map((name) => path.join(absoluteEvidenceDir, name))
    .sort()
}

function validateEvidenceFile(filePath) {
  const run = spawnSync(process.execPath, ['scripts/whatsapp-live-smoke-evidence.mjs'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      MAKOSH_WHATSAPP_LIVE_SMOKE_EVIDENCE: filePath,
    },
    encoding: 'utf8',
  })

  let document = null
  try {
    document = JSON.parse(fs.readFileSync(filePath, 'utf8'))
  } catch {
    return {
      ok: false,
      providerShape: '<unparseable>',
      filePath,
      evidence: [`${filePath} is not parseable JSON`],
    }
  }

  const providerShape = isPlainObject(document) ? document.provider_shape : '<missing>'
  if (run.status === 0) {
    return {
      ok: true,
      providerShape,
      filePath,
      evidence: [`${filePath} passed scripts/whatsapp-live-smoke-evidence.mjs`],
    }
  }

  const stdout = run.stdout?.trim()
  const stderr = run.stderr?.trim()
  return {
    ok: false,
    providerShape,
    filePath,
    evidence: [
      `${filePath} failed scripts/whatsapp-live-smoke-evidence.mjs`,
      ...(stdout ? [`stdout: ${stdout.slice(0, 800)}`] : []),
      ...(stderr ? [`stderr: ${stderr.slice(0, 800)}`] : []),
    ],
  }
}

function nativeUnsupportedCommands() {
  const nativeMd = readText('backend/src/integrations/whatsapp/runtime/native_md.rs')
  const match = nativeMd.match(
    /const\s+NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS:\s*&\[\s*&str\s*\]\s*=\s*&\[(?<body>[\s\S]*?)\];/u
  )
  if (!match?.groups?.body) {
    return null
  }
  return Array.from(match.groups.body.matchAll(/"([^"]+)"/gu), (item) => item[1]).sort()
}

function statusClosureState() {
  const status = readText('docs/integrations/whatsapp/status.md')
  if (status.includes('DOMAIN CLOSURE          = achieved')) {
    return 'achieved'
  }
  if (status.includes('DOMAIN CLOSURE          = not achieved')) {
    return 'not_achieved'
  }
  return 'unknown'
}

function adr0101State() {
  const adr = readText('docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md')
  const match = adr.match(/^Status:\s*(?<status>.+)$/mu)
  return match?.groups?.status?.trim() ?? 'unknown'
}

requireContains('static_readiness_targets_exist', 'Makefile', [
  'whatsapp-live-smoke-readiness:',
  'whatsapp-native-md-sdk-gap-readiness:',
  'whatsapp-live-smoke-evidence:',
  'whatsapp-business-cloud-edge-readiness:',
])

requireContains('acceptance_docs_track_current_blockers', 'docs/integrations/whatsapp/status.md', [
  'DOMAIN CLOSURE          = not achieved',
  'manual smoke',
  'remaining safe write APIs',
  'WebView live smoke',
  'Business Cloud public exposure/smoke',
])

requireContains('architecture_guard_contract_exists', 'backend/tests/communications_architecture_target.rs', [
  'whatsapp_provider_runtime_is_replaceable_trait_boundary',
  'runtime/native_md',
  'runtime/business_cloud',
  'domains',
  'engines',
])

requireContains('signal_hub_fixture_contract_exists', 'backend/tests/whatsapp_signal_hub.rs', [
  'sanitized WhatsApp event payload must remove',
  'signal.accepted.whatsapp',
  'provider-observed reconciliation',
  'whatsapp_native_md_unsupported_write_gap_is_explicit_and_structured',
  'unsupported writes',
])

requireContains('manual_smoke_evidence_contract_exists', 'scripts/whatsapp-live-smoke-evidence.mjs', [
  'commonGateIds',
  'personalGateIds',
  'businessCloudGateIds',
  'allowedEvidenceRefPrefixes',
  'requiredEvidenceRefPrefixGroups',
  'evidence_refs',
  'account_fingerprint must be sha256:<64 hex chars>',
  'evidence.${gateId}.status must be passed',
  'weak_reconciliation_refs_fail',
  'placeholder_refs_fail',
])

requireContains('native_md_upgrade_path_context_exists', 'scripts/whatsapp-native-md-sdk-gap-readiness.mjs', [
  'verifyRustAndCrateUpgradeContext()',
  'native_md_rust_baseline_context',
  'native_md_wa_rs_dependency_context',
  'native_md_crates_io_probe',
  'native_md_upgrade_requires_provider_api_not_toolchain_only',
])

requireContains('native_md_upgrade_docs_track_toolchain_limit', 'docs/integrations/whatsapp/status.md', [
  'native Rust/wa-rs upgrade path verifier',
  'Rust/toolchain upgrade is not treated as sufficient evidence',
  'MAKOSH_WA_RS_CRATES_IO_PROBE=1',
])

requireContains('live_smoke_evidence_collector_exists', 'scripts/whatsapp-live-smoke-collect-evidence.mjs', [
  'defaultObservationsPath = \'.local/whatsapp/live-smoke-observations.json\'',
  'whatsapp-live-smoke-evidence.mjs',
  '--observations-template',
  'assertNoSecretLikeContent',
  'mergeEvidence(template, observations)',
])

requireContains('live_smoke_evidence_collector_target_exists', 'Makefile', [
  'whatsapp-live-smoke-collect-evidence:',
  'node scripts/whatsapp-live-smoke-collect-evidence.mjs',
])

requireContains('live_smoke_evidence_collector_docs_exist', 'docs/integrations/whatsapp/live-smoke-checklist.md', [
  'make whatsapp-live-smoke-collect-evidence',
  'normalizer, not a bypass',
  'Gates without operator-provided sanitized',
])

requireContains('adr_0101_acceptance_scope_keeps_live_blocked', 'docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md', [
  'Status: Accepted',
  'Acceptance scope',
  'does not make any WhatsApp live',
  'remain blocked until their live-smoke evidence',
])

const validatedEvidence = new Map()
const invalidEvidence = []
for (const filePath of evidenceFiles()) {
  const result = validateEvidenceFile(filePath)
  if (result.ok && requiredEvidenceShapes.includes(result.providerShape)) {
    validatedEvidence.set(result.providerShape, result)
  } else {
    invalidEvidence.push(result)
  }
}

for (const providerShape of requiredEvidenceShapes) {
  const result = validatedEvidence.get(providerShape)
  if (result) {
    pass(`live_smoke_evidence.${providerShape}`, result.evidence)
  } else {
    const candidates = invalidEvidence
      .filter((item) => item.providerShape === providerShape)
      .flatMap((item) => item.evidence)
    block(`live_smoke_evidence.${providerShape}`, [
      candidates.length > 0
        ? candidates.join('\n')
        : `${evidenceDir}/live-smoke-evidence*.json has no valid ${providerShape} evidence artifact`,
    ])
  }
}

const unsupportedCommands = nativeUnsupportedCommands()
if (unsupportedCommands === null) {
  fail('native_md_unsupported_command_manifest', [
    'NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS manifest was not found',
  ])
} else if (unsupportedCommands.length === 0) {
  pass('native_md_unsupported_command_manifest', [
    'native_md unsupported command manifest is empty',
  ])
} else {
  block('native_md_unsupported_commands_remaining', [
    `native_md still marks unsupported commands: ${unsupportedCommands.join(', ')}`,
  ])
}

const adrState = adr0101State()
if (adrState === 'Accepted') {
  pass('adr_0101_accepted', ['ADR-0101 is Accepted'])
} else {
  block('adr_0101_not_accepted', [`ADR-0101 status is ${adrState}`])
}

const closureState = statusClosureState()
const noFailedChecks = checks.every((item) => item.status !== 'fail')
const closureEvidenceComplete = blockers.length === 0
const closureAchieved =
  noFailedChecks && closureEvidenceComplete && closureState === 'achieved' && adrState === 'Accepted'

if (closureAchieved) {
  pass('docs_status_claims_closure_only_after_evidence', [
    'docs/integrations/whatsapp/status.md claims DOMAIN CLOSURE = achieved and closure evidence is complete',
  ])
} else if (closureState === 'not_achieved') {
  pass('docs_status_does_not_overclaim_closure', [
    'docs/integrations/whatsapp/status.md keeps DOMAIN CLOSURE = not achieved while blockers remain',
  ])
} else if (closureState === 'achieved') {
  fail('docs_status_overclaims_closure', [
    'docs/integrations/whatsapp/status.md claims achieved but closure audit still has blockers',
  ])
} else {
  fail('docs_status_closure_state_unknown', [
    'docs/integrations/whatsapp/status.md must state DOMAIN CLOSURE = achieved or not achieved',
  ])
}

const result = {
  ok: noFailedChecks && (!requireClosed || closureAchieved),
  require_closed: requireClosed,
  closure_achieved: closureAchieved,
  generated_at: new Date().toISOString(),
  evidence_dir: evidenceDir,
  required_evidence_shapes: requiredEvidenceShapes,
  blockers,
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (!result.ok) {
  process.exitCode = 1
}
```

### `scripts/whatsapp-live-smoke-collect-evidence.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-live-smoke-collect-evidence.mjs`
- Size bytes / Размер в байтах: `8679`
- Included characters / Включено символов: `8679`
- Truncated / Обрезано: `no`

```javascript
#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const defaultObservationsPath = '.local/whatsapp/live-smoke-observations.json'
const observationsPath =
  process.env.MAKOSH_WHATSAPP_LIVE_SMOKE_OBSERVATIONS ?? defaultObservationsPath
const providerShapes = new Set([
  'whatsapp_web_companion',
  'whatsapp_native_md',
  'whatsapp_business_cloud',
])
const secretLikePatterns = [
  /session_blob/i,
  /session_material/i,
  /cookie_value/i,
  /raw_secret/i,
  /access_token_value/i,
  /refresh_token_value/i,
  /app_secret_value/i,
  /verify_token_value/i,
  /"authorization"\s*:/i,
  /"cookie"\s*:/i,
  /browser_profile_secret/i,
  /"qr_code"\s*:/i,
  /"pair_code"\s*:/i,
  /"media_key"\s*:/i,
  /"direct_path"\s*:/i,
  /"static_url"\s*:/i,
  /\+\d{7,15}/,
  /\b\d{8,15}@s\.whatsapp\.net\b/i,
]

function argValue(name) {
  const argv = process.argv.slice(2)
  const index = argv.indexOf(name)
  if (index >= 0 && typeof argv[index + 1] === 'string') {
    return argv[index + 1]
  }
  const prefix = `${name}=`
  const inline = argv.find((item) => item.startsWith(prefix))
  return inline ? inline.slice(prefix.length) : null
}

function absolutePath(relativePath) {
  return path.isAbsolute(relativePath) ? relativePath : path.join(repoRoot, relativePath)
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function sha256Fingerprint(value) {
  return `sha256:${createHash('sha256').update(value).digest('hex')}`
}

function providerShapeFrom(document) {
  const value =
    argValue('--provider-shape')
    ?? process.env.MAKOSH_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim()
    ?? document?.provider_shape
  if (!providerShapes.has(value)) {
    throw new Error(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  return value
}

function templateEvidence(providerShape) {
  const result = spawnSync(
    process.execPath,
    [
      'scripts/whatsapp-live-smoke-evidence.mjs',
      '--template',
      '--provider-shape',
      providerShape,
      '--status',
      'pending',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    }
  )
  if (result.status !== 0) {
    throw new Error(
      `failed to render evidence template: ${(result.stderr || result.stdout).trim()}`
    )
  }
  return JSON.parse(result.stdout)
}

function observationsTemplate(providerShape) {
  const now = new Date('2026-06-26T00:00:00.000Z').toISOString()
  return {
    schema_version: 1,
    run_id: 'replace-with-local-run-id',
    generated_at: now,
    provider_shape: providerShape,
    runtime_kind: 'replace-with-runtime-kind',
    account_fingerprint: 'sha256:replace-with-64-hex-account-fingerprint',
    operator_attestation: {
      low_risk_or_test_account: false,
      owner_visible_runtime: false,
      no_hidden_or_headless_runtime: false,
      secrets_not_recorded: false,
      no_direct_domain_mutation: false,
    },
    evidence: {
      'preflight.readiness_target': {
        observed_at: now,
        evidence_refs: ['command:make-whatsapp-live-smoke-readiness-run-id'],
        notes: 'replace with sanitized local command/run reference',
      },
    },
  }
}

function outputPath(providerShape) {
  const explicit = process.env.MAKOSH_WHATSAPP_LIVE_SMOKE_EVIDENCE ?? argValue('--output')
  if (explicit?.trim()) {
    return absolutePath(explicit.trim())
  }
  return absolutePath(`.local/whatsapp/live-smoke-evidence-${providerShape}.json`)
}

function accountFingerprint(observations, providerShape) {
  if (
    typeof observations.account_fingerprint === 'string'
    && /^sha256:[a-f0-9]{64}$/i.test(observations.account_fingerprint)
  ) {
    return observations.account_fingerprint
  }
  const accountId = process.env.MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID?.trim()
  if (accountId) {
    return sha256Fingerprint(`${providerShape}:${accountId}`)
  }
  return observations.account_fingerprint ?? ''
}

function assertNoSecretLikeContent(document) {
  const serialized = JSON.stringify(document)
  for (const pattern of secretLikePatterns) {
    if (pattern.test(serialized)) {
      throw new Error(`observations contain forbidden secret/private marker: ${pattern.source}`)
    }
  }
}

function mergeEvidence(template, observations) {
  const observedEvidence = observations.evidence
  if (!isPlainObject(observedEvidence)) {
    throw new Error('observations.evidence must be an object keyed by gate id')
  }

  // Gates without operator-provided sanitized refs remain pending in the template.
  for (const [gateId, observedGate] of Object.entries(observedEvidence)) {
    if (!isPlainObject(template.evidence[gateId])) {
      throw new Error(`observations.evidence.${gateId} is not a known gate for ${template.provider_shape}`)
    }
    if (!isPlainObject(observedGate)) {
      throw new Error(`observations.evidence.${gateId} must be an object`)
    }
    const refs = Array.isArray(observedGate.evidence_refs)
      ? observedGate.evidence_refs.filter((item) => typeof item === 'string' && item.trim())
      : typeof observedGate.evidence_ref === 'string' && observedGate.evidence_ref.trim()
        ? [observedGate.evidence_ref.trim()]
        : []
    if (refs.length === 0) {
      throw new Error(`observations.evidence.${gateId}.evidence_refs must include sanitized refs`)
    }
    template.evidence[gateId] = {
      status: observedGate.status === 'pending' ? 'pending' : 'passed',
      observed_at: observedGate.observed_at ?? new Date().toISOString(),
      evidence_ref: refs[0],
      evidence_refs: Array.from(new Set(refs.map((item) => item.trim()))),
      notes:
        typeof observedGate.notes === 'string' && observedGate.notes.trim()
          ? observedGate.notes.trim()
          : 'sanitized live-smoke observation reference',
    }
  }
}

function validateEvidence(filePath) {
  const result = spawnSync(process.execPath, ['scripts/whatsapp-live-smoke-evidence.mjs'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      MAKOSH_WHATSAPP_LIVE_SMOKE_EVIDENCE: filePath,
    },
    encoding: 'utf8',
  })
  return {
    ok: result.status === 0,
    status: result.status,
    stdout: result.stdout ? JSON.parse(result.stdout) : null,
    stderr: result.stderr?.trim() ?? '',
  }
}

function writeObservationsTemplate() {
  const providerShape =
    argValue('--provider-shape')
    ?? process.env.MAKOSH_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim()
    ?? 'whatsapp_native_md'
  if (!providerShapes.has(providerShape)) {
    throw new Error(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  console.log(JSON.stringify(observationsTemplate(providerShape), null, 2))
}

function collectEvidence() {
  const observationsFile = absolutePath(observationsPath)
  if (!fs.existsSync(observationsFile)) {
    throw new Error(
      `${observationsPath} does not exist; create sanitized observations or run with --observations-template`
    )
  }

  const observations = readJson(observationsFile)
  assertNoSecretLikeContent(observations)
  const providerShape = providerShapeFrom(observations)
  const template = templateEvidence(providerShape)
  template.run_id = observations.run_id
  template.generated_at = new Date().toISOString()
  template.provider_shape = providerShape
  template.runtime_kind = observations.runtime_kind
  template.account_fingerprint = accountFingerprint(observations, providerShape)
  template.operator_attestation = observations.operator_attestation
  mergeEvidence(template, observations)
  assertNoSecretLikeContent(template)

  const filePath = outputPath(providerShape)
  fs.mkdirSync(path.dirname(filePath), { recursive: true })
  fs.writeFileSync(filePath, `${JSON.stringify(template, null, 2)}\n`)
  const validation = validateEvidence(filePath)
  console.log(
    JSON.stringify(
      {
        ok: validation.ok,
        generated_at: new Date().toISOString(),
        observations_path: observationsPath,
        evidence_path: path.relative(repoRoot, filePath),
        provider_shape: providerShape,
        validation: validation.stdout ?? validation.stderr,
      },
      null,
      2
    )
  )
  if (!validation.ok) {
    process.exitCode = 1
  }
}

try {
  if (process.argv.includes('--observations-template')) {
    writeObservationsTemplate()
  } else {
    collectEvidence()
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error))
  process.exitCode = 1
}
```

### `scripts/whatsapp-live-smoke-evidence.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-live-smoke-evidence.mjs`
- Size bytes / Размер в байтах: `18371`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```javascript
#!/usr/bin/env node

import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const defaultEvidencePath = '.local/whatsapp/live-smoke-evidence.json'
const evidencePath =
  process.env.MAKOSH_WHATSAPP_LIVE_SMOKE_EVIDENCE ?? defaultEvidencePath

const providerShapes = new Set([
  'whatsapp_web_companion',
  'whatsapp_native_md',
  'whatsapp_business_cloud',
])

const commonGateIds = [
  'preflight.readiness_target',
  'preflight.runtime_api_probe',
  'runtime.boundary_provider_shape',
  'runtime.health_sanitized',
  'event_flow.raw_evidence_append_only',
  'event_flow.accepted_signal_events',
  'event_flow.projection_from_event_spine',
  'event_flow.no_direct_domain_writes',
  'commands.no_completion_without_provider_observed_evidence',
  'media.bytes_local_blob_only',
  'media.scanner_default_not_scanned',
  'media.no_clean_without_scanner',
  'redaction.api_responses',
  'redaction.raw_evidence',
  'redaction.event_payloads',
  'redaction.logs',
  'redaction.frontend_payloads',
]

const personalGateIds = [
  'auth.qr_or_pair_code_login',
  'auth.host_vault_session_binding',
  'auth.restart_restore_without_user_action',
  'auth.session_rotation_or_relink',
  'auth.multi_account_isolation',
  'inbound.private_message',
  'inbound.group_message',
  'inbound.reply_or_quote',
  'inbound.forward',
  'inbound.edit',
  'inbound.delete',
  'inbound.receipt',
  'inbound.reaction',
  'inbound.dialog',
  'inbound.participant',
  'inbound.presence',
  'inbound.call_metadata',
  'inbound.status',
  'inbound.status_view',
  'inbound.status_delete',
  'inbound.media_metadata',
  'inbound.media_download_ref',
  'inbound.sync_lifecycle',
  'outbound.send_text',
  'outbound.reply',
  'outbound.forward',
  'outbound.edit',
  'outbound.delete',
  'outbound.react',
  'outbound.unreact',
  'outbound.media_upload',
  'outbound.media_download',
  'outbound.voice_note',
  'outbound.status_publish',
  'outbound.mark_read',
  'outbound.mark_unread',
  'outbound.archive',
  'outbound.unarchive',
  'outbound.mute',
  'outbound.unmute',
  'outbound.pin',
  'outbound.unpin',
  'outbound.join_group',
  'outbound.leave_group',
  'search.message_search',
  'search.media_search',
  'search.participant_search',
  'search.chat_search',
]

const businessCloudGateIds = [
  'business_cloud.host_vault_access_token_binding',
  'business_cloud.host_vault_app_secret_binding',
  'business_cloud.host_vault_verify_token_binding',
  'business_cloud.edge_proxy_public_only_path',
  'business_cloud.edge_proxy_meta_challenge',
  'business_cloud.edge_proxy_signed_webhook',
  'business_cloud.makosh_api_not_public',
  'business_cloud.inbound_message_webhook',
  'business_cloud.receipt_webhook_reconciliation',
  'business_cloud.send_text',
  'business_cloud.send_template',
  'business_cloud.send_media',
  'business_cloud.send_voice_note',
  'business_cloud.rate_limit_retry_hint',
  'business_cloud.not_personal_whatsapp_substitute',
]

const secretLikePatterns = [
  /session_blob/i,
  /session_material/i,
  /cookie_value/i,
  /raw_secret/i,
  /access_token_value/i,
  /refresh_token_value/i,
  /app_secret_value/i,
  /verify_token_value/i,
  /"authorization"\s*:/i,
  /"cookie"\s*:/i,
  /browser_profile_secret/i,
  /"qr_code"\s*:/i,
  /"pair_code"\s*:/i,
  /"media_key"\s*:/i,
  /"direct_path"\s*:/i,
  /"static_url"\s*:/i,
  /\+\d{7,15}/,
  /\b\d{8,15}@s\.whatsapp\.net\b/i,
]

const allowedEvidenceRefPrefixes = [
  'audit:',
  'blob:',
  'command:',
  'doc:',
  'edge_proxy:',
  'event_log:',
  'log_scan:',
  'projection:',
  'raw_record:',
  'runtime_api:',
  'search:',
  'signal_hub:',
  'storage:',
  'ui:',
  'vault_binding:',
]

function requiredGateIds(providerShape) {
  if (providerShape === 'whatsapp_business_cloud') {
    return [...commonGateIds, ...businessCloudGateIds]
  }
  return [...commonGateIds, ...personalGateIds]
}

function requiredEvidenceRefPrefixGroups(providerShape, gateId) {
  const common = {
    'preflight.readiness_target': [['command:']],
    'preflight.runtime_api_probe': [['runtime_api:']],
    'runtime.boundary_provider_shape': [['runtime_api:']],
    'runtime.health_sanitized': [['runtime_api:']],
    'event_flow.raw_evidence_append_only': [['raw_record:']],
    'event_flow.accepted_signal_events': [['event_log:', 'signal_hub:']],
    'event_flow.projection_from_event_spine': [['projection:']],
    'event_flow.no_direct_domain_writes': [['audit:']],
    'commands.no_completion_without_provider_observed_evidence': [
      ['command:'],
      ['event_log:', 'signal_hub:'],
    ],
    'media.bytes_local_blob_only': [['blob:', 'storage:']],
    'media.scanner_default_not_scanned': [['projection:', 'storage:']],
    'media.no_clean_without_scanner': [['audit:', 'projection:']],
    'redaction.api_responses': [['runtime_api:']],
    'redaction.raw_evidence': [['raw_record:']],
    'redaction.event_payloads': [['event_log:', 'signal_hub:']],
    'redaction.logs': [['log_scan:']],
    'redaction.frontend_payloads': [['ui:']],
  }
  if (common[gateId]) {
    return common[gateId]
  }

  if (gateId.startsWith('auth.')) {
    if (gateId === 'auth.host_vault_session_binding') {
      return [['vault_binding:']]
    }
    if (gateId === 'auth.restart_restore_without_user_action') {
      return [['event_log:', 'runtime_api:']]
    }
    return [['runtime_api:', 'event_log:']]
  }
  if (gateId.startsWith('inbound.')) {
    return [['raw_record:'], ['event_log:', 'signal_hub:']]
  }
  if (gateId.startsWith('outbound.')) {
    if (gateId === 'outbound.media_download') {
      return [['command:'], ['event_log:', 'signal_hub:'], ['blob:', 'storage:']]
    }
    return [['command:'], ['event_log:', 'signal_hub:']]
  }
  if (gateId.startsWith('search.')) {
    return [['search:', 'projection:']]
  }

  if (providerShape === 'whatsapp_business_cloud') {
    if (gateId.startsWith('business_cloud.host_vault_')) {
      return [['vault_binding:']]
    }
    if (gateId.startsWith('business_cloud.edge_proxy_')) {
      return [['edge_proxy:']]
    }
    if (gateId === 'business_cloud.makosh_api_not_public') {
      return [['edge_proxy:', 'runtime_api:']]
    }
    if (gateId === 'business_cloud.inbound_message_webhook') {
      return [['event_log:', 'signal_hub:'], ['raw_record:']]
    }
    if (gateId === 'business_cloud.receipt_webhook_reconciliation') {
      return [['command:'], ['event_log:', 'signal_hub:'], ['raw_record:']]
    }
    if (gateId.startsWith('business_cloud.send_')) {
      return [['command:'], ['event_log:', 'signal_hub:']]
    }
    if (gateId === 'business_cloud.rate_limit_retry_hint') {
      return [['command:']]
    }
    if (gateId === 'business_cloud.not_personal_whatsapp_substitute') {
      return [['doc:', 'runtime_api:']]
    }
  }

  return [['audit:', 'doc:', 'event_log:', 'runtime_api:']]
}

function templateEvidenceRefs(providerShape, gateId, status) {
  const prefixGroups = requiredEvidenceRefPrefixGroups(providerShape, gateId)
  if (status !== 'passed') {
    return prefixGroups.map(
      (group) => `${group[0]}replace-with-sanitized-${gateId.replaceAll('.', '-')}-reference`
    )
  }
  return prefixGroups.map((group, index) => {
    const prefix = group[0]
    const suffix = `${providerShape}:${gateId.replaceAll('.', '-')}:${index + 1}`
    return `${prefix}${suffix}`
  })
}

function absolutePath(relativePath) {
  return path.isAbsolute(relativePath) ? relativePath : path.join(repoRoot, relativePath)
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function isIsoDate(value) {
  if (typeof value !== 'string' || value.trim() === '') {
    return false
  }
  const parsed = Date.parse(value)
  return Number.isFinite(parsed)
}

function makeGateTemplate(providerShape, gateId, status) {
  const evidenceRefs = templateEvidenceRefs(providerShape, gateId, status)
  return {
    status,
    observed_at: '2026-06-26T00:00:00.000Z',
    evidence_ref: evidenceRefs[0],
    evidence_refs: evidenceRefs,
    notes: 'sanitized evidence only; no account ids, phone numbers, message bodies, tokens, cookies, session material, media keys or provider URLs',
  }
}

function templateEvidence(providerShape = 'whatsapp_native_md', status = 'pending') {
  const evidence = {}
  for (const gateId of requiredGateIds(providerShape)) {
    evidence[gateId] = makeGateTemplate(providerShape, gateId, status)
  }
  return {
    schema_version: 1,
    run_id: 'replace-with-local-run-id',
    generated_at: new Date('2026-06-26T00:00:00.000Z').toISOString(),
    provider_shape: providerShape,
    runtime_kind: 'replace-with-runtime-kind',
    account_fingerprint: 'sha256:0000000000000000000000000000000000000000000000000000000000000000',
    operator_attestation: {
      low_risk_or_test_account: status === 'passed',
      owner_visible_runtime: status === 'passed',
      no_hidden_or_headless_runtime: status === 'passed',
      secrets_not_recorded: status === 'passed',
      no_direct_domain_mutation: status === 'passed',
    },
    evidence,
  }
}

function evidenceRefs(gate) {
  const refs = []
  if (typeof gate.evidence_ref === 'string') {
    refs.push(gate.evidence_ref)
  }
  if (Array.isArray(gate.evidence_refs)) {
    for (const item of gate.evidence_refs) {
      if (typeof item === 'string') {
        refs.push(item)
      }
    }
  }
  return Array.from(new Set(refs.map((item) => item.trim()).filter(Boolean)))
}

function evidenceRefErrors(providerShape, gateId, gate) {
  const refs = evidenceRefs(gate)
  const errors = []
  if (refs.length === 0) {
    return [`evidence.${gateId}.evidence_refs must include sanitized references`]
  }
  for (const ref of refs) {
    if (/replace-with|pending|todo|example|dummy|placeholder/i.test(ref)) {
      errors.push(`evidence.${gateId}.evidence_ref contains placeholder value: ${ref}`)
    }
    if (!allowedEvidenceRefPrefixes.some((prefix) => ref.startsWith(prefix))) {
      errors.push(
        `evidence.${gateId}.evidence_ref must start with one of ${allowedEvidenceRefPrefixes.join(', ')}: ${ref}`
      )
    }
  }
  const prefixGroups = requiredEvidenceRefPrefixGroups(providerShape, gateId)
  for (const group of prefixGroups) {
    if (!refs.some((ref) => group.some((prefix) => ref.startsWith(prefix)))) {
      errors.push(
        `evidence.${gateId}.evidence_refs must include at least one ${group.join(' or ')} reference`
      )
    }
  }
  return errors
}

function collectValidationErrors(document) {
  const errors = []
  if (!isPlainObject(document)) {
    return ['evidence root must be a JSON object']
  }

  if (document.schema_version !== 1) {
    errors.push('schema_version must be 1')
  }
  if (typeof document.run_id !== 'string' || document.run_id.trim() === '') {
    errors.push('run_id must be a non-empty string')
  }
  if (!isIsoDate(document.generated_at)) {
    errors.push('generated_at must be an ISO-like timestamp')
  }
  if (!providerShapes.has(document.provider_shape)) {
    errors.push(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  if (typeof document.runtime_kind !== 'string' || document.runtime_kind.trim() === '') {
    errors.push('runtime_kind must be a non-empty string')
  }
  if (
    typeof document.account_fingerprint !== 'string'
    || !/^sha256:[a-f0-9]{64}$/i.test(document.account_fingerprint)
  ) {
    errors.push('account_fingerprint must be sha256:<64 hex chars>, not a raw account id or phone number')
  } else if (/^sha256:0{64}$/i.test(document.account_fingerprint)) {
    errors.push('account_fingerprint must be replaced with the real hashed account fingerprint')
  }

  const attestation = document.operator_attestation
  if (!isPlainObject(attestation)) {
    errors.push('operator_attestation must be an object')
  } else {
    for (const key of [
      'low_risk_or_test_account',
      'owner_visible_runtime',
      'no_hidden_or_headless_runtime',
      'secrets_not_recorde
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `scripts/whatsapp-live-smoke-readiness.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-live-smoke-readiness.mjs`
- Size bytes / Размер в байтах: `17848`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```javascript
#!/usr/bin/env node

import fs from 'node:fs'
import http from 'node:http'
import https from 'node:https'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const strictEnv = process.env.MAKOSH_LIVE_SMOKE_STRICT_ENV === '1'
const probeRuntimeApi = process.env.MAKOSH_WHATSAPP_RUNTIME_API_PROBE === '1'
const runtimeApiBaseUrl =
  process.env.MAKOSH_WHATSAPP_RUNTIME_API_BASE_URL ?? 'http://127.0.0.1:8080'
const runtimeApiSecret = process.env.MAKOSH_LOCAL_API_SECRET?.trim() ?? ''
const smokeAccountId = process.env.MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID?.trim() ?? ''
const expectedProviderShape = process.env.MAKOSH_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim() ?? ''

const checks = []

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function pass(id, evidence) {
  checks.push({ id, status: 'pass', evidence })
}

function fail(id, evidence) {
  checks.push({ id, status: 'fail', evidence })
}

function requireContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const missing = needles.filter((needle) => !text.includes(needle))
  if (missing.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} contains ${needle}`))
  } else {
    fail(
      id,
      missing.map((needle) => `${relativePath} missing ${needle}`)
    )
  }
}

function requireNotContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const present = needles.filter((needle) => text.includes(needle))
  if (present.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} does not contain ${needle}`))
  } else {
    fail(
      id,
      present.map((needle) => `${relativePath} still contains ${needle}`)
    )
  }
}

function requireEnvWhenStrict(id, envNames) {
  if (!strictEnv) {
    pass(id, ['strict env checks disabled; set MAKOSH_LIVE_SMOKE_STRICT_ENV=1 for manual smoke'])
    return
  }
  const missing = envNames.filter((name) => !process.env[name]?.trim())
  if (missing.length === 0) {
    pass(id, envNames.map((name) => `${name} is set`))
  } else {
    fail(id, missing.map((name) => `${name} is required for strict live smoke readiness`))
  }
}

function parseJson(value) {
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

function requestRuntimeApi(method, pathname, headers = {}, body = '') {
  return new Promise((resolve, reject) => {
    const url = new URL(pathname, runtimeApiBaseUrl)
    const transport = url.protocol === 'https:' ? https : http
    const requestBody = Buffer.from(body)
    const req = transport.request(
      url,
      {
        method,
        headers: {
          ...headers,
          ...(requestBody.length > 0 ? { 'Content-Length': String(requestBody.length) } : {}),
        },
        timeout: 5_000,
      },
      (res) => {
        const chunks = []
        res.on('data', (chunk) => chunks.push(chunk))
        res.on('end', () => {
          resolve({
            statusCode: res.statusCode ?? 0,
            headers: res.headers,
            body: Buffer.concat(chunks).toString('utf8'),
          })
        })
      }
    )
    req.on('timeout', () => {
      req.destroy(new Error(`request timed out: ${method} ${url.href}`))
    })
    req.on('error', reject)
    if (requestBody.length > 0) {
      req.write(requestBody)
    }
    req.end()
  })
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function providerShapeErrors(value) {
  if (!expectedProviderShape) {
    return []
  }
  return value === expectedProviderShape
    ? []
    : [`provider_shape ${value ?? '<missing>'} did not match ${expectedProviderShape}`]
}

function assertNoSecretLeaks(id, responses) {
  const leakPatterns = [
    /session_blob/i,
    /session_material/i,
    /cookie_value/i,
    /raw_secret/i,
    /access_token_value/i,
    /app_secret_value/i,
    /verify_token_value/i,
    /refresh_token_value/i,
    /browser_profile_secret/i,
    /"authorization"\s*:/i,
    /"cookie"\s*:/i,
    /"qr_code"\s*:\s*"/i,
    /"pair_code"\s*:\s*"/i,
    /"media_key"\s*:\s*"/i,
    /"direct_path"\s*:\s*"/i,
    /"static_url"\s*:\s*"/i,
  ]
  const leaked = []
  for (const response of responses) {
    for (const pattern of leakPatterns) {
      if (pattern.test(response.body)) {
        leaked.push(`${response.id} matched ${pattern.source}`)
      }
    }
  }
  if (leaked.length === 0) {
    pass(id, ['runtime API responses contain no raw secret/session/media-ref payload markers'])
  } else {
    fail(id, leaked)
  }
}

async function probeJsonEndpoint(id, pathname, validate, responses) {
  const response = await requestRuntimeApi('GET', pathname, {
    Accept: 'application/json',
    'X-Макошь-Secret': runtimeApiSecret,
  })
  responses.push({ id, body: response.body })

  if (response.statusCode < 200 || response.statusCode >= 300) {
    fail(id, [`GET ${pathname} returned HTTP ${response.statusCode} (${response.body.length} bytes)`])
    return
  }

  const json = parseJson(response.body)
  if (!isPlainObject(json)) {
    fail(id, [`GET ${pathname} returned non-object JSON (${response.body.length} bytes)`])
    return
  }

  const errors = validate(json)
  if (errors.length === 0) {
    pass(id, [`GET ${pathname} returned the expected sanitized contract`])
  } else {
    fail(id, errors)
  }
}

async function probeRuntimeApiEndpoints() {
  if (!probeRuntimeApi) {
    pass('runtime_api_probe', [
      'runtime API probe disabled; set MAKOSH_WHATSAPP_RUNTIME_API_PROBE=1 with MAKOSH_LOCAL_API_SECRET and MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID to probe a running Макошь backend',
    ])
    return
  }

  const missing = []
  if (!runtimeApiSecret) {
    missing.push('MAKOSH_LOCAL_API_SECRET')
  }
  if (!smokeAccountId) {
    missing.push('MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID')
  }
  if (missing.length > 0) {
    fail('runtime_api_probe_configuration', missing.map((name) => `${name} is required for runtime API probe`))
    return
  }

  pass('runtime_api_probe_configuration', [
    `probing ${runtimeApiBaseUrl}`,
    `account id is set (${smokeAccountId.length} chars)`,
    expectedProviderShape
      ? `expected provider shape ${expectedProviderShape}`
      : 'provider-shape assertion disabled; set MAKOSH_WHATSAPP_SMOKE_PROVIDER_SHAPE to enable it',
  ])

  const responses = []
  const encodedAccountId = encodeURIComponent(smokeAccountId)
  const encodedStatusAccountId = encodeURIComponent(smokeAccountId)

  try {
    await probeJsonEndpoint(
      'runtime_api_global_capabilities',
      '/api/v1/integrations/whatsapp/capabilities',
      (json) => [
        ...(typeof json.version === 'string' ? [] : ['version must be a string']),
        ...(typeof json.runtime_mode === 'string' ? [] : ['runtime_mode must be a string']),
        ...(Array.isArray(json.provider_shapes) ? [] : ['provider_shapes must be an array']),
        ...(json.account_scope === null ? [] : ['global capabilities account_scope must be null']),
        ...(Array.isArray(json.capabilities) ? [] : ['capabilities must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_account_capabilities',
      `/api/v1/integrations/whatsapp/accounts/${encodedAccountId}/capabilities`,
      (json) => [
        ...(isPlainObject(json.account_scope) ? [] : ['account_scope must be an object']),
        ...(json.account_scope?.account_id === smokeAccountId
          ? []
          : ['account_scope.account_id must match MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.account_scope?.provider_shape === 'string'
          ? []
          : ['account_scope.provider_shape must be a string']),
        ...providerShapeErrors(json.account_scope?.provider_shape),
        ...(Array.isArray(json.capabilities) ? [] : ['capabilities must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_status',
      `/api/v1/integrations/whatsapp/runtime/status?account_id=${encodedStatusAccountId}`,
      (json) => [
        ...(json.account_id === smokeAccountId ? [] : ['account_id must match MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.provider_shape === 'string' ? [] : ['provider_shape must be a string']),
        ...providerShapeErrors(json.provider_shape),
        ...(typeof json.runtime_kind === 'string' ? [] : ['runtime_kind must be a string']),
        ...(typeof json.status === 'string' ? [] : ['status must be a string']),
        ...(typeof json.session_restore_available === 'boolean'
          ? []
          : ['session_restore_available must be a boolean']),
        ...(Array.isArray(json.runtime_blockers) ? [] : ['runtime_blockers must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_health',
      `/api/v1/integrations/whatsapp/runtime/health?account_id=${encodedStatusAccountId}`,
      (json) => [
        ...(json.account_id === smokeAccountId ? [] : ['account_id must match MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.provider_shape === 'string' ? [] : ['provider_shape must be a string']),
        ...providerShapeErrors(json.provider_shape),
        ...(typeof json.runtime_kind === 'string' ? [] : ['runtime_kind must be a string']),
        ...(typeof json.status === 'string' ? [] : ['status must be a string']),
        ...(typeof json.healthy === 'boolean' ? [] : ['healthy must be a boolean']),
        ...(isPlainObject(json.checks) ? [] : ['checks must be an object']),
        ...(typeof json.checked_at === 'string' ? [] : ['checked_at must be a string']),
      ],
      responses
    )

    assertNoSecretLeaks('runtime_api_probe_no_secret_leaks', responses)
  } catch (error) {
    fail('runtime_api_probe_request', [
      error instanceof Error ? error.message : 'runtime API probe failed with an unknown error',
    ])
  }
}

requireContains('webview_runtime_event_relay_dispatch', 'frontend/src-tauri/src/whatsapp_companion.rs', [
  'RUNTIME_EVENTS_BRIDGE_PATH',
  '"/api/v1/integrations/whatsapp/runtime-bridge/runtime-events"',
  'runtime_bridge_runtime_event_payload',
  'dispatch_runtime_bridge_runtime_event',
  'X-Макошь-Secret',
  'is_allowed_local_backend_url',
  'runtime_event_evidence_only_until_richer_typed_payload',
  'provider_observed_event_reconciliation_required',
])

requireContains('webview_remote_capability_is_narrow', 'frontend/src-tauri/capabilities/whatsapp-companion-relay.json', [
  '"local": false',
  'https://web.whatsapp.com',
  'whatsapp-companion-*',
  'allow-whatsapp-web-companion-relay-observation',
])

requireNotContains('webview_remote_capability_no_broad_core', 'frontend/src-tauri/capabilities/whatsapp-companion-relay.json', [
  'core:default',
])

requireNotContains('main_capability_does_not_allow_remote_relay', 'frontend/src-tauri/capabilities/default.json', [
  'allow-whatsapp-web-companion-relay-observation',
])

requireContains('webview_backend_health_contract_dispatch_state', 'backend/src/integrations/whatsapp/runtime/web_companion.rs', [
  'contract_injected_relay_dispatch_available',
  'tauri_allowlisted_companion_runtime_bridge_dispatch',
  'runtime_events_bridge_wired_smoke_pending',
  'NewWhatsappWebRuntimeEvent',
  'X-Макошь-Secret_from_tauri_process_env_only',
  'typed_projection',
  'manual_live_smoke_required',
])

requireNotContains('webview_backend_health_no_old_dispatch_blocker', 'backend/src/integrations/whatsapp/runtime/web_companion.rs', [
  'whatsapp_webview_backend_dispatch_not_implemented',
  'blocked_until_backend_dispatch_is_wired_and_live_smoked',
  'contract_injected_relay_preflight_available',
])

requireContains('signal_hub_static_guard_covers_webview_dispatch', 'backend/tests/whatsapp_signal_hub.rs', [
  'dispatch_runtime_bridge_runtime_event',
  'RUNTIME_EVENTS_BRIDGE_PATH',
  'is_allowed_local_backend_url',
  'runtime_event_evidence_only_until_richer_typed_payload',
  'dispatched_to_backend_runtime_bridge_runtime
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `scripts/whatsapp-native-md-sdk-gap-readiness.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/whatsapp-native-md-sdk-gap-readiness.mjs`
- Size bytes / Размер в байтах: `20273`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```javascript
#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const waRsVersion = '0.2.0'
const cratesIoProbe = process.env.MAKOSH_WA_RS_CRATES_IO_PROBE === '1'
const checks = []

const requiredApis = [
  {
    id: 'send_message_api',
    relativePath: 'src/send.rs',
    pattern: /pub\s+async\s+fn\s+send_message\s*\(/,
    evidence: 'Client::send_message is available for send_text/reply/react/unreact payload submission',
  },
  {
    id: 'revoke_message_api',
    relativePath: 'src/send.rs',
    pattern: /pub\s+async\s+fn\s+revoke_message\s*\(/,
    evidence: 'Client::revoke_message is available for delete/revoke submission',
  },
  {
    id: 'edit_message_api',
    relativePath: 'src/client.rs',
    pattern: /pub\s+async\s+fn\s+edit_message\s*\(/,
    evidence: 'Client::edit_message is available for edit submission',
  },
  {
    id: 'mark_as_read_api',
    relativePath: 'src/receipt.rs',
    pattern: /pub\s+async\s+fn\s+mark_as_read\s*\(/,
    evidence: 'Client::mark_as_read is available for read receipts',
  },
  {
    id: 'leave_group_api',
    relativePath: 'src/features/groups.rs',
    pattern: /pub\s+async\s+fn\s+leave\s*\(/,
    evidence: 'Client::groups().leave is available for leave_group',
  },
  {
    id: 'upload_api',
    relativePath: 'src/upload.rs',
    pattern: /pub\s+async\s+fn\s+upload\s*\(/,
    evidence: 'Client::upload is available for media/voice-note upload',
  },
  {
    id: 'download_from_params_api',
    relativePath: 'src/download.rs',
    pattern: /pub\s+async\s+fn\s+download_from_params\s*\(/,
    evidence: 'Client::download_from_params is available for media download refs',
  },
]

const unsupportedExpectations = [
  {
    id: 'no_status_publish_api',
    commandKinds: ['publish_status'],
    matches: (name) =>
      name === 'send_status'
      || name === 'publish_status'
      || (name.includes('status') && (name.includes('publish') || name.includes('post'))),
  },
  {
    id: 'no_dialog_state_write_api',
    commandKinds: ['archive', 'unarchive', 'mute', 'unmute', 'pin', 'unpin', 'mark_unread'],
    matches: (name) =>
      [
        'archive',
        'unarchive',
        'mute',
        'unmute',
        'pin',
        'unpin',
        'mark_unread',
        'mark_as_unread',
      ].includes(name),
  },
  {
    id: 'no_join_by_invite_api',
    commandKinds: ['join_group'],
    matches: (name) =>
      name === 'join_group'
      || name === 'join_by_invite'
      || name === 'accept_invite'
      || name === 'accept_group_invite',
  },
]

function pass(id, evidence) {
  checks.push({ id, status: 'pass', evidence })
}

function fail(id, evidence) {
  checks.push({ id, status: 'fail', evidence })
}

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function cargoDependencyVersion(cargoToml, crateName) {
  const escaped = crateName.replaceAll('-', String.raw`\-`)
  const pattern = new RegExp(`${escaped}\\s*=\\s*\\{[^\\n]*version\\s*=\\s*"([^"]+)"`)
  return cargoToml.match(pattern)?.[1] ?? null
}

function cargoLockPackageVersion(cargoLock, crateName) {
  const escaped = crateName.replaceAll('-', String.raw`\-`)
  const pattern = new RegExp(`name\\s*=\\s*"${escaped}"\\nversion\\s*=\\s*"([^"]+)"`, 'm')
  return cargoLock.match(pattern)?.[1] ?? null
}

function cargoInfoVersion(crateName) {
  const result = spawnSync('cargo', ['info', crateName], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 30_000,
  })
  if (result.status !== 0) {
    return {
      ok: false,
      evidence: [
        `cargo info ${crateName} failed with status ${result.status}`,
        ...(result.stderr?.trim() ? [result.stderr.trim().slice(0, 500)] : []),
      ],
    }
  }
  const version = result.stdout.match(/^version:\s*(?<version>\S+)/m)?.groups?.version
  if (!version) {
    return {
      ok: false,
      evidence: [`cargo info ${crateName} did not expose a version line`],
    }
  }
  return {
    ok: true,
    version,
    evidence: [`cargo info ${crateName} reports version ${version}`],
  }
}

function waRsSourceRoot() {
  if (process.env.MAKOSH_WA_RS_SOURCE_DIR?.trim()) {
    return process.env.MAKOSH_WA_RS_SOURCE_DIR.trim()
  }

  const cargoHome = process.env.CARGO_HOME?.trim() || path.join(os.homedir(), '.cargo')
  const registrySrc = path.join(cargoHome, 'registry', 'src')
  if (!fs.existsSync(registrySrc)) {
    return null
  }

  for (const registryNamespace of fs.readdirSync(registrySrc)) {
    const candidate = path.join(registrySrc, registryNamespace, `wa-rs-${waRsVersion}`)
    if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
      return candidate
    }
  }
  return null
}

function waRsCoreSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-core-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
    return candidate
  }
  return null
}

function waRsAppStateSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-appstate-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
    return candidate
  }
  return null
}

function waRsProtoSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-proto-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'whatsapp.rs'))) {
    return candidate
  }
  return null
}

function listRustFiles(root) {
  const result = []
  const stack = [root]
  while (stack.length > 0) {
    const current = stack.pop()
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const fullPath = path.join(current, entry.name)
      if (entry.isDirectory()) {
        stack.push(fullPath)
      } else if (entry.isFile() && entry.name.endsWith('.rs')) {
        result.push(fullPath)
      }
    }
  }
  return result.sort()
}

function publicSurfaceFiles(root) {
  const publicRoots = [
    path.join(root, 'src', 'bot.rs'),
    path.join(root, 'src', 'client.rs'),
    path.join(root, 'src', 'send.rs'),
    path.join(root, 'src', 'receipt.rs'),
    path.join(root, 'src', 'upload.rs'),
    path.join(root, 'src', 'download.rs'),
    path.join(root, 'src', 'features'),
  ]
  const files = []
  for (const publicRoot of publicRoots) {
    if (!fs.existsSync(publicRoot)) {
      continue
    }
    const stat = fs.statSync(publicRoot)
    if (stat.isDirectory()) {
      files.push(...listRustFiles(publicRoot))
    } else {
      files.push(publicRoot)
    }
  }
  return files.sort()
}

function publicFunctionNames(root) {
  const names = []
  for (const filePath of publicSurfaceFiles(root)) {
    const text = fs.readFileSync(filePath, 'utf8')
    const regex = /pub\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/g
    for (const match of text.matchAll(regex)) {
      names.push({
        name: match[1],
        file: path.relative(root, filePath),
      })
    }
  }
  return names
}

function publicFunctionNamesInFiles(root, files) {
  const names = []
  for (const filePath of files) {
    const text = fs.readFileSync(filePath, 'utf8')
    const regex = /pub\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/g
    for (const match of text.matchAll(regex)) {
      names.push({
        name: match[1],
        file: path.relative(root, filePath),
      })
    }
  }
  return names
}

function requireContains(relativePath, pattern, id, evidence) {
  const text = fs.readFileSync(relativePath, 'utf8')
  if (pattern.test(text)) {
    pass(id, [evidence])
  } else {
    fail(id, [`${relativePath} does not match ${pattern}`])
  }
}

function verifyLowLevelGapEvidence(root) {
  requireContains(
    path.join(root, 'src', 'request.rs'),
    /pub\s+async\s+fn\s+send_iq\s*\(/,
    'custom_iq_api_exists_but_is_low_level',
    'Client::send_iq exists for custom IQ stanzas, but unsupported app-state commands still need a safe encoder and smoke evidence'
  )

  const coreRoot = waRsCoreSourceRoot(root)
  const appStateRoot = waRsAppStateSourceRoot(root)
  if (!coreRoot) {
    fail('wa_rs_core_source_available', [
      `wa-rs-core ${waRsVersion} source not found next to ${root}`,
    ])
  } else {
    pass('wa_rs_core_source_available', [`using ${coreRoot}`])
    const groupsSource = fs.readFileSync(path.join(coreRoot, 'src', 'iq', 'groups.rs'), 'utf8')
    const joinInviteMarkers = [
      'JoinGroupIq',
      'AcceptInvite',
      'accept_invite',
      'join_by_invite',
      'GroupInviteJoin',
    ]
    const present = joinInviteMarkers.filter((marker) => groupsSource.includes(marker))
    if (present.length === 0) {
      pass('no_join_by_invite_iq_spec', [
        'wa-rs-core group IQ surface has invite-link fetch/reset, but no join-by-invite/accept-invite IQ spec',
      ])
    } else {
      fail('no_join_by_invite_iq_spec', present.map((marker) => `found ${marker}`))
    }
  }

  if (!appStateRoot) {
    fail('wa_rs_appstate_source_available', [
      `wa-rs-appstate ${waRsVersion} source not found next to ${root}`,
    ])
    return
  }

  pass('wa_rs_appstate_source_available', [`using ${appStateRoot}`])
  const appStateLib = fs.readFileSync(path.join(appStateRoot, 'src', 'lib.rs'), 'utf8')
  const appStateFiles = listRustFiles(path.join(appStateRoot, 'src'))
  const encoderFile = appStateFiles.find((filePath) => path.basename(filePath) === 'encode.rs')
  const publicFns = publicFunctionNamesInFiles(appStateRoot, appStateFiles)
  const outgoingEncoderHits = publicFns.filter((item) =>
    /^(encode|encrypt|build|create|send)_.*(patch|mutation|app_state|syncd)/u.test(item.name)
    || /^(patch|mutation|app_state|syncd)_.*(encode|encrypt|build|create|send)/u.test(item.name)
  )
  const exportedEncodeModule = /\bpub\s+mod\s+encode\b/u.test(appStateLib)
  if (!encoderFile && !exportedEncodeModule && outgoingEncoderHits.length === 0) {
    pass('no_public_outgoing_appstate_encoder', [
      'wa-rs-appstate exposes decode/hash/process helpers but no public outgoing patch/mutation encoder for archive/mute/pin/unread/status writes',
    ])
  } else {
    fail('no_public_outgoing_appstate_encoder', [
      ...(encoderFile ? [`found ${path.relative(appStateRoot, encoderFile)}`] : []),
      ...(exportedEncodeModule ? ['lib.rs exports pub mod encode'] : []),
      ...outgoingEncoderHits.map((hit) => `${hit.file}: pub fn ${hit.name}`),
    ])
  }
}

function verifyForwardTextReemitContract(root) {
  const protoRoot = waRsProtoSourceRoot(root)
  if (!protoRoot) {
    fail('wa_rs_proto_source_available_for_forward_context', [
      `wa-rs-proto ${waRsVersion} source not found next to ${root}`,
    ])
    return
  }

  pass('wa_rs_proto_source_available_for_forward_context', [`using ${protoRoot}`])
  requireContains(
    path.join(protoRoot, 'src', 'whatsapp.rs'),
    /pub forwarding_score: ::core::option::Option<u32>/,
    'wa_rs_proto_forwarding_score_available',
    'ExtendedTextMessage.ContextInfo exposes forwarding_score for forwarded text reemit'
  )
  requireContains(
    path.join(protoRoot, 'src', 'whatsapp.rs'),
    /pub is_forwarded: ::core::option::Option<bool>/,
    'wa_rs_proto_is_forwarded_available',
    'ExtendedTextMessage.ContextInfo exposes is_forwarded for forwarded text reemit'
  )
}

function verifyLocalWaRsApi(root) {
  pass('wa_rs_source_available', [`using ${root}`])

  for (const api of requiredApis) {
    requireContains(
      path.join(root, api.relativePath),
      api.pattern,
      api.id,
      `${api.relativePath}: ${api.evidence}`
    )
  }

  const publicFns = publicFunctionNames(root)
  for (const expectation of unsupportedExpectations) {
    const hits = publicFns.filter((item) => expectation.matches(item.name))
    if (hits.length === 0) {
      pass(expectation.id, [
        `${expectation.commandKinds.join(', ')} remains unsuppor
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
