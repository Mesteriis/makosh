# Frontend

Vue 3 + TypeScript desktop UI for Макошь, packaged by Tauri.

Current scope is a desktop/laptop shell for the local backend APIs with provider account setup wizards for Gmail, iCloud and raw IMAP, graph/project/task/Persona identity/document workflow surfaces, and local AI workflow surfaces. Mobile UI is out of scope while ADR-0031 is active.

## UI Styling Contract

The app-level CSS is loaded in this order from `src/App.vue`:

1. `src/assets/styles/tokens.css` defines design tokens and browser root defaults.
2. `src/assets/styles/app.css` defines global shell, view, state and responsive styles.

All components use scoped `<style>` blocks and Tailwind utility classes. No inline `style=` attributes in production components.

The supported desktop window minimum is `800 x 600`. At smaller widths or heights, a viewport guard is shown instead of the app. This is a desktop window constraint, not mobile UI support; ADR-0031 still keeps mobile design and validation out of scope.

## Scaffold

The frontend was scaffolded as a new Vue 3 + TypeScript + Vite project:

```sh
pnpm create vue@latest . -- --typescript --force
pnpm install
```

Tauri was initialized with:

```sh
pnpm tauri init --ci --app-name "Макошь" --window-title "Макошь" --frontend-dist "../dist" --dev-url "http://localhost:5173" --before-dev-command "pnpm dev" --before-build-command "pnpm build"
```

## Commands

```sh
make dev
make logs
make build
make migrate
make clean
make clean-vault
```

`make dev` is the supported desktop development loop. It starts PostgreSQL in
Docker, runs the backend via repo-local `bacon`, starts this Vue 3 + Vite
frontend natively at
`http://127.0.0.1:5174`.

The active session also exposes an aggregated plain-text log at:

```sh
make logs
```

`make build` is the supported release packaging entrypoint. It builds the
frontend, builds the backend release binary, prepares bundled Google OAuth,
TDLib and backend sidecar resources internally, and then runs `pnpm tauri
build`.

## Bundled TDLib Runtime

macOS release builds package the Telegram TDLib JSON runtime from
`frontend/src-tauri/resources/tdlib/`. Generated `libtdjson.dylib` files are not
committed; `make build` prepares the resource automatically before `tauri build`:

```sh
make build
```

The internal build step copies `libtdjson.dylib` from, in order:

1. `MAKOSH_TDJSON_SOURCE`
2. `MAKOSH_TDJSON_PATH`
3. Homebrew `tdlib`
4. `/opt/homebrew/lib/libtdjson.dylib`
5. `/usr/local/lib/libtdjson.dylib`

Release CI can build TDLib from source instead of relying on a system install:

```sh
MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make build
```

The backend still accepts `MAKOSH_TDJSON_PATH` as a development override, but a
packaged macOS app should resolve TDLib from the bundled Tauri resource path.
Linux is supported only as a development/container target and is not packaged as
a desktop TDLib bundle.

Telegram QR login also needs Telegram app credentials. Development runs can set
`MAKOSH_TELEGRAM_API_ID` and `MAKOSH_TELEGRAM_API_HASH` in the backend
environment. Packaged macOS builds can inject them into the Tauri launcher with
`MAKOSH_BUNDLED_TELEGRAM_API_ID` and `MAKOSH_BUNDLED_TELEGRAM_API_HASH`; the
launcher forwards those values to the backend sidecar as runtime
`MAKOSH_TELEGRAM_API_ID` and `MAKOSH_TELEGRAM_API_HASH`.

Google mail setup needs one project-owned OAuth Desktop app client. End users of
the packaged app should not create their own Google Cloud project. Release builds
copy the downloaded Desktop app JSON into the Tauri resource bundle:

```sh
make build
```

The internal build step reads `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` from
`docker/.env`, or `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` from the shell, and
copies the file to `frontend/src-tauri/resources/google-oauth/client_secret.json`.
That generated resource is ignored by Git. The packaged launcher passes the bundled resource path to the backend sidecar as
`MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.

## Bundled Backend Sidecar

macOS release builds package the Rust backend as a Tauri sidecar from
`frontend/src-tauri/binaries/`. Generated sidecar binaries are not committed;
`make build` prepares the current host binary before `tauri build`:

```sh
make build
```

## Architecture

The frontend is organized by domain under `src/domains/`:

- `src/domains/` — 14 domain modules (home, settings, personas, organizations, projects, tasks, calendar, documents, notes, knowledge, review, agents, timeline, communications, telegram, whatsapp)
- `src/shared/` — Shared UI primitives, stores, composables
- `src/platform/` — Platform abstractions (API client, SSE, routing, i18n, theming)
- `src/app/` — App shell, layout, view routing

Each domain follows a consistent structure:

```
domains/<name>/
  types/<name>.ts      — TypeScript interfaces
  api/<name>.ts        — API functions
  queries/<name>.ts    — TanStack Query hooks
  stores/<name>.ts     — Pinia store
  components/          — Vue 3 SFC components
  views/<name>.ts      — Page-level view component
```

Data flow: API → TanStack Query → Component (direct) or API → Pinia Store → Component.

Requests use `X-Макошь-Secret: <secret>` via the centralized `ApiClient` (see `src/platform/api/ApiClient.ts`).

Validate frontend packaging changes with:

```sh
make build
```

For the normal full-stack desktop workflow:

```sh
make dev
```
