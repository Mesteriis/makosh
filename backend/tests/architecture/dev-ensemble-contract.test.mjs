import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const sources = {
  adr: new URL(
    'docs/adr/ADR-0300-loopback-full-stack-development-assembly.md',
    PROJECT_ROOT,
  ),
  refreshAdr: new URL(
    'docs/adr/ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md',
    PROJECT_ROOT,
  ),
  rootMakefile: new URL('Makefile', PROJECT_ROOT),
  backendMakefile: new URL('Makefile', BACKEND_ROOT),
  assembly: new URL('scripts/dev-ensemble.sh', BACKEND_ROOT),
  authenticatedCompose: new URL(
    'development/authenticated/compose.yaml',
    BACKEND_ROOT,
  ),
  authenticatedNats: new URL(
    'development/authenticated/nats-server.conf',
    BACKEND_ROOT,
  ),
  authenticatedPostgresReconciliation: new URL(
    'development/authenticated/reconcile-postgres-admin.sql',
    BACKEND_ROOT,
  ),
  authenticatedProviderOwnerScopeReconciliation: new URL(
    'development/authenticated/reconcile-provider-owner-scopes.sql',
    BACKEND_ROOT,
  ),
  release: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  developmentAssembly: new URL('development/assembly/src/main.rs', BACKEND_ROOT),
  probe: new URL('scripts/probe-dev-gateway.mjs', BACKEND_ROOT),
  vite: new URL('frontend/vite.config.ts', PROJECT_ROOT),
  gateway: new URL('src/kernel/src/platform/gateway.rs', BACKEND_ROOT),
  cli: new URL('src/kernel/src/cli/mod.rs', BACKEND_ROOT),
  protocol: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/browser_session.proto',
    BACKEND_ROOT,
  ),
  ownerControl: new URL(
    'src/api/gateway/contracts/proto/makosh/gateway/v1/owner_control.proto',
    BACKEND_ROOT,
  ),
  ownerControlPlatformDispatch: new URL(
    'src/kernel/src/identity/owner_control/dispatch/platform.rs',
    BACKEND_ROOT,
  ),
};

