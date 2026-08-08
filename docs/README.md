# Документация Макошь clean-room

Статус: clean-room foundation реализован; business/data plane ещё закрыты
Дата: 2026-07-16

В этой директории находится только документация новой clean-room системы.
Документы предыдущего backend перенесены в
[`references/backend-legacy/docs`](../references/backend-legacy/docs/) и не
являются действующей policy.

## Активные решения

- [ADR index](adr/README.md)
- [ADR-0200: Модульная модель и изоляция runtime](adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md)
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](adr/ADR-0201-core-module-communication-and-nats.md)
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md)
- [ADR-0203: Управление локальной инфраструктурой и восстановление](adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md)
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md)
- [ADR-0205: Core Gateway и транспорт клиентских приложений](adr/ADR-0205-core-gateway-and-client-transport.md)
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md)
- [ADR-0207: Канонический реестр бизнес-доменов Макошь](adr/ADR-0207-canonical-business-domain-registry.md)
- [ADR-0208: Allowlist разработки доменов и запрет проекций](adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md)
- [ADR-0209: Kernel Event Hub и контроль подписок](adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md)
- [ADR-0210: Telemetry Hub и локальная диагностика](adr/ADR-0210-telemetry-hub-and-local-diagnostics.md)
- [ADR-0211: Backend workspace и физическая структура исходного кода](adr/ADR-0211-backend-workspace-and-source-layout.md)
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](adr/ADR-0212-crate-topology-and-compile-isolation.md)
- [ADR-0213: Конституция кода, ownership и автономность модулей](adr/ADR-0213-code-ownership-and-module-autonomy.md)
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md)
- [ADR-0215: Открытая регистрация модулей и capability grants](adr/ADR-0215-open-module-registration-and-capability-grants.md)
- [ADR-0216: Private Kernel Control Store на SQLite](adr/ADR-0216-private-kernel-control-store-with-sqlite.md)
- [ADR-0217: Нулевой внешний bootstrap Kernel](adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md)
- [ADR-0218: Owner/device identity, enrollment и offline recovery](adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md)
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md)
- [ADR-0220: Канонический durable envelope и эволюция контрактов](adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md)
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md)
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md)
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md)
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md)
- [ADR-0225: Первый production slice — recovery-only Kernel и фазовые ворота](adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md)
- [ADR-0226: Контекст для AI только через use-case workflows](adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md)
- [ADR-0227: Deployment profiles и server bootstrap pairing](adr/ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md)
- [ADR-0228: Development simulation profile](adr/ADR-0228-development-simulation-profile.md)
- [ADR-0229: Platform Clock contract и deterministic conformance](adr/ADR-0229-platform-clock-contract-and-deterministic-conformance.md)
- [ADR-0230: Blob Platform — opaque references and owner-local metadata](adr/ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md)
- [ADR-0231: Private Blob data session and Vault route](adr/ADR-0231-private-blob-data-session-and-vault-route.md)
- [ADR-0232: Browser client identity and same-origin Gateway session](adr/ADR-0232-browser-client-device-identity-and-same-origin-session.md)
- [ADR-0233: Scoped local recovery export и PostgreSQL dump](adr/ADR-0233-whole-instance-backup-and-fenced-restore.md)
- [ADR-0234: Browser-local key binding for synchronised passkeys](adr/ADR-0234-browser-local-key-binding-for-synchronised-passkeys.md)
- [ADR-0235: Private-LAN developer mode](adr/ADR-0235-private-lan-developer-mode.md)
- [ADR-0236: Integration owners, protocol adapters и configuration instances](adr/ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md)
- [ADR-0237: Временный private-LAN development без owner authority](adr/ADR-0237-temporary-private-lan-development-without-owner-authority.md)
- [ADR-0238: Secure-file FD boundary](adr/ADR-0238-secure-file-fd-boundary.md)

## Канонические summaries

- [Architecture overview](architecture/architecture-overview.md)
- [Component communication contract](architecture/component-communication.md)
- [Storage Control Plane](architecture/storage-control-plane.md)
- [Vault and credential leases](architecture/vault-and-credential-leases.md)
- [Process/container topology](architecture/container-diagram.md)
- [Executable architecture policy](../backend/architecture/README.md)

## Что ещё не определено

Активных подробных product, provider, UI, testing, deployment и operations
specifications пока нет. Entity model integrations предложена ADR-0236; exact
owner contracts и implementations ещё не выбраны. Общая module
settings model зафиксирована ADR-0222,
Vault boundary — ADR-0223, Storage Control и migration lifecycle — ADR-0224,
первый recovery-only production slice — ADR-0225, Clock boundary — ADR-0229,
а cross-owner AI context
boundary — ADR-0226. Конкретные owner contracts, schemas и migration bundles
ещё не определены. Остальные specifications должны
появляться заново после обсуждения соответствующей границы и не могут
восстанавливаться из legacy documentation как действующий contract.

Принятый ADR сам по себе не означает, что решение реализовано. Фактическое
состояние указывается внутри каждого ADR и меняется только после executable
evidence и targeted validation новой системы.
