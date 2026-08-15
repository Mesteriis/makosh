# Макошь frontend

Статус: активный clean-room client surface для Communications и Settings

Vue 3, Vite и Tauri source находятся в `frontend/`. Страницы Communications и
Settings используют generated ConnectRPC/Protobuf clients Core Gateway,
replayable client realtime и capability-driven availability. Provider
operational routes Mail, Telegram, WhatsApp и Zulip остаются frontend-частями
соответствующих integrations, а не Communications domain.

Предыдущая frontend documentation с legacy full-stack commands, API/auth
contract, sidecar packaging и transport client перенесена в
[`references/backend-legacy/frontend/README.md`](../references/backend-legacy/frontend/README.md).

## Текущие правила

- Не добавлять новый business API поверх legacy routes.
- Использовать generated clients; handwritten owner business REST запрещён.
- Не импортировать integration implementation в Communications domain UI.
- Provider screens загружаются только при фактической capability availability
  из Gateway bootstrap.
- Host/Tauri bridge не является business API.

Полный локальный browser contour запускается из корня:

```sh
make dev
```

Команда ждёт readiness и открывает `http://127.0.0.1:5173/`. Vite проксирует
только exact Gateway paths и добавляет process-local proof на server-side hop;
proof не попадает в browser bundle.

Для локального Telegram QR `make dev` может подключить owner-only корневой
`.env` как bounded native credential source. Принимается ровно одна полная пара
`HERMES_TELEGRAM_API_ID/HASH` либо `MAKOSH_TELEGRAM_API_ID/HASH`; API hash не
выдаётся Vite/browser и sealing выполняет loopback Owner Vault host. Source
можно переопределить absolute path через
`MAKOSH_DEV_TELEGRAM_CREDENTIALS_FILE`.

Для scoped frontend validation используйте scripts из `package.json`:

```sh
pnpm lint
pnpm typecheck
pnpm test:unit
pnpm build
```

Успешная frontend-команда сама по себе не доказывает live Gateway/provider
readiness; для end-to-end evidence нужен работающий root `make dev` или
отдельный managed integration contour.
