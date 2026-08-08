#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontendRoot = resolve(fileURLToPath(new URL('..', import.meta.url)));
const configPath = resolve(frontendRoot, 'src-tauri/tauri.conf.json');
const sourcePath = resolve(frontendRoot, 'src-tauri/src/lib.rs');
const recordingHostPath = resolve(frontendRoot, 'src-tauri/src/desktop_call_recording_host/mod.rs');
const mainCapabilityPath = resolve(frontendRoot, 'src-tauri/capabilities/default.json');
const appRootPath = resolve(frontendRoot, 'src/app/layout/AppLayoutRoot.vue');
const config = JSON.parse(readFileSync(configPath, 'utf8'));
const source = readFileSync(sourcePath, 'utf8');
const recordingHost = readFileSync(recordingHostPath, 'utf8');
const mainCapability = JSON.parse(readFileSync(mainCapabilityPath, 'utf8'));
const appRoot = readFileSync(appRootPath, 'utf8');
const failures = [];

const resources = config.bundle?.resources ?? {};
if (Object.keys(resources).some((resource) => resource.includes('google-oauth'))) {
  failures.push('Tauri bundle must not package legacy Google OAuth resources');
}
if (!Array.isArray(config.bundle?.externalBin) || !config.bundle.externalBin.includes('binaries/makosh-kernel')) {
  failures.push('Tauri bundle must declare the makosh-kernel sidecar');
}
if (/MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG|MAKOSH_LOCAL_API_SECRET/.test(source)) {
  failures.push('Tauri sidecar source must not forward legacy OAuth or local API secrets');
}
const ownerVaultPermissions = new Set([
  'allow-owner-vault-provisioning-host-start',
  'allow-owner-vault-provisioning-host-seal',
  'allow-owner-vault-provisioning-host-open-receipt',
  'allow-owner-vault-provisioning-host-cancel',
]);
if (mainCapability.permissions.some(
  (permission) => permission.startsWith('allow-') && !ownerVaultPermissions.has(permission),
)) {
  failures.push('Tauri main window must not receive provider host-bridge permissions before route admission');
}
if ([...ownerVaultPermissions].some((permission) => !mainCapability.permissions.includes(permission))) {
  failures.push('Tauri main window must receive the exact owner Vault host-adapter permissions');
}
if (!/admitted_route_exists[\s\S]*add_capability/.test(recordingHost)) {
  failures.push('Tauri recording host permissions must be added only after exact route admission');
}
if (/CommunicationsWorkspaceView|PersonasWorkspaceView|@\/integrations\//.test(appRoot)) {
  failures.push('Tauri recovery shell must not mount disabled product routes or provider host bridges');
}
if (!source.includes('#[cfg(feature = "whatsapp-host-webview")]\nmod whatsapp_companion;')) {
  failures.push('Tauri provider companion module must be excluded from the default recovery build');
}
if (!/#\[cfg\(all\(\s*feature = "whatsapp-host-webview",[\s\S]*?let builder = builder\.invoke_handler[\s\S]*?whatsapp_companion::start_hidden_whatsapp_webview/.test(source)) {
  failures.push('Tauri provider host commands must be excluded from the default recovery invoke handler');
}

if (failures.length > 0) {
  for (const failure of failures) process.stderr.write(`cleanroom-tauri-bundle: ${failure}\n`);
  process.exitCode = 1;
}
