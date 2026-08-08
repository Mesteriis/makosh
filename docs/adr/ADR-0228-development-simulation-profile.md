# ADR-0228: Development simulation profile

Статус: Принято
Дата: 2026-07-16
Состояние реализации: executable policy фиксирует full-platform development
contract; отдельный `makosh-development-kernel-operator` выполняет bounded
native TCP probe явно переданных development PostgreSQL/NATS endpoints и
one-shot TLS pairing simulation с file-backed ES256 proof. Compose contour даёт
real service smoke внутри private network. Runtime adapters и services ещё
реализуются последовательно по milestones; production platform adapters и их
gates остаются закрыты.

Зависит от:

- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0225: Первый production slice и phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).

## Контекст

Разработка использует тот же file-backed signer adapter, что и первый baseline
release; OCI registry и release signing environment по-прежнему не требуются
для development contour.

## Решение

Добавляется единственный непроизводственный contract
`development_full_platform_v1`. Он выбирается только явным development
invocation и предназначен для полного local platform development: Kernel,
Gateway, Vault, Storage Control, NATS, Blob, Clock, Scheduler, client flows и
remote-pairing protocol могут работать с development adapters и local services.

В этом profile разрешён file-backed ES256 signer c owner-private filesystem
storage, persistent development data/credentials, network listeners, remote
pairing, Vault и external services. Он может симулировать оба release target:
`macos_tauri_embedded_v1` и `linux_docker_server_v1`. Каждый запуск обязан
явно показывать insecure warning.

Development profile не является третьим deployment profile и не изменяет
release matrix ADR-0227. Он не выпускает release artifacts и не позволяет
использовать результаты simulation как production gate evidence. В нём должны
использоваться только development credentials и data: реальные owner keys,
production provider credentials и private user content не копируются в этот
контур.

Development network/services adapter обязан fail closed вне explicit development
invocation. File-backed signer adapter является отдельной replaceable boundary
и не является automatic downgrade для будущего adapter failure.

Первый development runtime — отдельный Cargo package с role `development`,
owner `development`, surface `runtime`; это не Kernel component и не получает
Docker socket. Он выполняет bounded TCP reachability probe для явно заданных
PostgreSQL и NATS addresses, а также remote-pairing simulation: ephemeral TLS
certificate, printed SHA-256 fingerprint, bounded listener, 256-bit bearer,
fresh challenge и file-backed ES256 proof. После one-shot consume runtime
создаёт owner-private receipt; только explicit Kernel development CLI повторно
проверяет его signature/hash и записывает initial owner в Control Store.
Compose service health проверяется отдельно из Compose network; эти checks не
являются Storage Control или Event Hub readiness attestation. Listener не
получает прямой SQLite access и не является production Kernel enrollment
endpoint.

Development Control Store также хранит `ExternalRuntimeAttestation`: она
привязана к approved module registration, exact distribution SHA-256, runtime
generation и текущему grant epoch. Повтор или откат generation отклоняется;
approval, suspension или revoke делают прежнюю attestation ineffective. Это
не Docker identity и не production service attestation: development CLI сама
читает explicitly selected absolute regular artifact, rejects symlink/path
replacement during digesting и сохраняет observed SHA-256. Она никогда не
сигналит контейнеры и не authorizes managed launch.

Для experimentation с owner-pinned managed artifact development CLI отдельно
сохраняет versioned `OwnerPinnedArtifactBinding`: canonical artifact path,
SHA-256, size, observed device/inode и ES256 signature current file signer,
domain-bound к instance и registration. Повторный preflight повторно читает
этот exact file и отвергает changed bytes или identity. Изменение bytes требует
новой explicit binding revision; никакой binding не является process launch,
supervision или доказательством `managed_launch_trust_v1`.

## Последствия

Разработчик может строить и тестировать весь local platform flow через тот же
file-backed signer contract. Этот baseline не доказывает release packaging,
network exposure или later adapter conformance.
