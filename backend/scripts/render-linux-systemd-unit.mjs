#!/usr/bin/env node

import { dirname } from 'node:path';
import { mkdirSync, renameSync, writeFileSync } from 'node:fs';

import { renderSystemdUnit } from './lib/linux-release-systemd.mjs';

function fail(message) {
  process.stderr.write(`linux-systemd-unit: ${message}\n`);
  process.exitCode = 1;
}

export function main(argv = process.argv.slice(2)) {
  if (argv.length !== 2) {
    fail('usage: render-linux-systemd-unit.mjs <compose.yaml> <makosh-platform.service>');
    return;
  }
  const [composePath, unitPath] = argv;
  try {
    const unit = renderSystemdUnit(composePath);
    const outputDirectory = dirname(unitPath);
    const temporaryPath = `${unitPath}.tmp-${process.pid}`;
    mkdirSync(outputDirectory, { recursive: true, mode: 0o700 });
    writeFileSync(temporaryPath, unit, { mode: 0o644 });
    renameSync(temporaryPath, unitPath);
  } catch (error) {
    fail(error.message);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
