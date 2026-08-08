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

- Chunk ID / ID чанка: `141-source-frontend-part-001`
- Group / Группа: `frontend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

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

### `frontend/postcss.config.js`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/postcss.config.js`
- Size bytes / Размер в байтах: `72`
- Included characters / Включено символов: `72`
- Truncated / Обрезано: `no`

```javascript
export default {
	plugins: {
		tailwindcss: {},
		autoprefixer: {}
	}
}
```

### `frontend/scripts/capture-layout-screenshots.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/capture-layout-screenshots.mjs`
- Size bytes / Размер в байтах: `21992`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```javascript
import { mkdir, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { chromium } from 'playwright';

const views = [
	{ id: 'home', label: 'Home', kind: 'primary' },
	{ id: 'communications', label: 'Communications', kind: 'primary' },
	{ id: 'communications-mail', label: 'Mail', kind: 'communication-section' },
	{ id: 'communications-telegram', label: 'Telegram', kind: 'communication-section' },
	{ id: 'communications-whatsapp', label: 'WhatsApp', kind: 'communication-section' },
	{ id: 'communications-calls', label: 'Calls', kind: 'communication-section' },
	{ id: 'timeline', label: 'Timeline', kind: 'primary' },
	{ id: 'persons', label: 'Persons', kind: 'primary' },
	{ id: 'projects', label: 'Projects', kind: 'primary' },
	{ id: 'tasks', label: 'Tasks', kind: 'primary' },
	{ id: 'calendar', label: 'Calendar', kind: 'primary' },
	{ id: 'documents', label: 'Documents', kind: 'primary' },
	{ id: 'notes', label: 'Notes', kind: 'primary' },
	{ id: 'knowledge-graph', label: 'Knowledge Graph', kind: 'primary' },
	{ id: 'ai-agents', label: 'AI Agents', kind: 'primary' },
	{ id: 'settings', label: 'Settings', kind: 'settings' }
];

const viewports = [
	{ id: '800x600', width: 800, height: 600, expectMultiColumn: false },
	{ id: '1024x768', width: 1024, height: 768, expectMultiColumn: false },
	{ id: '1366x768', width: 1366, height: 768, expectMultiColumn: true },
	{ id: '1920x1080', width: 1920, height: 1080, expectMultiColumn: true }
];

const mode = process.argv[2] ?? 'baseline';
if (!['baseline', 'after'].includes(mode)) {
	console.error('Usage: node scripts/capture-layout-screenshots.mjs baseline|after [url]');
	process.exit(1);
}

const url = process.argv[3] ?? 'http://localhost:5174/';
const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
const outputDir = path.join(os.tmpdir(), `makosh-layout-${mode}-${timestamp}`);

await mkdir(outputDir, { recursive: true });

const browser = await chromium.launch();
const results = [];
const failures = [];

const layoutStatusFixture = {
	version: 'layout-capture',
	surfaces: {
		messages: true,
		persons: true,
		search: true,
		documents: true,
		account_setup: true
	},
	vault_status: {
		state: 'locked',
		needs_entropy: false,
		needs_biometric: false,
		needs_recovery: false,
		version: 1,
		recoverable: true,
		entropy_progress: 100
	}
};

async function getPrimaryNavButton(page, label) {
	const primaryNav = page.locator('nav[aria-label="Primary workspaces"]');
	const button = primaryNav.getByRole('button').filter({ hasText: label });
	const count = await button.count();
	if (count !== 1) {
		const navLabels = await primaryNav.getByRole('button').evaluateAll((buttons) =>
			buttons.map((navButton) => (navButton.textContent ?? '').trim().replace(/\s+/g, ' '))
		);
		throw new Error(
			`Expected exactly one primary nav button containing visible text "${label}", found ${count}. ` +
				`Primary nav buttons: ${navLabels.length > 0 ? navLabels.join(', ') : '(none)'}`
		);
	}
	return button;
}

async function openPrimaryView(page, label) {
	const button = await getPrimaryNavButton(page, label);
	await button.click();
}

async function openCommunicationSection(page, label) {
	const subnav = page.locator('#sidebar-group-communications-sections, #communications-sidebar-sections').first();
	if (!(await subnav.isVisible().catch(() => false))) {
		await openPrimaryView(page, 'Communications');
	}
	await subnav.waitFor({ state: 'visible' });
	const button = subnav.getByRole('button').filter({ hasText: label });
	const count = await button.count();
	if (count !== 1) {
		const subnavLabels = await subnav.getByRole('button').evaluateAll((buttons) =>
			buttons.map((navButton) => (navButton.textContent ?? '').trim().replace(/\s+/g, ' '))
		);
		throw new Error(
			`Expected exactly one Communications section button containing visible text "${label}", found ${count}. ` +
				`Communications buttons: ${subnavLabels.length > 0 ? subnavLabels.join(', ') : '(none)'}`
		);
	}
	await button.click();
}

async function openSettings(page) {
	const button = page.locator('.sidebar-tools').getByRole('button').filter({ hasText: 'Settings' });
	const count = await button.count();
	if (count !== 1) {
		throw new Error(`Expected one Settings button in sidebar tools, found ${count}.`);
	}
	await button.click();
}

async function openView(page, view) {
	if (view.kind === 'communication-section') {
		await openCommunicationSection(page, view.label);
		return;
	}

	if (view.kind === 'settings') {
		await openSettings(page);
		return;
	}

	await openPrimaryView(page, view.label);
}

async function getViewportGuardDisplay(page, context) {
	return page.evaluate((currentContext) => {
		const guard = document.querySelector('.viewport-guard');
		if (guard === null) {
			throw new Error(`Expected .viewport-guard element to exist while ${currentContext}.`);
		}
		return getComputedStyle(guard).display;
	}, context);
}

function trackConsoleIssues(page, consoleIssues) {
	page.on('console', (message) => {
		if (['warning', 'error'].includes(message.type())) {
			consoleIssues.push({ level: message.type(), text: message.text() });
		}
	});
}

async function installLayoutCaptureRoutes(page) {
	await page.route('**/api/v1/status', async (route) => {
		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify(layoutStatusFixture)
		});
	});
}

async function captureViewport(viewport) {
	const viewportDir = path.join(outputDir, viewport.id);
	await mkdir(viewportDir, { recursive: true });

	const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
	const consoleIssues = [];
	trackConsoleIssues(page, consoleIssues);
	await installLayoutCaptureRoutes(page);

	try {
		await page.goto(url, { waitUntil: 'domcontentloaded' });
		await page.locator('nav[aria-label="Primary workspaces"]').waitFor({ state: 'visible' });

		for (const view of views) {
			await openView(page, view);
			await page.waitForTimeout(650);
			const layoutState = await page.evaluate(() => {
				const tokenNumber = (name, fallback) => {
					const probe = document.createElement('div');
					probe.style.position = 'absolute';
					probe.style.visibility = 'hidden';
					probe.style.pointerEvents = 'none';
					probe.style.height = `var(${name})`;
					document.body.append(probe);
					const value = Number.parseFloat(getComputedStyle(probe).height);
					probe.remove();
					return Number.isFinite(value) ? value : fallback;
				};
				const isElementVisible = (element) => {
					const rect = element.getBoundingClientRect();
					const style = getComputedStyle(element);
					return {
						rect,
						visible:
							style.display !== 'none' &&
							style.visibility !== 'hidden' &&
							rect.width > 0 &&
							rect.height > 0
					};
				};
				const widgetUnit = tokenNumber('--hh-widget-unit', 24);
				const widgetRow = tokenNumber('--hh-widget-row', widgetUnit);
				const layoutGap = tokenNumber('--hh-layout-gap', 0);
				const expectedWidgetHeight = (rows, zoneRows) =>
					rows * (widgetRow + layoutGap / Math.max(1, zoneRows)) - layoutGap;
				const isNearExpectedWidgetHeight = (height, rows, zoneRows) => {
					const expected = expectedWidgetHeight(rows, zoneRows);
					return Math.abs(height - expected) <= 2;
				};
				const outliers = [];
				for (const element of document.querySelectorAll('body *')) {
					const { rect, visible } = isElementVisible(element);
					if (!visible) continue;
					if (element.closest('.sidebar')) continue;
					const closestWidget = element.closest('.widget-frame[data-widget-id]');
					if (closestWidget !== null && closestWidget !== element) continue;
					const horizontalOutlier = rect.left < -1 || rect.right > window.innerWidth + 1;
					const verticalOutlier = rect.top < -1 || rect.bottom > window.innerHeight + 1;
					if (horizontalOutlier || verticalOutlier) {
						outliers.push({
							tag: element.tagName.toLowerCase(),
							className: typeof element.className === 'string' ? element.className : '',
							axis: [
								horizontalOutlier ? 'horizontal' : null,
								verticalOutlier ? 'vertical' : null
							].filter(Boolean),
							left: Math.round(rect.left),
							right: Math.round(rect.right),
							top: Math.round(rect.top),
							bottom: Math.round(rect.bottom),
							text: (element.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 80)
						});
					}
					if (outliers.length >= 10) break;
				}

				const visibleWidgetFrames = [];
				for (const element of document.querySelectorAll('.widget-frame[data-widget-id]')) {
					const { rect, visible } = isElementVisible(element);
					if (!visible) continue;
					const height = Math.round(rect.height);
					const style = getComputedStyle(element);
					const rows = Number.parseInt(element.getAttribute('data-widget-rows') ?? '', 10);
					const zoneRowsValue = Number.parseFloat(style.getPropertyValue('--hh-zone-rows'));
					const zoneRows = Number.isFinite(zoneRowsValue) ? zoneRowsValue : rows;
					const expectedHeight = Number.isInteger(rows)
						? Math.round(expectedWidgetHeight(rows, zoneRows))
						: null;
					const heightAligned = Number.isInteger(rows)
						? isNearExpectedWidgetHeight(height, rows, zoneRows)
						: false;
					visibleWidgetFrames.push({
						widgetId: element.getAttribute('data-widget-id') ?? '',
						rows: Number.isInteger(rows) ? rows : null,
						zoneRows: Number.isFinite(zoneRows) ? zoneRows : null,
						height,
						expectedHeight,
						left: Math.round(rect.left),
						top: Math.round(rect.top),
						moduleAligned: heightAligned,
						rowAligned: heightAligned
					});
				}

				const layoutColumnMetrics = [];
				const layoutSelectors = [
					'.dashboard-grid',
					'.three-pane',
					'.contacts-layout',
					'.docs-layout',
					'.notes-layout',
					'.project-dashboard-grid',
					'.tasks-layout',
					'.calendar-layout',
					'.knowledge-layout',
					'.agents-layout',
					'.settings-layout',
					'.timeline-layout'
				];
				for (const container of document.querySelectorAll(layoutSelectors.join(','))) {
					const { rect: containerRect, visible: containerVisible } = isElementVisible(container);
					if (!containerVisible) continue;

					const columnsByLeft = new Map();
					for (const child of Array.from(container.children)) {
						const { rect, visible } = isElementVisible(child);
						if (!visible) continue;
						const left = Math.round((rect.left - containerRect.left) / 8) * 8;
						const column = columnsByLeft.get(left) ?? {
							left,
							count: 0,
							top: Number.POSITIVE_INFINITY,
							bottom: Number.NEGATIVE_INFINITY,
							contentTop: Number.POSITIVE_INFINITY,
							contentBottom: Number.NEGATIVE_INFINITY
						};
						column.count += 1;
						column.top = Math.min(column.top, Math.round(rect.top - containerRect.top));
						column.bottom = Math.max(column.bottom, Math.round(rect.bottom - containerRect.top));
						column.contentTop = Math.min(column.contentTop, Math.round(rect.top - containerRect.top));
						column.contentBottom = Math.max(column.contentBottom, Math.round(rect.bottom - containerRect.top));
						columnsByLeft.set(left, column);
					}

					if (columnsByLeft.size < 2) continue;

					const columns = Array.from(columnsByLeft.values())
						.map((column) => ({
							...column,
							height: column.bottom - column.top,
							contentHeight: column.contentBottom - column.contentTop
						}))
						.sort((left, right) => left.left - right.left);
					const columnHeights = columns.map((column) => column.height);
					layoutColumnMetrics.push({
						selector: layoutSelectors.find((selector) => container.matches(selector)) ?? '',
						columnCount: columns.length,
						columns,
						columnHeightSpread: Math.max(...columnHeights) - Math.min(...columnHeights)
					});
				}

				const scrollModeAllowsAxis = (mode, axis) => {
					if (mode === 'both') return true
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/scripts/check-component-lines.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/check-component-lines.mjs`
- Size bytes / Размер в байтах: `4322`
- Included characters / Включено символов: `4322`
- Truncated / Обрезано: `no`

