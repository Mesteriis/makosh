### Summary / Резюме

На основе предоставленного (обрезанного) `Makefile` репозитория `makosh` создаётся страница русской Obsidian‑wiki `operations/configuration.md`. Страница документирует:

- назначение и общую структуру `Makefile` как точки входа для операций,
- переменные конфигурации (`?=`‑переменные, автоматическое обнаружение `sccache`),
- систему целей, агрегаты (например, `validate`, `test-fast`, `test`) и команды справки (`help`),
- видимые реализации ключевых целей (вплоть до места обрезания файла),
- ограничения полноты из‑за обрезания исходника после 12000 символов.

Все утверждения основаны исключительно на буквальном содержимом встроенного фрагмента. Выводы о поведении за пределами обрезанного текста не делаются.

---

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

```markdown
# Конфигурация операций

Операции разработки, тестирования, проверок и очистки проекта `makosh`
управляются через команды `make`. Основной файл — `Makefile` в корне
репозитория.

## Общие сведения

- `SHELL := /usr/bin/env bash`
- `.DEFAULT_GOAL := help` — при вызове `make` без аргументов выводится справка
- В файле объявлено большое количество `.PHONY`‑целей, охватывающих все этапы
  разработки

## Переменные конфигурации

Многие параметры могут быть переопределены через переменные окружения
(используется оператор `?=`). Ниже перечислены переменные из предоставленного
фрагмента.

| Переменная | По умолчанию | Назначение (из контекста) |
|---|---|---|
| `CARGO_TARGET_ROOT` | `$(CURDIR)/target` | Корневая директория артефактов Cargo |
| `CARGO_DEV_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/dev` | Артефакты для разработки |
| `CARGO_VALIDATE_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/validate` | Общая папка артефактов валидации |
| `CARGO_VALIDATE_CLIPPY_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/validate-clippy` | Артефакты Clippy |
| `CARGO_VALIDATE_TEST_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/validate-test` | Артефакты тестов валидации |
| `CARGO_BUILD_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/build` | Артефакты релизных сборок |
| `CARGO_COVERAGE_TARGET_DIR` | `$(CARGO_TARGET_ROOT)/coverage` | Артефакты покрытия |
| `MAKOSH_NEXTEST_JOBS` | `4` | Количество потоков для `cargo-nextest` |
| `CARGO_AUDIT_IGNORES` | `RUSTSEC-2023-0071` | Список игнорируемых advisory для `cargo-audit` |

*Примечание:* Полный список переменных может быть шире; исходный файл обрезан
после 12 000 символов.

### Обнаружение sccache

```makefile
SCCACHE_BIN := $(shell command -v sccache 2>/dev/null)

