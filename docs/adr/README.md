# Активные ADR clean-room backend

Все ADR предыдущей реализации были вынесены из active documentation 2026-07-15.
Исторический индекс находится в
[legacy documentation reference](../../references/backend-legacy/docs/archive/adr/README.md).
Legacy ADR являются только evidence и контекстом; они не возвращаются в active
policy через ссылки из новых документов.

## Статусы

- `Предложено` — решение обсуждается и ещё не принято.
- `Принято` — решение обязательно для новой реализации.
- `Заменено` — решение полностью заменено более новым active ADR.
- `Отклонено` — решение рассмотрено и не используется.

Поле `Состояние реализации` отделяет принятое решение от факта его реализации.
Статус `Принято` сам по себе не означает, что код уже существует.

## Активные решения

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md)
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md)
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md)
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md)
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md)
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md)
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md)
- [ADR-0207: Канонический реестр бизнес-доменов Макошь](ADR-0207-canonical-business-domain-registry.md)
- [ADR-0208: Allowlist разработки доменов и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md)
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md)
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md)
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md)
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md)
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md)
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md)
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md)
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md)
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md)
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md)
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md)
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md)
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md)
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md)
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md)
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md)
- [ADR-0225: Первый production slice — recovery-only Kernel и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md)
- [ADR-0226: Контекст для AI только через use-case workflows](ADR-0226-ai-context-acquisition-through-use-case-workflows.md)
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md)
- [ADR-0228: Development simulation profile](ADR-0228-development-simulation-profile.md)
- [ADR-0229: Platform Clock contract and deterministic conformance](ADR-0229-platform-clock-contract-and-deterministic-conformance.md)
- [ADR-0230: Blob Platform — opaque references and owner-local metadata](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md)
- [ADR-0231: Private Blob data session and Vault route](ADR-0231-private-blob-data-session-and-vault-route.md)
- [ADR-0232: Browser client identity and same-origin Gateway session](ADR-0232-browser-client-device-identity-and-same-origin-session.md)
- [ADR-0233: Scoped local recovery export and PostgreSQL dump](ADR-0233-whole-instance-backup-and-fenced-restore.md)
- [ADR-0234: Browser-local key binding for synchronised passkeys](ADR-0234-browser-local-key-binding-for-synchronised-passkeys.md)
- [ADR-0235: Private-LAN developer mode](ADR-0235-private-lan-developer-mode.md)
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md)
- [ADR-0237: Временный private-LAN development без owner authority](ADR-0237-temporary-private-lan-development-without-owner-authority.md)
- [ADR-0238: Secure-file FD boundary](ADR-0238-secure-file-fd-boundary.md)
- [ADR-0249: Communications profile for storage_control_v1](ADR-0249-communications-storage-control-v1-admission-profile.md)
- [ADR-0250: Communications profile for nats_data_plane_v1](ADR-0250-communications-nats-data-plane-v1-admission-profile.md)
- [ADR-0251: Opening client_gateway_v1 for owner contracts](ADR-0251-client-gateway-v1-opening-for-owner-contracts.md)
- [ADR-0252: first_owner_v1 Communications admission](ADR-0252-first-owner-v1-communications-admission.md)
- [ADR-0253: Communications legacy surface disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md)
- [ADR-0254: Communications derived search index](ADR-0254-communications-derived-search-index-and-private-content-boundary.md)
- [ADR-0255: Managed owner-key leases](ADR-0255-managed-owner-key-leases-for-derived-projections.md)
- [ADR-0256: Owner-declared client RPC route admission](ADR-0256-owner-declared-client-rpc-route-admission.md)
- [ADR-0257: Event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md)
- [ADR-0258: Correlated duplex managed-control transport](ADR-0258-correlated-duplex-managed-control-transport.md)
- [ADR-0259: Separate typed platform-control path](ADR-0259-separate-typed-platform-control-path.md)
- [ADR-0260: Communications attachment lifecycle event authority](ADR-0260-communications-attachment-lifecycle-event-authority.md)
- [ADR-0261: Communications attachment-anchor handoff](ADR-0261-communications-attachment-anchor-handoff.md)
- [ADR-0262: Mail attachment Blob-admission extension](ADR-0262-mail-attachment-blob-admission-extension.md)
- [ADR-0263: Mail integration settings and Storage admission artifacts](ADR-0263-mail-integration-settings-and-storage-admission.md)
- [ADR-0264: Communications message evidence history query](ADR-0264-communications-message-evidence-history-query.md)
- [ADR-0265: Provider operational client transport admission](ADR-0265-provider-operational-client-transport-admission.md)
- [ADR-0266: Telegram Kernel admission and event-only Communications handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md)
- [ADR-0267: Kernel-staged runtime artifacts and integration state roots](ADR-0267-kernel-staged-runtime-artifacts-and-integration-state-roots.md)
- [ADR-0268: Telegram release composition](ADR-0268-telegram-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0269: Mail release composition](ADR-0269-mail-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0270: Mail capability split](ADR-0270-mail-kernel-admission-and-route-specific-event-handoff.md)
- [ADR-0271: Zulip phase gate](ADR-0271-zulip-kernel-admission-and-event-only-communications-handoff.md)
- [ADR-0272: Zulip release composition](ADR-0272-zulip-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0273: Attachment Security engine](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md)
- [ADR-0274: Attachment Security Blob custody](ADR-0274-attachment-security-evidence-bound-blob-custody.md)
- [ADR-0275: Target-bound cross-owner Blob custody](ADR-0275-target-bound-cross-owner-blob-custody-delegation.md)
- [ADR-0276: WhatsApp phase gate](ADR-0276-whatsapp-kernel-admission-host-bridge-and-event-only-communications-handoff.md)
- [ADR-0277: Gmail API outbound mutation gate](ADR-0277-mail-gmail-api-outbound-mutation-gate.md)
- [ADR-0278: Gmail OAuth setup and refresh gate](ADR-0278-mail-gmail-oauth-setup-and-refresh-gate.md)
- [ADR-0279: Durable Blob custody scope and operation-scoped grants](ADR-0279-durable-blob-custody-scope-and-operation-scoped-grants.md)
- [ADR-0280: Mail event-gated outbound MIME attachments](ADR-0280-mail-event-gated-outbound-mime-attachments.md)
- [ADR-0281: Communications frontend clean-room composition](ADR-0281-communications-frontend-clean-room-composition.md)
- [ADR-0282: Full Communications and Settings capability reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md)
- [ADR-0283: Telegram automation management and preview boundary](ADR-0283-telegram-automation-management-and-preview-boundary.md)
- [ADR-0284: Telegram one-to-one audio calls operational boundary](ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md)
- [ADR-0285: Owner-local upgrade jobs and Telegram Calls realtime backfill](ADR-0285-owner-local-upgrade-jobs-and-telegram-calls-realtime-backfill.md)
- [ADR-0286: WhatsApp operational read and realtime boundary](ADR-0286-whatsapp-operational-read-and-realtime-boundary.md)
- [ADR-0287: Telegram operational realtime replay boundary](ADR-0287-telegram-operational-realtime-replay-boundary.md)
- [ADR-0288: Managed successor quiesce and Storage fence order](ADR-0288-managed-successor-quiesce-and-storage-fence-order.md)
- [ADR-0289: Telegram folder reassignment convergence boundary](ADR-0289-telegram-folder-reassignment-convergence-boundary.md)
- [ADR-0290: Telegram account runtime reconfiguration boundary](ADR-0290-telegram-account-runtime-reconfiguration-boundary.md)
- [ADR-0291: Zulip account, history, operational query and replay boundary](ADR-0291-zulip-account-history-query-and-replay-boundary.md)
- [ADR-0292: Managed integration settings apply and credential binding](ADR-0292-managed-integration-settings-apply-and-credential-binding.md)
- [ADR-0293: Scoped Vault credential retirement and deletion](ADR-0293-scoped-vault-credential-retirement-and-deletion.md)
- [ADR-0294: Mail account credential lifecycle and portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md)
- [ADR-0295: Owner write-only Vault provisioning through Core Gateway](ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md)
- [ADR-0296: Owner module Settings through Core Gateway](ADR-0296-owner-module-settings-through-core-gateway.md)
- [ADR-0297: Fresh owner-proof effective module Settings export](ADR-0297-fresh-owner-proof-effective-module-settings-export.md)
- [ADR-0298: Mail operational read projection and client contract](ADR-0298-mail-operational-read-projection-and-client-contract.md)
- [ADR-0299: Mail sync run history and provider-path health](ADR-0299-mail-sync-run-history-and-provider-path-health.md)
- [ADR-0300: Loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md)
- [ADR-0301: Bundled module discovery and development admission](ADR-0301-bundled-module-discovery-and-development-admission.md)
- [ADR-0302: Bundled managed Settings and first runtime bootstrap](ADR-0302-bundled-managed-settings-and-runtime-bootstrap.md)
- [ADR-0303: Provider-owned QR account linking](ADR-0303-provider-owned-qr-account-linking-and-transient-artifact-custody.md)
- [ADR-0304: Zulip account identity and Settings schema v3](ADR-0304-zulip-account-identity-and-settings-schema-v3.md)
- [ADR-0305: Mail-owned composition, drafts, templates and signatures](ADR-0305-mail-owned-composition-drafts-templates-and-signatures.md)
- [ADR-0306: Repeatable development release refresh and successor fencing](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md)
- [ADR-0307: Mail-owned message flag mutations and provider reconciliation](ADR-0307-mail-message-flag-mutations-and-provider-reconciliation.md)
- [ADR-0308: Mail message identity, IMAP mailbox roles and location authority](ADR-0308-mail-message-identity-imap-mailbox-roles-and-location-authority.md)
- [ADR-0309: Loopback browser Owner Vault provisioning host](ADR-0309-loopback-browser-owner-vault-provisioning-host.md)
- [ADR-0310: Telegram user-only TDLib QR account identity](ADR-0310-telegram-user-only-tdlib-qr-account-identity.md)
- [ADR-0311: Storage successor bundle step lineage](ADR-0311-storage-successor-bundle-step-lineage.md)
- [ADR-0312: Mail permanent delete confirmation and provider authority](ADR-0312-mail-permanent-delete-confirmation-and-provider-authority.md)
- [ADR-0313: Communications canonical read v2 detail and pagination](ADR-0313-communications-canonical-read-v2-detail-and-pagination.md)
- [ADR-0314: Core Gateway authenticated client Blob routing](ADR-0314-core-gateway-authenticated-client-blob-routing.md)
- [ADR-0315: Communications message body content read](ADR-0315-communications-message-body-content-read.md)
- [ADR-0316: Communications saved search derived projection](ADR-0316-communications-saved-search-derived-projection.md)
- [ADR-0317: Communications sender insights derived projection](ADR-0317-communications-sender-insights-derived-projection.md)
- [ADR-0318: Communications evidence export workflow](ADR-0318-communications-evidence-export-workflow.md)
- [ADR-0319: Owner-authorized legacy provider account recovery](ADR-0319-owner-authorized-legacy-provider-account-recovery.md)
- [ADR-0320: Mail multi-account configuration instances and runtime multiplexing](ADR-0320-mail-multi-account-configuration-instances-and-runtime-multiplexing.md)
- [ADR-0321: Legacy provider recovery bundle and native secret custody](ADR-0321-legacy-provider-recovery-bundle-and-native-secret-custody.md)
- [ADR-0322: Loopback native Owner device-proof host](ADR-0322-loopback-native-owner-device-proof-host.md)
- [ADR-0323: Gmail preauthorization with unresolved mailbox identity](ADR-0323-gmail-preauthorization-with-unresolved-mailbox-identity.md)
- [ADR-0324: Empty Protobuf client RPC request semantics](ADR-0324-empty-protobuf-client-rpc-request-semantics.md)
- [ADR-0325: Bounded asynchronous Mail sync execution](ADR-0325-bounded-asynchronous-mail-sync-execution.md)
- [ADR-0326: Audience-scoped Vault request sequence](ADR-0326-vault-audience-sequenced-replay-fencing.md)
- [ADR-0327: Durable target-bound Blob delegation across source successors](ADR-0327-durable-target-bound-blob-delegation-across-source-successors.md)
- [ADR-0328: Storage bootstrap quarantine for policy-invalid owner bundles](ADR-0328-storage-bootstrap-quarantine-for-policy-invalid-owner-bundles.md)
- [ADR-0329: Full-stack development Attachment scanner contour](ADR-0329-full-stack-development-attachment-scanner-contour.md)
- [ADR-0365: Communication recipient suggestion workflow and source boundary](ADR-0365-communication-recipient-suggestion-workflow-and-source-boundary.md)
- [ADR-0366: Communication task candidate extraction and reviewed Task promotion](ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md)
- [ADR-0367: Authenticated client device context for owner runtimes](ADR-0367-authenticated-client-device-context-for-owner-runtimes.md)
- [ADR-0368: Reviewed task candidate promotion workflow](ADR-0368-reviewed-task-candidate-promotion-workflow.md)
- [ADR-0369: Communication note candidate extraction and reviewed Knowledge promotion](ADR-0369-communication-note-candidate-extraction-and-reviewed-knowledge-promotion.md)
- [ADR-0370: Verified Knowledge note owner admission](ADR-0370-verified-knowledge-note-owner-admission.md)
- [ADR-0371: Bounded attachment text extraction workflow](ADR-0371-bounded-attachment-text-extraction-workflow.md)
- [ADR-0372: Kernel-staged runtime resources for managed workflows](ADR-0372-kernel-staged-runtime-resources-for-managed-workflows.md)