```javascript
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const checkedExtensions = new Set(['.ts', '.tsx', '.vue']);

const sourceRoot = path.resolve('src');
const warningLimit = 500;
const failureLimit = 700;
const criticalLimit = 1000;
const maxReportedFiles = 20;

export function classifyLineCount(lineCount) {
	return {
		warning: lineCount >= warningLimit,
		failure: lineCount >= failureLimit,
		critical: lineCount >= criticalLimit
	};
}

export function isProductionSourceFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	if (!checkedExtensions.has(path.extname(normalized))) {
		return false;
	}
	if (normalized.includes('/__tests__/')) {
		return false;
	}
	return !/\.(test|spec)\.[cm]?[jt]sx?$/.test(normalized);
}

export function isLineCountCheckedSourceFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	if (!checkedExtensions.has(path.extname(normalized))) {
		return false;
	}
	if (normalized === 'src/gen' || normalized.startsWith('src/gen/') || normalized.includes('/src/gen/')) {
		return false;
	}
	return isProductionSourceFile(normalized);
}

async function collectSourceFiles(directory) {
	if (!(await exists(directory))) {
		return [];
	}

	const entries = await readdir(directory, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(directory, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectSourceFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && isLineCountCheckedSourceFile(entryPath)) {
			files.push(entryPath);
		}
	}

	return files;
}

function countLines(source) {
	return source.endsWith('\n') ? source.split('\n').length - 1 : source.split('\n').length;
}

function formatFileReport(files) {
	return files
		.slice(0, maxReportedFiles)
		.map(({ relativePath, lineCount }) => `- ${relativePath}: ${lineCount} lines`)
		.join('\n');
}

async function exists(root) {
	try {
		await stat(root);
		return true;
	} catch (error) {
		if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
			return false;
		}
		throw error;
	}
}

export async function collectLineCountReport(root = sourceRoot) {
	const files = await collectSourceFiles(root);
	const fileReports = [];

	for (const filePath of files) {
		const source = await readFile(filePath, 'utf8');
		fileReports.push({
			relativePath: path.relative(process.cwd(), filePath),
			lineCount: countLines(source)
		});
	}

	const warningFiles = [];
	const failureFiles = [];
	const criticalFiles = [];

	for (const file of fileReports) {
		const classification = classifyLineCount(file.lineCount);
		if (classification.warning) warningFiles.push(file);
		if (classification.failure) failureFiles.push(file);
		if (classification.critical) criticalFiles.push(file);
	}

	const byLineCountDescending = (left, right) => right.lineCount - left.lineCount;

	return {
		totalFiles: fileReports.length,
		warningFiles: warningFiles.sort(byLineCountDescending),
		failureFiles: failureFiles.sort(byLineCountDescending),
		criticalFiles: criticalFiles.sort(byLineCountDescending)
	};
}

async function main() {
	const report = await collectLineCountReport();

	if (report.warningFiles.length > 0) {
		console.warn(
			[
				`Source SRP line-count warnings: ${report.warningFiles.length} source file(s) are at or above ${warningLimit} lines.`,
				`Longest warning files, capped at ${maxReportedFiles}:`,
				formatFileReport(report.warningFiles)
			].join('\n')
		);
	}

	if (report.failureFiles.length > 0) {
		console.error(
			[
				`Source SRP line-count failures: ${report.failureFiles.length} source file(s) are at or above ${failureLimit} lines.`,
				`Critical architecture failures at or above ${criticalLimit} lines: ${report.criticalFiles.length}.`,
				`Longest failing files, capped at ${maxReportedFiles}:`,
				formatFileReport(report.failureFiles)
			].join('\n')
		);
		process.exit(1);
	}

	console.log(
		`Source SRP line-count check passed: ${report.totalFiles} source file(s), no files at or above ${failureLimit} lines.`
	);
}

const entryPointUrl = process.argv[1] ? pathToFileURL(process.argv[1]).href : '';

if (import.meta.url === entryPointUrl) {
	await main();
}
```

### `frontend/scripts/check-component-lines.test.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/check-component-lines.test.mjs`
- Size bytes / Размер в байтах: `1841`
- Included characters / Включено символов: `1841`
- Truncated / Обрезано: `no`

```javascript
import { describe, expect, it } from 'vitest';
import {
	checkedExtensions,
	classifyLineCount,
	isLineCountCheckedSourceFile,
	isProductionSourceFile
} from './check-component-lines.mjs';

describe('check-component-lines policy', () => {
	it('checks production Vue and TypeScript source files', () => {
		expect(checkedExtensions.has('.vue')).toBe(true);
		expect(checkedExtensions.has('.ts')).toBe(true);
		expect(checkedExtensions.has('.tsx')).toBe(true);
		expect(isProductionSourceFile('src/domains/communications/components/ComposeDrawer.vue')).toBe(true);
		expect(isProductionSourceFile('src/domains/communications/queries/realtimeMailPatches.ts')).toBe(true);
	});

	it('excludes test files from the production source gate', () => {
		expect(isProductionSourceFile('src/platform/bootstrap/realtime.test.ts')).toBe(false);
		expect(isProductionSourceFile('src/domains/foo/__tests__/foo.ts')).toBe(false);
	});

	it('excludes generated and test files from the line-count architecture gate', () => {
		expect(isLineCountCheckedSourceFile('src/platform/bootstrap/realtime.test.ts')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/domains/foo/__tests__/foo.ts')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/domains/foo/__tests__/foo.vue')).toBe(false);
		expect(isLineCountCheckedSourceFile('src/gen/makosh/signal_hub/v1/signal_hub_pb.ts')).toBe(false);
	});

	it('treats 700 lines as a failure and 1000 lines as critical', () => {
		expect(classifyLineCount(499)).toEqual({ warning: false, failure: false, critical: false });
		expect(classifyLineCount(500)).toEqual({ warning: true, failure: false, critical: false });
		expect(classifyLineCount(700)).toEqual({ warning: true, failure: true, critical: false });
		expect(classifyLineCount(1000)).toEqual({ warning: true, failure: true, critical: true });
	});
});
```

### `frontend/scripts/check-no-inline-styles.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/check-no-inline-styles.mjs`
- Size bytes / Размер в байтах: `3855`
- Included characters / Включено символов: `3855`
- Truncated / Обрезано: `no`

```javascript
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';

const sourceRoots = ['src', 'public'];
const checkedExtensions = new Set([
	'.html',
	'.js',
	'.jsx',
	'.mjs',
	'.ts',
	'.tsx',
	'.vue'
]);
const forbiddenStylePatterns = [
	{
		pattern: /(?:^|[\s<])(?<binding>:style|v-bind:style|style)\s*=/,
		message: 'inline style attributes are forbidden; move styles to CSS files or an approved runtime layout primitive'
	}
];

const dynamicLayoutStyleAllowlist = new Set([
	'src/domains/communications/components/AttachmentSearchPanel.vue',
	'src/domains/communications/components/DraftStrip.vue',
	'src/domains/communications/components/CommunicationFolderStrip.vue',
	'src/domains/communications/components/CommunicationList.vue',
	'src/domains/communications/components/MailResourceOverviewStrip.vue',
	'src/domains/communications/components/SavedSearchStrip.vue',
	'src/domains/documents/components/DocumentsList.vue',
	'src/domains/knowledge/components/KnowledgeGraphCanvas.vue',
	'src/domains/notes/components/NotesList.vue',
	'src/domains/personas/components/PersonsList.vue',
	'src/domains/tasks/components/TaskList.vue',
	'src/integrations/telegram/components/TelegramChatList.vue',
	'src/domains/timeline/components/TimelineStream.vue',
	'src/integrations/whatsapp/components/WhatsAppSessionList.vue'
]);

async function collectSourceFiles(root) {
	if (!(await exists(root))) {
		return [];
	}

	const entries = await readdir(root, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(root, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectSourceFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && checkedExtensions.has(path.extname(entry.name))) {
			if (isTestFile(entryPath)) {
				continue;
			}
			files.push(entryPath);
		}
	}

	return files;
}

function isTestFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	return normalized.includes('/__tests__/') || /\.(test|spec)\.[cm]?[jt]sx?$/.test(normalized);
}

async function exists(root) {
	try {
		await stat(root);
		return true;
	} catch (error) {
		if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
			return false;
		}
		throw error;
	}
}

async function findStyleContractViolations() {
	const files = (await Promise.all(sourceRoots.map(collectSourceFiles))).flat();
	const violations = [];

	for (const file of files) {
		const source = await readFile(file, 'utf8');
		const scanLines = scanLinesForFile(file, source);
		const normalizedFile = file.split(path.sep).join('/');

		for (const { line, lineNumber } of scanLines) {
			for (const { pattern, message } of forbiddenStylePatterns) {
				const match = pattern.exec(line);
				if (match) {
					const binding = match.groups?.binding;
					const isAllowedDynamicLayout =
						binding !== 'style' && dynamicLayoutStyleAllowlist.has(normalizedFile);
					if (!isAllowedDynamicLayout) {
						violations.push(`${file}:${lineNumber}: ${message}`);
					}
				}
			}
	}
	}

	return violations;
}

function scanLinesForFile(file, source) {
	const lines = source.split('\n');
	const extension = path.extname(file);
	if (extension !== '.vue' && extension !== '.html') {
		return [];
	}

	if (extension === '.html') {
		return lines.map((line, index) => ({ line, lineNumber: index + 1 }));
	}

	const ranges = [];
	let inTemplate = false;
	for (const [index, line] of lines.entries()) {
		if (line.includes('<template')) {
			inTemplate = true;
		}
		if (inTemplate) {
			ranges.push({ line, lineNumber: index + 1 });
		}
		if (line.includes('</template>')) {
			inTemplate = false;
		}
	}

	return ranges;
}

const violations = await findStyleContractViolations();

if (violations.length > 0) {
	console.error(violations.join('\n'));
	process.exit(1);
}
```

### `frontend/scripts/generate-proto.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/generate-proto.mjs`
- Size bytes / Размер в байтах: `1164`
- Included characters / Включено символов: `1164`
- Truncated / Обрезано: `no`

```javascript
import { mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const frontendRoot = resolve(__dirname, '..')
const repoRoot = resolve(frontendRoot, '..')
const protoRoot = join(repoRoot, 'contracts', 'proto')
const outputDir = join(frontendRoot, 'src', 'gen')
const pluginPath = join(frontendRoot, 'node_modules', '.bin', 'protoc-gen-es')
const protoFiles = [
  join(protoRoot, 'makosh', 'common', 'v1', 'common.proto'),
  join(protoRoot, 'makosh', 'events', 'v1', 'event_envelope.proto'),
  join(protoRoot, 'makosh', 'signal_hub', 'v1', 'signal_hub.proto'),
  join(protoRoot, 'makosh', 'communications', 'v1', 'communications.proto')
]

mkdirSync(outputDir, { recursive: true })

const result = spawnSync(
  'protoc',
  [
    `-I${protoRoot}`,
    `--plugin=protoc-gen-es=${pluginPath}`,
    `--es_out=${outputDir}`,
    '--es_opt',
    'target=ts',
    ...protoFiles
  ],
  {
    cwd: frontendRoot,
    stdio: 'inherit'
  }
)

if (result.status !== 0) {
  process.exit(result.status ?? 1)
}
```

