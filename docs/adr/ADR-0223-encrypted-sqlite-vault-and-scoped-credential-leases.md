# ADR-0223: Encrypted SQLite Vault и scoped credential leases

Статус: Принято
Дата: 2026-07-16
Состояние реализации: `vault_v1` открыт на exact five-package Vault owner
cut. Public `makosh-vault-protocol` проверяет bounded typed purpose,
ordered secret classes/actions, exact `configuration_instance` scope и TTL.
`makosh-vault-key-provider-file` создаёт или открывает только owner-private
0600 regular wrapping-key file без symlink traversal. Separate
`makosh-vault-runtime` создаёт SQLCipher database через raw key API с private
0700 directory/0600 file boundary. Unlocked store держит одну dedicated
bounded SQLite actor connection (queue 64, deadline 2 seconds), поэтому
record operations не открывают конкурирующие database connections. File wrapping key шифрует отдельный
случайный `VaultRootKey` в authenticated `vault.anchor`; explicit recovery slot
может завернуть тот же root key отдельным `VaultRecoveryKeyV1` через atomic
replace anchor без database rewrite. SQLCipher key
производится из root key через separate HKDF label. Conformance проверяет
reopen same key, wrong-key rejection, tampered-anchor rejection, отсутствие
plaintext marker и symlink rejection. Store также использует single-epoch
XChaCha20-Poly1305 record envelope с AAD exact owner/configuration/purpose/
class/revision и rejects scope substitution до decrypt. Owner-private Kernel IPC
уже binds/starts only the designated Vault artifact from the current verified
release, passes exact verified contracts through the inherited channel and
fences the typed ciphertext relay by current external session, capability,
binding revision, Kernel/Vault generation and grant epoch. Rebind stops the
prior managed Vault process. `RecoveryKeyV1` теперь имеет one-time checked
English 24-word BIP-39 entropy representation, без seed/PBKDF2/passphrase
semantics. Persistence adapter now exports an offline classified encrypted
snapshot through SQLite Backup API: only `vault.db`, encrypted `vault.anchor`
and a root-authenticated manifest are emitted, then re-opened and verified.
The adapter can also restore only into a new empty private contour after
recovery-key verification and binds a new platform wrapping slot. Это
classified Vault-component recovery, а не whole-instance restore: existing
contour replacement, component ordering и final generation fencing остаются
обязанностью `whole_instance_backup_v1`. Existing recovery slot теперь можно atomically rotate
only after current recovery-key unwrap; platform slot и encrypted database при
этом не меняются. Typed memory-only `CredentialLeaseV1`
fencing собран в private `VaultService`: до decrypt он требует exact one-time
lease audience и scope owner/configuration/purpose/class/revision. Этот core
ещё не доступен через IPC и не доказывает Kernel authorization. HPKE frame
sender contract доступен только через public `makosh-vault-protocol`, а ephemeral
receiver private key остаётся в private Vault runtime. `VaultTransportSessionV1`
держит opaque HPKE frame вместе с exact binding, а private bounded replay guard
отклоняет stale generation, wrong direction и повторный request ID только после
успешной HPKE authentication; generation и direction проверяются до decrypt.
Это ещё не public Vault service и не создаёт
secret-bearing socket. Explicit audience revoke
и runtime-generation advance немедленно инвалидируют unresolved service leases.
Store поддерживает one-time create с unique scope/revision и атомарную credential
replacement: только adjacent revision, authenticated prior record и delete/insert
в одной actor transaction.
Offline persistence уже поддерживает root rotation без in-place rekey: staged
SQLCipher copy переупаковывает record envelopes под новый root-derived key,
private reservation фиксирует SHA-256 DB/anchor до первой замены, а pending
rotation fail-closes normal open. Multi-slot anchor требует current Recovery Key
и не меняется при его отсутствии.
`Resolve` additionally requires the declared `VaultActionV1::Resolve`; a lease
for create or replacement cannot be repurposed as a secret-read lease.
`makosh-vault-runtime serve` открывает только initialized store и владеет private
`vault.sock`: он выдаёт ограниченный status с runtime generation и ephemeral
HPKE public key, но не публикует paths, keys или credential metadata. Runtime
protocol уже содержит bounded opaque `VaultCiphertextRouteV1`; Kernel сверяет
route с current external runtime identity/generation/grant epoch и relays
bounded opaque bytes только по inherited managed-runtime channel после
descriptor handshake. На этом же authenticated inherited channel применяется
отдельный typed `ManagedVaultRuntimeControlV1`: Kernel читает только `ready`,
exact persisted runtime generation и 32-byte ephemeral HPKE public key; mismatch
или error не создают ready state и останавливают новый child. Ciphertext остаётся
отдельным oneof-variant, поэтому status нельзя спутать с secret-bearing frame.
Vault runtime исполняет этот relay приватно: до decrypt он
проверяет binding, direction и replay, а response шифрует только для response
recipient из authenticated request binding. Control Store уже
имеет отдельный Kernel-owned platform-process binding/launch fence без owner
module registration; owner через private proof-bound control IPC принимает и
запускает только единственный Vault artifact из текущего verified signed release,
без client-supplied path/digest. Rebind останавливает prior child, а route после
registration revoke fail-closes. Public protocol уже
имеет fixed-major `ResolveLease`, `StoreLease` и `ReplaceLease` command codecs: lease ID остаётся
внутри HPKE plaintext, а SHA-256 digest exact encoded command проверяется against
binding после decrypt. Private session executor передаёт authenticated command
только в `VaultService`; тот получает audience только из binding и выполняет
lease-bound create, adjacent-revision replacement либо resolve exact
scope/class/revision без enumeration. Это
не является capability-route
evidence и не открывает IPC command surface.
Пока runtime исполняет только эти три actions, он fail-closed отклоняет lease
issuance для `Retire`, `Delete` и `IssueSessionStoreKey`; объявление action в
purpose не является implementation.
Binding фиксирует ephemeral X25519 public key получателя ответа в HPKE AAD;
private Vault runtime выдаёт результат только как отдельный `FromVault`
ciphertext frame для этого ключа. Kernel не получает plaintext и не может
подменить response recipient.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR уточняет перечисленные решения, но не заменяет их. Он определяет
отдельный Макошь Vault для credential material и не меняет ownership Kernel
Control Store, PostgreSQL, provider operational state, blobs или client/device
identity.

