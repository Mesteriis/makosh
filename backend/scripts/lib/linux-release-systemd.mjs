import { dirname, isAbsolute } from 'node:path';

const safeAbsolutePath = /^\/[A-Za-z0-9_./-]+$/;

/**
 * Compose and systemd, rather than Kernel, own container start and stop.
 * The path is deliberately restricted instead of attempting shell escaping in
 * a unit file because systemd's command parsing is not a shell contract.
 */
export function renderSystemdUnit(composePath) {
  if (!isAbsolute(composePath)) {
    throw new TypeError('Compose path must be absolute');
  }
  if (!safeAbsolutePath.test(composePath)) {
    throw new TypeError('Compose path must contain only safe characters');
  }
  const workingDirectory = dirname(composePath);
  const compose = `/usr/bin/docker compose --project-name makosh-platform --file ${composePath}`;
  return [
    '[Unit]',
    'Description=Макошь external Compose platform',
    'Wants=docker.service',
    'After=docker.service network-online.target',
    '',
    '[Service]',
    'Type=oneshot',
    'RemainAfterExit=yes',
    `WorkingDirectory=${workingDirectory}`,
    `ExecStartPre=/usr/bin/test -r ${composePath}`,
    `ExecStart=${compose} up --detach --remove-orphans`,
    `ExecStop=${compose} down --timeout 30`,
    '',
    '[Install]',
    'WantedBy=multi-user.target',
    '',
  ].join('\n');
}
