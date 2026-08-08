#!/usr/bin/env node

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { dirname, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const backendRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const repositoryRoot = dirname(backendRoot);
const frontendRoot = resolve(repositoryRoot, 'frontend');
const base = process.env.MAKOSH_TEST_BASE || 'HEAD';
const dryRun = process.argv.includes('--dry-run');

const changed = changedPaths();
if (changed.length === 0) {
  console.log('impacted-test: no changed files');
  process.exit(0);
}

const plan = buildPlan(changed);
console.log(`impacted-test: ${changed.length} changed path(s), ${plan.length} command(s)`);
for (const command of plan) {
  console.log(`impacted-test: ${display(command)}`);
  if (dryRun) continue;
  const result = spawnSync(command.command, command.arguments, {
    cwd: command.cwd,
    stdio: 'inherit',
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function changedPaths() {
  const diff = (arguments_) => command('git', arguments_)
    .split('\n')
    .filter(Boolean);
  return [...new Set([
    ...diff(['diff', '--name-only', '--diff-filter=ACMRD', base, '--']),
    ...diff(['diff', '--cached', '--name-only', '--diff-filter=ACMRD', '--']),
    ...diff(['ls-files', '--others', '--exclude-standard']),
  ])].sort();
}

function buildPlan(paths) {
  const plan = [];
  const backendPaths = paths.filter((path) => path.startsWith('backend/'));
  const frontendPaths = paths.filter((path) => path.startsWith('frontend/'));
  const lifecycleTests = backendPaths
    .filter((path) => path.startsWith('backend/tests/lifecycle/') && path.endsWith('.test.mjs') && existsInRepository(path))
    .map((path) => relative(backendRoot, resolve(repositoryRoot, path)));
  const architectureTests = backendPaths
    .filter((path) => path.startsWith('backend/tests/architecture/') && path.endsWith('.test.mjs') && existsInRepository(path))
    .map((path) => relative(backendRoot, resolve(repositoryRoot, path)));

  const backendGlobal = backendPaths.some((path) => isBackendGlobal(path));
  if (backendGlobal) {
    plan.push(nodeCommand(['scripts/check-architecture-policy.mjs']));
    plan.push(nodeCommand(['scripts/check-cargo-boundaries.mjs']));
    plan.push(nodeCommand(['scripts/check-srp-policy.mjs']));
    plan.push(nodeCommand(['--test', ...allArchitectureTests()]));
  }
  if (architectureTests.length > 0 && !backendGlobal) {
    plan.push(nodeCommand(['--test', ...architectureTests]));
  }
  if (lifecycleTests.length > 0) {
    plan.push(nodeCommand(['--test', ...lifecycleTests]));
  }

  const packages = backendPaths.length > 0
    ? impactedCargoPackages(backendPaths)
    : { workspace: false, names: [] };
  if (packages.workspace) {
    plan.push(cargoCommand(['test', '--locked', '--workspace']));
  } else if (packages.names.length > 0) {
    plan.push(cargoCommand(['test', '--locked', ...packages.names.flatMap((name) => ['-p', name])]));
  }

  if (frontendPaths.length > 0) {
    const frontendGlobal = frontendPaths.some((path) => isFrontendGlobal(path));
    const frontendSource = frontendPaths
      .filter((path) => (path.startsWith('frontend/src/') || path.startsWith('frontend/stories/')) && existsInRepository(path))
      .map((path) => relative(frontendRoot, resolve(repositoryRoot, path)));
    if (frontendGlobal) {
      plan.push({ command: 'pnpm', arguments: ['test'], cwd: frontendRoot });
    } else if (frontendSource.length > 0) {
      plan.push({
        command: 'pnpm',
        arguments: ['exec', 'vitest', 'related', '--run', '--passWithNoTests', ...frontendSource],
        cwd: frontendRoot,
      });
    }
    if (frontendPaths.some((path) => path.startsWith('frontend/tests/visual/') || path.includes('.storybook/'))) {
      plan.push({ command: 'pnpm', arguments: ['test:visual'], cwd: frontendRoot });
    }
    if (frontendPaths.some((path) => path.startsWith('frontend/src-tauri/'))) {
      plan.push({ command: 'cargo', arguments: ['test', '--manifest-path', 'src-tauri/Cargo.toml'], cwd: frontendRoot });
    }
  }
  return deduplicate(plan);
}

function impactedCargoPackages(paths) {
  if (paths.some((path) => path === 'backend/Cargo.lock' || path === 'backend/Cargo.toml' || !existsInRepository(path))) {
    return { workspace: true, names: [] };
  }
  const metadata = JSON.parse(command('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], backendRoot));
  const packages = metadata.packages.map((pkg) => ({
    name: pkg.name,
    directory: dirname(pkg.manifest_path),
  }));
  const names = new Set();
  for (const path of paths) {
    const absolute = resolve(repositoryRoot, path);
    const owner = packages
      .filter((pkg) => absolute === pkg.directory || absolute.startsWith(`${pkg.directory}${sep}`))
      .sort((left, right) => right.directory.length - left.directory.length)[0];
    if (owner) names.add(owner.name);
  }
  return { workspace: false, names: [...names].sort() };
}

function isBackendGlobal(path) {
  return path === 'backend/Makefile'
    || path === 'backend/architecture/policy.json'
    || path.startsWith('backend/scripts/')
    || path.startsWith('backend/tests/architecture/')
    || path.startsWith('backend/.cargo/')
    || !existsInRepository(path);
}

function allArchitectureTests() {
  const directory = resolve(backendRoot, 'tests/architecture');
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.test.mjs'))
    .map((entry) => `tests/architecture/${entry.name}`)
    .sort();
}

function isFrontendGlobal(path) {
  return path === 'frontend/package.json'
    || path === 'frontend/pnpm-lock.yaml'
    || path.startsWith('frontend/.storybook/')
    || path.startsWith('frontend/vite.config')
    || !existsInRepository(path);
}

function existsInRepository(path) {
  return existsSync(resolve(repositoryRoot, path));
}

function nodeCommand(arguments_) {
  return { command: 'node', arguments: arguments_, cwd: backendRoot };
}

function cargoCommand(arguments_) {
  return { command: 'cargo', arguments: ['+1.97.0', ...arguments_], cwd: backendRoot };
}

function command(command_, arguments_, cwd = repositoryRoot) {
  return execFileSync(command_, arguments_, { cwd, encoding: 'utf8' }).trim();
}

function deduplicate(commands) {
  const seen = new Set();
  return commands.filter((command_) => {
    const key = `${command_.cwd}\0${command_.command}\0${command_.arguments.join('\0')}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function display(command_) {
  const cwd = relative(repositoryRoot, command_.cwd) || '.';
  return `(cd ${cwd} && ${command_.command} ${command_.arguments.join(' ')})`;
}
