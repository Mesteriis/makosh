# ADR-0232: Browser client identity and same-origin Gateway session

Статус: Принято
Дата: 2026-07-18
Состояние реализации: частично реализован private foundation: Control Store
registry/revoke fence, single-use epoch-fenced pairing state, Gateway-memory
session core и WebAuthn verifier с exact HTTPS origin/RP и ES256/UV.
Backup-eligible credential допускается только вместе с отдельным
browser-local proof по ADR-0234. Authentication ceremony остаётся server-held,
bounded и short-lived; caller-selected credential ID сначала резолвится как
active record через Kernel authority, затем снова fenced при assertion
persistence. Exact `makosh-gateway-session-contract` отделяет
browser authority от Gateway implementation; Kernel реализует его через
private Control Store adapter. Pairing state уже хранит server-side WebAuthn
ceremony и удаляется только после успешного verifier + persistence callback.
Verified credential materializes только в typed `BrowserEnrollmentV1`, который
несёт captured identity fence pairing; Kernel adapter повторно сверяет current
owner/fence и atomically записывает browser device в Control Store.
Verified assertion atomically advances the persisted device counter under the
current identity-epoch fence; a non-zero counter cannot repeat or regress,
while a zero-counter authenticator remains valid only while it reports zero.
Это решение включает browser client в scope `client_gateway_v1`, но не
открывает первый business owner. Частичный Gateway transport foundation
существует отдельным `makosh-gateway-runtime`: он fail-closed валидирует
local/paired-remote profiles, запрещает remote plaintext и HTTP/3 early data,
обслуживает paired TCP/TLS peer по HTTP/2 и HTTP/3 peer через same Gateway
router. Kernel-managed listener lifecycle имеет explicit opt-in local profile
и отдельный explicit paired-remote CLI profile; remote profile принимает
operator-supplied DER identity, не выпускает certificate автоматически и
поднимает HTTP/2 и H3 на одном address/port. Detached
`BrowserRealtimeRouter` также принимает только exact `GET
/api/realtime/v1/events`: он authorizes same-origin HttpOnly cookie, принимает
replay cursor исключительно через `Last-Event-ID`, отдаёт typed
`ClientRealtimeFrameV1` как SSE и прекращает stream с explicit `ReplayGap` при
live-buffer overrun или invalid live frame. Subscription source получает exact
authorized browser session и обязан owner-filter frames до передачи Gateway;
internal `DurableEnvelopeV1` не используется. Неполный CLI-набор fail-closed
до Control Store bootstrap, а shutdown Kernel закрывает listener. Private owner-control
operation `BeginBrowserPairing` принимает только current owner-device session,
извлекает owner/device из server-held session и создаёт server-held WebAuthn
ceremony в shared Kernel-memory state. В ответ он возвращает только opaque
256-bit pairing ID и bounded expiry. `BrowserPairingRouter` монтируется лишь
на explicit configured TLS profile: exact `GET /browser/v1/pairing/{id}/registration`
отдаёт options с `no-store`, а `POST .../finish` требует exact HTTPS `Origin`,
WebAuthn verifier и atomic Control Store admission against captured identity
fence. Wrong Origin и malformed request не расходуют pairing; success не
возвращает device ID, credential, session ID или bearer token. Pairing ID сам
по себе не создаёт ceremony, owner rights или session. Этот partial path не
является public owner API и не даёт browser business capability.
Signed installed macOS release может доставить только required
`browser.bootstrap` artifact на `/`: Kernel повторно проверяет его immutable
bytes по release manifest и Gateway держит документ в memory с `no-store` и
CSP; directory/assets fallback и legacy frontend отсутствуют. Canonical source
одного документа — `frontend/browser-bootstrap/index.html`; release input
должен bind-ить его как required `browser.bootstrap`, и
`pnpm --dir frontend browser:bootstrap:check` проверяет отсутствие external
assets, bearer/legacy credentials, session/token storage и превышения лимита
Gateway. Документ осуществляет owner-approved pairing, WebAuthn registration
и cookie-backed sign-in; persistent browser state ограничен public credential
ID, не token/session/secret. В configured TLS surface по-прежнему отсутствуют
client-safe realtime owner и generated business client surface. После sign-in
доступен только typed owner-neutral ConnectRPC `BrowserSessionService/GetStatus`:
он подтверждает bounded expiry текущей cookie session и не выдаёт owner data,
opaque session ID, credential или grant. Если Kernel запускается из installed signed macOS bundle и browser
listener явно включён, отсутствие exact required `browser.bootstrap` является
release integrity error и fail-closed до bind listener; только development
binary вне `.app` может не иметь bootstrap artifact.