Vault packages/process не входят в `kernel_recovery_only_v1` и открываются
только `vault_v1` после `managed_launch_trust_v1` ADR-0225.

## Контекст

Макошь должен безопасно хранить:

- passwords и app passwords;
- API/client secrets;
- OAuth refresh credentials;
- provider auth keys и небольшие session credential blobs;
- ключи для больших integration-owned encrypted session stores.

Эти данные нужны independently restartable integration runtimes, но не являются
settings, business truth, event payload или общей PostgreSQL state. Kernel
обязан запускаться и сохранять recovery surface без Vault, а падение Vault не
должно останавливать modules, которым credentials не нужны.

Legacy `HostVault` в `references/backend-legacy/` является только evidence. В
нём были полезны отдельный SQLite store, root key вне database, explicit
`locked/unlocked` lifecycle, XChaCha20-Poly1305, случайный nonce, AAD и
zeroization. Переносить реализацию как шаблон нельзя:

- шифровались значения, но `secret_ref`, account, purpose, timestamps и
  manifest metadata оставались plaintext;
- root key напрямую хранился в platform key store;
- общий process мог читать secret по произвольному строковому reference;
- account/purpose/module identity, grant epoch и runtime generation не
  авторизовывались;
- recovery phrase фактически являлась экспортом root key;
- secret и manifest изменялись не одной transaction;
- большие provider session databases смешивались с небольшими credentials.

Новая система сохраняет идею локального Vault, но создаёт новый contract и
storage format без compatibility с legacy database.

## Решение

### Owner и process boundary

Vault имеет exclusive owner `platform/vault`. Он не является частью Kernel,
domain или integration:

~~~text
Kernel supervisor
    ↓ lifecycle, GrantSet context, fencing, opaque routing
makosh-vault-runtime                 отдельный managed OS process
    ├─ makosh-vault-store-sqlcipher
    └─ makosh-vault-key-provider-file
            ↓
