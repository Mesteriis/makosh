### Summary

Создаётся страница `operations/configuration.md` в русской wiki репозитория `makosh`. На основе трёх встроенных файлов `.github/workflows` документируются конфигурации CI/CD: основной пайплайн (`ci.yml`), ночные проверки (`nightly.yml`) и развёртывание документационного портала (`pages.yml`). Каждое утверждение подтверждается исходным кодом.

### Proposed pages

**Путь**: `operations/configuration.md`

```markdown
# Конфигурация CI/CD

## Workflow `ci.yml`

Основной workflow непрерывной интеграции. Запускается на события `pull_request` и `push` в ветку `main`.

### Jobs

#### `architecture` — Проверка архитектурных границ
- Команда: `make test-architecture`
- Окружение: `ubuntu-latest`, Node 24, Rust 1.88, `cargo-nextest`
- Таймаут: 10 минут

#### `backend-fmt` — Форматирование backend-кода
- Команда: `make backend-fmt-check`
- Rust с компонентом `rustfmt`, версия 1.88
- Переменная окружения: `CARGO_TARGET_ROOT=/tmp/makosh-cargo-target-backend-fmt`
- Таймаут: 15 минут

#### `backend-clippy` — Статический анализ backend-кода
- Команда: `make backend-clippy`
- Rust с компонентом `clippy`, версия 1.88
- Кэширование `~/.cargo/registry` и `~/.cargo/git`
- Переменная окружения: `CARGO_TARGET_ROOT=/tmp/makosh-cargo-target-backend-clippy`
- Таймаут: 25 минут

#### `backend-unit` — Модульные тесты backend
- Команда: `make test-unit`
- Node 24, `cargo-nextest`
- Кэширование Cargo
- Переменная окружения: `CARGO_TARGET_ROOT=/tmp/makosh-cargo-target-backend-unit`
- Артефакты (выгружаются всегда):
  - `reports/test-performance/unit.json`
  - `reports/test-performance/unit.md`
  - `target/nextest/default/junit.xml`
- Таймаут: 20 минут

#### `backend-snapshot` — Snapshot-тесты backend
- Команда: `make test-snapshot`
- Node 24, `cargo-nextest`
- Переменная окружения: `CARGO_TARGET_ROOT=/tmp/makosh-cargo-target-backend-snapshot`
- Таймаут: 15 минут

#### `backend-integration` — Интеграционные тесты backend
- Запускается **только** при `push` в `main`
- Команда: `make test-integration`
- Требуется Docker Compose
- `cargo-nextest`
- Артефакты (выгружаются всегда):
  - `reports/test-performance/integration.json`
  - `reports/test-performance/integration.md`
  - `target/nextest/integration/junit.xml`
- Таймаут: 45 минут

#### `coverage` — Покрытие кода
- Запускается **только** при `push` в `main`
- Команда: `make coverage-ci`
- Rust с компонентом `llvm-tools-preview`, версия 1.88
- Устанавливаются `cargo-nextest` и `cargo-llvm-cov`
- Артефакты (выгружаются всегда):
  - `target/coverage/lcov.info`
  - `target/nextest/ci/junit.xml`
- Таймаут: 60 минут

#### `security` — Проверки безопасности
- Запускается **только** при `push` в `main`
- Команда: `make security`
- Устанавливаются `cargo-audit` и `cargo-deny`
- Таймаут: 20 минут

#### `frontend-lint` — Линтинг frontend
- Команда: `make frontend-lint`
- Node 24, pnpm (версия из `frontend/package.json`), `corepack`
- Установка зависимостей: `pnpm install --frozen-lockfile` в `frontend/`
- Таймаут: 15 минут

#### `frontend-test` — Тесты frontend
- Команда: `make frontend-test`
- Тот же pnpm/setup, что и в `frontend-lint`
- Таймаут: 15 минут

#### `frontend-build` — Сборка frontend
- Команда: `make frontend-build`
- Тот же pnpm/setup, что и в `frontend-lint`
- Таймаут: 15 минут

## Workflow `nightly.yml`

Ночные проверки качества. Запускается по расписанию (cron `0 3 * * *` — ежедневно в 3:00 UTC) и вручную (`workflow_dispatch`).

### Jobs

#### `backend-e2e` — End-to-end тесты backend
- Команда: `make test-e2e`
- Node 24, Rust 1.88, `cargo-nextest`
- Требуется Docker Compose
- Таймаут: 60 минут

#### `mutants` — Мутационное тестирование
- Команда: `make mutants`
- Rust 1.88, `cargo-nextest`, `cargo-mutants`
- Таймаут: 180 минут

## Workflow `pages.yml`

Публикация документационного портала через GitHub Pages.

- Запускается при `push` в `main`, если изменены файлы в `docs/site/**` или сам workflow-файл `.github/workflows/pages.yml`, а также вручную (`workflow_dispatch`).
- Права: `contents: read`, `pages: write`, `id-token: write`.
- Группа параллелизма: `github-pages`, без отмены текущих запусков.
- Окружение развёртывания: `github-pages`.

### Job: `deploy`

Шаги:
1. Checkout
2. Configure Pages (`actions/configure-pages@v5`)
3. Upload Pages artifact — источник: `docs/site`
4. Deploy Pages (`actions/deploy-pages@v4`)

Итоговый URL публикуемой страницы доступен через `steps.deployment.outputs.page_url`.
```

### Source coverage

- **`.github/workflows/ci.yml`** — описан каждый job: имя, команда, используемые версии (Node 24, Rust 1.88), таймауты, переменные окружения (`CARGO_TARGET_ROOT`), кэширование (backend-clippy, backend-unit), условия запуска (`push` в `main` для integration/coverage/security), выгружаемые артефакты (backend-unit, backend-integration, coverage) и инструменты (`cargo-nextest`, `cargo-llvm-cov`).
- **`.github/workflows/nightly.yml`** — описан триггер (cron и `workflow_dispatch`), jobs `backend-e2e` и `mutants`, их команды, таймауты, требуемые инструменты и наличие Docker Compose.
- **`.github/workflows/pages.yml`** — описан триггер (push по определённым путям + `workflow_dispatch`), права, параллелизм, окружение, шаги деплоя (`actions/configure-pages@v5`, `actions/upload-pages-artifact@v3`, `actions/deploy-pages@v4`), источник артефакта (`docs/site`) и URL публикации.

### Drift candidates

В предоставленном контексте расхождений не обнаружено — страница строится непосредственно по встроенным исходным файлам.