### `frontend/scripts/split-css.py`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/scripts/split-css.py`
- Size bytes / Размер в байтах: `14938`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```python
#!/usr/bin/env python3
"""Split app.css into component-scoped CSS files.
Parses flat rule blocks, classifies each, preserves @media wrappers."""

import os

def read_file(path):
    with open(path, 'r') as f:
        return f.read()

def write_file(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write(content)

# --- CSS block parsing (handles nesting) ---

def parse_css_blocks(text):
    """Parse CSS into blocks. Each block is (selector, body, is_atrule, full_text).
    For @media, returns inner blocks separately."""
    blocks = []
    i = 0
    n = len(text)

    while i < n:
        # Skip whitespace and comments
        while i < n and text[i] in ' \t\n\r':
            i += 1
        if i >= n:
            break

        # Skip comments
        if i + 1 < n and text[i] == '/' and text[i+1] == '*':
            j = text.find('*/', i+2)
            if j == -1:
                break
            i = j + 2
            continue

        # Find the start of a CSS rule
        start = i
        # Find opening brace
        brace = text.find('{', i)
        if brace == -1:
            break

        selector = text[start:brace].strip()

        # Find matching closing brace
        depth = 1
        j = brace + 1
        while j < n and depth > 0:
            if text[j] == '{':
                depth += 1
            elif text[j] == '}':
                depth -= 1
            j += 1

        if depth != 0:
            break

        body = text[brace+1:j-1]
        full = text[start:j]

        blocks.append({
            'selector': selector,
            'body': body,
            'full': full,
            'is_atrule': selector.startswith('@'),
        })

        i = j

    return blocks

def get_first_class_from_selector(sel):
    """Get the first CSS class from a single selector."""
    # Strip pseudo-classes/elements
    s = sel.split(':')[0].split('::')[0].strip()
    # Get last class in case of descendant selectors
    parts = s.split()
    for p in parts:
        if p.startswith('.'):
            return p
    return ''

def selector_matches_prefixes(sel_text, prefixes):
    """Check if ALL comma-separated selectors match any of the prefixes."""
    selectors = [s.strip() for s in sel_text.split(',')]
    for s in selectors:
        cls = get_first_class_from_selector(s)
        if not cls:
            return False
        matched = False
        for pfx in prefixes:
            if cls.startswith(pfx):
                matched = True
                break
        if not matched:
            return False
    return True

def block_matches_prefixes(block, prefixes):
    """Check if a block matches (handles @media by checking inner rules)."""
    if block['is_atrule']:
        sel = block['selector']
        if sel.startswith('@keyframes'):
            # Check keyframes name against prefixes
            parts = sel.split()
            name = parts[1] if len(parts) > 1 else ''
            for pfx in prefixes:
                if pfx.startswith('@keyframes'):
                    kf_name = pfx.replace('@keyframes ', '').replace('@keyframes', '')
                    if name.startswith(kf_name):
                        return True
            return False
        elif sel.startswith('@media'):
            # Check ALL inner blocks match
            inner_blocks = parse_css_blocks(block['body'])
            if not inner_blocks:
                return False
            for ib in inner_blocks:
                if not block_matches_prefixes(ib, prefixes):
                    return False
            return True
        else:
            return False
    else:
        return selector_matches_prefixes(block['selector'], prefixes)

def extract_from_media_block(block, prefixes):
    """Extract matching inner rules from a @media block.
    Returns (matched_inner_blocks, unmatched_inner_blocks)."""
    inner_blocks = parse_css_blocks(block['body'])
    matched = []
    unmatched = []

    for ib in inner_blocks:
        if block_matches_prefixes(ib, prefixes):
            matched.append(ib)
        else:
            unmatched.append(ib)

    return matched, unmatched

def blocks_to_css(blocks):
    """Convert list of block dicts to CSS text."""
    return '\n\n'.join(b['full'].strip() for b in blocks) + '\n'

def make_media_block(media_selector, inner_blocks):
    """Create a @media block wrapping inner blocks."""
    inner_css = '\n'.join(b['full'].strip() for b in inner_blocks)
    return f'{media_selector} {{\n{inner_css}\n}}'

# --- Group definitions ---

GROUP_PREFIXES = {
    'vault': ['.vault-'],
    'sidebar': [
        '.sidebar', '.brand', '.brand-mark-button', '.brand-mark',
        '.brand-name', '.brand-subtitle', '.brand-copy',
        '.nav-group', '.nav-group-label', '.nav-entry',
        '.primary-nav', '.nav-disclosure',
        '.communications-subnav', '.communications-rail-dropdown',
        '.subnav-', '.sidebar-tools', '.settings-link',
        '.sidebar-rail-dropdown-backdrop',
        '.sidebar-settings-', '.sidebar-group-create', '.sidebar-config-',
        '.sidebar-preview-',
    ],
    'topbar': [
        '.topbar', '.topbar-title', '.top-actions',
        '.user-menu', '.user-menu-',
        '.menu-button', '.icon-button', '.segmented',
        '.search-bar', '.view-header', '.view-title-with-icon',
        '.hero-mark', '.section-tabs', '.pill-tabs',
        '.filter-bar', '.filter-tabs',
        '.primary-button', '.ghost-button', '.link-row', '.link-button',
        '.kbd',
    ],
    'notifications': [
        '.notifications-', '.notification-',
        '.workspace', '.workspace-status-strip',
    ],
    'panels': [
        '.panel', '.panel-', '.widget-frame', '.widget-',
        '.info-card', '.metric-grid', '.metric-card',
        '.stacked-rail', '.empty-panel', '.muted-copy',
        '.detail-list', '.detail-row', '.health-row', '.round-icon',
        '.chip', '.status-chip', '.health-chip', '.deadline',
        '.bar-row', '.mini-check', '.collection-row',
        '.source-card', '.source-strip', '.table-head', '.task-row',
        '.doc-row', '.person-compact', '.profile-panel', '.quick-icons',
        '.chat-pane', '.chat-body', '.chat-actions', '.bubble',
        '.date-divider', '.composer', '.conversation-list',
        '.feed-panel', '.feed-row', '.person-list',
        '.schedule-panel', '.schedule-list', '.compact-project',
        '.project-card-row', '.full-band', '.score-ring', '.donut',
        '.chart-panel',
        '.persons-list-panel', '.profile-head', '.person-row',
        '.hero-row', '.layout-edit-controls', '.layout-zone',
        '.communication-empty-', '.related-row', '.inline-error',
        '.inline-metrics', '.inline-copy', '.summary-numbers',
        '.home-metrics', '.radial-graph', '.graph-center', '.graph-chip',
        '.source-footer', '.source-badge',
        '.status-list', '.timeline-mini', '.big-score', '.progress',
        '.task-stack', '.task-actions', '.task-row-actions',
        '.feed-list', '.left-panels', '.new-tile',
        '.mail-', '.state-badge', '.draft-', '.health-strip',
        '.search-hint', '.importance-dot',
        '.layout-widget-', '.widget-drawer',
    ],
    'pages': [
        '.home-page', '.dashboard-grid', '.communications-page',
        '.three-pane', '.context-rail', '.persons-page', '.projects-page',
        '.tasks-page', '.tasks-layout', '.calendar-page', '.calendar-layout',
        '.documents-page', '.documents-layout', '.notes-page', '.notes-layout',
        '.knowledge-page', '.agents-page', '.agents-layout',
        '.organizations-page', '.org-layout', '.timeline-page', '.timeline-layout',
        '.settings-page', '.settings-layout', '.agent-card',
        '.org-row', '.event-row', '.note-card', '.document-row',
        '.project-side', '.telegram-rail', '.whatsapp-rail',
        '.person-detail', '.identity-candidate',
        '.graph-canvas', '.graph-node',
        '.week-board', '.event-list', '.new-event-form', '.org-detail-grid',
        '.timeline-event-row', '.task-table', '.task-group',
        '.docs-table', '.category-grid', '.tag-cloud', '.notes-list',
        '.settings-', '.setting-', '.appearance-', '.brightness-', '.accent-swatch',
        '.account-', '.wizard-', '.setup-', '.provider-', '.qr-',
        '.org-list-panel', '.org-detail-',
        '.project-hero', '.project-logo', '.project-meta-strip',
        '.project-empty-state', '.project-dashboard-grid', '.project-switcher',
        '.graph-', '.knowledge-', '.agent-', '.ai-', '.evidence-',
        '.timeline-', '.spark-', '.large-timeline',
        '.person-hero', '.person-cards', '.person-detail',
        '.doc-mini', '.notes-main', '.agent-main',
        '.identity-', '.node-detail-',
        '.conversation-', '.attachment-',
        '.event-block', '.time-grid', '.now-line', '.week-header',
        '.event-actions', '.event-detail', '.event-meta', '.event-type-chip',
        '.agenda-list', '.brief-', '.participant-chip',
        '.telegram-chat-pane', '.telegram-grid', '.telegram-inline-form',
        '.whatsapp-chat-pane', '.whatsapp-grid',
        '.calendar-layout', '.knowledge-layout', '.agents-layout',
        '.persons-layout', '.docs-layout', '.notes-layout',
        '.communications-grid', '.compact-form', '.checkbox-row',
        '.form-actions', '.form-row', '.oauth-box',
        '.drawer-backdrop', '.modal-backdrop', '.rail-dot',
        '.graph-chip-',
        '.citation-', '.background-option-', '.background-preview',
    ],
}

OUTPUT_FILES = {
    'vault': 'frontend/src/lib/components/vault/vault.css',
    'sidebar': 'frontend/src/lib/components/shell/sidebar.css',
    'topbar': 'frontend/src/lib/components/shell/topbar.css',
    'notifications': 'frontend/src/lib/components/shell/notifications.css',
    'panels': 'frontend/src/lib/components/shared/panels.css',
    'pages': 'frontend/src/lib/pages/pages.css',
}

IMPORTS = {
    'vault': ('frontend/src/lib/components/vault/VaultOnboarding.svelte', "import './vault.css';"),
    'sidebar': ('frontend/src/lib/components/shell/Sidebar.svelte', "import './sidebar.css';"),
    'topbar': ('frontend/src/lib/components/shell/Topbar.svelte', "import './topbar.css';"),
    'notifications': ('frontend/src/lib/components/shell/NotificationsDrawer.svelte', "import './notifications.css';"),
    'panels': ('frontend/src/lib/components/shared/WidgetEditChrome.svelte', "import './panels.css';"),
    'pages': ('frontend/src/routes/+layout.svelte', "import '$lib/pages/pages.css';"),
}

def process_blocks(blocks):
    """Process blocks recursively. Returns (extracted_by_group, remaining_blocks)."""
    extracted = {name: [] for name in GROUP_PREFIXES}
    remaining = []

    for block in blocks:
        if block['is_atrule'] and block['selector'].startswith('@media'):
            # Process @media block - extract inner rules per group
            inner_blocks = parse_css_blocks(block['body'])
            inner_extracted, inner_remaining = process_blocks(inner_blocks)

            # Add extracted inner blocks to their groups, wrapped in @media
            for group_name, inner_list in inner_extracted.items():
                if inner_list:
                    media_wrapper = make_media_block(block['selector'], inner_list)
                    extracted[group_name].append({
                        'selector': block['selector'],
                        'body': '\n'.join(b['full'] for b in inner_list),
                        'full': media_wrapper,
                        'is_atrule': True,
                    })

            # Keep remaining inner blocks in @media wrapper
            if inner_remaining:
                media_wrapper = make_media_block(block['selector'], inner_remaining)
                remaining.append({
                    'selector': block['selector'],
                    'body': '\n'.join(b['full'] for b in inner_remaining),
                    'full': media_wrapper,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src-tauri/build.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/build.rs`
- Size bytes / Размер в байтах: `634`
- Included characters / Включено символов: `634`
- Truncated / Обрезано: `no`

```rust
const APP_COMMANDS: &[&str] = &[
    "open_whatsapp_web_companion",
    "whatsapp_web_companion_manifest",
    "whatsapp_web_companion_relay_observation",
    "open_yandex_telemost_companion",
    "yandex_telemost_companion_manifest",
    "yandex_telemost_prepare_audio_device",
    "yandex_telemost_recording_start",
    "yandex_telemost_recording_stop",
    "yandex_telemost_speaker_timeline_append",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run tauri build script")
}
```

### `frontend/src-tauri/src/lib.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/src/lib.rs`
- Size bytes / Размер в байтах: `7040`
- Included characters / Включено символов: `7040`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

mod whatsapp_companion;
mod yandex_telemost_companion;

