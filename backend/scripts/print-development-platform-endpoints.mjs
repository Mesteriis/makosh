// The Compose-only development contour publishes these fixed loopback ports.
// Container-network addresses are intentionally not used: they are unreachable
// from the macOS host running Cargo tests.
process.stdout.write('MAKOSH_DEVELOPMENT_POSTGRES_URL=postgres://makosh_development@127.0.0.1:35432/makosh_development\n');
process.stdout.write('MAKOSH_DEVELOPMENT_PGBOUNCER_URL=postgres://makosh_development@127.0.0.1:36432/makosh_development\n');
process.stdout.write('MAKOSH_DEVELOPMENT_POSTGRES_HOST=127.0.0.1\n');
process.stdout.write('MAKOSH_DEVELOPMENT_POSTGRES_PORT=35432\n');
process.stdout.write('MAKOSH_DEVELOPMENT_PGBOUNCER_HOST=127.0.0.1\n');
process.stdout.write('MAKOSH_DEVELOPMENT_PGBOUNCER_PORT=36432\n');
process.stdout.write('MAKOSH_DEVELOPMENT_NATS_URL=nats://127.0.0.1:34222\n');
