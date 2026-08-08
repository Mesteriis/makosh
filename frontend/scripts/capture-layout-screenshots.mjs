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
					if (mode === 'both') return true;
					if (axis === 'vertical') return mode === 'vertical';
					if (axis === 'horizontal') return mode === 'horizontal';
					return false;
				};
				const unauthorizedScrollContainers = [];
				const workspace = document.querySelector('.workspace');
				for (const element of workspace?.querySelectorAll('*') ?? []) {
					const { rect, visible } = isElementVisible(element);
					if (!visible) continue;

					const style = getComputedStyle(element);
					const scrollsVertically =
						/(auto|scroll|overlay)/.test(style.overflowY) &&
						element.scrollHeight > element.clientHeight + 1;
					const scrollsHorizontally =
						/(auto|scroll|overlay)/.test(style.overflowX) &&
						element.scrollWidth > element.clientWidth + 1;
					if (!scrollsVertically && !scrollsHorizontally) continue;

					const widget = element.closest('.widget-frame[data-widget-id]');
					const widgetScrollMode = widget?.getAttribute('data-widget-scroll') ?? 'none';
					const verticalAllowed =
						!scrollsVertically || (widget !== null && scrollModeAllowsAxis(widgetScrollMode, 'vertical'));
					const horizontalAllowed =
						!scrollsHorizontally ||
						(widget !== null && scrollModeAllowsAxis(widgetScrollMode, 'horizontal'));

					if (!verticalAllowed || !horizontalAllowed) {
						unauthorizedScrollContainers.push({
							tag: element.tagName.toLowerCase(),
							className: typeof element.className === 'string' ? element.className : '',
							widgetId: widget?.getAttribute('data-widget-id') ?? null,
							widgetScrollMode,
							scrollsVertically,
							scrollsHorizontally,
							clientWidth: element.clientWidth,
							scrollWidth: element.scrollWidth,
							clientHeight: element.clientHeight,
							scrollHeight: element.scrollHeight,
							left: Math.round(rect.left),
							right: Math.round(rect.right),
							top: Math.round(rect.top),
							bottom: Math.round(rect.bottom),
							text: (element.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 80)
						});
					}
					if (unauthorizedScrollContainers.length >= 10) break;
				}

				const shell = document.querySelector('.desktop-shell');
				const sidebar = document.querySelector('.sidebar');
				const shellRect = shell?.getBoundingClientRect() ?? null;
				const sidebarRect = sidebar?.getBoundingClientRect() ?? null;
				const workspaceRect = workspace?.getBoundingClientRect() ?? null;
				const roundMetric = (value) => Math.round(value * 100) / 100;
				const shellMetrics = {
					shellBottomGap: shellRect === null ? null : roundMetric(window.innerHeight - shellRect.bottom),
					sidebarBottomGap: sidebarRect === null ? null : roundMetric(window.innerHeight - sidebarRect.bottom),
					workspaceBottomGap: workspaceRect === null ? null : roundMetric(window.innerHeight - workspaceRect.bottom)
				};

				return {
					h1: document.querySelector('h1')?.textContent?.trim() ?? null,
					scrollX: window.scrollX,
					scrollY: window.scrollY,
					bodyScrollWidth: document.body.scrollWidth,
					bodyScrollHeight: document.body.scrollHeight,
					documentScrollWidth: document.documentElement.scrollWidth,
					documentScrollHeight: document.documentElement.scrollHeight,
					outliers,
					shellMetrics,
					widgetFrameMetrics: {
						widgetUnit,
						widgetRow,
						layoutGap,
						visibleCount: visibleWidgetFrames.length,
						nonModularHeights: visibleWidgetFrames.filter((frame) => !frame.moduleAligned && !frame.rowAligned)
					},
					layoutColumnMetrics,
					unauthorizedScrollContainers
				};
			});
			const state = {
				h1: layoutState.h1,
				scrollX: layoutState.scrollX,
				scrollY: layoutState.scrollY,
				bodyScrollWidth: layoutState.bodyScrollWidth,
				bodyScrollHeight: layoutState.bodyScrollHeight,
				documentScrollWidth: layoutState.documentScrollWidth,
				documentScrollHeight: layoutState.documentScrollHeight,
				guardDisplay: await getViewportGuardDisplay(page, `capturing ${view.label} view state at ${viewport.id}`),
				outliers: layoutState.outliers,
				shellMetrics: layoutState.shellMetrics,
				widgetFrameMetrics: layoutState.widgetFrameMetrics,
				layoutColumnMetrics: layoutState.layoutColumnMetrics,
				unauthorizedScrollContainers: layoutState.unauthorizedScrollContainers
			};
			const screenshotPath = path.join(viewportDir, `${view.id}.png`);
			await page.screenshot({ path: screenshotPath, fullPage: false });
			results.push({ type: 'view', viewport, id: view.id, label: view.label, screenshotPath, state });

			if (
				state.bodyScrollWidth > viewport.width + 1 ||
				state.documentScrollWidth > viewport.width + 1 ||
				state.outliers.length > 0
			) {
				failures.push({
					type: 'viewport-outlier',
					viewport: viewport.id,
					view: view.id,
					bodyScrollWidth: state.bodyScrollWidth,
					documentScrollWidth: state.documentScrollWidth,
					outliers: state.outliers
				});
			}
			if (
				state.bodyScrollHeight > viewport.height + 1 ||
				state.documentScrollHeight > viewport.height + 1
			) {
				failures.push({
					type: 'vertical-document-overflow',
					viewport: viewport.id,
					view: view.id,
					bodyScrollHeight: state.bodyScrollHeight,
					documentScrollHeight: state.documentScrollHeight
				});
			}
			if (state.scrollX !== 0 || state.scrollY !== 0) {
				failures.push({
					type: 'document-scroll-offset',
					viewport: viewport.id,
					view: view.id,
					scrollX: state.scrollX,
					scrollY: state.scrollY
				});
			}
			if (
				state.shellMetrics.sidebarBottomGap === null ||
				Math.abs(state.shellMetrics.sidebarBottomGap) > 1
			) {
				failures.push({
					type: 'sidebar-viewport-height',
					viewport: viewport.id,
					view: view.id,
					shellMetrics: state.shellMetrics
				});
			}
			if (state.unauthorizedScrollContainers.length > 0) {
				failures.push({
					type: 'unauthorized-scroll-container',
					viewport: viewport.id,
					view: view.id,
					containers: state.unauthorizedScrollContainers
				});
			}
			if (state.widgetFrameMetrics.nonModularHeights.length > 0) {
				failures.push({
					type: 'non-modular-widget-heights',
					viewport: viewport.id,
					view: view.id,
					widgets: state.widgetFrameMetrics.nonModularHeights
				});
			}
		}

		await openPrimaryView(page, 'Home');
		await page.getByRole('button', { name: 'Collapse sidebar' }).click();
		await page.waitForTimeout(100);
		const railState = await page.evaluate(() => {
			const sidebar = document.querySelector('.sidebar');
			const subnav = document.querySelector('#communications-sidebar-sections');
			const sidebarRect = sidebar?.getBoundingClientRect() ?? null;
			return {
				isRail: sidebar?.classList.contains('rail') ?? false,
				sidebarWidth: sidebar ? Math.round(sidebar.getBoundingClientRect().width) : null,
				sidebarBottomGap:
					sidebarRect === null ? null : Math.round((window.innerHeight - sidebarRect.bottom) * 100) / 100,
				hasVisibleSubnav:
					subnav !== null &&
					getComputedStyle(subnav).display !== 'none' &&
					subnav.getBoundingClientRect().height > 0
			};
		});
		const railScreenshotPath = path.join(viewportDir, 'rail-home.png');
		await page.screenshot({ path: railScreenshotPath, fullPage: false });
		results.push({ type: 'rail', viewport, screenshotPath: railScreenshotPath, state: railState });
		if (
			!railState.isRail ||
			railState.sidebarWidth !== 64 ||
			railState.hasVisibleSubnav ||
			railState.sidebarBottomGap === null ||
			Math.abs(railState.sidebarBottomGap) > 1
		) {
			failures.push({ type: 'rail-mode', viewport: viewport.id, state: railState });
		}

		results.push({ type: 'console', viewport, issues: consoleIssues });
	} finally {
		await page.close();
	}
}