#[derive(Default)]
struct BackendSidecar {
    child: Mutex<Option<CommandChild>>,
}

impl Drop for BackendSidecar {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            if let Some(child) = child.take() {
                let _ = child.kill();
            }
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            whatsapp_companion::open_whatsapp_web_companion,
            whatsapp_companion::whatsapp_web_companion_manifest,
            whatsapp_companion::whatsapp_web_companion_relay_observation,
            yandex_telemost_companion::open_yandex_telemost_companion,
            yandex_telemost_companion::yandex_telemost_companion_manifest,
            yandex_telemost_companion::yandex_telemost_prepare_audio_device,
            yandex_telemost_companion::yandex_telemost_recording_start,
            yandex_telemost_companion::yandex_telemost_recording_stop,
            yandex_telemost_companion::yandex_telemost_speaker_timeline_append,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(BackendSidecar::default());
            app.manage(yandex_telemost_companion::TelemostLocalRecorder::default());
            if !cfg!(debug_assertions) {
                start_backend_sidecar(app.handle())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_backend_sidecar<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("MAKOSH_DISABLE_BACKEND_SIDECAR").is_some() {
        log::info!("Макошь backend sidecar disabled by MAKOSH_DISABLE_BACKEND_SIDECAR");
        return Ok(());
    }

    let mut command = app
        .shell()
        .sidecar("makosh-backend")?
        .env("MAKOSH_HTTP_ADDR", "127.0.0.1:8080")
        .env(
            "MAKOSH_LOCAL_API_SECRET",
            std::env::var_os("MAKOSH_LOCAL_API_SECRET")
                .unwrap_or_else(|| "change-me-local-api-secret".into()),
        );

    for key in [
        "DATABASE_URL",
        "MAKOSH_SECRET_VAULT_KEY",
        "MAKOSH_OLLAMA_BASE_URL",
        "MAKOSH_OLLAMA_CHAT_MODEL",
        "MAKOSH_OLLAMA_EMBED_MODEL",
        "MAKOSH_OLLAMA_TIMEOUT_SECONDS",
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH",
        "MAKOSH_GOOGLE_OAUTH_CLIENT_ID",
        "MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command = command.env(key, value);
        }
    }
    if let Some(value) = std::env::var_os("MAKOSH_TELEGRAM_API_ID")
        .or_else(|| option_env!("MAKOSH_BUNDLED_TELEGRAM_API_ID").map(std::ffi::OsString::from))
    {
        command = command.env("MAKOSH_TELEGRAM_API_ID", value);
    }
    if let Some(value) = std::env::var_os("MAKOSH_TELEGRAM_API_HASH")
        .or_else(|| option_env!("MAKOSH_BUNDLED_TELEGRAM_API_HASH").map(std::ffi::OsString::from))
    {
        command = command.env("MAKOSH_TELEGRAM_API_HASH", value);
    }

    if std::env::var_os("MAKOSH_TDJSON_PATH").is_none() {
        if let Some(tdjson_path) = bundled_tdjson_path(app) {
            command = command.env("MAKOSH_TDJSON_PATH", tdjson_path);
        }
    }
    if std::env::var_os("MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH").is_none()
        && std::env::var_os("MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON").is_none()
        && std::env::var_os("MAKOSH_GOOGLE_OAUTH_CLIENT_ID").is_none()
    {
        if let Some(google_oauth_client_config_path) = bundled_google_oauth_client_config_path(app)
        {
            command = command.env(
                "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH",
                google_oauth_client_config_path,
            );
        }
    }

    let (mut events, child) = command.spawn()?;
    let pid = child.pid();
    app.state::<BackendSidecar>()
        .child
        .lock()
        .map_err(|_| std::io::Error::other("backend sidecar state lock poisoned"))?
        .replace(child);

    tauri::async_runtime::spawn(async move {
        log::info!("Макошь backend sidecar started with pid {pid}");
        while let Some(event) = events.recv().await {
            match event {
                CommandEvent::Stdout(line) => log_sidecar_line(log::Level::Info, &line),
                CommandEvent::Stderr(line) => log_sidecar_line(log::Level::Warn, &line),
                CommandEvent::Error(error) => {
                    log::error!("Макошь backend sidecar event error: {error}");
                }
                CommandEvent::Terminated(payload) => {
                    log::warn!(
                        "Макошь backend sidecar terminated: code={:?} signal={:?}",
                        payload.code,
                        payload.signal
                    );
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn log_sidecar_line(level: log::Level, bytes: &[u8]) {
    let line = String::from_utf8_lossy(bytes).trim().to_owned();
    if line.is_empty() {
        return;
    }
    log::log!(level, "Макошь backend sidecar: {line}");
}

fn bundled_tdjson_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    let tdlib_dir = resource_dir.join("tdlib");
    let platform_path = tdlib_dir
        .join(tdlib_platform_dir())
        .join(tdlib_library_file_name());
    if platform_path.is_file() {
        return Some(platform_path);
    }

    let universal_path = tdlib_dir
        .join("macos-universal")
        .join(tdlib_library_file_name());
    universal_path.is_file().then_some(universal_path)
}

fn bundled_google_oauth_client_config_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    let client_config_path = resource_dir.join("google-oauth").join("client_secret.json");
    client_config_path.is_file().then_some(client_config_path)
}

fn tdlib_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "macos-arm64";
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "macos-x64";
    }
    #[allow(unreachable_code)]
    "unknown"
}

fn tdlib_library_file_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "libtdjson.dylib";
    }
    #[allow(unreachable_code)]
    "libtdjson"
}
```

### `frontend/src-tauri/src/main.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/src/main.rs`
- Size bytes / Размер в байтах: `179`
- Included characters / Включено символов: `179`
- Truncated / Обрезано: `no`

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}
```

