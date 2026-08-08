# ADR-0300: Loopback full-stack development assembly

Статус: Принято
Дата: 2026-07-27
Состояние реализации: реализовано для loopback development profile. Root
lifecycle, locally signed release, platform foundation, distinct
Communications/Attachment Security/integration admission, active
Storage/Event/Vault bindings, replayable Gateway SSE source, Vite same-origin
proxy для generated RPC, SSE и descriptor-declared `client_blob`, readiness
barrier и browser open подтверждены executable и live evidence. Provider
runtime с обязательными Settings намеренно остаётся
`unconfigured`, пока владелец не создаст учётную запись по ADR-0302; это
fail-closed состояние полного ensemble, а не отсутствие assembly unit.

Уточняет:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0228: Development simulation profile](ADR-0228-development-simulation-profile.md);
- [ADR-0232: Browser client identity and same-origin Gateway session](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0237: Временный private-LAN development без owner authority](ADR-0237-temporary-private-lan-development-without-owner-authority.md).
- [ADR-0301: Bundled module discovery и owner-authorized development admission](ADR-0301-bundled-module-discovery-and-development-admission.md).

## Контекст

Поддерживаемый root command surface уже объявляет `make dev`, но прежняя цель
только запускала Compose, Kernel и Vite как два background child process.
Она не:

- открывала browser Gateway;
- соединяла same-origin generated clients с Gateway;
- ждала readiness;
- открывала приложение в браузере;
- корректно работала с system Bash 3.2 на macOS: `wait -n` отсутствует, поэтому
  Vite мог завершиться, а wrapper и Kernel оставались висеть;
- подготавливала отдельную development owner identity для pristine
  `.local/kernel-dev`.

ADR-0237 намеренно оставляет private-LAN plaintext listener technical-only.
Расширять его до owner authority или использовать LAN address как proof
владельца запрещено. Обычный production browser profile требует HTTPS,
WebAuthn и signed application bundle, поэтому он также не является быстрым
Vite development contour.

Нужна отдельная assembly boundary. Она не является Kernel, domain,
integration, workflow или release assembly и не меняет production admission.

## Решение

Вводится непроизводственный contract:

```text
loopback_full_stack_dev_assembly_v1
```

Его единственный публичный entrypoint:

```sh
make dev
```

Root `Makefile` только делегирует этот contract backend-owned development
assembly. Runtime orchestration находится в одном script unit под
`backend/scripts/`; Makefile не дублирует lifecycle logic.

### Exact topology

```text
make dev
  -> development Compose
       -> PostgreSQL
       -> PgBouncer
       -> NATS
  -> makosh-kernel serve
       -> Core Gateway at 127.0.0.1:9444
  -> Vite at 127.0.0.1:5173
       -> exact same-origin Gateway proxy
  -> readiness barrier
  -> system browser at http://127.0.0.1:5173/
```

PostgreSQL, NATS и provider runtimes не становятся frontend dependencies.
Browser продолжает обращаться только к Core Gateway через generated contracts.
Vite является development delivery/proxy adapter, а не business API или
production reverse proxy.

`make dev` поднимает инфраструктурные зависимости, Kernel/Gateway и клиентский
delivery contour. Полный development ensemble дополнительно materialize-ит
отдельный signed local distribution и через ADR-0301 выполняет generic
owner-authorized admission/start platform, domain и integration units. Он не
создаёт provider credentials, не подменяет отсутствующие registrations и не
имитирует available provider surface. UI получает фактическую capability
availability из Gateway bootstrap.

### Loopback development proxy proof

Development assembly создаёт новый 256-bit random proof для каждого запуска:

- proof хранится только в owner-private `0600` temporary regular file;
- в argv и environment передаётся только путь к файлу;
- proof не попадает в URL, browser JavaScript, Vite bundle, logs, diagnostics,
  Control Store, backup или provider state;
- Kernel и Vite читают exact file до serving;
- Vite удаляет любой одноимённый browser header и добавляет proof только на
  server-side proxy hop;
