import { spawnSync } from 'node:child_process';

export function waitForChildExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return Promise.resolve();
  }
  return new Promise((resolve) => child.once('exit', resolve));
}

export function runWithFileSizeLimit(binary, dataDir, maxBytes, arguments_, input) {
  const blockLimit = Math.max(1, Math.floor(maxBytes / 512));
  const script = `
    trap '' XFSZ
    limit="$1"; binary="$2"; data_dir="$3"; shift 3
    ulimit -f "$limit"
    exec "$binary" --data-dir "$data_dir" "$@"
  `;
  return spawnSync(
    '/bin/zsh',
    ['-c', script, 'makosh-file-limit', blockLimit, binary, dataDir, ...arguments_],
    { encoding: 'utf8', input },
  );
}