### `frontend/src-tauri/src/whatsapp_companion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/src/whatsapp_companion.rs`
- Size bytes / Размер в байтах: `31838`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "whatsapp_web_companion";
const RUNTIME_KIND: &str = "webview_companion";
const WINDOW_LABEL_PREFIX: &str = "whatsapp-companion";
const WHATSAPP_WEB_URL: &str = "https://web.whatsapp.com/";
const DEFAULT_BACKEND_HTTP_ADDR: &str = "127.0.0.1:8080";
const RUNTIME_EVENTS_BRIDGE_PATH: &str =
    "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRequest {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) driver_id: &'static str,
    pub(crate) window_label: String,
    pub(crate) target_url: &'static str,
    pub(crate) opened_window: bool,
    pub(crate) focused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) tauri_ipc_available_to_companion_window: bool,
    pub(crate) event_flow: &'static str,
    pub(crate) event_extractor: WhatsAppWebCompanionExtractorContract,
    pub(crate) bridge_routes: WhatsAppWebCompanionBridgeRoutes,
    pub(crate) command_channel: WhatsAppWebCompanionCommandChannel,
    pub(crate) secret_policy: WhatsAppWebCompanionSecretPolicy,
    pub(crate) remaining_blockers: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionExtractorContract {
    pub(crate) state: &'static str,
    pub(crate) initialization_script: &'static str,
    pub(crate) script_scope: &'static str,
    pub(crate) origin_guard: &'static str,
    pub(crate) navigation_guard: &'static str,
    pub(crate) relay_channel: &'static str,
    pub(crate) runtime_bridge_dispatch: &'static str,
    pub(crate) allowed_observations: Vec<&'static str>,
    pub(crate) forbidden_reads: Vec<&'static str>,
    pub(crate) next_gate: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionBridgeRoutes {
    pub(crate) authorized_session_path: &'static str,
    pub(crate) runtime_event_path: &'static str,
    pub(crate) sync_lifecycle_path: &'static str,
    pub(crate) message_paths: Vec<&'static str>,
    pub(crate) conversation_paths: Vec<&'static str>,
    pub(crate) media_paths: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionCommandChannel {
    pub(crate) kind: &'static str,
    pub(crate) claim_path: &'static str,
    pub(crate) failure_path: &'static str,
    pub(crate) completion_rule: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionSecretPolicy {
    pub(crate) session_material: &'static str,
    pub(crate) cookies: &'static str,
    pub(crate) browser_profile_secrets: &'static str,
    pub(crate) qr_pair_code_artifacts: &'static str,
    pub(crate) message_bodies: &'static str,
    pub(crate) media_bytes: &'static str,
    pub(crate) postgres_storage: &'static str,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRelayObservationRequest {
    pub(crate) account_id: String,
    pub(crate) event_family: String,
    pub(crate) provider_event_id: String,
    pub(crate) observed_at: String,
    #[serde(default)]
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionRelayObservationReceipt {
    pub(crate) account_id: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) window_label: String,
    pub(crate) event_family: String,
    pub(crate) provider_event_id: String,
    pub(crate) observed_at: String,
    pub(crate) target_runtime_bridge_path: &'static str,
    pub(crate) typed_runtime_bridge_path: &'static str,
    pub(crate) relay_state: &'static str,
    pub(crate) relay_channel: &'static str,
    pub(crate) sanitized_metadata: Value,
    pub(crate) runtime_event_kind: String,
    pub(crate) import_batch_id: String,
    pub(crate) runtime_bridge_http_status: u16,
    pub(crate) event_flow: &'static str,
    pub(crate) completion_rule: &'static str,
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_manifest(
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    manifest_for_account(&request.account_id, false, false)
}

#[tauri::command]
pub(crate) async fn open_whatsapp_web_companion(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let window_label = companion_window_label(&request.account_id)?;
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|error| format!("failed to show WhatsApp companion window: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("failed to focus WhatsApp companion window: {error}"))?;
        return manifest_for_account(&request.account_id, false, true);
    }

    let url = WHATSAPP_WEB_URL
        .parse()
        .map_err(|error| format!("invalid WhatsApp Web URL: {error}"))?;
    let initialization_script =
        companion_initialization_script(&request.account_id, &window_label)?;
    let window = WebviewWindowBuilder::new(&app, window_label, WebviewUrl::External(url))
        .title("WhatsApp Web Companion")
        .visible(true)
        .resizable(true)
        .inner_size(1160.0, 780.0)
        .initialization_script(initialization_script)
        .on_navigation(|url| url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com"))
        .build()
        .map_err(|error| format!("failed to open WhatsApp companion window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus WhatsApp companion window: {error}"))?;

    manifest_for_account(&request.account_id, true, false)
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_relay_observation(
    webview_window: WebviewWindow,
    request: WhatsAppWebCompanionRelayObservationRequest,
) -> Result<WhatsAppWebCompanionRelayObservationReceipt, String> {
    let account_id = required_account_id(&request.account_id)?;
    let expected_window_label = companion_window_label(account_id)?;
    if webview_window.label() != expected_window_label {
        return Err(format!(
            "WhatsApp companion relay rejected caller window {} for account {}",
            webview_window.label(),
            account_id
        ));
    }

    let event_family = required_ascii_slug("event_family", &request.event_family)?;
    let provider_event_id = required_ascii_slug("provider_event_id", &request.provider_event_id)?;
    let observed_at = required_observed_at(&request.observed_at)?;
    let typed_runtime_bridge_path =
        runtime_bridge_path_for_event_family(event_family).ok_or_else(|| {
            format!("unsupported WhatsApp companion relay event family {event_family}")
        })?;
    let sanitized_metadata = sanitize_relay_metadata(request.metadata);
    let runtime_event_kind = format!("webview_companion.{event_family}.observed");
    let import_batch_id = runtime_bridge_import_batch_id(account_id, provider_event_id);
    let runtime_event_payload = runtime_bridge_runtime_event_payload(
        account_id,
        event_family,
        provider_event_id,
        observed_at,
        typed_runtime_bridge_path,
        &runtime_event_kind,
        &import_batch_id,
        sanitized_metadata.clone(),
    );
    let runtime_bridge_http_status = tauri::async_runtime::spawn_blocking(move || {
        dispatch_runtime_bridge_runtime_event(runtime_event_payload)
    })
    .await
    .map_err(|error| format!("WhatsApp companion relay dispatch task failed: {error}"))??;

    Ok(WhatsAppWebCompanionRelayObservationReceipt {
        account_id: account_id.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        window_label: expected_window_label,
        event_family: event_family.to_owned(),
        provider_event_id: provider_event_id.to_owned(),
        observed_at: observed_at.to_owned(),
        target_runtime_bridge_path: RUNTIME_EVENTS_BRIDGE_PATH,
        typed_runtime_bridge_path,
        relay_state: "dispatched_to_backend_runtime_bridge_runtime_event",
        relay_channel: "tauri_allowlisted_companion_runtime_bridge_dispatch",
        sanitized_metadata,
        runtime_event_kind,
        import_batch_id,
        runtime_bridge_http_status,
        event_flow: "visible_webview_companion -> tauri_allowlisted_relay_preflight -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation",
        completion_rule: "provider_observed_event_reconciliation_required",
    })
}

fn manifest_for_account(
    account_id: &str,
    opened_window: bool,
    focused_existing_window: bool,
) -> Result<WhatsAppWebCompanionManifest, String> {
    Ok(WhatsAppWebCompanionManifest {
        account_id: required_account_id(account_id)?.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        driver_id: "tauri_visible_webview_companion",
        window_label: companion_window_label(account_id)?,
        target_url: WHATSAPP_WEB_URL,
        opened_window,
        focused_existing_window,
        owner_visible: true,
        hidden_headless_mode: "forbidden",
        tauri_ipc_available_to_companion_window: true,
        event_flow: "visible_webview_companion -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation",
        event_extractor: WhatsAppWebCompanionExtractorContract {
            state: "contract_injected_relay_dispatch_available",
            initialization_script: "installed_on_visible_companion_window",
            script_scope: "main_frame_only",
            origin_guard: "https://web.whatsapp.com",
            navigation_guard: "https://web.whatsapp.com_only",
            relay_channel: "tauri_allowlisted_companion_runtime_bridge_dispatch",
            runtime_bridge_dispatch: "runtime_events_bridge_wired_smoke_pending",
            allowed_observations: vec![
                "runtime_lifecycle_metadata",
                "sync_lifecycle_metadata",
                "message_identity_metadata",
                "receipt_metadata",
                "reaction_metadata",
                "dialog_metadata",
                "participant_metadata",
                "presence_metadata",
                "call_metadata",
                "status_metadata",
                "media_metadata_without_bytes",
            ],
            forbidden_reads: vec![
                "cookies",
                "web_storage",
                "indexed_db",
                "browser_profile_secrets",
                "session_material",
                "message_bodies",
                "media_bytes",
            ],
            next_gate: "manual_live_smoke_before_public_availability",
        },
        bridge_routes: WhatsAppWebCompanionBridgeRoutes {
            authorized_session_path: "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            runtime_event_path: "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            sync_lifecycle_path: "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            message_paths: vec![
                "/api/v1/integrations/whatsapp/runtime-bridge/messages",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
                "/api/
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src-tauri/src/yandex_telemost_companion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/src/yandex_telemost_companion.rs`
- Size bytes / Размер в байтах: `30833`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "yandex_telemost_user";
const RUNTIME_KIND: &str = "yandex_telemost_webview_runtime";
const WINDOW_LABEL_PREFIX: &str = "yandex-telemost";
const TELEMOST_ALLOWED_HOST_RU: &str = "telemost.yandex.ru";
const TELEMOST_ALLOWED_HOST_COM: &str = "telemost.yandex.com";
const DEFAULT_FFMPEG_PATH: &str = "ffmpeg";
const DEFAULT_LINUX_MONITOR: &str = "makosh_telemost.monitor";

#[derive(Default)]
pub(crate) struct TelemostLocalRecorder {
    sessions: Mutex<HashMap<String, RecordingProcess>>,
}

impl Drop for TelemostLocalRecorder {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (_, mut session) in sessions.drain() {
                let _ = session.child.kill();
                let _ = session.child.wait();
            }
        }
    }
}

struct RecordingProcess {
    child: Child,
    manifest: YandexTelemostRecordingSession,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostCompanionOpenRequest {
    pub(crate) account_id: String,
    pub(crate) join_url: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) window_label: String,
    pub(crate) opened_window: bool,
    pub(crate) focused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) allowed_hosts: Vec<&'static str>,
    pub(crate) speaker_timeline: SpeakerTimelineContract,
    pub(crate) recorder: LocalRecorderContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SpeakerTimelineContract {
    pub(crate) state: &'static str,
    pub(crate) source: &'static str,
    pub(crate) truth_status: &'static str,
    pub(crate) output_files: Vec<&'static str>,
    pub(crate) cadence_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalRecorderContract {
    pub(crate) state: &'static str,
    pub(crate) audio_format: &'static str,
    pub(crate) consent_attestation_required: bool,
    pub(crate) ffmpeg_path_env: &'static str,
    pub(crate) ffmpeg_input_env: &'static str,
    pub(crate) default_linux_input: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostAudioDevicePrepareRequest {
    #[serde(default)]
    pub(crate) device_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostAudioDeviceManifest {
    pub(crate) platform: &'static str,
    pub(crate) state: &'static str,
    pub(crate) input_hint: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostRecordingStartRequest {
    pub(crate) account_id: String,
    pub(crate) join_url: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) window_label: Option<String>,
    #[serde(default)]
    pub(crate) audio_input: Option<String>,
    #[serde(default)]
    pub(crate) consent_attested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostRecordingStopRequest {
    pub(crate) recording_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostRecordingSession {
    pub(crate) recording_session_id: String,
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) window_label: String,
    pub(crate) output_dir: String,
    pub(crate) audio_path: String,
    pub(crate) speaker_jsonl_path: String,
    pub(crate) speaker_txt_path: String,
    pub(crate) ffmpeg_pid: Option<u32>,
    pub(crate) started_at_epoch_ms: u128,
    pub(crate) consent_attested: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostRecordingStopReceipt {
    pub(crate) recording_session_id: String,
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) audio_path: String,
    pub(crate) speaker_jsonl_path: String,
    pub(crate) speaker_txt_path: String,
    pub(crate) stopped_at_epoch_ms: u128,
    pub(crate) state: &'static str,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct YandexTelemostSpeakerTimelineAppendRequest {
    pub(crate) account_id: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) recording_session_id: Option<String>,
    pub(crate) speaker_label: String,
    #[serde(default)]
    pub(crate) confidence: Option<f32>,
    #[serde(default)]
    pub(crate) observed_at_epoch_ms: Option<u128>,
    #[serde(default)]
    pub(crate) source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostSpeakerTimelineAppendReceipt {
    pub(crate) recording_session_id: Option<String>,
    pub(crate) accepted: bool,
    pub(crate) reason: &'static str,
}

#[tauri::command]
pub(crate) async fn yandex_telemost_companion_manifest(
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let label = companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    Ok(manifest_for_request(request, label, false, false))
}

#[tauri::command]
pub(crate) async fn open_yandex_telemost_companion(
    app: AppHandle,
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let window_label =
        companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|error| format!("failed to show Telemost window: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("failed to focus Telemost window: {error}"))?;
        return Ok(manifest_for_request(request, window_label, false, true));
    }

    let url = request
        .join_url
        .parse()
        .map_err(|error| format!("invalid Yandex Telemost join URL: {error}"))?;
    let initialization_script = telemost_initialization_script(&request, &window_label)?;
    let window = WebviewWindowBuilder::new(&app, window_label.clone(), WebviewUrl::External(url))
        .title("Yandex Telemost · Макошь")
        .visible(true)
        .resizable(true)
        .inner_size(1220.0, 820.0)
        .initialization_script(initialization_script)
        .on_navigation(|url| {
            url.scheme() == "https"
                && matches!(
                    url.host_str(),
                    Some(TELEMOST_ALLOWED_HOST_RU) | Some(TELEMOST_ALLOWED_HOST_COM)
                )
        })
        .build()
        .map_err(|error| format!("failed to open Yandex Telemost window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus Telemost window: {error}"))?;

    Ok(manifest_for_request(request, window_label, true, false))
}

#[tauri::command]
pub(crate) async fn yandex_telemost_prepare_audio_device(
    request: YandexTelemostAudioDevicePrepareRequest,
) -> Result<YandexTelemostAudioDeviceManifest, String> {
    prepare_audio_device(request)
}

#[tauri::command]
pub(crate) async fn yandex_telemost_recording_start(
    app: AppHandle,
    state: State<'_, TelemostLocalRecorder>,
    request: YandexTelemostRecordingStartRequest,
) -> Result<YandexTelemostRecordingSession, String> {
    if !request.consent_attested {
        return Err("recording requires explicit consent_attested=true; Макошь will not start hidden conference capture".to_owned());
    }
    validate_join_url(&request.join_url)?;
    let account_id = required_slug("account_id", &request.account_id)?;
    let window_label = match request
        .window_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => value.to_owned(),
        None => companion_window_label(account_id, request.conference_id.as_deref())?,
    };
    let session_id = recording_session_id(account_id, request.conference_id.as_deref());
    let output_dir = recording_output_dir(&app, account_id, &session_id)?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create Telemost recording dir: {error}"))?;
    let audio_path = output_dir.join("audio.mp3");
    let speaker_jsonl_path = output_dir.join("speaker-timeline.jsonl");
    let speaker_txt_path = output_dir.join("speaker-timeline.txt");
    write_timeline_header(&speaker_txt_path, &request, &session_id)?;
    let mut command = ffmpeg_recording_command(request.audio_input.as_deref(), &audio_path)?;
    let child = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start ffmpeg Telemost recorder: {error}"))?;
    let manifest = YandexTelemostRecordingSession {
        recording_session_id: session_id.clone(),
        account_id: account_id.to_owned(),
        conference_id: request.conference_id.clone(),
        join_url: request.join_url.clone(),
        window_label,
        output_dir: output_dir.to_string_lossy().into_owned(),
        audio_path: audio_path.to_string_lossy().into_owned(),
        speaker_jsonl_path: speaker_jsonl_path.to_string_lossy().into_owned(),
        speaker_txt_path: speaker_txt_path.to_string_lossy().into_owned(),
        ffmpeg_pid: Some(child.id()),
        started_at_epoch_ms: now_epoch_ms(),
        consent_attested: true,
    };
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Telemost recorder state lock poisoned".to_owned())?;
    if sessions.contains_key(&session_id) {
        return Err(format!(
            "Telemost recording session `{session_id}` is already running"
        ));
    }
    sessions.insert(
        session_id,
        RecordingProcess {
            child,
            manifest: manifest.clone(),
        },
    );
    Ok(manifest)
}

#[tauri::command]
pub(crate) async fn yandex_telemost_recording_stop(
    state: State<'_, TelemostLocalRecorder>,
    request: YandexTelemostRecordingStopRequest,
) -> Result<YandexTelemostRecordingStopReceipt, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Telemost recorder state lock poisoned".to_owned())?;
    let mut process = sessions
        .remove(request.recording_session_id.trim())
        .ok_or_else(|| {
            format!(
                "Telemost recording session `{}` was not found",
                request.recording_session_id.trim()
            )
        })?;
    let _ = process.child.kill();
    let _ = process.child.wait();
    append_text_line(
        Path::new(&process.manifest.speaker_txt_path),
        &format!(
            "{}\tSYSTEM\trecording_stop\tconfidence=1.00\tsource=local_recorder",
            now_epoch_ms()
        ),
    )?;
    Ok(YandexTelemostRecordingStopReceipt {
        recording_session_id: process.manifest.recording_session_id,
        account_id: process.manifest.account_id,
        conference_id:
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/__tests__/apiClient.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/__tests__/apiClient.test.ts`
- Size bytes / Размер в байтах: `3905`
- Included characters / Включено символов: `3903`
- Truncated / Обрезано: `no`

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ApiClient } from '@/platform/api/ApiClient'

describe('ApiClient', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
	})

	it('throws if accessed before init', () => {
		expect(() => ApiClient.instance).toThrow('ApiClient not initialized')
	})

	it('initializes with baseUrl and secret', () => {
		const client = ApiClient.init('http://localhost:3000', 'test-secret')
		expect(client).toBeInstanceOf(ApiClient)
		expect(ApiClient.instance).toBe(client)
	})

	it('rejects an empty secret', () => {
		expect(() => ApiClient.init('http://localhost:3000', '   ')).toThrow(
			'X-Макошь-Secret cannot be empty'
		)
	})

	it('strips trailing slash from baseUrl', () => {
		const client = ApiClient.init('http://localhost:3000/', 'secret')
		// Private field access for test — we verify behavior via GET request
		expect(client).toBeInstanceOf(ApiClient)
	})

	it('sends X-Макошь-Secret header with GET requests', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ data: 'test' })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'my-secret')
		await ApiClient.instance.get('/api/v1/test')

		expect(mockFetch).toHaveBeenCalledTimes(1)
		const [url, options] = mockFetch.mock.calls[0]
		expect(url).toBe('http://localhost:3000/api/v1/test')
		expect(options.headers['X-Макошь-Secret']).toBe('my-secret')
		expect(options.method).toBe('GET')

		vi.unstubAllGlobals()
	})

	it('handles 204 No Content responses', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 204,
			json: () => Promise.resolve('should not be called')
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		const result = await ApiClient.instance.get('/api/v1/empty')

		expect(result).toBeUndefined()

		vi.unstubAllGlobals()
	})

	it('throws ApiError on non-ok response', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 403,
			text: () => Promise.resolve('forbidden')
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')

		await expect(
			ApiClient.instance.get('/api/v1/protected')
		).rejects.toMatchObject({
			message: 'forbidden',
			status: 403
		})

		vi.unstubAllGlobals()
	})

	it('sends JSON body with POST requests', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ id: 1 })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		await ApiClient.instance.post('/api/v1/create', { name: 'test' })

		const [, options] = mockFetch.mock.calls[0]
		expect(options.method).toBe('POST')
		expect(options.headers['Content-Type']).toBe('application/json')
		expect(JSON.parse(options.body)).toEqual({ name: 'test' })

		vi.unstubAllGlobals()
	})

	it('sends DELETE request without body', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ deleted: true })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		await ApiClient.instance.delete('/api/v1/item/1')

		const [, options] = mockFetch.mock.calls[0]
		expect(options.method).toBe('DELETE')
		expect(options.body).toBeUndefined()

		vi.unstubAllGlobals()
	})

	it('falls back to default error message when text() fails', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 500,
			text: () => Promise.reject(new Error('parse failed'))
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')

		await expect(
			ApiClient.instance.get('/api/v1/error')
		).rejects.toMatchObject({
			status: 500
		})

		vi.unstubAllGlobals()
	})
})
```

### `frontend/src/__tests__/sanitizeEmailHtml.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/__tests__/sanitizeEmailHtml.test.ts`
- Size bytes / Размер в байтах: `3080`
- Included characters / Включено символов: `3080`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { renderMessageBody, sanitizeEmailHtml } from '@/shared/sanitize/emailHtml'

describe('sanitizeEmailHtml', () => {
	it('removes active content, event handlers, inline styles, and javascript urls', () => {
		const sanitized = sanitizeEmailHtml(`
			<div onclick="steal()" style="color: red">
				<script>alert('x')</script>
				<a href="javascript:alert(1)" onmouseover="steal()">open</a>
				<img src="javascript:alert(1)" onerror="steal()" alt="tracker">
				<form action="/send"><button>Send</button></form>
				<svg><script>alert('svg')</script></svg>
			</div>
		`)

		expect(sanitized).toContain('<div>')
		expect(sanitized).toContain('<a>open</a>')
		expect(sanitized).toContain('<img alt="tracker">')
		expect(sanitized).not.toContain('script')
		expect(sanitized).not.toContain('onclick')
		expect(sanitized).not.toContain('onmouseover')
		expect(sanitized).not.toContain('onerror')
		expect(sanitized).not.toContain('style=')
		expect(sanitized).not.toContain('javascript:')
		expect(sanitized).not.toContain('<form')
		expect(sanitized).not.toContain('<svg')
	})

	it('keeps a constrained set of email formatting tags and safe links', () => {
		const sanitized = sanitizeEmailHtml(`
			<p>Hello <b>Alex</b>, <i>see</i> <a href="https://example.com?q=1&x=2">details</a>.</p>
			<blockquote cite="https://ignored.example">Quoted</blockquote>
			<table><tr><td colspan="2">Cell</td></tr></table>
		`)

		expect(sanitized).toContain('<p>Hello <strong>Alex</strong>, <em>see</em> ')
		expect(sanitized).toContain(
			'<a href="https://example.com?q=1&amp;x=2" target="_blank" rel="noreferrer noopener">details</a>'
		)
		expect(sanitized).toContain('<blockquote>Quoted</blockquote>')
		expect(sanitized).toContain('<table><tr><td colspan="2">Cell</td></tr></table>')
		expect(sanitized).not.toContain('cite=')
	})

	it('rejects obfuscated unsafe urls without throwing on malformed entities', () => {
		const sanitized = sanitizeEmailHtml(`
			<a href="java&#x0a;script&colon;alert(1)">bad link</a>
			<img src="mailto:person@example.com" alt="bad image">
			<a href="&#999999999999999999999999;">invalid entity</a>
		`)

		expect(sanitized).toContain('<a>bad link</a>')
		expect(sanitized).toContain('<img alt="bad image">')
		expect(sanitized).toContain('<a>invalid entity</a>')
		expect(sanitized).not.toContain('javascript')
		expect(sanitized).not.toContain('mailto:person@example.com')
	})
})

describe('renderMessageBody', () => {
	it('renders HTML bodies through the sanitizer', () => {
		const rendered = renderMessageBody({
			bodyHtml: '<p>Safe</p><script>alert(1)</script>',
			bodyText: 'fallback'
		})

		expect(rendered.kind).toBe('html')
		expect(rendered.html).toBe('<p>Safe</p>')
	})

	it('escapes plain text and preserves line breaks', () => {
		const rendered = renderMessageBody({
			bodyHtml: null,
			bodyText: 'Hello <script>alert(1)</script>\nSecond line'
		})

		expect(rendered.kind).toBe('plain')
		expect(rendered.html).toBe('Hello &lt;script&gt;alert(1)&lt;/script&gt;<br>Second line')
	})
})
```