authorized module runtime
    получает только process-bound CredentialLeaseV1
~~~

Kernel:

- owner-authorized bind/start фиксирует only designated Vault artifact current
  verified release; Kernel stage-ит exact verified descriptor/settings bytes в
  private one-shot files и передаёт их только `serve-inherited` child;
- запускает и bounded-перезапускает Vault; quiesce/drain ещё требует отдельной
  runtime-conformance evidence;
- проверяет managed executable по ADR-0219;
- вычисляет effective `GrantSet`; private external runtime route требует
  owner-approved `vault.lease.resolve` и relays only fenced versioned
  ciphertext frames в current Vault child;
- видит только sanitized state, generation и blocker code;
- не линкует Vault runtime, SQLCipher, crypto или file-key implementation;
- не получает `VaultRootKey`, record keys или credential plaintext.

Vault runtime:

- является единственным владельцем encrypted SQLite path и connections;
- unwrap-ит ключи, выполняет crypto/storage операции и выдаёт leases;
- не зависит от PostgreSQL, PgBouncer, NATS, provider SDK или module packages;
- не является generic KV/database service;
- не интерпретирует Mail, Telegram, Zulip, WhatsApp или domain semantics.

Module runtime:

- зависит только от public `makosh-vault-protocol`;
- не получает SQLite path, SQL, key slots, root key или enumeration API;
- получает material только в рамках approved purpose и process-bound lease;
- хранит plaintext в памяти минимально необходимое время и zeroize-ит его при
  stop/revoke/expiry настолько, насколько это поддерживает provider SDK.

Первая реализация Vault является только bundled `managed` process. External
Vault registration и альтернативная implementation/topology запрещены.

### Cargo packages

Первая package topology фиксирована:

~~~text
backend/src/platform/vault/protocol/
    makosh-vault-protocol
    platform:vault:contract

backend/src/platform/vault/managed_client/
    makosh-managed-vault-client
    platform:vault:contract

backend/src/platform/vault/key_provider/
    makosh-vault-key-provider
    platform:vault:contract

backend/src/platform/vault/runtime/
    makosh-vault-runtime
    platform:vault:runtime
    component: vault_service

backend/src/platform/vault/store_sqlcipher/
    makosh-vault-store-sqlcipher
    platform:vault:persistence

backend/src/platform/vault/key_provider_file/
    makosh-vault-key-provider-file
    platform:vault:implementation
~~~

`makosh-vault-key-provider` является внутренним adapter port владельца Vault.
Kernel, modules и Gateway не зависят от него. Новый Vault package или platform
adapter требует изменения этого ADR и executable policy.

`makosh-managed-vault-client` является единственным публичным contract для
managed module runtime, которому требуется provider credential. Он принимает
только Kernel-inherited authenticated local FD и строит HPKE ciphertext frames
для scoped lease request; он не открывает Vault store, не получает root/wrapping
keys, не импортирует Vault runtime или key-provider и не создаёт alternate Vault
transport. Его dependency разрешена только runtime packages с действующим
owner-approved grant; Kernel и Gateway не зависят от него. Остальные Vault
packages остаются закрытой owner implementation boundary.

### Threat boundary

Первая версия защищает от:

- offline theft Vault database, journal, temporary migration files или backup;
- чтения account/provider metadata из plaintext SQLite;
- соседнего module без grant либо с другим account/purpose;
- replay stale lease после restart, revoke или epoch change;
- случайного раскрытия plaintext внутри Kernel routing/logging;
- потери platform wrapping key при наличии recovery package и Recovery Key.

Первая версия не обещает защиту от:

- полного compromise Vault process, пока он unlocked;
- host root/administrator или полного compromise owner OS account;
- malicious runtime, который уже получил plaintext и скопировал его;
- provider-side compromise после успешной authentication.

Lease ограничивает выдачу и lifetime авторизации, но не может отозвать bytes,
уже скопированные чужим process. Revoke поэтому включает fencing и
quiesce/stop затронутого runtime, а provider credential при необходимости
отзывается или rotates у внешнего provider.

### Encrypted SQLite profile

Vault использует SQLCipher full-database encryption и отдельный
XChaCha20-Poly1305 envelope для credential payload.

