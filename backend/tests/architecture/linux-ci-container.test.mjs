import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import test from 'node:test';

const here = path.dirname(fileURLToPath(import.meta.url));
const makefile = path.resolve(here, '../../Makefile');

test('Linux target compilation uses an isolated native Linux compiler container', async () => {
  const source = await readFile(makefile, 'utf8');

  assert.match(source, /LINUX_CI_IMAGE \?= rust:1\.97\.0-bookworm/);
  assert.match(source, /LINUX_CI_PLATFORM \?= linux\/amd64/);
  assert.match(source, /check-target:[\s\S]*x86_64-unknown-linux-gnu[\s\S]*check-linux-container/);
  assert.match(source, /check-linux-container:[\s\S]*\$\(DOCKER\) run --rm --platform \$\(LINUX_CI_PLATFORM\)/);
  assert.match(source, /src="\$\(CURDIR\)",dst=\/workspace,readonly/);
  assert.match(source, /CARGO_TARGET_DIR=\/tmp\/makosh-target/);
  assert.match(source, /cargo \+\$\(RUST_TOOLCHAIN\) check --locked --workspace --target x86_64-unknown-linux-gnu/);
});
