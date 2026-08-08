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
	'frontend/storybook-static',
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
	'frontend/storybook-static/',
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

function isStorybookFile(relativePath) {
	return relativePath.startsWith('frontend/stories/') || relativePath.startsWith('frontend/.storybook/');
}

function isStorybookGeneratedRuntimeFile(relativePath) {
	return relativePath === 'frontend/public/mockServiceWorker.js';
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

				if (!isDocFile(file) && !isTestFile(file) && !isStorybookFile(file) && secretPattern.test(line)) {
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
					if (isStorybookGeneratedRuntimeFile(file)) continue;
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