SQLCipher скрывает schema, indexes и metadata at rest, а record envelope:

- связывает ciphertext с exact owner/configuration/purpose/class/revision;
- не позволяет переставить ciphertext между logical records;
- даёт независимый record/key epoch и controlled crypto-suite migration.

Initial SQLite profile:

~~~text
journal_mode = DELETE
synchronous = FULL
foreign_keys = ON
trusted_schema = OFF
temp_store = MEMORY
extension loading = disabled
ATTACH = forbidden
single writer actor
~~~

Правила:

- local parent directory имеет mode `0700`, files `0600`;
- SQLCipher key передаётся через raw-key API, а не interpolated SQL;
- SQLite connection принадлежит одной dedicated blocking thread/actor;
- requests typed, bounded и имеют deadline;
- mutation выполняется одной короткой transaction;
- raw SQL, row types и file paths не пересекают persistence boundary;
- unknown cipher/schema/record major version fails closed;
- schema и embedded migrations принадлежат
  `makosh-vault-store-sqlcipher`;
- migration возможна только после успешного unlock;
- destructive downgrade, plaintext export и automatic fallback запрещены.

WAL не включается в первой версии: low-write Vault не получает достаточного
выигрыша, чтобы оправдать отдельный checkpoint/sidecar lifecycle. Изменение
journal mode требует crash, plaintext-leak и backup conformance tests.