ifneq ($(strip $(SCCACHE_BIN)),)
export RUSTC_WRAPPER := $(SCCACHE_BIN)
endif
```

Если утилита `sccache` найдена в `PATH`, она автоматически используется как
обёртка компилятора Rust.

## Система целей

### Справка

Цель `help` (цель по умолчанию) печатает следующий список команд:

```
make docker-env    Create docker/.env from docker/.env.example when missing
make dev           Start PostgreSQL, backend watcher, and Vite dev server
make logs          Tail the active live development log
make build         Build backend, frontend, and Tauri release artifacts
make migrate       Start PostgreSQL if needed and run backend-managed migrations
make validate      Run architecture, backend, and frontend validation
make test-fast     Run the fast local test loop (unit + architecture + snapshots + frontend)
make test          Run the full local test suite entry point
make test-ci       Run the CI-oriented backend nextest profile and frontend unit tests
make test-unit     Run Rust unit tests through cargo-nextest without Docker
make test-integration Run container-backed backend integration targets
make test-e2e      Run backend end-to-end/API nextest targets
make test-architecture Run architecture test targets and JS contract checks
make test-snapshot Run backend snapshot tests
make coverage      Run coverage summary via cargo-llvm-cov + nextest
make coverage-html Generate HTML coverage output in target/coverage/html
make coverage-ci   Generate LCOV coverage output in target/coverage/lcov.info
make snapshot-accept Accept updated insta snapshots
make mutants       Run cargo-mutants with nextest
make audit         Run cargo-audit
make deny          Run cargo-deny
make security      Run audit and deny
make udeps         Run cargo-udeps on nightly Rust
make watch-test    Watch files and rerun make test-fast
make watch-unit    Watch files and rerun make test-unit
make watch-integration Watch files and rerun make test-integration
make cache-stats   Show sccache stats
make cache-reset   Reset sccache stats
make test-performance-report Rebuild reports from existing nextest JUnit XML files
make whatsapp-live-smoke-readiness Run static WhatsApp live-smoke readiness checks
make whatsapp-native-md-sdk-gap-readiness Verify native MD wa-rs command gap inventory
make whatsapp-live-smoke-evidence Validate sanitized WhatsApp manual live-smoke evidence
make whatsapp-live-smoke-collect-evidence Build and validate evidence from sanitized live-smoke observations
make whatsapp-domain-closure-audit Report WhatsApp domain closure blockers
make whatsapp-domain-closure-gate Fail until WhatsApp domain closure evidence is complete
make whatsapp-business-cloud-edge-readiness Run Business Cloud edge proxy readiness checks
make whatsapp-business-cloud-edge-config Validate the Business Cloud edge proxy compose profile
make whatsapp-business-cloud-edge-up Start the Business Cloud edge proxy compose profile
make whatsapp-business-cloud-edge-stop Stop the Business Cloud edge proxy compose service
make whatsapp-business-cloud-edge-logs Tail the Business Cloud edge proxy compose logs
make vault-backup  Create a timestamped PostgreSQL + vault backup
make vault-restore Interactively restore PostgreSQL + vault from a backup
make clean         Remove build artifacts, temporary files, and logs
make clean-dev     Remove dev watcher Cargo artifacts and local dev logs
make clean-validate  Remove validation Cargo artifacts
make clean-build   Remove release/Tauri build artifacts
make clean-data    Delete local PostgreSQL data after confirmation
make clean-vault   Delete local vault data after confirmation
```

### Агрегаты

Некоторые цели являются составными и только объявляют зависимости:

- `validate` → `architecture-check` + `code-boundaries-check` + `backend-validate` + `frontend-validate`
- `lint-architecture` → `architecture-check` + `code-boundaries-check`
- `lint-rust` → `backend-fmt-check` + `backend-clippy`
- `lint-frontend` → `frontend-lint`
- `backend-validate` → `backend-fmt-check` + `backend-clippy` + `backend-test`
- `test` → `test-fast` + `test-integration`
- `test-fast` → `test-unit` + `test-architecture` + `test-snapshot` + `frontend-test`
- `security` → `audit` + `deny`

## Реализация команд (видимые правила)

Далее приведены правила, полностью или частично попавшие в доступный фрагмент.
Для целей, чьё тело обрезано, указано только наличие в `.PHONY` и описание из
`help`.

### Окружение, разработка и сборка

- **`docker-env`**
  `@bash -lc 'source scripts/lib/env.sh; ensure_docker_env_file'`
- **`dev`**
  `@./scripts/dev.sh`
- **`logs`**
  `@./scripts/logs.sh`
- **`build`**
  `@./scripts/build.sh`
- **`migrate`**
  `@./scripts/migrate.sh`

### Проверки архитектуры и стиля

- **`architecture-check`**
  ```
  @node scripts/check-architecture-contract.test.mjs
  @node scripts/check-architecture.mjs --self-test
  @node scripts/check-architecture.mjs
  ```
- **`code-boundaries-check`**
  `@node scripts/check-code-boundaries.mjs`
- **`backend-fmt-check`**
  `@cargo fmt --check --manifest-path backend/Cargo.toml`
- **`backend-clippy`**
  ```
  @CARGO_TARGET_DIR="$(CARGO_VALIDATE_CLIPPY_TARGET_DIR)" CARGO_INCREMENTAL=0 \
    cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
  ```

### Тестирование (unit, integration, e2e)

- **`backend-test`** (используется в `backend-validate`)
  ```
  @CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" \
    ./scripts/test/run-nextest.sh default --all-targets
  @node scripts/test/analyze-nextest-junit.mjs \
    --input target/nextest/default/junit.xml \
    --suite backend-full \
    --output reports/test-performance/backend-full
  ```
- **`test-unit`**
  Загружает `scripts/lib/rust-tooling.sh`, проверяет наличие `cargo-nextest`,
  запускает `cargo nextest run --workspace --lib --profile default`
  с переменной `NEXTEST_SHOW_PROGRESS` (по умолчанию `bar`), затем анализирует
  JUnit‑отчёт.
- **`test-integration`**
  Использует `./scripts/test/run-nextest.sh integration` с целями из
  `BACKEND_INTEGRATION_TARGETS`, после чего анализирует JUnit.
- **`test-e2e`**
  Аналогично, использует `BACKEND_E2E_TARGETS`.

### Архитектурные и snapshot‑тесты

- **`test-architecture`**
  Выполняет проверки `check-architecture-contract.test.mjs`,
  `check-architecture.mjs --self-test`, `check-architecture.mjs`, затем
  `cargo nextest` с целями из `BACKEND_ARCHITECTURE_TARGETS`.
- **`test-snapshot` / `snapshot-test`**
  `cargo nextest run` с целями из `BACKEND_SNAPSHOT_TARGETS`.
- **`snapshot-accept`**
  То же, что `snapshot-test`, но с `INSTA_UPDATE=always`.

### CI‑тесты

- **`test-ci`**
  `./scripts/test/run-nextest.sh ci --all-targets`, затем анализ JUnit,
  затем `frontend-test`.

### Покрытие

- **`coverage`**
  `./scripts/test/run-llvm-cov.sh ci --summary-only`
- **`coverage-html`**
  Создаёт папку `target/coverage/html` и вызывает `run-llvm-cov.sh ci --html
  --output-dir target/coverage/html`
- **`coverage-ci`**
  Создаёт папку `target/coverage` и вызывает `run-llvm-cov.sh ci --lcov
  --output-path target/coverage/lcov.info`

### Мутационное тестирование и аудит безопасности

- **`mutants`**
  Проверяет наличие `cargo-mutants` и `cargo-nextest`, затем выполняет
  `cd backend && cargo mutants --test-tool nextest`.
- **`audit`** (правило обрезано)
  Начало: `@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand audit …`;
  Полное тело не видно. Описание из `help`: “Run cargo-audit”.
- **`deny`, `security`, `udeps`**
  Правила отсутствуют в доступном фрагменте (обрезаны). Описания даны в `help`.

### Наблюдение, кэш, отчёты, WhatsApp, хранилище, очистка

Для следующих целей доступен только текст из `help`; их правила обрезаны:

- `watch-test`, `watch-unit`, `watch-integration`
- `cache-stats`, `cache-reset`
- `test-performance-report`
- `whatsapp-live-smoke-readiness`, `whatsapp-native-md-sdk-gap-readiness`,
  `whatsapp-live-smoke-evidence`, `whatsapp-live-smoke-collect-evidence`,
  `whatsapp-domain-closure-audit`, `whatsapp-domain-closure-gate`,
  `whatsapp-business-cloud-edge-readiness`,
  `whatsapp-business-cloud-edge-config`, `whatsapp-business-cloud-edge-up`,
  `whatsapp-business-cloud-edge-stop`, `whatsapp-business-cloud-edge-logs`
- `vault-backup`, `vault-restore`
- `clean`, `clean-dev`, `clean-validate`, `clean-build`, `clean-data`, `clean-vault`

## Ограничения контекста

Исходный файл `Makefile` обрезан после 12 000 символов. Вся информация на этой
странице основана исключительно на предоставленном фрагменте. Поведение команд,
правила которых не попали во фрагмент, не документируется и не предполагается.
```

