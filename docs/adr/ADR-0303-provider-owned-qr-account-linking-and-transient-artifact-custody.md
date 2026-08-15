# ADR-0303: Provider-owned QR account linking and transient artifact custody

Статус: Принято
Дата: 2026-07-27
Состояние реализации: Frontend cutover реализован. Telegram runtime и
generated `telegram.authorization.v1` передают transient `qr_link`, first-party
setup больше не заявляет QR authorization до TDLib `ready`, а frontend
локально рендерит QR, опрашивает status и поддерживает 2FA continuation.
WhatsApp integration-owned adapter открывает existing owner-visible Tauri
WebView; plain browser показывает exact native-host requirement без fixture.
Frontend unit/type/boundary tests, native Tauri feature build и live browser
negative contour зелёные. Ручное сканирование реального provider QR не
выполнялось и не заявляется как evidence.

Client UX update 2026-08-14 делает Telegram QR первой и основной областью
account dialog. Если TDLib application credentials ещё не provisioned, вместо
provider-looking fixture показывается пустая QR custody area с exact blocker и
отдельной owner action для одноразовой application configuration. После
write-only Vault provisioning тот же dialog запрашивает real transient QR без
перехода через технический wizard step.

Уточняет:

- [ADR-0204: integration/provider boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0240: Telegram clean-room boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0241: WhatsApp clean-room boundary](ADR-0241-whatsapp-clean-room-provider-boundary.md);
- [ADR-0266: Telegram admission](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0276: WhatsApp host bridge admission](ADR-0276-whatsapp-kernel-admission-host-bridge-and-event-only-communications-handoff.md);
- [ADR-0281: frontend clean-room composition](ADR-0281-communications-frontend-clean-room-composition.md);
- [ADR-0300: loopback development assembly](ADR-0300-loopback-full-stack-development-assembly.md).

## Контекст

Telegram TDLib runtime уже получает provider-issued `tg://login` link и
возвращает его через отдельный authorization query. Frontend, однако:

- передавал `qr_authorized = true` до сканирования;
- показывал secret-bearing login link как обычную внешнюю ссылку;
- не рендерил QR и не обновлял authorization state.

WhatsApp account setup применяет managed settings и запрашивает private host
bridge, но не открывает уже реализованный owner-visible Tauri companion.
Поэтому пользователь получает «configured profile» без доступного QR.

Объединять оба provider flow в generic account/QR backend нельзя. Telegram
выдаёт bounded link через integration-owned public authorization contract.
WhatsApp Web сам показывает QR внутри OS-managed WebView и не должен отдавать
DOM/session artifact в Gateway, Settings или module storage.

## Решение

### Telegram

Telegram QR linking остаётся частью Telegram integration:

```text
TDLib authorization state
  -> Telegram runtime projection
  -> telegram.authorization.v1
  -> authenticated Core Gateway opaque route
  -> integration-owned frontend controller
  -> local QR rendering
```

Account provisioning передаёт `qr_authorized = false`. Это поле является
наблюдаемым результатом authorization lifecycle, а не client assertion.
Frontend после managed setup опрашивает bounded authorization status до
`waiting_qr_scan`, `waiting_password`, terminal ready или sanitized error.

`qr_link`:

- хранится только в памяти Telegram runtime и active client view;
- не попадает в Settings, Vault, PostgreSQL, NATS, SSE, logs, errors,
  analytics или browser history;
- не открывается как external navigation;
- локально преобразуется в QR image без network request;
- очищается сразу после смены authorization state или unmount.

2FA password идёт через существующий typed Telegram authorization command и не
сохраняется во frontend state после submit.

### WhatsApp

WhatsApp linking остаётся host-owned:

```text
integration Settings UI
  -> Tauri command with exact configured account_id
  -> owner-visible account-scoped WebView
  -> https://web.whatsapp.com provider page
  -> owner scans provider-rendered QR
  -> OS-managed WebView profile owns session
```

Frontend не извлекает QR из DOM, cookies, local storage, IndexedDB или WebView
profile. Tauri command открывает только exact account-scoped companion,
созданный native host unit. QR/Pair Code остаётся `visible_only_in_owner_controlled_webview`.

Plain browser development не имеет native host capability. Он обязан показать
`desktop_host_required` и не генерировать fixture, placeholder, demo QR или
provider-looking artifact. `make dev` остаётся full loopback browser assembly,
но не заявляет native WhatsApp pairing.

### Kernel/Core agreement

Kernel/Core:

- не вводят generic QR/account service;
- не декодируют Telegram QR link и не читают WhatsApp WebView;
- для Telegram проверяют existing exact capability, device/session и route
  binding, затем переносят typed payload opaque;
- для WhatsApp stage-ят только existing private host route и exact runtime
  generation/grant epoch binding;
- не сохраняют provider link artifact и не создают account truth.

Telegram и WhatsApp capabilities независимы. Наличие одной не выдаёт права на
другую. Browser session не эквивалентна native host authority.

### Functional units

```text
telegram runtime/api        provider authorization state and typed QR link
telegram frontend           polling, local QR rendering and 2FA UX
whatsapp Tauri host         visible provider WebView and OS profile
whatsapp frontend           exact native-host invocation and status UX
app settings composition    places provider panels only
development assembly        starts services; owns no provider semantics
```

Ни domain, ни release/development assembly не получают integration semantics.

## Gate `provider_qr_account_linking_frontend_v1`

Gate открывается только при наличии:

1. Telegram provisioning с `qr_authorized = false`;
2. real TDLib `qr_link` query and bounded refresh;
3. local QR rendering without network/external navigation;
4. QR removal after state change/unmount;
5. explicit 2FA continuation;
6. WhatsApp exact Tauri companion invocation for configured account;
7. no WhatsApp QR extraction or persistence;
8. explicit `desktop_host_required` in plain browser;
9. no fixture/demo QR paths;
10. unit, type, boundary and live UI evidence.

Gate не открывает Telegram/WhatsApp full operational umbrella и не добавляет
integration packages в Communications domain.
