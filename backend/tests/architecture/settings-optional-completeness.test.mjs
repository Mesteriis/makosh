import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0386-owner-declared-optional-settings-and-schema-successor-completeness.md',
    PROJECT_ROOT,
  ),
  protocol: new URL(
    'src/platform/runtime_protocol/proto/makosh/runtime/v1/recovery.proto',
    BACKEND_ROOT,
  ),
  validation: new URL(
    'src/platform/runtime_protocol/src/validation/descriptor.rs',
    BACKEND_ROOT,
  ),
  kernel: new URL('src/kernel/src/modules/settings/schema.rs', BACKEND_ROOT),
  mail: new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT),
};

test('Settings completeness is declarative and provider-neutral', async () => {
  const [adr, protocol, validation, kernel, mail] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );

  assert.match(protocol, /SettingValueV1 default_value = 11; bool optional = 12;/);
  assert.match(validation, /pub fn settings_snapshot_is_complete_v1/);
  assert.match(validation, /filter\(\|definition\| !definition\.optional\)/);
  assert.match(kernel, /settings_snapshot_is_complete_v1/);
  const kernelProduction = kernel.split('#[cfg(test)]')[0];
  assert.doesNotMatch(
    kernelProduction,
    /mail\.|gmail|imap|smtp|carddav|telegram|whatsapp|zulip/i,
  );
  assert.match(mail, /fn required_definition/);
  assert.match(mail, /definition\.optional = false/);
  assert.match(mail, /optional: true/);
  assert.match(adr, /Proto default `false`/);
  assert.match(adr, /Kernel не интерпретирует/);
});
