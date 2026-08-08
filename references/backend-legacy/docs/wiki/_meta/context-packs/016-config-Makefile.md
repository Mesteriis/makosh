# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `016-config-Makefile`
- Group / Группа: `Makefile`
- Role / Роль: `config`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/configuration.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `Makefile`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/Makefile`
- Size bytes / Размер в байтах: `15804`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
SHELL := /usr/bin/env bash

.DEFAULT_GOAL := help

CARGO_TARGET_ROOT ?= $(CURDIR)/target
CARGO_DEV_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/dev
CARGO_VALIDATE_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/validate
CARGO_VALIDATE_CLIPPY_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/validate-clippy
CARGO_VALIDATE_TEST_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/validate-test
CARGO_BUILD_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/build
CARGO_COVERAGE_TARGET_DIR ?= $(CARGO_TARGET_ROOT)/coverage
MAKOSH_NEXTEST_JOBS ?= 4
CARGO_AUDIT_IGNORES ?= RUSTSEC-2023-0071
CARGO_AUDIT_IGNORE_FLAGS = $(foreach advisory,$(CARGO_AUDIT_IGNORES),--ignore $(advisory))
BACKEND_ARCHITECTURE_TARGETS = $(shell node scripts/test/backend-test-targets.mjs targets architecture)
BACKEND_E2E_TARGETS = $(shell node scripts/test/backend-test-targets.mjs targets e2e)
BACKEND_INTEGRATION_TARGETS = $(shell node scripts/test/backend-test-targets.mjs targets integration)
BACKEND_SNAPSHOT_TARGETS = $(shell node scripts/test/backend-test-targets.mjs targets snapshot)
SCCACHE_BIN := $(shell command -v sccache 2>/dev/null)

ifneq ($(strip $(SCCACHE_BIN)),)
export RUSTC_WRAPPER := $(SCCACHE_BIN)
endif

.PHONY: help docker-env dev logs build migrate validate lint-architecture lint-rust lint-frontend architecture-check code-boundaries-check backend-fmt-check backend-clippy backend-test backend-validate frontend-lint frontend-test frontend-build frontend-validate test test-fast test-ci test-unit test-integration test-e2e test-architecture test-snapshot snapshot-test snapshot-accept coverage coverage-html coverage-ci mutants audit deny security udeps watch-test watch-unit watch-integration cache-stats cache-reset test-performance-report whatsapp-live-smoke-readiness whatsapp-native-md-sdk-gap-readiness whatsapp-live-smoke-evidence whatsapp-live-smoke-collect-evidence whatsapp-domain-closure-audit whatsapp-domain-closure-gate whatsapp-business-cloud-edge-readiness whatsapp-business-cloud-edge-config whatsapp-business-cloud-edge-up whatsapp-business-cloud-edge-stop whatsapp-business-cloud-edge-logs vault-backup vault-restore clean clean-dev clean-validate clean-build clean-data clean-vault

help:
	@printf '%s\n' 'Макошь development commands:'
	@printf '%s\n' '  make docker-env    Create docker/.env from docker/.env.example when missing'
	@printf '%s\n' '  make dev           Start PostgreSQL, backend watcher, and Vite dev server'
	@printf '%s\n' '  make logs          Tail the active live development log'
	@printf '%s\n' '  make build         Build backend, frontend, and Tauri release artifacts'
	@printf '%s\n' '  make migrate       Start PostgreSQL if needed and run backend-managed migrations'
	@printf '%s\n' '  make validate      Run architecture, backend, and frontend validation'
	@printf '%s\n' '  make test-fast     Run the fast local test loop (unit + architecture + snapshots + frontend)'
	@printf '%s\n' '  make test          Run the full local test suite entry point'
	@printf '%s\n' '  make test-ci       Run the CI-oriented backend nextest profile and frontend unit tests'
	@printf '%s\n' '  make test-unit     Run Rust unit tests through cargo-nextest without Docker'
	@printf '%s\n' '  make test-integration Run container-backed backend integration targets'
	@printf '%s\n' '  make test-e2e      Run backend end-to-end/API nextest targets'
	@printf '%s\n' '  make test-architecture Run architecture test targets and JS contract checks'
	@printf '%s\n' '  make test-snapshot Run backend snapshot tests'
	@printf '%s\n' '  make coverage      Run coverage summary via cargo-llvm-cov + nextest'
	@printf '%s\n' '  make coverage-html Generate HTML coverage output in target/coverage/html'
	@printf '%s\n' '  make coverage-ci   Generate LCOV coverage output in target/coverage/lcov.info'
	@printf '%s\n' '  make snapshot-accept Accept updated insta snapshots'
	@printf '%s\n' '  make mutants       Run cargo-mutants with nextest'
	@printf '%s\n' '  make audit         Run cargo-audit'
	@printf '%s\n' '  make deny          Run cargo-deny'
	@printf '%s\n' '  make security      Run audit and deny'
	@printf '%s\n' '  make udeps         Run cargo-udeps on nightly Rust'
	@printf '%s\n' '  make watch-test    Watch files and rerun make test-fast'
	@printf '%s\n' '  make watch-unit    Watch files and rerun make test-unit'
	@printf '%s\n' '  make watch-integration Watch files and rerun make test-integration'
	@printf '%s\n' '  make cache-stats   Show sccache stats'
	@printf '%s\n' '  make cache-reset   Reset sccache stats'
	@printf '%s\n' '  make test-performance-report Rebuild reports from existing nextest JUnit XML files'
	@printf '%s\n' '  make whatsapp-live-smoke-readiness Run static WhatsApp live-smoke readiness checks'
	@printf '%s\n' '  make whatsapp-native-md-sdk-gap-readiness Verify native MD wa-rs command gap inventory'
	@printf '%s\n' '  make whatsapp-live-smoke-evidence Validate sanitized WhatsApp manual live-smoke evidence'
	@printf '%s\n' '  make whatsapp-live-smoke-collect-evidence Build and validate evidence from sanitized live-smoke observations'
	@printf '%s\n' '  make whatsapp-domain-closure-audit Report WhatsApp domain closure blockers'
	@printf '%s\n' '  make whatsapp-domain-closure-gate Fail until WhatsApp domain closure evidence is complete'
	@printf '%s\n' '  make whatsapp-business-cloud-edge-readiness Run Business Cloud edge proxy readiness checks'
	@printf '%s\n' '  make whatsapp-business-cloud-edge-config Validate the Business Cloud edge proxy compose profile'
	@printf '%s\n' '  make whatsapp-business-cloud-edge-up Start the Business Cloud edge proxy compose profile'
	@printf '%s\n' '  make whatsapp-business-cloud-edge-stop Stop the Business Cloud edge proxy compose service'
	@printf '%s\n' '  make whatsapp-business-cloud-edge-logs Tail the Business Cloud edge proxy compose logs'
	@printf '%s\n' '  make vault-backup  Create a timestamped PostgreSQL + vault backup'
	@printf '%s\n' '  make vault-restore Interactively restore PostgreSQL + vault from a backup'
	@printf '%s\n' '  make clean         Remove build artifacts, temporary files, and logs'
	@printf '%s\n' '  make clean-dev     Remove dev watcher Cargo artifacts and local dev logs'
	@printf '%s\n' '  make clean-validate  Remove validation Cargo artifacts'
	@printf '%s\n' '  make clean-build   Remove release/Tauri build artifacts'
	@printf '%s\n' '  make clean-data    Delete local PostgreSQL data after confirmation'
	@printf '%s\n' '  make clean-vault   Delete local vault data after confirmation'

docker-env:
	@bash -lc 'source scripts/lib/env.sh; ensure_docker_env_file'

dev:
	@./scripts/dev.sh

logs:
	@./scripts/logs.sh

build:
	@./scripts/build.sh

migrate:
	@./scripts/migrate.sh

validate: architecture-check code-boundaries-check backend-validate frontend-validate

lint-architecture: architecture-check code-boundaries-check

lint-rust: backend-fmt-check backend-clippy

lint-frontend: frontend-lint

architecture-check:
	@node scripts/check-architecture-contract.test.mjs
	@node scripts/check-architecture.mjs --self-test
	@node scripts/check-architecture.mjs

code-boundaries-check:
	@node scripts/check-code-boundaries.mjs

backend-fmt-check:
	@cargo fmt --check --manifest-path backend/Cargo.toml

backend-clippy:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_CLIPPY_TARGET_DIR)" CARGO_INCREMENTAL=0 cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings

backend-test:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh default --all-targets
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite backend-full --output reports/test-performance/backend-full

backend-validate: backend-fmt-check backend-clippy backend-test

test-unit:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --workspace --lib --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS)'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite unit --output reports/test-performance/unit

test-integration:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh integration $(foreach target,$(BACKEND_INTEGRATION_TARGETS),--test $(target))
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/integration/junit.xml --suite integration --output reports/test-performance/integration

test-e2e:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh integration $(foreach target,$(BACKEND_E2E_TARGETS),--test $(target))
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/integration/junit.xml --suite e2e --output reports/test-performance/e2e

test-architecture:
	@node scripts/check-architecture-contract.test.mjs
	@node scripts/check-architecture.mjs --self-test
	@node scripts/check-architecture.mjs
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_ARCHITECTURE_TARGETS),--test $(target))'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite architecture --output reports/test-performance/architecture

test-snapshot: snapshot-test

snapshot-test:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_SNAPSHOT_TARGETS),--test $(target))'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite snapshot --output reports/test-performance/snapshot

snapshot-accept:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; INSTA_UPDATE=always CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_SNAPSHOT_TARGETS),--test $(target))'

test-fast: test-unit test-architecture test-snapshot frontend-test

test-ci:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh ci --all-targets
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/ci/junit.xml --suite backend-ci --output reports/test-performance/backend-ci
	@$(MAKE) frontend-test

test: test-fast test-integration

coverage:
	@CARGO_TARGET_DIR="$(CARGO_COVERAGE_TARGET_DIR)" ./scripts/test/run-llvm-cov.sh ci --summary-only

coverage-html:
	@mkdir -p target/coverage/html
	@CARGO_TARGET_DIR="$(CARGO_COVERAGE_TARGET_DIR)" ./scripts/test/run-llvm-cov.sh ci --html --output-dir target/coverage/html

coverage-ci:
	@mkdir -p target/coverage
	@CARGO_TARGET_DIR="$(CARGO_COVERAGE_TARGET_DIR)" ./scripts/test/run-llvm-cov.sh ci --lcov --output-path target/coverage/lcov.info

mutants:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand mutants "cargo install --locked cargo-mutants"; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; cd backend && cargo mutants --test-tool nextest'

audit:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand audit "cargo install --locked cargo-audit"; cargo audit $(CARGO_AUDIT_IGNORE
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