- Gateway принимает development owner routes только при одновременном
  совпадении proof, exact `Host`, допустимого `Origin` и отсутствии forwarding
  headers;
- Gateway listener и Vite listener bind только literal loopback addresses;
- direct request к Gateway без proof не получает owner authority;
- proof удаляется при shutdown, а restart всегда меняет его.

Это process-local development capability, а не owner/device identity,
production session, durable grant или LAN trust. Обычные local/paired browser
profiles не читают proof file и сохраняют WebAuthn/cookie boundary.

Gateway session contract возвращает отдельный closed enum
`LOCAL_DEVELOPMENT`. Он не называется `LAN_DEVELOPMENT`, потому что
private-LAN technical profile ADR-0237 не выдаёт owner session.

### Development owner bootstrap

Default data directory остаётся:

```text
<repository>/.local/kernel-dev
```

Kernel `status` публикует только sanitized states:

```text
owner_identity=missing|enrolled|unavailable
owner_device_signer=missing|ready|mismatch|unavailable
```

Для pristine default development instance assembly:

1. idempotently создаёт file-backed ES256 development device key;
2. выполняет initial enrollment с закрытыми IDs `development-owner` и
   `development-desktop`;
3. никогда не заменяет существующего owner или key.

Если owner уже enrolled, но signer missing/mismatch, запуск fail closed с
sanitized remediation. Ни Control Store reset, ни удаление data, ни повторная
enrollment автоматически не выполняются.

### Lifecycle и readiness

Assembly обязан:

- поддерживать macOS system Bash 3.2 и не использовать `wait -n`;
- проверять обязательные команды и занятость exact ports до запуска child
  processes;
- передавать SIGINT/SIGTERM обоим child processes;
- завершать sibling process, если Kernel или Vite неожиданно завершился;
- оставлять Compose services запущенными для последующих development cycles;
- дождаться direct Gateway `/readyz` с proxy proof;
- дождаться `/readyz` через Vite same-origin proxy;
- открыть browser только после обоих успешных checks;
- иметь bounded startup deadline и выводить sanitized component failure.

Accepted process может оставаться foreground: это сохраняет live logs и один
явный lifecycle owner.

## Проверка

Gate `loopback_full_stack_dev_assembly_v1` открывается только при наличии:

1. architecture test на единственный root delegation и assembly ownership;
2. Kernel tests на loopback-only configuration, missing/wrong proof, wrong
   Host/Origin, forwarding headers и unchanged paired/LAN profiles;
3. session/protocol tests на exact `LOCAL_DEVELOPMENT` enum;
4. frontend tests на exact proxy allowlist и отсутствие proof в client bundle;
5. shell syntax/contract test без `wait -n`, wildcard bind и secret argv;
6. live `make dev` evidence: Compose healthy, Gateway ready, Vite ready,
   same-origin session status доступен и browser page загружена;
7. cleanup evidence: interrupt закрывает Kernel/Vite, proof file удалён,
   Compose остаётся healthy.

Наличие ADR или unit tests без live root command не закрывает gate.

Фактическое evidence от 2026-07-27:

- architecture policy/SRP/Cargo boundaries прошли, все 536 architecture tests
  успешны;
- targeted Gateway recovery suite: 34 tests успешны;
- frontend lint, typecheck, clean-room boundary, 120 test files/276 tests и
  production build успешны;
- live root `make dev` подтвердил healthy Compose, direct и same-origin
  readiness, session status, client bootstrap, descriptor-declared
  `/api/blobs/` delivery и browser page без console errors;
- interrupt завершил Kernel/Vite и освободил оба listener, удалил per-run proof,
  оставив Compose healthy; повторный запуск использовал существующего enrolled
  development owner и снова достиг readiness.

## Последствия

- Разработчик получает один повторяемый foreground command и видит текущий UI
  в browser.
- Development proxy proof ограничивает bypass exact local assembly hop; он не
  заменяет production WebAuthn.
- Private-LAN developer listener ADR-0237 остаётся technical-only.
- Assembly не становится domain, integration или production release unit.
- Незарегистрированные либо неготовые modules честно остаются unavailable в UI.