### `frontend/src/app/router.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/router.ts`
- Size bytes / Размер в байтах: `2100`
- Included characters / Включено символов: `2100`
- Truncated / Обрезано: `no`

```typescript
import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

import HomeView from './views/HomeView.vue'
import CommunicationsView from './views/CommunicationsView.vue'
import TimelineView from './views/TimelineView.vue'
import PersonsView from './views/PersonsView.vue'
import ProjectsView from './views/ProjectsView.vue'
import TasksView from './views/TasksView.vue'
import CalendarView from './views/CalendarView.vue'
import DocumentsView from './views/DocumentsView.vue'
import NotesView from './views/NotesView.vue'
import KnowledgeView from './views/KnowledgeView.vue'
import ReviewView from './views/ReviewView.vue'
import SettingsView from './views/SettingsView.vue'
import AgentsView from './views/AgentsView.vue'
import OrganizationsView from './views/OrganizationsView.vue'
import EventTracingView from './views/EventTracingView.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/home' },
  { path: '/home', name: 'home', component: HomeView },
  { path: '/communications', name: 'communications', component: CommunicationsView },
  { path: '/timeline', name: 'timeline', component: TimelineView },
  { path: '/persons', name: 'persons', component: PersonsView },
  { path: '/projects', name: 'projects', component: ProjectsView },
  { path: '/tasks', name: 'tasks', component: TasksView },
  { path: '/calendar', name: 'calendar', component: CalendarView },
  { path: '/documents', name: 'documents', component: DocumentsView },
  { path: '/notes', name: 'notes', component: NotesView },
  { path: '/knowledge', name: 'knowledge', component: KnowledgeView },
  { path: '/review', name: 'review', component: ReviewView },
  { path: '/event-tracing', name: 'event-tracing', component: EventTracingView },
  { path: '/settings', name: 'settings', component: SettingsView },
  { path: '/agents', name: 'agents', component: AgentsView },
  { path: '/organizations', name: 'organizations', component: OrganizationsView }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
```

### `frontend/src/config/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/config/index.ts`
- Size bytes / Размер в байтах: `104`
- Included characters / Включено символов: `104`
- Truncated / Обрезано: `no`

```typescript
import { loadFrontendConfig } from '../platform/config/env'

export const config = loadFrontendConfig()
```

### `frontend/src/domains/agents/api/agents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/api/agents.ts`
- Size bytes / Размер в байтах: `1876`
- Included characters / Включено символов: `1876`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
	AiStatus,
	AiAgentListResponse,
	AiRunListResponse,
	AiAnswerRequest,
	AiAnswerResponse,
	AiMeetingPrepRequest,
	AiMeetingPrepResponse,
	AiTaskCandidateRefreshRequest,
	AiTaskCandidateRefreshResponse,
	OwnerPersonaResponse
} from '../types/agents'

export async function fetchAiStatus(): Promise<AiStatus> {
	return ApiClient.instance.get<AiStatus>('/api/v1/ai/status', 'AI status request failed')
}

export async function fetchAiAgents(): Promise<AiAgentListResponse> {
	return ApiClient.instance.get<AiAgentListResponse>('/api/v1/ai/agents', 'AI agents request failed')
}

export async function fetchAiRuns(limit = 25): Promise<AiRunListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	return ApiClient.instance.get<AiRunListResponse>(
		`/api/v1/ai/runs?${params.toString()}`,
		'AI run history request failed'
	)
}

export async function fetchOwnerPersona(): Promise<OwnerPersonaResponse> {
	return ApiClient.instance.get<OwnerPersonaResponse>(
		'/api/v1/persons/owner',
		'Owner persona request failed'
	)
}

export async function requestAiAnswer(request: AiAnswerRequest): Promise<AiAnswerResponse> {
	return ApiClient.instance.post<AiAnswerResponse>('/api/v1/ai/answers', request, 'AI answer request failed')
}

export async function requestAiMeetingPrep(request: AiMeetingPrepRequest): Promise<AiMeetingPrepResponse> {
	return ApiClient.instance.post<AiMeetingPrepResponse>(
		'/api/v1/ai/meeting-prep',
		request,
		'AI meeting prep request failed'
	)
}

export async function refreshAiTaskCandidates(
	request: AiTaskCandidateRefreshRequest
): Promise<AiTaskCandidateRefreshResponse> {
	return ApiClient.instance.post<AiTaskCandidateRefreshResponse>(
		'/api/v1/ai/task-candidates/refresh',
		request,
		'AI task candidate refresh request failed'
	)
}
```

### `frontend/src/domains/agents/queries/useAgentsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/queries/useAgentsQuery.ts`
- Size bytes / Размер в байтах: `1144`
- Included characters / Включено символов: `1144`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchAiAgents, fetchAiRuns, fetchAiStatus, fetchOwnerPersona } from '../api/agents'

export function useAiWorkspaceQuery() {
	return useQuery({
		queryKey: ['ai-workspace'],
		queryFn: async () => {
			const [agentResponse, runResponse, ownerResponse] = await Promise.all([
				fetchAiAgents(),
				fetchAiRuns(25),
				fetchOwnerPersona()
			])
			const agents = agentResponse.items
			const runs = runResponse.items
			const ownerPersona = ownerResponse.owner_persona
			let aiStatus: Awaited<ReturnType<typeof fetchAiStatus>> | null = null
			let error = ''
			try {
				aiStatus = await fetchAiStatus()
			} catch (statusError) {
				error = statusError instanceof Error ? statusError.message : 'Unknown AI status error'
			}
			return { agents, runs, status: aiStatus, ownerPersona, error }
		},
		refetchOnMount: 'always' as const,
		staleTime: 30_000
	})
}

export function useAiRunsQuery() {
	return useQuery({
		queryKey: ['ai-runs'],
		queryFn: async () => {
			const response = await fetchAiRuns(25)
			return { runs: response.items, error: '' }
		},
		staleTime: 10_000
	})
}
```

### `frontend/src/domains/agents/stores/agents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/stores/agents.ts`
- Size bytes / Размер в байтах: `7351`
- Included characters / Включено символов: `7351`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	AiStatus,
	AiAgent,
	AiRun,
	AiAnswerResponse,
	AiMeetingPrepResponse,
	AiTaskCandidateRefreshResponse,
	AiCitation,
	OwnerPersona,
	AgentCard
} from '../types/agents'
import {
	requestAiAnswer,
	requestAiMeetingPrep,
	refreshAiTaskCandidates,
	fetchAiRuns
} from '../api/agents'