Эти ADR фиксируют runtime, communication, storage, infrastructure lifecycle и
границу между provider-specific experience и provider-neutral context, а также
единый client gateway для desktop и Android. Конституция Kernel ограничивает
его техническим control plane и фиксирует boot/recovery state machine.
ADR-0225 закрывает inventory первого production slice: разрешены только шесть
foundation packages recovery-only Kernel, а domains, integrations, workflows и
engines пока имеют пустой фактический inventory. Любое расширение требует
открытия соответствующего phase gate через ADR, policy и executable evidence.
Канонический реестр фиксирует тринадцать начальных business domains и отделяет
их от integrations, workflows и projections.
Текущий implementation allowlist разрешает только Communications, Contacts,
Organizations, Tasks, Calendar, Documents и AI; остальные домены и все
product projections заблокированы.
Event Hub является Kernel control plane над NATS catalog/subscriptions, а
Telemetry Hub обеспечивает независимые от PostgreSQL/NATS локальные logs,
metrics, traces и crash diagnostics через отдельный supervised Collector.
ADR-0211 помещает весь production backend code в `backend/src`, а policy,
scripts, infrastructure и tests — в отдельные backend-owned roots внутри
`backend/`.
ADR-0212 запрещает compile-graph aggregation, отделяет Kernel/Gateway от
owner-specific packages и фиксирует owner-local package topology, включая
узкий Communications ingress для всех integrations. Telegram в ADR является
примером protocol-specific split, а не особым архитектурным случаем.
ADR-0213 определяет SRP через owner, ответственность и причину изменения,
задаёт практическую интерпретацию SOLID/KISS/DRY/YAGNI и проверяемую автономность
каждого module в build, tests, lifecycle, data и failure boundaries.
ADR-0214 отделяет Scheduler от Kernel/Event Hub, оставляет исполняемый job code
в module-владельце и фиксирует durable schedules, owner-local execution,
default reconciliation и горячее изменение runtime policy без загрузки кода из
database.
ADR-0215 разрешает любому локальному process пройти недоверенную регистрацию,
но до явного approval оставляет его без capabilities. Effective grants являются
пересечением module request, owner settings и hard Kernel policy; `managed` и
`external` lifecycle имеют разные restart guarantees, а обязательная подпись
binary не является admission condition первой версии.
ADR-0216 сохраняет registrations, grant epochs и desired infrastructure state
в private kernel-owned SQLite через отдельный persistence adapter. Kernel
стартует и поднимает local recovery surface без PostgreSQL, PgBouncer, NATS,
Vault и modules; business data и secrets в Control Store запрещены.
ADR-0217 запрещает обязательный bootstrap configuration file и любые
Макошь-specific environment overlays. Default data directory определяется
операционной системой, explicit `--data-dir` выбирает отдельный instance, а
недоверенный Control Store оставляет только restricted local recovery.
ADR-0218 отделяет logical OwnerAuthority от OS identity и module
processes. Каждое device имеет отдельную отзываемую ES256 keypair,
private key остаётся в platform signer, а online recovery недоверенного
Control Store ограничен sanitized `status/validate/export`.
ADR-0219 сохраняет open `pending` registration без publisher signature,
но требует signed distribution entry либо owner-pinned digest для
любого `managed` process. Kernel проверяет exact bytes перед каждым
launch, не скачивает code и не выполняет automatic rollback.
ADR-0220 фиксирует binary `DurableEnvelopeV1`, exact contract/schema binding,
byte-for-byte outbox-to-NATS delivery, пять message kinds, отдельный technical
DLQ record и строгую границу между internal data plane и client SSE.
ADR-0221 разделяет signed distribution inventory, runtime descriptor,
effective grants и observed state. `ModuleDescriptorV1` является exact
Protobuf declaration, а capability становится единицей approval, readiness,
dependency resolution и revoke; managed binding pin-ит descriptor digest.
ADR-0222 делает Settings Registry обязательным Kernel component. Module
владеет schema и смыслом полей, Kernel — typed desired/effective revisions в
private Control Store, validation/application и supervised restart. Secrets,
business/runtime state и Scheduler records настройками не являются.
ADR-0223 выделяет Vault в отдельный verified managed process. Kernel вычисляет
grants и маршрутизирует только HPKE ciphertext, а Vault хранит bounded credential
material в SQLCipher с record-level AEAD и выдаёт process-bound leases. Bulk
provider session state остаётся у integration owner. Exact `vault_v1`
production packages, storage format и conformance tests реализованы;
whole-instance backup открыт ADR-0233.
ADR-0224 выделяет Storage Control в отдельный managed control-plane process.
Kernel supervises PostgreSQL, PgBouncer и Storage Control; modules выполняют
business SQL напрямую через PgBouncer, а Storage Control владеет bootstrap,
roles/grants/budgets, migration admission и readiness. Runtime credentials
выдаёт Vault, а PgBouncer не считается единственной security boundary. Target
принят, но production packages и process-level isolation tests отсутствуют.
ADR-0225 зафиксировал исходный recovery-only production graph. Последующие
атомарные gates открыли managed platform runtimes, NATS, Blob, Scheduler,
public client Gateway, whole-instance backup и первый owner Communications.
Текущий Kernel по-прежнему честно сообщает `module_control_plane`; отдельный
production state `ready` не заявляется.
ADR-0226 запрещает AI прямой доступ к таблицам и query APIs других owners.
Cross-owner AI context собирает отдельный use-case workflow через явные public
contracts в distinct generated request с common `AiContextReceiptV1` и
concrete use-case context. Global fragment union, opaque payload bytes,
generic Context API и durable Context projection остаются заблокированы.
ADR-0228 вводит отдельный full-platform development profile для local
development всех platform components с software trust adapters и local services.
Он не является deployment profile и никогда не служит evidence для production
gates.
ADR-0229 открывает `clock_v1`: UTC and monotonic reading, explicit
discontinuity policy and deterministic fake clock. It does not open Scheduler,
module timers or timezone/DST calendar evaluation.
ADR-0230 фиксирует Blob Platform boundary: opaque references, owner-local
metadata, Vault-scoped encryption authority, bounded range/path handling and
fenced retention/GC. `blob_v1` открыт после runtime conformance.
ADR-0231 фиксирует следующий mandatory Blob vertical slice: private direct
socket authenticated by a short-lived generation-bound session grant and
ciphertext-only inherited Vault routing. Kernel never receives Blob plaintext.
ADR-0232 включает browser как отдельный first-party client: он получает
owner-approved, revocable, device-bound WebAuthn ES256 identity и только
short-lived same-origin HttpOnly Gateway session. Его owner-neutral
`browser_client_v1` gate открыт отдельно; ADR-0251 затем открывает
`client_gateway_v1` для owner contracts без Gateway-owned business facade.
ADR-0233 открывает `whole_instance_backup_v1`: signed/encrypted media включает
Control Store, Vault, PostgreSQL, Blob, Scheduler и Event Hub topology через
component-owned offline ports, с empty-target restore и generation fencing.
ADR-0234 допускает synchronised WebAuthn passkeys только как одну часть
двухключевой browser identity: session требует ещё и подписи отдельного
non-extractable browser-local WebCrypto key. Новый Mac с синхронизированным
passkey должен пройти новый CLI-approved pairing.
ADR-0235 заменён: persistent LAN owner bypass оказался несовместим с owner
device proof boundary.
ADR-0236 предлагается как уточнение integration granularity: integration
является owner/runtime boundary, protocol/SDK client — owner-local adapter, а
настроенное подключение — opaque configuration instance. Решение не выбирает
первый owner и не открывает `first_owner_v1`.
ADR-0237 оставляет `--dangerous-lan-development` только временным technical
listener без owner APIs: он не сохраняется и не даёт owner authority.
ADR-0238 вводит один FD-bound secure-file contract для bounded no-symlink
readers private material и release inputs; rollout readers остаётся явным
admission prerequisite.
ADR-0239 остаётся историей раннего Mail/IMAP slice. ADR-0252 заменяет временный
owner exception exact admission домена Communications; provider integrations
остаются отдельными units и не входят в owner inventory домена.
ADR-0240 фиксирует Telegram как отдельного integration owner с собственными
operational contracts/state и только typed evidence boundary в Communications.
ADR-0256 реализован как owner-neutral descriptor-declared ClientRpc routing;
Kernel/Gateway не импортируют owner implementations и не декодируют payload.
ADR-0265 запрещает считать legacy Communications REST provider transport.
ADR-0266 задаёт первый exact Telegram phase gate: Kernel владеет только
admission/routing/fencing control plane, а Telegram → Communications handoff
остаётся event-only через integration outbox, NATS и Communications inbox.
ADR-0267 убирает native artifact path и provider session-store directory из
settings: exact runtime dependency приходит из verified managed binding, а
private state root stage-ит Kernel без знания provider semantics.
ADR-0268 выделяет Telegram release composition в отдельную integration-owned
assembly unit: она материализует canonical descriptor/settings/storage bytes и
неподписанный exact artifact fragment, а generic distribution compiler
подписывает только полный release без передачи signing authority integration.
ADR-0269 применяет ту же authority boundary к Mail как к отдельному integration
owner: Mail-owned assembly unit материализует canonical
descriptor/settings/storage bytes и unsigned fragment без native dependency,
а Kernel/Communications/Gateway и Mail runtime не зависят от этой build unit.
ADR-0270 разделяет Mail operational sync/delivery и provider credential
purposes на независимые capability units: integration использует Kernel/Core
для admission и opaque routing, а Communications получает Mail evidence
только через durable typed events.
ADR-0271 задаёт отдельный Zulip phase gate: Kernel/Core владеет только
platform admission, leases, fencing и opaque client routing; Zulip provider
evidence пересекает Communications boundary только через owner-local outbox,
NATS и Communications inbox. Command и operation query становятся разными
capability units, а runtime обязан перейти на один correlated V2 frame pump.
ADR-0272 выделяет Zulip release composition в отдельную integration-owned
assembly unit с exact runtime/settings/storage artifacts и двухэлементным
unsigned fragment. Она не имеет signing authority и не входит в Kernel,
Gateway или Communications.
ADR-0273 вводит отдельный `attachment_security` engine owner: integration
публикует provider-neutral scan candidate, engine durably join-ит его с
canonical Communications `blob_admitted` и публикует typed safety verdict из
собственного outbox. Kernel/Core получает отдельный managed Engine launch
contract и остаётся control plane, а Communications не импортирует scanner
implementation.
ADR-0274 закрывает обнаруженный live conformance разрыв: direct read
integration-owned Blob остаётся запрещён, revision-2 candidate переносит
bounded source custody proof, а engine выполняет evidence-bound transfer в
собственную Blob custody перед one-use read. Kernel не декодирует candidate и
не переносит bytes/verdict.
ADR-0275 устраняет скрытое смешение module owner и human owner в Blob custody:
same-owner proof сохраняет прежний fence, а cross-owner delegation обязана
криптографически bind-ить exact target owner/registration/capability. Audience
принадлежит public owner contract, поэтому integration не импортирует target
runtime implementation, а Kernel не выбирает recipient по business event.
ADR-0276 задаёт отдельный WhatsApp phase gate: Kernel/Core владеет только
admission, leases, fencing, private host-route staging и opaque public client
routing; WhatsApp evidence пересекает Communications boundary только через
owner-local outbox, NATS и Communications inbox. Host bridge, command и query
являются разными capability units, а runtime обязан использовать один
correlated V2 control reader без cloned FD.
ADR-0277 открывает отдельный Gmail delivery gate внутри Mail integration:
outbound-only GrantSet разрешает bounded Gmail HTTPS mutation, owner-local
durable acceptance/query и neutral event replay без IMAP/SMTP/attachment
capabilities или Communications facade.
ADR-0278 задаёт Mail-owned Gmail OAuth setup/refresh gate: Core/Kernel
переносит только opaque owner routes и action-specific Vault ciphertext,
Mail владеет PKCE/operation/binding state, а access и refresh credentials
остаются разными secret classes и capability responsibilities. Gate открыт
после live exact-form/CAS/revoke/negative-output conformance; Communications в
credential lifecycle не участвует.
ADR-0279 разделяет ephemeral Blob access fence и durable at-rest custody:
descriptor объявляет exact custody scope и operation set, Kernel выдаёт только
operation-scoped session, а ciphertext, Vault content-key revision и technical
quota ledger переживают restart/re-registration без generic read/write grant.
ADR-0282 расширяет завершённый frontend ownership/transport cutover до полного
capability reconstruction: Communications остаётся provider-neutral domain,
Mail/Telegram/WhatsApp/Zulip — независимыми integrations, Settings —
app-composition, а cross-owner и AI use cases получают отдельные workflow
units и atomic admission gates.
ADR-0284 разделяет Telegram Calls на independently admitted history, signaling
и real tgcalls media gates: provider calls остаются Telegram integration
surface, а cross-provider evidence и transcription принадлежат отдельным
composition/workflow units. History и signaling gates реализованы отдельными
Query/Command/Realtime capabilities и durable owner-local operation journal;
real audio и Calls umbrella остаются закрыты.
ADR-0285 вводит owner-local upgrade job без fake Scheduler schedule:
owner-neutral Job Platform protocol несёт exact upgrade command, а Telegram
Calls persistence владеет durable execution, lease и checkpoint для
restart-safe V3-to-V4 realtime backfill. Реализованный V6 bundle остаётся
DDL-only; owner executor сохраняет прежние cursors через отдельный replay-order
mapping, а Kernel/Scheduler/Communications не получают Telegram handler или
owner SQL.
ADR-0286 разделяет WhatsApp operational closure на отдельные read и realtime
gates: integration владеет typed projections, bounded search и replay journal,
а Kernel/Gateway только fence-ят exact routes и grants. Metadata-only history
не превращается в fake content; upgrade требует bounded provider resync, а
frontend остаётся вторичным integration-owned consumer.
ADR-0287 добавляет отсутствующую Telegram operational realtime capability:
integration владеет account-scoped ordered journal и explicit cursor reset, а
Kernel/Gateway только допускают exact opaque route. Lifecycle/query aliases и
выдача internal durable envelope клиенту запрещены.
ADR-0288 закрывает общую managed-successor гонку: durable `revoking` остаётся
authority fence, supervisor до physical Storage fence запрещает autorestart
exact predecessor worker, а новый runtime generation резервируется только
после fence и join. Provider-specific retries и перенос lifecycle в integration
запрещены.
ADR-0289 фиксирует честную Telegram folder reassignment semantics: один durable
command сходится к exact target через fresh provider delta и обязательную
финальную проверку, а partial success повторно планируется от текущего TDLib
state. Provider atomicity, stale saved plan и fake terminal `ok` запрещены.
ADR-0290 заменяет fake lifecycle restart отдельным Telegram-owned
`telegram.reconfiguration.v1`: client задаёт только exact intent и expected
epoch, runtime получает fresh Vault leases, физически заменяет TDLib client и
завершает durable target epoch только после restore. Kernel переносит opaque
route и grant, не интерпретируя Telegram lifecycle.
ADR-0291 разделяет полный Zulip experience на account lifecycle, bounded
provider history convergence, owner-local operational query и realtime replay:
Kernel/Core допускают только exact opaque routes и leases, Zulip integration
владеет projection/storage/runtime, а Communications получает neutral evidence
только через durable events.
ADR-0292 устраняет обход Settings Registry при managed integration launch:
Kernel выполняет provider-neutral desired/effective replacement, а credential
revision хранится только как integration-owned Vault binding. Settings, Vault,
integration persistence, runtime и release assembly остаются отдельными
функциональными units.
ADR-0293 закрывает недостающий Vault lifecycle primitive: exact scoped
`retire` удаляет active ciphertext и создаёт durable tombstone, а отдельный
`delete` повышает tombstone до deleted. Kernel согласует только declared
action/grant/runtime fences и не интерпретирует provider logout; integration
выбирает purpose через свой typed lifecycle contract.
ADR-0294 переносит credential revisions из Mail Settings в Mail-owned CAS
bindings: Bind и sanitized Query являются отдельными generated contracts,
текущий runtime quiesce-ит изменённый provider path, а exact Vault revision
активируется только Settings successor generation. Retire/Delete, explicit
Retry и lifecycle Status ведут Mail-owned per-purpose journal, quiesce-ят все
provider paths до exact Vault mutation и сохраняют account tombstone; typed
portability остаётся отдельным незакрытым gate.
ADR-0295 вводит отсутствующий first-party write-only provisioning path:
Core Gateway требует operation-bound fresh device proof, Kernel проверяет
exact approved Vault-purpose capability и переносит только HPKE ciphertext, а
Vault атомарно сохраняет mutation и durable idempotency receipt без record ID
или credential read-back. Backend и client adapters остаются разными gates.
ADR-0296 открывает отсутствующий public Settings path без экспорта private
owner-control protocol: Core Gateway принимает только typed provider-neutral
update/apply intent, Kernel требует fresh active-device proof и сохраняет
authority у Settings Registry, а managed integration replacement остаётся
generic successor operation ADR-0292.
ADR-0297 добавляет отсутствующий fresh-proof export effective Settings:
Core Gateway возвращает только typed client-visible values после проверки
current revision, admitted schema hash и active device, не импортируя Mail или
raw runtime descriptor. Mail собирает свой versioned portability artifact и
resumable multi-receipt import только в first-party integration UI.
ADR-0298 разделяет Mail provider operational projection и Communications
canonical content: Mail владеет bounded folders/threads/messages query, Core
Gateway только маршрутизирует exact contract, а full body app получает через
отдельный Communications content contract по opaque observation anchor.
ADR-0299 отделяет Mail-owned sync run journal и provider-path health от
Scheduler schedules и Communications analytics: exact query возвращает только
bounded sanitized run evidence, restart помечает stale generation как
interrupted, а newsletter detection остаётся Communications-derived use case.
ADR-0300 вводит отдельную непроизводственную assembly boundary для root
`make dev`: loopback Core Gateway и Vite соединяются exact same-origin proxy с
ephemeral server-side proof, readiness проверяется до открытия browser, а
private-LAN technical profile не получает owner authority.
ADR-0301 закрывает отсутствующий generic seam между signed bundled artifact и
pending registration: Kernel проверяет installed manifest и создаёт только
proposal, owner отдельно approve/bind/start-ит units, а development assembly
координирует exact platform/domain/integration plan без provider secrets.
ADR-0302 определяет deterministic development bootstrap для managed Settings и
runtime: assembly применяет только declared typed defaults и generic owner
control operations, не забирая provider semantics у integration units.
ADR-0303 фиксирует provider-owned QR linking: Telegram передаёт transient
TDLib link через existing opaque authorization route и рендерит QR локально,
а WhatsApp оставляет QR внутри owner-visible Tauri WebView. Kernel не становится
generic QR/account service, browser не подделывает native pairing.
ADR-0304 заменяет ложную Zulip bot-only identity на Settings schema major 3 с
`zulip.account_email`: Zulip integration владеет email/API semantics, Kernel
применяет только generic typed settings и не выбирает bot/user behavior.
ADR-0307 разделяет Mail operational mutations по различным failure semantics:
read/star flags идут через exact convergent command и owner-local journal,
folder moves/delete остаются отдельным gate, а Kernel/Core Gateway переносят
opaque payload и не становятся generic provider-command service.
ADR-0308 вводит обязательную identity foundation для Mail location mutations:
client использует стабильный Mail-owned `message_id`, IMAP locator
`mailbox/UIDVALIDITY/UID` остаётся private, special-use roles берутся из
bounded provider discovery, а permanent delete получает отдельный destructive
capability и grant.
ADR-0309 добавляет недостающий development-only host adapter для browser
`make dev`: отдельный native process переиспользует HPKE/Vault ceremony,
Vite добавляет private proxy proof только для exact loopback route, а
Gateway/Kernel остаются blind к credential plaintext. Telegram integration
после реального account setup сама запускает TDLib QR lifecycle.
ADR-0310 удаляет ложный Telegram bot path из active clean-room contract:
Telegram integration поддерживает только TDLib user account, client больше не
выбирает `provider_kind`, а QR создаётся только из transient provider
`tg://login` после защищённого account setup.
ADR-0311 устраняет повторный DDL при cumulative Storage bundle successor:
exact predecessor step наследуется только при совпадающем digest и получает
immutable acceptance row текущей revision, а digest drift и downgrade
отклоняются до mutation.
ADR-0312 отделяет необратимое Mail удаление от reversible location: отдельные
command/query grants требуют current Trash projection, stale-revision fence и
явное confirmation; IMAP использует только UIDPLUS `UID EXPUNGE`, а Gmail
требует отдельного broad-scope consent без автоматического повышения OAuth
authority. Kernel/Core Gateway остаются owner-neutral и переносят opaque bytes.
ADR-0313 закрывает следующий Communications reconstruction gate: existing
metadata-only owner query получает exact message detail и scoped opaque keyset
continuation для всех repeated list/search operations. Frontend собирает
participants, attachment anchors, references и evidence history только через
Communications owner contract; provider fallback, content и Blob locators
запрещены.
ADR-0322 закрывает обнаруженный live разрыв loopback owner mutations:
отдельный native app signer подписывает только exact Kernel challenge initial
development device, Gateway/Kernel сохраняют fresh P-256 proof, а private key
не попадает в browser, Vite, integration или domain.
ADR-0323 допускает честный Gmail pre-authorization target, когда legacy
external identity не является mailbox: Mail runtime использует provider alias
`me`, разрешает current OAuth и sync, но оставляет delivery `not_configured`
без valid `from_address`; fake address, token import и provider semantics в
Kernel/Communications запрещены.
ADR-0324 фиксирует transport semantics пустого Protobuf request: Core Gateway
передаёт exact zero-length payload descriptor-declared owner runtime, а
owner-specific decoder сохраняет authority над schema validation. Искусственные
поля и Mail-specific Gateway special case запрещены.
ADR-0325 переводит Mail sync в durable asynchronous operation: client получает
быстрое acceptance, provider I/O выполняется вне control loop, а IMAP получает
bounded chunks, retries и whole-run deadline без импорта Communications.
ADR-0326 заменяет исчерпаемый Vault replay set на audience-scoped monotonic
sequence high-watermark: owner-neutral runtime protocol владеет exact opaque
request ID рядом с ciphertext route, private Vault хранит bounded число runtime
sessions, а Mail/Blob/Communications не получают доступ к replay lifecycle.
ADR-0327 сохраняет current source registration/grant fencing для target-bound
Blob delegation, но разрешает durable event пережить benign successor
generation source-процесса без hidden synchronous lease между integration и
engine.
ADR-0328 не позволяет policy-invalid owner bundle обрушить весь Storage:
Storage Runtime до выдачи credentials/roles/pool исключает invalid binding из
effective bootstrap, остальные owners продолжают работу, а replacement идёт
через обычный immutable successor без reset Control Store или ослабления AST
policy.
ADR-0329 добавляет реальный ClamAV daemon в authenticated full-stack `make dev`
как loopback-only scanner infrastructure, сохраняя engine build units и
разрешая только одно exact owner-local восстановление jobs, исчерпанных до
появления scanner contour.
ADR-0330 вводит отдельный provider-neutral delivery-intent workflow owner:
canonical route planning, sealed owner-local body custody и runtime/assembly
являются отдельными build units, а provider execution остаётся integration.
ADR-0331 разделяет outbound delivery на четыре provider-owned event contract
build units без общего facade: durable command несёт только opaque cursors и
target-bound Blob receipt/proof, а terminal results остаются typed и
provider-owned.
ADR-0332 добавляет workflow-owned transactional event boundary: четыре exact
provider command encoder/result decoder adapter, owner-local outbox и
idempotent terminal-result inbox работают без provider facade, cross-owner
storage или payload decode в Kernel/Core.
ADR-0333 заменяет неисполняемое workflow-local sealing тела delivery intent на
managed Blob write с exact target-bound custody proof. Workflow persistence
хранит только receipt/proof и canonical route; четыре integration runtimes
остаются независимыми target owners.
ADR-0334 делает только receipt-bound Blob write идемпотентным при retry:
существующий deterministic reference принимается лишь после полного SHA-256
сравнения внутри Blob runtime, а обычный write остаётся create-only.
ADR-0336 вводит недостающий owner-neutral managed `query_rpc`: caller объявляет
exact contract dependency, Kernel разрешает единственный current approved
provider и проверяет grants/runtime fences, не декодируя owner payload.
Первым consumer становится delivery-intent route resolution через public
Communications contract; `client_rpc`, durable events и replayable client
realtime остаются отдельными interaction kinds.
ADR-0337 вводит недостающий owner-neutral managed `client_realtime`: owner
атомарно хранит client-safe transition и monotonic cursor, Kernel проверяет
exact descriptor capability/runtime fences, а общий Gateway SSE выполняет
bounded replay/live fan-out без owner API, cross-owner SQL или выдачи
`DurableEnvelopeV1` клиенту.
ADR-0338 переводит client-safe system health на общий replayable Gateway SSE:
Kernel вычисляет sanitized status, Gateway публикует change-only frame, а
frontend выполняет один bootstrap без периодического polling.
ADR-0339 вводит отдельный capability-routed managed `request_rpc` для typed
mutation с immediate receipt: provider inventory, authorization, runtime
delivery и no-retry semantics не смешиваются с `query_rpc` или `client_rpc`.
ADR-0340 определяет отдельный `communication_bulk_action` workflow: batch и
targets сохраняются до fan-out, каждая цель использует стабильный operation ID
и public delivery-intent `request_rpc`, а provider completion остаётся вне
bulk state.
ADR-0341 согласует отдельный `communication_delayed_delivery` workflow с
Kernel, Scheduler и delivery-intent: schedule control идёт durable
command/result через event spine, due execution получает стандартный
`ScheduledJobCommandV1`, private body остаётся в workflow-owned Blob custody,
а cancellation race решает Scheduler. `scheduler_v1` уже подтверждён live
managed evidence; workflow gate остаётся закрыт до module-originated
schedule-control contract и отдельных delayed-delivery units.
ADR-0342 вводит недостающий durable module-to-Scheduler seam: module может
создать или отменить только one-shot schedule собственного approved JobKind,
Scheduler сохраняет command/result до ACK, а Kernel/Event Hub проверяют
topology и fences без decode business payload. Protocol foundation реализован;
durable runtime gate остаётся закрыт до persistence/JetStream/live evidence.
ADR-0344 добавляет отдельный delayed-delivery execution-store adapter:
execution port и owner-local persistence связываются явным typed mapping без
SQL в orchestration, persistence dependency на execution или смешивания с
Blob/request transport adapters.
ADR-0346 определяет отдельный `communication_cross_channel_forward` workflow:
Communications подготавливает source evidence как target-bound Blob delegation,
workflow сохраняет provenance и передаёт exact body в delivery-intent, а
Kernel/Core только маршрутизируют exact contracts и не становятся provider
facade.
ADR-0347 фиксирует event-only source preparation для cross-channel forward:
workflow публикует durable command, Communications отвечает durable result с
target-bound Blob receipt, а direct RPC, generic content API и cross-owner SQL
запрещены.
ADR-0348 отделяет module-to-module delivery-intent ingress от client RPC:
workflow публикует bodyless durable command с fixed target-bound Blob receipt,
delivery-intent атомарно отвечает submitted/rejected result, а provider
selection остаётся внутри delivery-intent workflow. Exact contract build unit
и cross-channel transactional event persistence реализованы; runtime adapters
и live managed evidence остаются открыты.
ADR-0349 фиксирует provider-neutral Communications call evidence как отдельные
contract/core/persistence/runtime units и event-only ingress от integrations.
Live managed Telegram → NATS → Communications → Gateway SSE conformance
реализован, поэтому capability gate открыт без открытия Telegram media gate.
ADR-0350 разделяет module owner и authenticated human owner в managed
domain/integration launch: первый остаётся authority для grants, Event Hub и
storage, второй — для provider/domain tenancy и client realtime.
ADR-0351 открывает Review как отдельного domain owner для Макошь
pending/reviewed/dismissed, pin, importance и snooze. Contract/core и
owner-local idempotent command/query/realtime persistence units реализованы
без Communications dependency. Самостоятельный managed Review runtime,
отдельная unsigned release assembly, signed Kernel admission, Gateway и
restart-safe shared SSE replay прошли live conformance; gate открыт.
ADR-0352 делает Event Hub launch configuration capability-scoped: Kernel
передаёт eventless pair только domain без approved event route requests, а
client realtime продолжает идти через общий replayable SSE. Eventless Review
live conformance подтверждает этот контракт без фиктивного NATS grant.
ADR-0353 разделяет AI Reply на Communications-owned event-backed source
handoff, отдельный `communication_reply_suggestion` workflow, AI inference
engine и Ollama integration. Client content ticket не переиспользуется,
private body не проходит через NATS/Gateway, а все четыре gates остаются
закрыты до отдельных build units и live inference evidence. Первый
Communications-owned source contract build unit уже реализован, но сам source
handoff gate ждёт persistence/runtime и live event evidence.
ADR-0354 вводит узкое delegated implementation правило для provider extension
ports: только Integration module может реализовать exact foreign-owned
`request_rpc` после explicit capability approval. Domain/workflow/engine,
query/client/realtime surfaces и event authority сохраняют same-owner
ограничения. Первым consumer является Ollama integration, реализующая
AI-owned provider generation port без присвоения AI business authority.
ADR-0355 расширяет capability-scoped Event Hub launch на Integration runtime:
eventful integration получает exact endpoint/credential pair, eventless
integration — exact `empty + 0`. Half-configured пары и фиктивные NATS grants
запрещены; первым live consumer является локальная Ollama integration.
ADR-0356 отделяет semantic AI request identity от обновляемого Blob authority:
reply workflow переносит Communications custody, создаёт отдельный
AI-target-bound Blob, хранит только bounded recovery receipts и Ack-ает source
event после terminal inference и cleanup обоих Blob. Runtime unit, unsigned
assembly и signed release artifacts реализованы; live managed orchestration
остаётся открыта. Dev release compiler уже включает exact runtime и Storage
artifacts в подписанный manifest.
ADR-0357 вводит bounded canonical `message_subject` и coordinated revision 2
для Communications ingress/AI source: Mail передаёт exact IMAP/Gmail subject,
Communications сохраняет его как evidence и формирует один typed
sender/subject/body Blob, а reply workflow переводит его в отдельный AI-owned
content contract без integration import, raw private-content persistence или
body-only compatibility facade. Slice реализован; canonical evidence major 1
также переведён на revision 2 с новым schema digest.
ADR-0358 расширяет capability-scoped Event Hub launch на Engine runtime:
event-backed engine получает exact endpoint/credential pair, а engine без
approved event route получает `empty + 0` и не требует Event Hub topology.
Half-configured пары, фиктивные credentials и пустые event capabilities
запрещены. Runtime protocol, Kernel composition и signed managed AI negative
conformance реализованы; первым eventless consumer является AI inference
engine.
ADR-0359 выделяет attachment archive inspection из Communications в отдельный
bounded engine. On-demand request, provider-neutral scan candidate и canonical
`safe_for_delivery` объединяются owner-local, source bytes переходят только
через target-bound Blob custody, а ZIP adapter читает metadata без extraction.
API/core/ZIP/persistence units реализованы. Отдельно согласован target-owned
ingress contract: Archive публикует durable delegation command, Attachment
Security сверяет собственный safe scan/current custody и возвращает fresh
redelegated proof durable result event без engine-to-engine RPC. Ingress code,
typed routes и bounded exact envelopes реализованы; owner-local handoff
persistence на стороне Archive также реализует deterministic request outbox,
exact result inbox и создаёт parser job только после fresh delegated proof.
Attachment Security также реализует отдельные durable
command-consumer/result-publisher capabilities, exact replay inbox, проверку
completed safe verdict/current custody только по owner-local state, fenced
delegation jobs, managed-control redelegation и exact result outbox. Отдельные
Archive runtime/assembly, authenticated Gateway/SSE и managed live contours
реализованы; production gate закрыт.
ADR-0360 вводит отдельную managed control operation для target-bound
redelegation уже принятого Blob. Kernel проверяет predecessor proof, exact
evidence lineage и current custodian runtime/grant, но не читает bytes или Blob
metadata; следующий target получает новый proof только через typed durable
event и выполняет обычный transfer, где Blob runtime остаётся final custody
authority. Runtime protocol, Kernel issuance, typed Blob client и Blob
data-plane lineage validation реализованы; первый business event flow и live
conformance остаются в Archive Inspection gate.
ADR-0365 выделяет Smart CC в отдельный `communication_recipient_suggestion`
workflow. Communications передаёт bounded body только через distinct durable
source events и target-bound Blob, workflow возвращает typed role/rationale
candidates и не меняет recipients. AI/Ollama, Contacts resolution, provider
commands и presentation strings не входят в gate; Kernel/Gateway остаются
owner-neutral и используют существующие admission, routing и shared SSE.
ADR-0366 разделяет перенос task candidates между четырьмя owners:
Communications передаёт source custody, отдельный extraction workflow создаёт
immutable candidate, Review хранит human decision, а Tasks материализует Task
только по typed durable approved-candidate command. Cross-owner flow идёт через
events и target-bound Blob, существующий Review attention API не становится
generic facade, а deterministic V1 не притягивает AI/Ollama без измеренной
необходимости.
ADR-0367 проводит уже аутентифицированный browser device principal через
owner-neutral Gateway client envelope до exact managed runtime. Actor не берётся
из business payload, Kernel не интерпретирует owner semantics, а Review может
сохранять отзываемое human decision evidence без Gateway facade.
ADR-0368 закрывает отсутствующий cross-owner переход между Review approval и
Tasks command отдельным `reviewed_task_candidate_promotion` workflow. Workflow
владеет только durable correlation/inbox/outbox, не читает Blob и не импортирует
domain implementations; terminal Tasks result возвращается в Review через
отдельный Review-owned typed contract и только затем меняет promotion
projection/SSE.
ADR-0369 переносит legacy note extraction как отдельный deterministic workflow:
Communications отдаёт source через target-bound Blob, Review владеет human
decision, а accepted result может стать durable Knowledge truth только через
отдельный promotion workflow и target command.
ADR-0370 узко разблокирует Knowledge для `VerifiedKnowledgeNoteV1`: exact
command/core units принимают только reviewed candidate с provenance и
Knowledge-bound Blob. Generic Notes, Graph/Search/Context/Memory projections и
direct domain calls остаются запрещены; managed owner gate ещё требует
persistence/runtime/assembly и live evidence.
ADR-0371 выделяет text extraction из Communications в отдельный bounded
workflow: source authority приходит только через Communications и Attachment
Security events, bytes переходят через target-bound Blob, parser adapters и
derived text остаются отдельными build units. Exact eleven-unit topology,
managed UTF-8/PDF/DOCX/`eng+rus` OCR, restart/outage/stale/privacy contours и
production gate реализованы.
ADR-0372 обобщает Kernel-staged runtime artifacts на integration, workflow и
engine без допуска domain: exact native executable и read-only model data
приходят только из signed distribution через descriptor/grant intersection.
Первый consumer — `eng+rus` OCR workflow; Settings, system Tesseract и
machine-local path fallback запрещены. Exact OCR resource contour реализован.
ADR-0373 выделяет attachment preview из Communications в отдельный bounded
workflow. Source custody приходит только через Attachment Security events,
renderer выбирается по bytes/magic, private content выдаётся one-use ticket
через exact `client_blob`, а text/image/PDF/DOCX/media adapters, persistence,
runtime и assembly остаются отдельными build units. Gate остаётся `planned` до
полного managed/live evidence.
ADR-0376 фиксирует explicit owner-authorized replay для canonical durable bytes,
которые пережили bounded JetStream retention. Replay остаётся producer-local,
публикует те же bytes/message ID, не сбрасывает outbox state и не создаёт Kernel
facade. Исторический Preview закрывается отдельным workflow gate только после
двух producer-owned replay operations и terminal browser SSE/client_blob proof.
ADR-0377 закрывает обнаруженный live-browser gap до admission: public replay
Start становится provider-neutral и принимает только authenticated operation и
canonical attachment anchor. Exact durable message выбирают и аудируют только
Communications и Mail в собственных replay indexes после двух typed event
commands; Kernel/Gateway/Event Hub не получают business selection, frontend не
получает provider registrations и продолжает terminal lifecycle через один SSE.
ADR-0378 выделяет attachment translation в отдельный workflow owner. Source
Text Extraction передаётся только exact durable events и target-bound Blob,
AI Engine получает distinct attachment-translation use-case contract, а
Communications, provider integration, Kernel и Gateway не становятся facade.
ADR-0379 разделяет Mail address-book sync между Mail integration, отдельным
`mail_contacts_sync` workflow и минимальным Contacts command owner. Provider
protocol/ETag остаются в Mail, canonical identity truth — в Contacts, а
direction, correlation, checkpoints и retry — в workflow; взаимодействие идёт
только через typed durable events/commands и owner-local storage. Contacts
command gate реализован пятью изолированными units и доказан live managed
Vault/Storage/NATS contour; сам Mail sync workflow остаётся planned.
ADR-0380 задаёт managed configuration instances и typed Settings bootstrap для
workflow runtime без account/provider authority в Kernel или Scheduler.
ADR-0381 добавляет шестую Contacts-owned source-port unit: private contact
snapshot переходит в Mail только через exact target-bound Blob, а durable event
остаётся bodyless и revision-bound.
ADR-0382 фиксирует Mail-owned address-book execution: Google People и CardDAV
являются отдельными provider units, provider выбирается только typed Mail
Settings, OAuth/CardDAV credentials имеют разные exact purposes, а durable
inbox/result outbox остаются в отдельной Mail persistence unit.
ADR-0383 закрывает обязательную обратную reconciliation после Mail provider
write: returned provider entry ID/ETag закрепляются только Contacts-owned exact
command через NATS, workflow ждёт Contacts terminal result, а Mail, Kernel и
Gateway не получают права писать canonical Contacts link.
ADR-0384 разделяет retryable infrastructure outage и ambiguous provider write:
NATS outbox replay сохраняет exact bytes, `OUTCOME_UNKNOWN` запрещает
автоматический повтор provider mutation, recovery идёт через последующее
provider observation, causal cross-subject delivery ждёт prerequisites,
Contacts provenance refresh не создаёт feedback write, а
revoke/generation/grant fence срабатывает до IO. Managed gate реализован;
browser Start/Get/shared-SSE остаётся отдельным условием общего workflow gate.
ADR-0385 добавляет отдельный owner-proof Settings apply для managed workflow:
Kernel запускает fresh workflow successor с exact configuration-instance
catalog и не подменяет workflow integration launch/state/host-bridge semantics.
ADR-0386 добавляет owner-declared optionality в общий Settings protocol: Kernel
проверяет только безусловно обязательные values, а conditional provider
semantics остаются в integration runtime; это позволяет additive Mail schema
successor сохранить существующие account targets без provider heuristics в ядре.
ADR-0387 оставляет общий managed readiness timeout неизменным, но даёт Storage
launch bounded deadline по exact числу active bindings, потому что Storage до
ready обязан получить platform и per-runtime Vault credentials и применить
авторизованный topology workload.
ADR-0388 фиксирует live Storage fence reconciliation для development release
refresh: account-scoped Settings apply может опередить локальный assembly
checkpoint, поэтому `make dev` сверяет owner-authorized binding status и
продолжает successor только от live revisions.