---

### Source coverage / Покрытие источников

**Файл:** `Makefile` (обрезан после 12 000 символов)

Факты, покрытые страницей:

- Установка `SHELL`, `.DEFAULT_GOAL := help`
- Переменные: `CARGO_TARGET_ROOT`, `CARGO_DEV_TARGET_DIR`, `CARGO_VALIDATE_TARGET_DIR`, `CARGO_VALIDATE_CLIPPY_TARGET_DIR`, `CARGO_VALIDATE_TEST_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR`, `CARGO_COVERAGE_TARGET_DIR`, `MAKOSH_NEXTEST_JOBS`, `CARGO_AUDIT_IGNORES`, `SCCACHE_BIN`, `RUSTC_WRAPPER` (условно)
- Формирование `CARGO_AUDIT_IGNORE_FLAGS` и списков целей (`BACKEND_ARCHITECTURE_TARGETS`, `BACKEND_E2E_TARGETS`, `BACKEND_INTEGRATION_TARGETS`, `BACKEND_SNAPSHOT_TARGETS`) через вызовы `node scripts/test/backend-test-targets.mjs`
- Полный текст цели `help` (перечень команд с описаниями)
- Правила для `docker-env`, `dev`, `logs`, `build`, `migrate`
- Правила для `validate`, `lint-architecture`, `lint-rust`, `lint-frontend`, `backend-validate`
- Правила для `architecture-check`, `code-boundaries-check`, `backend-fmt-check`, `backend-clippy`, `backend-test`
- Правила для `test-unit`, `test-integration`, `test-e2e`, `test-architecture`, `test-snapshot`/`snapshot-test`, `snapshot-accept`
- Правила для `test-fast`, `test`, `test-ci`
- Правила для `coverage`, `coverage-html`, `coverage-ci`
- Правило для `mutants`
- Частично видимое правило для `audit` (обрезано)
- Упоминание целей `frontend-test`, `frontend-lint`, `frontend-build`, `frontend-validate` в `.PHONY` и зависимостях, но без их тел

Всё остальное (где правила не видны) задокументировано только на уровне описаний из `help`.

---

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно.
Единственный доступный артефакт — обрезанный `Makefile`; сравнить его с другими источниками (существующей wiki, ADR, скриптами) невозможно, так как они не встроены в context pack.

Если бы потребовалось полное покрытие, следовало бы предоставить полный `Makefile` и соответствующие wiki‑страницы.