export const useAgentsStore = defineStore('agents-ui', () => {
	const aiStatus = ref<AiStatus | null>(null)
	const aiAgents = ref<AiAgent[]>([])
	const aiRuns = ref<AiRun[]>([])
	const ownerPersona = ref<OwnerPersona | null>(null)
	const aiError = ref('')
	const isAiLoading = ref(false)
	const isAiAnswerSubmitting = ref(false)
	const isAiMeetingPrepSubmitting = ref(false)
	const isAiTaskRefreshSubmitting = ref(false)
	const selectedAgentIndex = ref(0)
	const aiQuestion = ref('What does the local memory say about Макошь V3?')
	const aiMeetingTopic = ref('Prepare a V3 implementation review brief')
	const aiTaskQuery = ref('Find open task candidates from local messages and documents')
	const aiAnswerResult = ref<AiAnswerResponse | null>(null)
	const aiMeetingPrepResult = ref<AiMeetingPrepResponse | null>(null)
	const aiTaskRefreshResult = ref<AiTaskCandidateRefreshResponse | null>(null)

	const agentCards = computed<AgentCard[]>(() =>
		aiAgents.value.map((agent) => agentCardView(agent, aiRuns.value))
	)

	const selectedAgent = computed<AgentCard | null>(() =>
		agentCards.value[selectedAgentIndex.value] ?? agentCards.value[0] ?? null
	)

	function setWorkspace(data: {
		agents: AiAgent[]
		runs: AiRun[]
		status: AiStatus | null
		ownerPersona: OwnerPersona | null
		error: string
	}) {
		aiAgents.value = data.agents
		aiRuns.value = data.runs
		aiStatus.value = data.status
		ownerPersona.value = data.ownerPersona
		aiError.value = data.error
		if (selectedAgentIndex.value >= aiAgents.value.length) {
			selectedAgentIndex.value = 0
		}
	}

	function setLoading(v: boolean) {
		isAiLoading.value = v
	}

	function selectAgent(index: number) {
		selectedAgentIndex.value = index
	}

	async function submitAiAnswer() {
		const query = aiQuestion.value.trim()
		if (!query || isAiAnswerSubmitting.value) return
		isAiAnswerSubmitting.value = true
		try {
			const result = await requestAiAnswer({
				command_id: `ai-answer-${crypto.randomUUID()}`,
				query,
				agent_id: selectedAgent.value?.agentId ?? 'MNEMOSYNE'
			})
			aiAnswerResult.value = result
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiAnswerResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI answer error'
		} finally {
			isAiAnswerSubmitting.value = false
		}
	}

	async function prepareAiBrief() {
		const topic = aiMeetingTopic.value.trim()
		if (!topic || isAiMeetingPrepSubmitting.value) return
		isAiMeetingPrepSubmitting.value = true
		try {
			const result = await requestAiMeetingPrep({
				command_id: `ai-meeting-prep-${crypto.randomUUID()}`,
				topic
			})
			aiMeetingPrepResult.value = result
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiMeetingPrepResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI meeting prep error'
		} finally {
			isAiMeetingPrepSubmitting.value = false
		}
	}

	async function refreshTasksFromAi() {
		const query = aiTaskQuery.value.trim()
		if (!query || isAiTaskRefreshSubmitting.value) return
		isAiTaskRefreshSubmitting.value = true
		try {
			const result = await refreshAiTaskCandidates({
				command_id: `ai-task-refresh-${crypto.randomUUID()}`,
				query
			})
			aiTaskRefreshResult.value = result
			aiError.value = ''
			await loadAiRunsOnly()
		} catch (error) {
			aiTaskRefreshResult.value = null
			aiError.value = error instanceof Error ? error.message : 'Unknown AI task refresh error'
		} finally {
			isAiTaskRefreshSubmitting.value = false
		}
	}

	async function loadAiRunsOnly() {
		try {
			const response = await fetchAiRuns(25)
			aiRuns.value = response.items
		} catch (error) {
			aiError.value = error instanceof Error ? error.message : 'Unknown AI run history error'
		}
	}

	return {
		aiStatus,
		aiAgents,
		aiRuns,
		ownerPersona,
		aiError,
		isAiLoading,
		isAiAnswerSubmitting,
		isAiMeetingPrepSubmitting,
		isAiTaskRefreshSubmitting,
		selectedAgentIndex,
		aiQuestion,
		aiMeetingTopic,
		aiTaskQuery,
		aiAnswerResult,
		aiMeetingPrepResult,
		aiTaskRefreshResult,
		agentCards,
		selectedAgent,
		setWorkspace,
		setLoading,
		selectAgent,
		submitAiAnswer,
		prepareAiBrief,
		refreshTasksFromAi,
		loadAiRunsOnly
	}
})

function agentCardView(agent: AiAgent, aiRuns: AiRun[]): AgentCard {
	const visual = agentVisual(agent.agent_id)
	const runs = aiRuns.filter((run) => run.agent_id === agent.agent_id)
	const completed = runs.filter((run) => run.status === 'completed').length
	const success = runs.length > 0 ? Math.round((completed / runs.length) * 100) : 0

	return {
		agentId: agent.agent_id,
		name: agent.persona_email ?? aiAgentPersonaEmail(agent.agent_id),
		summary: agent.role,
		icon: visual.icon,
		tasks: runs.length,
		success,
		status: agent.status,
		tone: visual.tone,
		model: agent.default_model
	}
}

function aiAgentPersonaEmail(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}

function agentVisual(agentId: string): { icon: string; tone: string } {
	switch (agentId) {
		case 'HESTIA':
			return { icon: 'tabler:calendar-stats', tone: 'mint' }
		case 'MAKOSH':
			return { icon: 'tabler:route', tone: 'blue' }
		case 'MNEMOSYNE':
			return { icon: 'tabler:database-search', tone: 'purple' }
		case 'ATHENA':
			return { icon: 'tabler:target-arrow', tone: 'amber' }
		default:
			return { icon: 'tabler:sparkles', tone: 'cyan' }
	}
}

export function aiRuntimeSummary(aiStatus: AiStatus | null, isAiLoading: boolean): string {
	if (!aiStatus) return isAiLoading ? 'Loading' : 'Unknown'
	return aiStatus.status === 'ok' ? 'Ready' : 'Unavailable'
}

export function aiModelSummary(aiStatus: AiStatus | null): string {
	if (!aiStatus) return 'No status'
	return `${aiStatus.chat_model} / ${aiStatus.embedding_model}`
}

export function runStatusLabel(run: AiRun): string {
	if (run.status === 'completed') return 'Completed'
	if (run.status === 'failed') return 'Failed'
	return 'Requested'
}

export function formatDuration(durationMs: number | null | undefined): string {
	if (durationMs == null) return 'n/a'
	if (durationMs < 1000) return `${durationMs} ms`
	return `${(durationMs / 1000).toFixed(1)} s`
}

export function formatDateTime(date: string): string {
	const d = new Date(date)
	if (Number.isNaN(d.getTime())) return 'Invalid date'
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(d)
}

export function safeCitations(value: unknown): AiCitation[] {
	if (!Array.isArray(value)) return []
	return value.filter(isAiCitation)
}

function isAiCitation(value: unknown): value is AiCitation {
	return (
		typeof value === 'object' &&
		value !== null &&
		typeof (value as Record<string, unknown>).source_kind === 'string' &&
		typeof (value as Record<string, unknown>).source_id === 'string' &&
		typeof (value as Record<string, unknown>).title === 'string' &&
		typeof (value as Record<string, unknown>).excerpt === 'string'
	)
}
```

### `frontend/src/domains/agents/types/agents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/types/agents.ts`
- Size bytes / Размер в байтах: `2739`
- Included characters / Включено символов: `2739`
- Truncated / Обрезано: `no`

```typescript
export interface AiStatus {
	runtime: string
	status: string
	version: string | null
	chat_model: string
	embedding_model: string
	embedding_dimension: number
	chat_model_available: boolean
	embedding_model_available: boolean
}

export interface AiAgent {
	agent_id: string
	display_name: string
	role: string
	default_model: string
	status: string
	persona_id?: string
	persona_type?: string
	persona_email?: string
}

export interface AiAgentListResponse {
	items: AiAgent[]
}

export interface AiCitation {
	source_kind: string
	source_id: string
	title: string
	excerpt: string
	score: number
	graph_node_id?: string
}

export interface AiRun {
	run_id: string
	agent_id: string
	status: string
	chat_model: string
	embedding_model: string
	prompt_template_version: string
	model_config: Record<string, unknown>
	query: string
	answer: string | null
	citations: AiCitation[] | unknown[]
	error_summary: string | null
	actor_id: string
	causation_id: string | null
	correlation_id: string | null
	requested_event_id: string | null
	completed_event_id: string | null
	failed_event_id: string | null
	started_at: string
	completed_at: string | null
	duration_ms: number | null
	created_at: string
	updated_at: string
}

export interface AiRunListResponse {
	items: AiRun[]
}

export interface OwnerPersona {
	person_id: string
	display_name: string
	email_address: string
	persona_type: string
	is_self: boolean
	created_at: string
	updated_at: string
}

export interface OwnerPersonaResponse {
	owner_persona: OwnerPersona | null
}

export interface AiAnswerRequest {
	command_id: string
	query: string
	agent_id?: string
	correlation_id?: string
}

export interface AiAnswerResponse {
	run_id: string
	agent_id: string
	status: string
	answer: string
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AiMeetingPrepRequest {
	command_id: string
	topic: string
	project_id?: string
	person_id?: string
	correlation_id?: string
}

export interface AiMeetingPrepResponse {
	run_id: string
	agent_id: string
	status: string
	briefing: string
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AiTaskCandidateRefreshRequest {
	command_id: string
	query: string
	correlation_id?: string
}

export interface AiTaskCandidateRefreshResponse {
	run_id: string
	agent_id: string
	status: string
	created_count: number
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AgentCard {
	agentId: string
	name: string
	icon: string
	tone: string
	summary: string
	status: string
	model: string
	tasks: number
	success: number
}
```

### `frontend/src/domains/calendar/api/calendar.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/api/calendar.ts`
- Size bytes / Размер в байтах: `5158`
- Included characters / Включено символов: `5158`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CalendarAccountsResponse,
  CalendarSourcesResponse,
  CalendarEventsResponse,
  CalendarEvent,
  CalendarAccount,
  EventParticipantsResponse,
  EventContextPack,
  EventAgenda,
  MeetingNotesResponse,
  MeetingOutcomesResponse,
  DeadlinesResponse,
  CalendarFetchParams
} from '../types/calendar'

export async function fetchCalendarAccounts(provider?: string): Promise<CalendarAccountsResponse> {
  const params = new URLSearchParams()
  if (provider) params.set('provider', provider)
  return ApiClient.instance.get<CalendarAccountsResponse>(
    `/api/v1/calendar/accounts?${params.toString()}`,
    'Calendar accounts request failed'
  )
}

export async function createCalendarAccount(
  body: { provider: string; account_name: string; email?: string }
): Promise<CalendarAccount> {
  return ApiClient.instance.post<CalendarAccount>(
    '/api/v1/calendar/accounts',
    body,
    'Create calendar account failed'
  )
}

export async function fetchCalendarSources(accountId: string): Promise<CalendarSourcesResponse> {
  return ApiClient.instance.get<CalendarSourcesResponse>(
    `/api/v1/calendar/accounts/${encodeURIComponent(accountId)}/sources`,
    'Calendar sources request failed'
  )
}

export async function fetchCalendarEvents(
  params: CalendarFetchParams = {}
): Promise<CalendarEventsResponse> {
  const sp = new URLSearchParams()
  if (params.account_id) sp.set('account_id', params.account_id)
  if (params.source_id) sp.set('source_id', params.source_id)
  if (params.from) sp.set('from', params.from)
  if (params.to) sp.set('to', params.to)
  if (params.status) sp.set('status', params.status)
  if (params.event_type) sp.set('event_type', params.event_type)
  if (params.limit) sp.set('limit', String(params.limit))
  return ApiClient.instance.get<CalendarEventsResponse>(
    `/api/v1/calendar/events?${sp.toString()}`,
    'Calendar events request failed'
  )
}

export async function createCalendarEvent(
  body: {
    title: string
    start_at: string
    end_at: string
    description?: string
    location?: string
    event_type?: string
    account_id?: string
    source_id?: string
    timezone?: string
    all_day?: boolean
  }
): Promise<CalendarEvent> {
  return ApiClient.instance.post<CalendarEvent>('/api/v1/calendar/events', body, 'Create event failed')
}

export async function deleteCalendarEvent(eventId: string): Promise<{ deleted: boolean }> {
  return ApiClient.instance.delete<{ deleted: boolean }>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}`,
    'Delete event failed'
  )
}

export async function fetchEventParticipants(eventId: string): Promise<EventParticipantsResponse> {
  return ApiClient.instance.get<EventParticipantsResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/participants`,
    'Participants request failed'
  )
}

export async function fetchEventContextPack(eventId: string): Promise<EventContextPack | null> {
  return ApiClient.instance.get<EventContextPack | null>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/context-pack`,
    'Context pack request failed'
  )
}