ADR-0389 вводит честный `unconfigured` initial launch для workflows, которым
нужен configuration-instance target: Kernel не запускает child без snapshot,
а owner Settings apply остаётся единственным путём к готовому runtime.

ADR-0390 отделяет call transcription workflow от Communications text и generic
LLM: запись требует explicit consent и source-owned Blob custody, распознавание
принадлежит отдельному Speech-to-Text engine, а concrete Whisper execution —
отдельной integration. Transcript bytes доступны только через actor-bound
client Blob, не через PostgreSQL, durable events, query или SSE.

ADR-0391 фиксирует concrete Whisper STT provider как отдельную integration:
canonical transcript является private Blob document, whisper.cpp executable и
model приходят только как pinned managed artifacts, а process/runtime/storage и
assembly остаются отдельными build units без Communications или workflow
dependencies.

ADR-0392 делает original-write Blob custody transfer replay-stable: opaque
target reference выводится из подписанных content/evidence/source/target
semantics, но не из volatile proof time/signature/runtime-fence bytes. Live
authority продолжает проверяться на каждом transfer, а direct cross-owner read
и persistence custody proofs остаются запрещены.

ADR-0393 переводит terminal/progress status Communications evidence export с
frontend polling на существующий owner-authenticated shared Gateway SSE:
workflow сохраняет owner-local replay transitions, frontend использует один
общий hub, а query остаётся только initial/manual recovery snapshot.

ADR-0394 фиксирует первый desktop call recording producer как отдельную
integration с owner-local consent receipt и Blob custody. Tauri остаётся
visible OS-capture adapter за fenced host bridge, Kernel не интерпретирует
аудио/consent, а transcription получает только target-owned durable event.