Foundation также имеет lookup по credential ID через private Control Store и
не разрешает создать session по выбранному вызывающим `device_id`: принимается
только opaque material успешного assertion, после чего active browser identity
повторно резолвится до выдачи session.
`BrowserGatewaySessionService` собирает exact-Origin mutation check,
server-held authentication ceremony, assertion persistence и выдачу только
secure cookie; HTTP adapter не получает session ID или server-held
ceremony state. При explicit TLS admission Kernel создаёт этот service
из private Control Store authority; SSE без admitted client-safe owner
возвращает unavailable, а не имитирует пустой replay stream.
Detached `BrowserAuthenticationRouter` реализует только `POST` begin/finish
authentication JSON routes с bounded body, `Cache-Control: no-store` и без
CORS. Kernel монтирует его на explicit local либо explicitly selected
paired-remote TLS profile; без полной operator configuration listener не
существует. Это не является owner admission и не открывает public owner API.
Realtime router также detached: он не делает source of truth, не получает
owner-business route и не публикует URL cursor или payload plaintext в SSE
transport.
`GatewayApplicationRouter` является единственной текущей HTTP composition для
этих adapters: он реализует Hyper service для уже admitted listener,
разрешает только health/readiness, browser authentication и exact realtime
route, а любые query-string routes fail closed. `GatewayLoopbackListenerV1`
принимает только loopback address и обслуживает local HTTP/1.1 connections до
explicit owner shutdown;
paired-remote listener остаётся TLS + HTTP/2. Это не меняет lifecycle:
владелец Kernel listener и public admission всё ещё отсутствует.

Frontend foundation теперь имеет отдельный `BrowserGatewayFetch`: он принимает
только relative same-origin path, использует только `credentials: same-origin`
и отклоняет `X-Макошь-Secret`/`Authorization` до network call.
`createBrowserGatewayConnectTransport` собирает поверх него только typed
Connect transport с relative base URL и отключённым GET; он не принимает
interceptors, headers или business methods. `BrowserGatewayAuthenticator`
использует тот же fetch boundary только для двух authentication routes,
передаёт WebAuthn assertion и не возвращает browser session identifier. Эти
клиентские foundations не подключены к legacy UI и не являются public Gateway
listener или business client.
`BrowserGatewayRealtime` использует generated clean-room
`ClientRealtimeFrameV1` contract через same-origin `EventSource` на exact
`/api/realtime/v1/events`. Browser сам повторяет SSE `Last-Event-ID` при
reconnect, поэтому client не хранит и не передаёт cursor в URL. Adapter
закрывает subscription и требует explicit consumer reaction на `ReplayGap` или
malformed frame; он не интерпретирует owner payload.

Зависит от:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0225: Первый production slice и phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Browser является first-party client поверх единственного Core Gateway, но не
является Tauri WebView, module runtime или доверенным host bridge. Старый
frontend хранит base URL и `X-Макошь-Secret`; этот surface является legacy
evidence и не может быть адаптирован для browser client. При этом ADR-0218
задаёт отдельные ES256 identities для desktop/Android, но не определяет
origin-bound browser credential и его pairing lifecycle.

## Решение

### Browser — отдельное device identity

Browser получает отдельный revocable `DeviceIdentity`, а не копию desktop key
и не общий owner secret. Его WebAuthn private key остаётся в authenticator,
а local browser key — non-extractable `CryptoKey` в browser profile. Control
Store сохраняет только credential ID, COSE ES256 public key, browser-local
public point, RP ID, device/owner relation, identity epoch, active/revoked
state и минимальный counter/provenance для проверки assertion. Биометрия,
attestation certificate и private keys нигде Макошь не сохраняются.

Первичная регистрация browser credential доступна только из уже
owner-authorized desktop/Android pairing ceremony с single-use, short-lived,
operation-bound challenge. Browser не может enrol себя через anonymous public
endpoint, восстановить удалённый device, повысить capability или заменить RP
ID. Revoke browser identity atomically increments identity epoch и немедленно
инвалидирует все его Gateway sessions/replay cursors.

Credential обязан быть WebAuthn ES256 и user-verifying. Backup-eligible/synced
credential допустим только вместе с browser-local proof ADR-0234. Registration и
assertion сверяют exact RP ID, exact HTTPS origin, challenge, type,
authenticator-data RP ID hash, user-presence/user-verification flags, ES256
signature и monotonic counter policy. `attestation = none`; Макошь не делает
trust decision по model/biometric data authenticator. Это следует standard
WebAuthn assertion model, где authenticator signs assertion data and the
client-data hash for the relying party origin.

