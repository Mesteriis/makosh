const serviceOrder = ['postgres', 'pgbouncer', 'vault', 'telemetry', 'storage-control', 'nats', 'kernel'];

const serviceVolumes = {
  kernel: 'kernel-control',
  vault: 'vault-data',
  telemetry: 'telemetry-data',
  'storage-control': 'storage-control-data',
  postgres: 'postgres-data',
  pgbouncer: 'pgbouncer-data',
  nats: 'nats-data',
};

const dependencies = {
  pgbouncer: ['postgres'],
  'storage-control': ['postgres', 'pgbouncer', 'vault'],
  nats: ['vault'],
  kernel: ['vault', 'telemetry', 'storage-control', 'nats'],
};

function quote(value) {
  return JSON.stringify(value);
}

function renderDependencies(names) {
  if (!names) return [];
  return [
    '    depends_on:',
    ...names.flatMap((name) => [
      `      ${name}:`,
      '        condition: service_healthy',
    ]),
  ];
}

function renderService(name, image) {
  const lines = [
    `  ${name}:`,
    `    image: ${image}`,
    '    init: true',
    '    read_only: true',
    '    security_opt:',
    '      - no-new-privileges:true',
    '    cap_drop:',
    '      - ALL',
    '    pids_limit: 256',
    '    mem_limit: 512m',
    '    networks:',
    '      - makosh-private',
    '    volumes:',
    `      - ${serviceVolumes[name]}:/var/lib/makosh`,
    '    healthcheck:',
    '      test: ["CMD", "/usr/local/bin/makosh-platform-healthcheck"]',
    '      interval: 10s',
    '      timeout: 3s',
    '      retries: 6',
    '      start_period: 15s',
  ];

  if (name === 'postgres') {
    lines.push(
      '    environment:',
      '      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_bootstrap_password',
      '    secrets:',
      '      - postgres_bootstrap_password',
    );
  }
  if (name === 'storage-control') {
    lines.push(
      '    secrets:',
      '      - postgres_bootstrap_password',
    );
  }
  lines.push(...renderDependencies(dependencies[name]));
  return lines;
}

/**
 * Render a Compose descriptor only from a manifest that has already passed
 * validateManifest(). The caller owns signature verification before writing it.
 */
export function renderCompose(manifest, secretsDirectory) {
  const lines = [
    '# Generated from a Cosign-verified Макошь release manifest. Do not edit.',
    'name: makosh-platform',
    'services:',
    ...serviceOrder.flatMap((name) => renderService(name, manifest.services[name].image)),
    'networks:',
    '  makosh-private:',
    '    internal: true',
    'volumes:',
    ...serviceOrder.map((name) => `  ${serviceVolumes[name]}:`),
    'secrets:',
    '  postgres_bootstrap_password:',
    `    file: ${quote(`${secretsDirectory}/postgres_bootstrap_password`)}`,
    '',
  ];
  return lines.join('\n');
}