export async function fetchEventAgenda(eventId: string): Promise<EventAgenda | null> {
  return ApiClient.instance.get<EventAgenda | null>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/agenda`,
    'Agenda request failed'
  )
}

export async function fetchEventBrief(eventId: string): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/brief`,
    'Brief request failed'
  )
}

export async function fetchMeetingNotes(eventId: string): Promise<MeetingNotesResponse> {
  return ApiClient.instance.get<MeetingNotesResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/notes`,
    'Notes request failed'
  )
}

export async function fetchMeetingOutcomes(eventId: string): Promise<MeetingOutcomesResponse> {
  return ApiClient.instance.get<MeetingOutcomesResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/outcomes`,
    'Outcomes request failed'
  )
}

export async function fetchDeadlines(status?: string): Promise<DeadlinesResponse> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  return ApiClient.instance.get<DeadlinesResponse>(
    `/api/v1/calendar/deadlines?${params.toString()}`,
    'Deadlines request failed'
  )
}

export async function fetchCalendarWatchtower(): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    '/api/v1/calendar/watchtower',
    'Watchtower request failed'
  )
}

export async function fetchWeeklyBrief(): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    '/api/v1/calendar/weekly-brief',
    'Weekly brief request failed'
  )
}

export async function searchCalendarEvents(q: string): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    `/api/v1/calendar/search?q=${encodeURIComponent(q)}`,
    'Calendar search failed'
  )
}
```

### `frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts`
- Size bytes / Размер в байтах: `681`
- Included characters / Включено символов: `681`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchCalendarAccounts, fetchCalendarEvents } from '../api/calendar'
import type { CalendarAccount, CalendarEvent } from '../types/calendar'

export function useCalendarAccountsQuery() {
  return useQuery<CalendarAccount[]>({
    queryKey: ['calendar-accounts'],
    queryFn: async () => {
      const res = await fetchCalendarAccounts()
      return res.items
    }
  })
}

export function useCalendarEventsQuery(limit = 200) {
  return useQuery<CalendarEvent[]>({
    queryKey: ['calendar-events', limit],
    queryFn: async () => {
      const res = await fetchCalendarEvents({ limit })
      return res.items
    }
  })
}
```

### `frontend/src/domains/calendar/stores/calendar.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/stores/calendar.ts`
- Size bytes / Размер в байтах: `4405`
- Included characters / Включено символов: `4405`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { format, formatDistanceToNow } from 'date-fns'
import type { CalendarEvent, CalendarViewMode, WeeklyBrief } from '../types/calendar'

export const useCalendarStore = defineStore('calendar-ui', () => {
  const viewMode = ref<CalendarViewMode>('week')
  const searchQuery = ref('')
  const selectedEvent = ref<CalendarEvent | null>(null)
  const calendarError = ref('')
  const isCalendarLoading = ref(false)
  const showNewEventForm = ref(false)
  const newEventTitle = ref('')
  const newEventStart = ref('')
  const newEventEnd = ref('')
  const newEventType = ref('meeting')
  const weeklyBrief = ref<WeeklyBrief | null>(null)
  const eventBrief = ref<Record<string, unknown> | null>(null)
  const eventAgenda = ref<Record<string, unknown> | null>(null)

  function setViewMode(mode: CalendarViewMode) {
    viewMode.value = mode
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function selectEvent(evt: CalendarEvent | null) {
    selectedEvent.value = evt
    if (!evt) {
      eventBrief.value = null
      eventAgenda.value = null
    }
  }

  function setCalendarError(error: string) {
    calendarError.value = error
  }

  function setCalendarLoading(loading: boolean) {
    isCalendarLoading.value = loading
  }

  function toggleNewEventForm() {
    showNewEventForm.value = !showNewEventForm.value
  }

  function resetNewEventForm() {
    newEventTitle.value = ''
    newEventStart.value = ''
    newEventEnd.value = ''
    newEventType.value = 'meeting'
    showNewEventForm.value = false
  }

  function setWeeklyBrief(brief: WeeklyBrief | null) {
    weeklyBrief.value = brief
  }

  function setEventBrief(brief: Record<string, unknown> | null) {
    eventBrief.value = brief
  }

  function setEventAgenda(agenda: Record<string, unknown> | null) {
    eventAgenda.value = agenda
  }

  return {
    viewMode, searchQuery, selectedEvent, calendarError, isCalendarLoading,
    showNewEventForm, newEventTitle, newEventStart, newEventEnd, newEventType,
    weeklyBrief, eventBrief, eventAgenda,
    setViewMode, setSearchQuery, selectEvent, setCalendarError, setCalendarLoading,
    toggleNewEventForm, resetNewEventForm,
    setWeeklyBrief, setEventBrief, setEventAgenda
  }
})

// --- Utility functions ---

export function formatEventDate(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE, MMM d')
}

export function formatEventTime(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'HH:mm')
}

export function formatEventDateTime(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE, MMM d HH:mm')
}

export function formatEventDayShort(dateStr: string): string {
  const d = new Date(dateStr)
  return format(d, 'EEE d')
}

export function formatRelativeTime(dateStr: string): string {
  const d = new Date(dateStr)
  const now = new Date()
  if (d < now) return `${formatDistanceToNow(d)} ago`
  return formatDistanceToNow(d, { addSuffix: true })
}

export function eventTypeTone(eventType: string | null): string {
  switch (eventType) {
    case 'meeting': return 'blue'
    case 'deadline': return 'red'
    case 'focus': return 'green'
    default: return 'neutral'
  }
}

export function eventTypeLabel(type: string): string {
  return type.charAt(0).toUpperCase() + type.slice(1)
}

export const WEEK_DAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN']

export const EVENT_TYPE_OPTIONS = [
  'meeting', 'focus', 'deadline', 'personal', 'travel', 'tax', 'review', 'planning'
] as const

export function getWeekStart(): Date {
  const now = new Date()
  const start = new Date(now)
  start.setDate(now.getDate() - now.getDay() + 1)
  start.setHours(0, 0, 0, 0)
  return start
}

export function getWeekEnd(weekStart: Date): Date {
  const end = new Date(weekStart)
  end.setDate(weekStart.getDate() + 7)
  return end
}

export function getWeekColumns(weekStart: Date): string[] {
  return WEEK_DAYS.map((d, i) => {
    const d2 = new Date(weekStart)
    d2.setDate(weekStart.getDate() + i)
    return `${d} ${d2.getDate()}`
  })
}

export function filterWeekEvents(events: CalendarEvent[], weekStart: Date): CalendarEvent[] {
  const end = getWeekEnd(weekStart)
  return events.filter(e => {
    const start = new Date(e.start_at)
    return start >= weekStart && start < end
  })
}
```

### `frontend/src/domains/calendar/types/calendar.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/calendar/types/calendar.ts`
- Size bytes / Размер в байтах: `3617`
- Included characters / Включено символов: `3617`
- Truncated / Обрезано: `no`

```typescript
export interface CalendarAccount {
  account_id: string
  provider: string
  account_name: string
  email: string | null
  credentials_reference: string | null
  sync_status: string
  capabilities: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CalendarAccountsResponse {
  items: CalendarAccount[]
}

export interface CalendarSource {
  source_id: string
  account_id: string
  provider_calendar_id: string | null
  name: string
  color: string | null
  timezone: string | null
  visibility: string
  read_only: boolean
  sync_enabled: boolean
  capabilities: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CalendarSourcesResponse {
  items: CalendarSource[]
}

export interface CalendarEvent {
  event_id: string
  source_event_id: string | null
  account_id: string | null
  source_id: string | null
  title: string
  description: string | null
  location: string | null
  start_at: string
  end_at: string
  timezone: string | null
  all_day: boolean
  recurrence_rule: string | null
  status: string
  visibility: string
  event_type: string | null
  importance_score: number | null
  readiness_score: number | null
  sync_status: string
  created_at: string
  updated_at: string
}

export interface CalendarEventsResponse {
  items: CalendarEvent[]
}

export interface EventParticipant {
  id: string
  event_id: string
  person_id: string | null
  email: string
  display_name: string | null
  role: string
  response_status: string
  organization_id: string | null
  timezone: string | null
  confidence: number
  created_at: string
}

export interface EventParticipantsResponse {
  items: EventParticipant[]
}

export interface EventContextPack {
  id: string
  event_id: string
  summary: string | null
  participants_summary: string | null
  documents: unknown[]
  tasks: unknown[]
  open_questions: unknown[]
  risks: unknown[]
  suggested_agenda: unknown[]
  suggested_actions: unknown[]
  generated_at: string
  model: string | null
  created_at: string
  updated_at: string
}

export interface EventAgenda {
  id: string
  event_id: string
  items: unknown[]
  source: string
  created_by: string | null
  created_at: string
  updated_at: string
}

export interface MeetingNote {
  id: string
  event_id: string
  content: string
  format: string
  source: string
  linked_note_id: string | null
  created_at: string
  updated_at: string
}

export interface MeetingNotesResponse {
  items: MeetingNote[]
}

export interface MeetingOutcome {
  id: string
  event_id: string
  outcome_type: string
  title: string
  description: string | null
  owner_person_id: string | null
  due_date: string | null
  source: string
  confidence: number
  linked_entity_id: string | null
  created_at: string
  updated_at: string
}

export interface MeetingOutcomesResponse {
  items: MeetingOutcome[]
}

export interface DeadlineEvent {
  id: string
  source_entity_type: string | null
  source_entity_id: string | null
  title: string
  due_at: string
  severity: string
  status: string
  linked_calendar_event_id: string | null
  created_at: string
  updated_at: string
}

export interface DeadlinesResponse {
  items: DeadlineEvent[]
}

export type CalendarViewMode = 'day' | 'week' | 'month' | 'agenda'

export interface WeeklyBrief {
  upcoming_events_this_week: number
  overdue_deadlines: number
  past_events_without_notes: number
  [key: string]: unknown
}

export interface CalendarFetchParams {
  account_id?: string
  source_id?: string
  from?: string
  to?: string
  status?: string
  event_type?: string
  limit?: number
}
```

### `frontend/src/domains/communications/api/aiState.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/api/aiState.test.ts`
- Size bytes / Размер в байтах: `1730`
- Included characters / Включено символов: `1730`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchMessageAiState,
  updateMessageAiState
} from './aiState'

describe('communication AI state API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('gets and updates first-class message AI state', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg:1', ai_state: 'NEW' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message_id: 'msg:1', ai_state: 'REVIEW_REQUIRED' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchMessageAiState('msg:1')
    await updateMessageAiState('msg:1', {
      ai_state: 'REVIEW_REQUIRED',
      review_reason: 'Needs owner review'
    })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/communications/messages/msg%3A1/ai-state')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/communications/messages/msg%3A1/ai-state')
    expect(fetchMock.mock.calls[1][1].method).toBe('PUT')
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      ai_state: 'REVIEW_REQUIRED',
      review_reason: 'Needs owner review'
    })
  })
})
```