async function captureViewportGuard() {
	const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
	await installLayoutCaptureRoutes(page);
	try {
		await page.goto(url, { waitUntil: 'domcontentloaded' });
		await page.locator('nav[aria-label="Primary workspaces"]').waitFor({ state: 'visible' });

		await page.setViewportSize({ width: 799, height: 600 });
		await page.waitForTimeout(50);
		const widthGuard = await getViewportGuardDisplay(page, 'checking the 799px width guard');

		await page.setViewportSize({ width: 800, height: 599 });
		await page.waitForTimeout(50);
		const heightGuard = await getViewportGuardDisplay(page, 'checking the 599px height guard');

		results.push({ type: 'guard', widthGuard, heightGuard });
		if (widthGuard !== 'grid' || heightGuard !== 'grid') {
			failures.push({ type: 'viewport-guard', widthGuard, heightGuard });
		}
	} finally {
		await page.close();
	}
}

try {
	for (const viewport of viewports) {
		await captureViewport(viewport);
	}
	await captureViewportGuard();

	await writeFile(path.join(outputDir, 'summary.json'), JSON.stringify(results, null, 2));

	const widgetSummary = results
		.filter((result) => result.type === 'view')
		.map((result) => ({
			viewport: result.viewport.id,
			id: result.id,
			visibleWidgets: result.state.widgetFrameMetrics.visibleCount,
			widgetRow: result.state.widgetFrameMetrics.widgetRow,
			nonModularWidgets: result.state.widgetFrameMetrics.nonModularHeights.length,
			horizontalOutliers: result.state.outliers.length,
			bodyScrollHeight: result.state.bodyScrollHeight,
			documentScrollHeight: result.state.documentScrollHeight,
			sidebarBottomGap: result.state.shellMetrics.sidebarBottomGap,
			unauthorizedScrollContainers: result.state.unauthorizedScrollContainers.length,
			columnSpreads: result.state.layoutColumnMetrics.map((metric) => ({
				selector: metric.selector,
				spread: metric.columnHeightSpread
			}))
		}));
	console.log(`Widget frame summary: ${JSON.stringify(widgetSummary)}`);
	if (failures.length > 0) {
		await writeFile(path.join(outputDir, 'failures.json'), JSON.stringify(failures, null, 2));
		console.error(`Layout capture failed: ${JSON.stringify(failures.slice(0, 5))}`);
		console.log(outputDir);
		process.exit(1);
	}
} finally {
	await browser.close();
}

console.log(outputDir);
