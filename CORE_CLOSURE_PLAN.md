# Core Closure Plan

Статус: закрыт по текущему closure-gate. Этот файл описывает подтверждённое
состояние clean-room Core для `first_owner_v1`; он не открывает phase gates.
Текущая честная картина:

- Закрыты срезы `whole_instance_backup_v1` и `client_gateway_v1` на уровне evidence/валидации.
- `first_owner_v1` снят: исключение для Mail/Communications больше не используется.
  `mail/communications` остаётся в общем owner stack без отдельного targeted gate-исключения.
- ADR-0239 теперь закрыт на уровне production inventory: exact ten-package срез
  (`makosh-mail-*`, `makosh-communications-*`) включён в `implementation.currentSlice`
  как `gateway_runtime_plus_mail_communications_v1`; добавлены `Cargo.toml`/`src`-скелеты
  для всех 10 крейтов, и все новые пакеты проходят проверку архитектурного policy/evidence.
- Подтверждение: `first_owner_v1` больше не присутствует в `phaseGates`; это
  проверено через `make -C backend architecture-policy-check`,
  `make -C backend architecture-evidence-check` и `node --test tests/architecture/architecture-policy.test/part-01.test.mjs`.

Короткая матрица закрытия:

- DONE (evidence-validated):
  - `whole_instance_backup_v1`
  - `client_gateway_v1`
  - `first_owner_v1` (phase gate снят)
  - ADR-0239 read-only Mail/IMAP срез: контракт + runtime entry points + 24/24 тестов `makosh-mail-communications-slice-testkit` и unit smoke по каждому пакету
  - owner-control runtime dispatcher для Mail/Communications
  - валидация owner-control IPC-команд Mail/Communications (reject `magic-*`,
    недостаточно аргументов для `sync`/`ingest`) через `node --test tests/lifecycle/owner-control-ipc.test.mjs`
- Далее (после закрытия этого closure-gate):
  - по этому плану gate-пути на текущем `first_owner_v1`-контуре отсутствуют; дальше продолжается owner stack execution
    в пределах уже открытых срезов и текущего этапа Recovery/Client Gateway.

## Сделано

- `make -C backend ci` — единственная canonical CI-команда. На текущем дереве он
  выполняется полностью и стабильно (включая architecture/SRP/boundary policy,
  workspace tests, dependency checks, audit и SBOM).
- Architecture evidence связывает текущие policy/docs hashes с результатами
  validation. Release compiler проверяет воспроизводимость unsigned content
  перед signing.
- Control Store содержит operation journal и hardened LAN-development policy.
- Gateway имеет connection, request и deadline budgets; restartable workers
  изолированы от critical recovery/owner control workers.
- Secure-file policy применяется к чувствительным file readers.
- Vault и Blob имеют component-owned offline backup/verify/restore primitives.
  Blob restore использует private staging и atomic publish.
- ADR-0239 фиксирует exact first-owner Mail/IMAP inventory и read-only limits.
  Реализован минимальный production-скелет (contracts/implementation/persistence/runtime entry points):
  это уже достаточная база для policy gate, но runtime-флоу Mail/Communications
  пока ещё требует развёртки на уровне полноценной диспетчеризации синхронизации.

## Активный срез: Whole-instance recovery

Нужно завершить единый offline recovery contour:

1. Exact signed media layout для Control Store, Vault, PostgreSQL/Storage,
   Blob, Event Hub и conditional Scheduler — реализован и покрыт component
   process-port round trip.
2. Kernel-owned coordinator, который вызывает только verified,
   component-owned offline ports и не импортирует их implementation crates —
   реализован; owner-authorized capture + restore CLI готовы, top-level
   restore now performs explicit target lock/authorization/verification and
   event-hub topology replay through an offline reconciliation relay.
3. Quiesce/capture order, retention/encryption, empty-target restore,
   generation/lease/session fencing — реализован; координация/порты закрывают порядок,
   очистку staging-артефактов и проверку `EMPTY_TARGET`/фазов.
4. Recreate JetStream topology и exact-byte outbox/inbox replay — реализовано:
   coordinator-пути обрабатывают `ReconcileTopology` через offline relay и проходят
   тесты маршрутизации/топологии + event-hub recovery topology (в рамках
   `makosh-kernel-recovery-testkit`).
5. Disposable full restore evidence: corrupt media, wrong/replayed authority,
   Blob orphan, PostgreSQL migration ledger, replay and fencing cases — покрыто
   recovery testkit-сценариями (`capture_coordinator`, `restore_coordinator`,
   `control_store_media`, `recovery_media`, `recovery_fence`) и верификацией
   пустого целевого target-слоя.

`whole_instance_backup_v1` на уровне contour/валидации уже закрыт, а gate остаётся fail-closed в `phaseGates.notAuthorized` (production-конфигурация на текущем этапе). evidence-матрица и contour уже полностью покрыты:

- `make -C backend ci` выполняется стабильно: в `makosh-kernel-recovery-testkit`
  тест `paired_remote_listener_serves_technical_routes_only_after_tls_handshake` более не
  флапает после фиксация ALPN в тестовом TLS handshaked-endpoint'е (`backend/tests/support/kernel-recovery/src/tests/gateway_runtime/paired_peer.rs`).
- `RUSTC_WRAPPER= cargo test -p makosh-kernel-recovery-testkit -- --nocapture` ✅ (189 passed, 9 ignored),
- отдельные recovery-scenarios (`capture_coordinator`, `restore_coordinator`, `control_store_media`, `recovery_media`, `recovery_fence`, `process_port`, `gateway_realtime_frames`) — зелёные.

## Следующий срез: Client Gateway admission

`client_gateway_v1` закрыт по evidence:

- owner-device proof и session authorization;
- typed ConnectRPC receipts/status/errors;
- SSE replay/gap/reset/disconnect semantics;
- HTTP/2 TLS, HTTP/3 fallback/no-0RTT и abuse/redaction evidence.

`client_gateway_v1` больше закрыт через:
- `make -C backend architecture-policy-check` (прошёл),
- recovery-тесты `RUSTC_WRAPPER= cargo test -p makosh-kernel-recovery-testkit gateway -- --nocapture` (31 passed).

`first_owner_v1` больше не является gate-забором: он убран из policy как fail-closed
requirement после завершения его подтверждающего slice. Mail/Communications теперь
идёт в рамках общего owner stack без отдельного targeted исключения.

## После admission: Mail/IMAP → Communications

- Завершён exact ten-package inventory из ADR-0239 на уровне крейтов
  (`Cargo.toml` + сборочные entry-point'ы).
- Далее поэтапно реализовывать IMAPS/993 read-only contour, Vault ciphertext handoff,
  bounded sync/body extraction, Blob ticket и durable observation ingress.
- Доказать disposable Dovecot E2E до generated Gateway client и read-only UI.
- На этом этапе уже подтверждён runtime-контур owner-control: неизвестные команды
  и invalid-аргументы `sync` для Mail/Communications отвергаются.

Mail/Communications срез реализован как часть общего owner-пакетного прогресса:
он больше не использует targeted исключение из `first_owner_v1`, но по-прежнему
опирается на Whole-instance recovery, Client Gateway admission и conditional Blob/Scheduler
evidence для полноценных owner-контрактов.

Важно: текущий уровень `Mail/IMAP` — это read-only `ADR-0239` контур (synthetic sync/ingest path + local contracts), а не полноценный production IMAP/Dovecot end-to-end клиент с реальными сетевыми сессиями и Blob-циклом.

## Финальный closure

- `make -C backend ci` выполняется полностью; все шаги завершились без падений
  при повторном прогоне в этой конфигурации.
- `make -C backend architecture-policy-check` и `make -C backend architecture-evidence-check`
  выполняются зелёно на текущем состоянии.
- `node --test tests/architecture/architecture-policy.test/part-01.test.mjs` (из `backend/`) — зелёный.
- `node --test tests/architecture/release-distribution-compiler.test.mjs` (из `backend/`):
  зелёный.
- `node --test tests/lifecycle/owner-control-ipc.test.mjs` (из `backend/`):
  зелёный (проверка ограниченного runtime allowlist и валидации аргументов).
- `make -C backend check-target` на macOS проходит (локальная траектория check только на
  `aarch64-apple-darwin`).
- `node --test tests/architecture/release-distribution-compiler.test.mjs
  tests/architecture/macos-release-manifest.test.mjs
  tests/architecture/linux-release-manifest.test.mjs
  tests/architecture/local-platform-release-input.test.mjs` — все green.
- `two-clean-environment reproducible release artifacts` и signing provenance закрыты
  через тесты `release-distribution-compiler` (independent unsigned manifest,
  `verify-release-reproducibility` CLI, key generation/signature invariants).
- native Linux release/container validation на production уровне закрыт: проверка через
  `check-linux-container` проходит после фикса в `backend/Makefile` (установка
  `clang`, `libclang-dev`, `protobuf-compiler`, добавление `rustfmt` и явная настройка
  `PATH/PROTOC` для x86_64-unknown-linux-gnu toolchain).
- `ADR-0239` реализован на уровне exact ten-package inventory, policy/evidence и owner-control runtime dispatcher.
  `first_owner_v1` из политики снят, `phaseGates.ownerAdmissionExceptions` больше не содержит
  отдельного Mail/Communications exception, а owner-control IPC теперь поддерживает
  bounded Mail/Communications runtime-команды в рамках общего owner stack.

По запросу: `first_owner_v1` теперь **снят полностью**; корректная траектория —
`first_owner_v1` больше не управляется targetted exception, и Mail/Communications идёт как
часть общего owner stack.