Текущий verifier pin-ит `webauthn-rs-core` 0.5.5. Его narrow wrapper явно
фиксирует единственный HTTPS origin, `ES256`, `UserVerificationPolicy::Required`
и `attestation = none`. Никакой browser response,
state ceremony или credential material не сериализуется в client state.

После verifier Gateway foundation не передаёт credential в Kernel как untyped
callback data. Он строит `BrowserEnrollmentV1` из verified material, owner/RP
pairing и captured `GatewayIdentityFenceV1`; private Kernel adapter требует
тот же current owner и exact instance/generation/identity epoch перед
single-writer admission. Ошибка persistence не расходует pairing, а stale
fence не создаёт или не заменяет browser identity. Device ID для этого пути
создаёт Gateway, а не browser caller.

### Same-origin session

Browser bundle доставляется только из signed application bundle Core Gateway
на exact same-origin. В production origin всегда HTTPS; loopback development
может использовать browser-secure localhost exception, но не wildcard origin,
LAN HTTP или arbitrary Vite origin. Gateway не выдаёт CORS allowance и не
принимает browser caller identity из Host, IP, user-agent или Origin alone.

После успешной WebAuthn assertion Gateway создаёт Kernel-memory-only session,
fenced по instance generation, browser device identity epoch, capability set и
short expiry. Browser получает только `Secure; HttpOnly; SameSite=Strict;
Path=/` session cookie without `Domain`, `Max-Age` or persistent storage.
Cookie не содержит owner key, WebAuthn assertion, raw capability или durable
token; opaque value имеет смысл только в memory of the current Kernel run.

Все state-changing requests требуют exact same-origin `Origin` check и
fresh operation-bound WebAuthn user-verification proof for privileged owner
operation. Query, ConnectRPC and the one SSE stream use the same session
boundary; SSE cursor remains non-secret device-local state. Session, cookie,
assertion and cursor never enter URLs, localStorage, IndexedDB, diagnostics,
analytics, logs or typed public errors.

### Gateway surface

Browser has no handwritten REST business client. Owner-specific business
operations remain generated ConnectRPC contracts, while ordinary HTTP stays
limited to health/readiness, browser auth ceremony, OAuth callback, Blob and
one replayable SSE stream. Gateway maps the cookie only to a client session;
it never exposes Kernel Control Store, NATS, PostgreSQL, PgBouncer, Vault,
module IPC or `DurableEnvelopeV1` to browser code.

Browser delivery does not admit any owner package, provider operation,
business query or generic content API. It can expose only client technical
bootstrap/authentication and client-safe Gateway frames until `first_owner_v1`
selects exact public owner contracts.

## Проверка

`client_gateway_v1` requires executable conformance for at least:

- registration/assertion success and rejection of wrong origin, RP ID,
  challenge, type, signature, UV/UP flags and counter regression; dual-proof
  validation for backup-eligible credential is specified by ADR-0234;
- pairing replay, expiry, unauthorised registration, revoke and epoch/session
  fencing;
- absent wildcard CORS, absent plaintext/non-loopback bind and absent token in
  URL/browser persistent storage/logs/errors;
- exact cookie attributes plus Origin/CSRF rejection for mutation;
- one browser SSE stream with replay, duplicate, gap and explicit disconnect;
- browser inability to reach module ports, NATS, PostgreSQL or private Kernel
  sockets.

Наличие этого ADR, protocol type или unit test само по себе не открывает
`client_gateway_v1`; требуется согласованное изменение ADR, executable policy
и runtime evidence по ADR-0225.

`browser_client_v1` является отдельным, меньшим phase gate для этого
owner-neutral contour. Он требует exact gateway/local-listener inventory,
WebAuthn pairing and session fencing, signed bootstrap delivery, same-origin
Connect session confirmation, no-secret fetch boundary, fail-closed SSE без
admitted owner и negative privacy/abuse checks. Его открытие не authorizes
owner query/command contracts, durable receipts, Android HTTP/3 conformance
или first owner; всё это остаётся в `client_gateway_v1` и `first_owner_v1`.

## Последствия

- Browser получает phishing-resistant, origin-bound device proof без Макошь
  secret в JavaScript memory/storage.
- Backup-eligible passkeys use the separate browser-local key binding of
  ADR-0234; the WebAuthn credential alone never proves the browser device.
- Gateway implementation обязана добавить owner-approved browser pairing и
  multi-device public identity registry до public listener.
