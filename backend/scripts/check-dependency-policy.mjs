#!/usr/bin/env node

import { readdir, readFile } from "node:fs/promises";
import { join, relative } from "node:path";

const root = process.cwd();
const manifests = [];

async function collect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (["target", ".git"].includes(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await collect(path);
    if (entry.isFile() && entry.name === "Cargo.toml") manifests.push(path);
  }
}

function dependencyEntries(text) {
  return [...text.matchAll(/^(?<name>[A-Za-z0-9_-]+)\s*=\s*\{(?<spec>[^\n]+)\}$/gm)];
}

await collect(root);
const violations = [];
for (const manifest of manifests) {
  const text = await readFile(manifest, "utf8");
  for (const entry of dependencyEntries(text)) {
    const { name, spec } = entry.groups;
    const location = `${relative(root, manifest)}: ${name}`;
    const path = /\bpath\s*=\s*"[^"]+"/.test(spec);
    const workspace = /\bworkspace\s*=\s*true/.test(spec);
    const version = spec.match(/\bversion\s*=\s*"([^"]+)"/);
    if (path && !name.startsWith("makosh-")) {
      violations.push(`${location} path dependency must be a Макошь package`);
    }
    if (!path && !workspace && (!version || !version[1].startsWith("="))) {
      violations.push(`${location} registry dependency must use an exact version`);
    }
    if (version?.[1] === "*") {
      violations.push(`${location} must not use a wildcard version`);
    }
  }
}

if (violations.length > 0) {
  console.error("dependency-policy-check: failed");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}
console.log(`dependency-policy-check: ok (${manifests.length} manifests checked)`);