test('root make dev owns one loopback full-stack browser assembly', async () => {
  const {
    adr,
    refreshAdr,
    rootMakefile,
    backendMakefile,
    assembly,
    authenticatedCompose,
    authenticatedNats,
    authenticatedPostgresReconciliation,
    authenticatedProviderOwnerScopeReconciliation,
    release,
    developmentAssembly,
    probe,
    vite,
    gateway,
    cli,
    protocol,
    ownerControl,
    ownerControlPlatformDispatch,
  } = Object.fromEntries(
    await Promise.all(
      Object.entries(sources).map(async ([name, path]) => [
        name,
        await readFile(path, 'utf8'),
      ]),
    ),
  );

  assert.match(adr, /loopback_full_stack_dev_assembly_v1/);
  assert.match(refreshAdr, /repeatable_development_release_refresh_v1/);
  assert.match(rootMakefile, /build test dev docker tauri clean:[\s\S]*\$\(MAKE\) -C backend \$@/);
  assert.match(backendMakefile, /^dev:\n\t@\.\/scripts\/dev-ensemble\.sh$/m);
  assert.doesNotMatch(backendMakefile, /wait -n/);

  assert.match(assembly, /materialize-dev-release\.sh/);
  assert.match(assembly, /development\/authenticated\/compose\.yaml/);
  assert.match(
    assembly,
    /MAKOSH_DEV_AUTHENTICATED_COMPOSE_PROJECT_NAME:-makosh-storage-authenticated-development/,
  );
  assert.match(
    assembly,
    /docker compose --project-name "\$authenticated_compose_project_name" -f "\$compose_file"/,
  );
  assert.match(
    assembly,
    /docker compose --project-name "\$legacy_compose_project_name" -f "\$legacy_compose_file"/,
  );
  assert.match(assembly, /run_compose up --detach --wait/);
  assert.match(
    assembly,
    /run_compose exec --no-TTY --user postgres postgres[\s\S]*--file -[\s\S]*reconcile-postgres-admin\.sql/,
  );
  assert.match(
    assembly,
    /run_compose exec --no-TTY --user postgres postgres[\s\S]*--file -[\s\S]*reconcile-provider-owner-scopes\.sql/,
  );
  assert.match(
    assembly,
    /run_compose exec --no-TTY pgbouncer[\s\\]+test -r \/etc\/pgbouncer\/pgbouncer\.ini[\s\\]+-a -r \/etc\/makosh\/runtime\/databases\.ini[\s\\]+-a -r \/etc\/makosh\/auth\/users\.txt/,
  );
  assert.match(
    assembly,
    /if ! run_compose exec[\s\S]*run_compose up --detach --no-deps --force-recreate --wait pgbouncer[\s\S]*fi/,
  );
  assert.match(
    assembly,
    /if ! test "\$expected_postgres_hash" = "\$current_postgres_hash"; then[\s\S]*--force-recreate --wait postgres[\s\S]*fi/,
  );
  assert.doesNotMatch(assembly, /--force-recreate[^\n]*(?:nats|clamav)/);
  assert.match(assembly, /PostgreSQL, PgBouncer, NATS and ClamAV infrastructure/);
  assert.match(authenticatedCompose, /image: clamav\/clamav:1\.5\.3-debian13-slim/);
  assert.match(
    authenticatedCompose,
    /- makosh-authenticated-development-nats:\/var\/lib\/makosh/,
    'JetStream state must survive a normal development ensemble restart',
  );
  assert.match(
    authenticatedCompose,
    /- makosh-authenticated-development-postgres:\/var\/lib\/postgresql\/data/,
    'authenticated PostgreSQL must not rely on an anonymous Docker volume',
  );
  assert.match(
    authenticatedCompose,
    /^volumes:\n  makosh-authenticated-development-nats:\n  makosh-authenticated-development-postgres:/m,
  );
  assert.match(
    authenticatedCompose,
    /\.\/nats-server\.conf:\/etc\/nats\/makosh-development\.conf:ro/,
  );
  assert.match(
    authenticatedPostgresReconciliation,
    /pg_read_file\('\/run\/secrets\/storage_postgres_admin_password'\)/,
  );
  assert.match(
    authenticatedPostgresReconciliation,
    /ALTER ROLE %I PASSWORD %L/,
  );
  assert.doesNotMatch(
    authenticatedPostgresReconciliation,
    /(?:password|credential)\s*[:=]\s*['"][^'"]+['"]/i,
  );
  assert.match(
    authenticatedProviderOwnerScopeReconciliation,
    /\('telegram', 'telegram_owner_scope'\)[\s\S]*\('whatsapp', 'whatsapp_owner_scope'\)[\s\S]*\('zulip', 'zulip_owner_scope'\)/,
  );
  assert.match(
    authenticatedProviderOwnerScopeReconciliation,
    /logical_owner_id = \$2[\s\S]*USING current_prefix, 'development-owner'/,
  );
  assert.doesNotMatch(
    authenticatedProviderOwnerScopeReconciliation,
    /DELETE|TRUNCATE|DROP|provider.*(?:message|session|payload)/i,
  );
  assert.match(authenticatedNats, /^max_control_line: 16384$/m);
  assert.doesNotMatch(authenticatedNats, /authorization|password|token|users/i);
  assert.match(
    authenticatedCompose,
    /127\.0\.0\.1:\$\{MAKOSH_ATTACHMENT_SECURITY_CLAMAV_PORT:-3310\}:3310/,
  );
  assert.match(authenticatedCompose, /test: \["CMD", "clamdscan", "--ping", "1"\]/);
  assert.doesNotMatch(authenticatedCompose, /(?:^|\s)(?:0\.0\.0\.0|::):.*3310/m);
  assert.match(assembly, /provision-platform/);
  assert.match(assembly, /start-ensemble/);
  assert.match(assembly, /Admitting the exact clean-room development module plan/);
  assert.match(developmentAssembly, /attachment_preview\.runtime\.v1/);
  assert.match(developmentAssembly, /attachment_preview\.storage\.v1/);
  assert.match(assembly, /development_assembly=stale/);
  assert.match(assembly, /--distribution-generation "\$distribution_generation"/);
  assert.match(assembly, /start_kernel_for_admission\(\)/);
  assert.match(
    assembly,
    /assembly_status=.*[\s\S]*if test "\$assembly_status" != "development_assembly=current"; then[\s\S]*start_kernel_for_admission[\s\S]*wait_for_gateway[\s\S]*admit[\s\S]*stop_kernel[\s\S]*fi[\s\S]*start_kernel[\s\S]*wait_for_gateway/,
    'a stale development catalog must be admitted without starting Scheduler before the full Kernel contour',
  );
  assert.match(assembly, /--browser-gateway-development-admission-mode/);
  assert.match(assembly, /--browser-gateway-listen-address "\$gateway_address"/);
  assert.match(assembly, /--browser-gateway-development-proxy-proof-file "\$proof_file"/);
  assert.match(assembly, /MAKOSH_DEV_GATEWAY_PROOF_FILE="\$proof_file"/);
  assert.match(assembly, /probe-dev-gateway\.mjs/);
  assert.match(assembly, /curl .*"\$browser_origin\/readyz"/);
  assert.match(assembly, /no_browser="\$\{MAKOSH_DEV_NO_BROWSER:-1\}"/);
  assert.match(assembly, /if test "\$no_browser" = 0; then[\s\S]*open "\$browser_url"/);
  assert.match(assembly, /Browser opening skipped/);
  assert.match(assembly, /trap cleanup EXIT/);
  assert.doesNotMatch(
    assembly,
    /run_compose down/,
    'the authenticated development storage contour must survive normal make dev shutdown',
  );
  assert.doesNotMatch(assembly, /wait -n|0\.0\.0\.0|--browser-gateway-development-proxy-proof [^"-]/);
  assert.doesNotMatch(assembly, /rm -rf .*kernel-dev|rm -rf .*control|reset/);

  assert.match(release, /makosh-communications-assembly/);
  assert.match(release, /makosh-attachment-security-assembly/);
  assert.match(release, /build-attachment-text-extraction-ocr-macos\.sh/);
  assert.match(release, /makosh-attachment-text-extraction-assembly/);
  assert.match(
    release,
    /attachment_text_extraction\.release-artifacts\.json/,
  );
  assert.match(release, /makosh-mail-assembly/);
  assert.match(release, /makosh-telegram-assembly/);
  assert.match(release, /makosh-whatsapp-assembly/);
  assert.match(release, /makosh-zulip-assembly/);
  assert.match(release, /build-distribution-release\.mjs/);
  assert.match(release, /next_distribution_generation/);
  assert.match(release, /--generation "\$distribution_generation"/);
  assert.match(release, /development-distribution-generation/);

  assert.match(developmentAssembly, /begin_managed_storage_binding_revocation/);
  assert.match(
    developmentAssembly,
    /begin_managed_storage_binding_revocation[\s\S]*previous\.storage_binding_revision/,
  );
  assert.match(developmentAssembly, /upgrade_bundled_managed_registration/);
  assert.match(developmentAssembly, /successor_fences/);
  assert.match(developmentAssembly, /ReservationReleaseV1::Predecessor/);
  assert.match(
    developmentAssembly,
    /ReservationReleaseV1::Predecessor[\s\S]*write_state\(state_path, &state\)[\s\S]*remove_reservation\(reservation_path\)[\s\S]*refresh_plan/,
  );
  assert.match(developmentAssembly, /version=3/);
  assert.match(
    developmentAssembly,
    /ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT:[\s\S]*attachment_text_extraction\.runtime\.v1/,
  );
  assert.match(
    developmentAssembly,
    /runtime_artifact_id: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,[\s\S]*runtime_kind: ModuleRuntimeKindV1::Workflow/,
  );

  assert.match(probe, /host: '127\.0\.0\.1'/);
  assert.match(probe, /origin: 'http:\/\/127\.0\.0\.1:5173'/);
  assert.match(probe, /'x-makosh-development-proxy-proof': proof/);
  assert.doesNotMatch(probe, /console\.(?:log|error)|process\.stdout|process\.stderr/);

  assert.match(vite, /host: '127\.0\.0\.1'/);
  assert.match(vite, /strictPort: true/);
  assert.match(vite, /'\^\/makosh\\\\\.'/);
  assert.match(vite, /'\/api\/realtime\/v1\/events'/);
  assert.match(vite, /'\/healthz'/);
  assert.match(vite, /'\/readyz'/);
  assert.match(vite, /request\.setHeader\(DEVELOPMENT_PROXY_PROOF_HEADER, gateway\.proof\)/);
  assert.doesNotMatch(vite, /define:\s*\{|VITE_.*PROOF/);

  assert.match(gateway, /BrowserGatewayExposureV1::LoopbackDevelopmentProxy/);
  assert.match(gateway, /starts_signed_development_foundation/);
  assert.match(gateway, /GatewayLoopbackListenerV1::bind/);
  assert.match(gateway, /with_loopback_development_proxy_policy/);
  assert.match(cli, /browser_gateway_development_proxy_proof_file: Option<PathBuf>/);
  assert.match(cli, /browser_gateway_development_admission_mode: bool/);
  assert.match(gateway, /development_admission_mode: bool/);
  assert.match(gateway, /starts_development_scheduler/);
  assert.match(gateway, /runs_scheduler_lifecycle/);
  assert.match(cli, /metadata\.permissions\(\)\.mode\(\) & 0o077/);
  assert.match(protocol, /BROWSER_GATEWAY_ACCESS_MODE_V1_LOCAL_DEVELOPMENT = 3/);
  assert.match(ownerControl, /message GetManagedStorageBindingStatusRequestV1/);
  const storageStatusStart = ownerControlPlatformDispatch.indexOf(
    'fn get_managed_storage_binding_status(',
  );
  const storageStatusEnd = ownerControlPlatformDispatch.indexOf(
    '\nfn issue_external_storage_binding(',
    storageStatusStart,
  );
  const storageStatus = ownerControlPlatformDispatch.slice(
    storageStatusStart,
    storageStatusEnd,
  );
  assert.match(storageStatus, /sessions\.authorize/);
  assert.match(storageStatus, /platform_storage_binding/);
  assert.doesNotMatch(storageStatus, /effective_bundled_managed_launch_binding/);
  assert.match(ownerControl, /uint64 credential_lease_revision = 5/);
  assert.match(ownerControl, /message UpgradeBundledManagedRegistrationRequestV1/);
});
