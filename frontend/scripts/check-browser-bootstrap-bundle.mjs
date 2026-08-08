import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const bundlePath = resolve(import.meta.dirname, '../browser-bootstrap/index.html');
const document = readFileSync(bundlePath, 'utf8');

if (Buffer.byteLength(document, 'utf8') > 512 * 1024) {
  throw new Error('browser bootstrap exceeds the Gateway response limit');
}

const required = [
  '<!doctype html>',
  'navigator.credentials.create',
  'navigator.credentials.get',
  '/browser/v1/pairing/${pairingId}/registration',
  '/browser/v1/authentication/begin',
  '/makosh.gateway.v1.BrowserSessionService/GetStatus',
  "credentials: 'same-origin'",
  'credentialStorageKey',
];
for (const value of required) {
  if (!document.includes(value)) throw new Error(`browser bootstrap is missing ${value}`);
}

if ((document.match(/<script\b/gi) ?? []).length !== 1 || /<script\b[^>]*\bsrc=/i.test(document)) {
  throw new Error('browser bootstrap must contain exactly one inline script');
}
const script = document.match(/<script>([\s\S]*)<\/script>/i)?.[1];
if (!script) throw new Error('browser bootstrap script is unavailable');
try {
  new Function(script);
} catch {
  throw new Error('browser bootstrap inline script is invalid');
}
if (/<(?:img|link|iframe|audio|video|source)\b[^>]*(?:src|href)=/i.test(document)) {
  throw new Error('browser bootstrap must not load an external asset');
}
if (/authorization|x-makosh-secret|bearer\s+/i.test(document)) {
  throw new Error('browser bootstrap must not contain a bearer or legacy credential path');
}
if (!/localStorage\.setItem\(credentialStorageKey/.test(document)
  || /localStorage\.setItem\([^)]*(?:session|token|secret)/i.test(document)) {
  throw new Error('browser bootstrap may persist only the public credential identifier');
}

process.stdout.write('browser bootstrap bundle policy passed\n');