[SQLCipher](https://www.zetetic.net/sqlcipher/design/) используется как
page-encryption implementation boundary, а не как generic database dependency
Kernel или modules.

### Key hierarchy

~~~text
PlatformWrappingKey                 RecoveryKeyV1
owner-private file adapter          хранится владельцем вне Макошь
          │                              │
          └──── authenticated KeySlotV1 ─┘
                            ↓
                     VaultRootKey
                 ┌──────────┴──────────┐
          SQLCipher key          record-domain keys
~~~

- `VaultRootKey` — 32 random bytes из OS CSPRNG.
- `PlatformWrappingKey` — отдельные 32 random bytes в owner-private regular
  file `0600`, которым владеет отдельный `FileWrappingKeyAdapter` Vault.
- Owner/device ES256 signing key ADR-0218 не используется для Vault encryption
  или wrapping.
- `RecoveryKeyV1` — независимые 32 random bytes; Макошь показывает их один раз
  и не сохраняет plaintext.
- Human representation `RecoveryKeyV1` — 24-word BIP-39 entropy encoding с
  checksum и English word list. Используется только entropy-to-mnemonic
  encoding; BIP-39 seed/PBKDF2/passphrase semantics не используются. Формат
  следует только части entropy/mnemonic стандарта
  [BIP-39](https://bips.dev/39/).
- User-created password/passphrase unlock в V1 отсутствует. Его появление
  потребует отдельного Argon2id contract.
- Wrapping suite V1 — XChaCha20-Poly1305 с random 24-byte nonce.
- SQLCipher key и record-domain keys выводятся через HKDF-SHA-256 с
  `VaultInstanceId`, distinct fixed info labels и key epoch.
- Credential record использует unique random nonce; AAD включает
  `VaultInstanceId`, record ID, logical owner, opaque configuration instance,
  purpose, class, revision, suite и key epoch.
- Root/key plaintext существует только в unlocked Vault memory, не
  сериализуется и zeroize-ится при lock/stop/failure.

`vault.anchor` содержит только version, `VaultInstanceId` и authenticated
`KeySlotV1` records. Он принадлежит Vault, а не Kernel Control Store. Key slot
содержит kind, suite, key epoch, nonce и wrapped root key, но не credential
metadata.

Platform wrapping slot и recovery slot позволяют:

- менять wrapping-key adapter/slot без database rewrite;
- восстанавливать Vault на новом устройстве;
- менять recovery key без изменения credential records;
- не делать recovery phrase равной root key.

### State machine и unlock policy

Vault использует явные состояния:

~~~text
uninitialized
    → sealed → unlocking → unlocked
                    └────→ recovery_required

unlocked ↔ rotating
unlocked → quiescing → stopped
sealed / recovery_required → stopped
~~~

Unlock modes:

- `file_adapter_auto` — baseline unlock через отдельный owner-private
  file-backed wrapping-key adapter;
- `manual_local` — local interactive unlock через configured file adapter;
- `recovery_offline` — stopped Kernel/Vault, exclusive lock и
  `RecoveryKeyV1`.

Initialize, recovery export/import, root rotation и изменение recovery slot
всегда требуют fresh operation-bound owner proof. File-backed baseline не
требует отдельной interactive confirmation ceremony.

`file_adapter_auto` является non-secret Vault-owned operator setting. Она
применяется только при trustworthy Control Store. Unavailable/untrusted
Control Store запрещает online unlock и lease issuance независимо от
сохранённого desired mode.

Startup:

1. Kernel достигает `recovery_only` без Vault.
2. При trustworthy Control Store supervisor проверяет exact Vault executable.
3. Vault process запускается в `sealed`.
4. Разрешённый unlock unwrap-ит `VaultRootKey`.
5. Vault открывает SQLCipher и проверяет cipher/schema/integrity.
6. Создаётся новая `vault_runtime_generation`.
7. Только затем capability становится ready и выдаёт leases.

Vault failure блокирует только methods/capabilities с Vault dependency. Kernel,
Control Store, recovery surface и modules без credential dependency продолжают
работать.

### Secret record и metadata privacy

Logical record:

~~~text
SecretRecordV1
  secret_record_id
  logical_owner_id
  opaque_configuration_instance_id
  vault_purpose_id
  secret_class
  secret_revision
  key_epoch
  state
  not_before?
  expires_at?
  bounded encrypted payload
~~~

Closed `secret_class` V1:

- `provider_credential`;
- `oauth_refresh_credential`;
- `session_credential_blob`;
- `platform_credential`;
- `session_store_key`.

Record states:

- `active`;
- `retiring`;
- `revoked`;
- `tombstoned`.

Vault не хранит email, phone, username, provider account ID, display label,
provider URL или arbitrary JSON metadata. Integration owner хранит связь
своего account с opaque `configuration_instance_id` и purpose. Replacement
registration не наследует credential binding или grants автоматически.

Generic `ListSecrets`, `GetSecret(secret_ref)` и client read-back отсутствуют.
Owner provisioning является write-only: после записи client видит только
`configured`, revision, expiry/rotation status и sanitized blocker.

Health, logs, errors, SSE и NATS не содержат record ID, purpose, account
metadata, counts по providers, ciphertext/plaintext length или database path.

### Разделение credentials и provider session state

Vault хранит:

- passwords, app passwords, API/client secrets;
- OAuth refresh credentials;
- private auth keys/cookies, если они являются небольшим credential material;
- небольшой opaque session blob, достаточный для impersonation;
- wrapping keys для integration-owned encrypted session stores.

Vault не хранит:

- messages, contacts, attachments, documents, media или prompts;
- settings, cursors, checkpoints, pts/qts/seq, mailbox position;
- retry/reconciliation state, outbox/inbox или jobs;
- provider operational projections;
- большие/high-churn TDLib/provider session databases;
- экспортированные cookies/local storage hidden WhatsApp WebView.

Limits V1:

- обычный credential payload — не более 64 KiB;
- `session_credential_blob` — не более 4 MiB;
- больший или часто изменяемый state остаётся в private integration store;
- Vault выдаёт ему только `SessionStoreKeyLease`.

WhatsApp hidden WebView использует отдельный OS-managed per-account profile и
не экспортирует cookies/storage в generic Vault. Mail passwords/OAuth refresh
credentials, Telegram auth material и Zulip API key следуют тому же общему
Vault contract без provider-specific API внутри Vault.

PostgreSQL/PgBouncer bootstrap, admin и runtime credentials являются
`platform_credential` ADR-0224. Только Storage Control получает narrow
`create`/`replace_cas`/`retire` actions и exact-purpose `resolve` для bounded
bootstrap/role provisioning; plaintext остаётся в его memory только до
передачи PostgreSQL tool/connection и zeroize. Module runtime получает другой
scoped `resolve` для exact `StorageBindingV1`. Vault не знает SQL, schemas,
table names или PgBouncer admin model и не является database session revocation
service.

### Descriptor request и authorization

`ModuleDescriptorV1` может запросить:

~~~text
VaultPurposeRequestV1
  purpose_id
  allowed_secret_class[]
  actions[]
  target_scope
  requested_lease_ttl
~~~

Текущий descriptor parser уже fail-closed проверяет этот request: purpose
является bounded identifier, `allowed_secret_class` и `actions` не пусты,
строго упорядочены без duplicate и содержат только известные V1 enum values;
`target_scope` ровно `configuration_instance`, а requested TTL находится в
диапазоне `1..=3600` секунд. Эта проверка не выдаёт Vault capability и не
заменяет authorization/runtime/lease checks будущего Vault process.

Actions V1:

- `resolve`;
- `create`;
- `replace_cas`;
- `retire`;
- `delete`;
- `issue_session_store_key`.

Purpose ID является stable bounded identifier владельца contract. Wildcard
owner/resource, arbitrary provider account label, email/phone и secret
reference в descriptor запрещены.

Vault operation разрешена только как пересечение:

~~~text
VaultPurposeRequestV1
∩ owner-approved GrantSet
∩ hard Kernel/Vault policy
∩ current runtime session and generation
~~~

Vault повторно проверяет signed authorization context перед decrypt/unwrap.
`pending`, `suspended`, `revoked`, wrong audience/purpose и stale epoch
отклоняются до чтения credential payload.

### CredentialLeaseV1

~~~text
CredentialLeaseV1
  lease_id
  vault_instance_id
  vault_runtime_generation
  secret_revision
  logical_owner_id
  configuration_instance_id
  purpose_id
  actions
  audience_module_registration_id
  audience_runtime_instance_id
  grant_epoch
  issued_at
  expires_at
  single_resolve
  sealed_material
~~~

Для storage `configuration_instance_id`, purpose и `secret_revision` связаны с
exact current `StorageBindingV1`; смена storage/runtime/grant/role generation
создаёт новую binding/credential revision. Vault сверяет exact opaque binding,
audience, purpose, revision и grant epoch до delivery, а Storage Control
отдельно проверяет storage/role generations и завершает session fencing
ADR-0224. Vault не интерпретирует database semantics.

Initial policy:

- default TTL — 10 минут;
- hard maximum — 1 час;
- lease и resolved material не сохраняются в SQLite;
- `Resolve` выполняется не более одного раза;
- renewal создаёт новый lease и заново проверяет current `GrantSet`;
- Vault restart/lock/restore или generation change инвалидирует все leases;
- module restart, suspend/revoke или grant epoch change инвалидирует его
  leases;
- revoke закрывает transport session и блокирует renewal;
- истечение/revoke Vault lease запрещает дальнейший resolve/renewal, но не
  считается отзывом уже открытой SQL session; полный role/PgBouncer/PostgreSQL
  session fencing подтверждает Storage Control ADR-0224;
- secret material никогда не проходит через NATS, durable events, SSE,
  settings, Control Store, argv, environment, logs или filesystem spool.

### Secret-bearing transport

`VaultTransportSessionV1` использует standard HPKE
[RFC 9180](https://www.rfc-editor.org/rfc/rfc9180.html) suite:

~~~text
DHKEM(X25519, HKDF-SHA256)
HKDF-SHA256
ChaCha20Poly1305
~~~

HPKE context `info` и AAD связывают frame с:

- `vault_runtime_generation`;
- authenticated owner device session или `ModuleRegistrationId`;
- `runtime_instance_id`;
- `grant_epoch`;
- request ID;
- operation digest;
- direction и protocol major.

Kernel/Gateway авторизует и маршрутизирует HPKE ciphertext, но не имеет
recipient private key и не видит plaintext. Module-to-module socket не
появляется: transport остаётся частью versioned capability routing.

Vault transport keypair ephemeral и привязан к одной runtime generation.
Replay, wrong direction/context, unknown suite/major и malformed ciphertext
fail closed до credential mutation или delivery.

Owner provisioning использует тот же sealed payload boundary. Tauri/Android
host adapter выполняет HPKE operation; browser business API не получает Vault
root/platform keys и не имеет generic secret read method.

### Mutations и rotation

Public Vault control/data operations:

~~~text
Status
Initialize
Unlock
Lock
PutCredential
ReplaceCredential(expected_revision)
RetireCredential
DeleteCredential
IssueLease
RenewLease
RevokeLease
CreateBackup
RotatePlatformSlot
RotateRecoverySlot
RotateRootKey
Quiesce
Drain
Stop
~~~

- `Initialize` разрешён только pristine Vault instance с fresh owner proof.
- Module write требует отдельного scoped action; resolve grant не даёт write.
- `ReplaceCredential` использует compare-and-swap по revision.
- Provider credential rotation выполняет integration/provider workflow, а
  Vault только атомарно хранит versions.
- Old/new credential overlap явный, bounded и не включается автоматически.
- `DeleteCredential` создаёт tombstone; physical secure erase SQLite page,
  filesystem snapshot или старого backup не обещается.

Три независимых rotation:

1. credential revision через CAS и explicit retire;
2. platform/recovery wrapping slot без database rewrite;
3. Vault root/SQLCipher/record keys через explicit
   `quiesce → encrypted copy/rekey → verify → atomic swap`.

Automatic rollback/fallback после rotation запрещён. Failed rotation сохраняет
однозначно старый либо проверенный новый generation.

### Backup и recovery

Backup:

- выполняется только unlocked Vault;
- требует fresh owner proof и platform user presence;
- bounded-quiesce-ит writes;
- использует SQLCipher-compatible SQLite backup/export, а не `cp` открытого
  файла;
- включает encrypted database snapshot, `vault.anchor`, schema/cipher/key
  epochs и authenticated manifest;
- не включает `RecoveryKeyV1`;
- проверяется пробным unwrap/open/integrity check до публикации.

SQLite Online Backup API создаёт consistent snapshot работающей database:
[SQLite Backup API](https://www.sqlite.org/backup.html).

Restore:

- выполняется только offline при остановленных Kernel и Vault;
- требует explicit `--data-dir`, exclusive instance lock, local interactive
  confirmation и `RecoveryKeyV1`;
- проверяет package/anchor/DB integrity до mutation;
- создаёт новый platform wrapping slot;
- повышает Vault generation и инвалидирует все leases;
- не восстанавливает Kernel grants, OwnerAuthority или device identity;
- не rebind-ит replacement registration к старым secrets автоматически.

Recovery Key даёт только decrypt authority конкретного Vault backup. Он не
является OwnerAuthority, client session или module grant.

Wrong recovery key, missing wrapping-key file, corruption либо incompatible version
никогда не создают empty Vault и не перезаписывают working key slot.
Состояние становится `sealed` или `recovery_required` до explicit action.

### Failure, privacy и observability

- Process exit допускает bounded restart по ADR-0203.
- Restart создаёт новый runtime generation и не сохраняет leases.
- `sealed` не является crash и не запускает restart loop.
- Integrity/cipher/schema failure становится `recovery_required` без
  automatic init/reset/restore.
- Core dumps Vault отключены; memory locking используется where supported.
- Secret buffers используют non-clone secret types и best-effort zeroization.
- Public errors typed и redacted; raw crypto/SQL/file-key errors остаются
  внутри bounded mapping без values/paths/account metadata.
- Telemetry содержит state transition, generation, duration и reason code, но
  не secret IDs, purpose, payload length или account/provider identity.
- Automated tests используют только synthetic marker bytes и platform test
  adapters; live provider credentials запрещены.

## Отклонённые варианты

### Хранить credentials в Kernel Control Store

Отклонено: создаёт boot cycle, расширяет compromise Kernel и смешивает
technical control state с secret material.

### Хранить credentials в PostgreSQL

Отклонено: Vault должен работать независимо от PostgreSQL, а module roles,
backup и query surfaces расширяют область доступа.

### Встроить Vault implementation в Kernel process

Отклонено: root key/plaintext попадают в общий failure/compromise domain и
исчезает independently restartable boundary.

### Plain SQLite только с ciphertext value columns

Отклонено: schema, manifest, account/purpose metadata, journal и access pattern
артефакты остаются шире необходимого. Full-database encryption является
основным at-rest boundary.

### Только SQLCipher без record envelope

Отклонено: page encryption не задаёт typed AAD, record revision/key epoch и
защиту от semantic ciphertext swapping внутри storage implementation.

### Общий `read_secret(secret_ref)`

Отклонено: строковый reference не доказывает module, account, purpose, epoch
или process audience и создаёт enumeration confused-deputy API.

### Recovery phrase равна Vault root key

Отклонено: root нельзя независимо rotate/re-wrap, а leak recovery value сразу
становится прямым DB key material.

### Один Vault blob для любой provider session

Отклонено: high-churn databases превращают Vault в generic blob/session store,
увеличивают write contention и связывают provider lifecycle с secret DB.

### User-created password как default unlock

Отклонено в V1: random platform/recovery keys сильнее и не требуют выбора KDF
параметров. Password-based portable profile требует отдельного решения.

### Silent fallback между key adapters

Отклонено: первый release использует только explicit
`FileWrappingKeyAdapter`; adapter failure даёт `sealed` и не разрешает
automatic fallback, regeneration или reinitialization. Замена adapter требует
явной migration/re-wrap процедуры и conformance tests.

## Проверка решения

До изменения `Состояние реализации` обязательны:

- Kernel boot/recovery без Vault;
- Vault является отдельным verified managed process;
- Vault crash не завершает Kernel или независимые modules;
- restart создаёт новую generation и инвалидирует leases;
- `pending`/`suspended`/`revoked`/stale epoch denied до decrypt;
- wrong registration/runtime/account-purpose audience отклоняется;
- duplicate/replayed `Resolve` отклоняется;
- TTL/renew/revoke и hard maximum проверяются fake clock;
- plaintext markers отсутствуют в DB, journal, temp migration и backup bytes;
- tampered anchor, key slot, SQLCipher page, record nonce/AAD/ciphertext fail
  closed без overwrite;
- wrong Recovery Key не меняет file key slot, anchor или database;
- missing file key даёт `sealed`, не auto-initialize;
- recovery restore проходит на новом host adapter и повышает generation;
- fault injection для put/replace/delete/migration/backup/rekey/atomic swap;
- concurrent requests проходят single-writer bounded queue без partial state;
- CAS conflict не теряет active credential revision;
- key rotation поддерживает mixed record epochs только в declared migration
  window;
- revoke quiesce/stop-ит affected runtime и не обещает remote side-effect undo;
- Control Store не содержит keys, slots, secret IDs/bindings или leases;
- settings не содержат secret values/references/bindings;
- events/NATS/SSE/logs/errors/health/crash reports не содержат secret/private
  metadata;
- Kernel и modules не зависят от Vault runtime/store/key-provider adapter;
- Vault не зависит от PostgreSQL/NATS/provider/module packages;
- release build использует только declared `FileWrappingKeyAdapter`;
- automated suite не использует live accounts.

Static architecture policy доказывает package ownership, dependency direction,
forbidden carriers и declared lifecycle/lease invariants. SQLCipher, crypto,
file-key, IPC, memory, crash, backup и recovery guarantees доказываются только
production conformance/integration tests.

## Последствия

Положительные:

- Kernel остаётся bootable и diagnosable без Vault;
- compromise или crash integration не открывает credentials соседних modules;
- metadata и secret values защищены at rest;
- restart/revoke имеют typed generation/epoch fencing;
- realtime integrations получают automatic desktop unlock без постоянных UI
  prompts;
- recovery не экспортирует root key и отделено от OwnerAuthority;
- Telegram/Mail/Zulip и будущие integrations используют один capability
  contract без общего provider abstraction.

Стоимость:

- появляется отдельный managed process, encrypted SQLite format и key hierarchy;
- нужны file-key adapter и filesystem-conformance tests;
- dual SQLCipher + record AEAD требует crypto conformance и rotation tooling;
- HPKE transport и sealed provisioning усложняют IPC;
- большие provider session stores требуют собственного encryption lifecycle;
- physical secure erase одной записи или старого backup не обещается;
- будущий adapter не считается supported до отдельной реализации, explicit
  re-wrap migration и conformance.
