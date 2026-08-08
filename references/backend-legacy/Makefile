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
MAKOSH_LAB_PROVIDER = $(if $(PROVIDER),$(PROVIDER),zulip)
MAKOSH_LAB_ACTION = $(if $(ACTION),$(ACTION),help)
MAKOSH_LAB_EXTRA_ARGS =
ifneq ($(strip $(EXECUTE)),)
MAKOSH_LAB_EXTRA_ARGS += --execute
endif
ifneq ($(strip $(TESTCONTAINERS)),)
MAKOSH_LAB_EXTRA_ARGS += --testcontainers
endif
ifneq ($(strip $(BACKEND)),)
MAKOSH_LAB_EXTRA_ARGS += --backend-evidence
endif
ifneq ($(strip $(SCENARIO)),)
MAKOSH_LAB_EXTRA_ARGS += --scenario "$(SCENARIO)"
endif
ifneq ($(strip $(REPORT_DIR)),)
MAKOSH_LAB_EXTRA_ARGS += --report-dir "$(REPORT_DIR)"
endif
ifneq ($(strip $(REQUIRE_CLOSED)),)
MAKOSH_LAB_EXTRA_ARGS += --require-closed
endif

ifneq ($(strip $(SCCACHE_BIN)),)
export RUSTC_WRAPPER := $(SCCACHE_BIN)
endif

.PHONY: help docker-env docker-app-config docker-app-up docker-app-stop docker-app-logs dev dev-desktop logs build migrate validate lint-architecture lint-rust lint-frontend architecture-check code-boundaries-check backend-fmt-check backend-clippy backend-test backend-validate frontend-lint frontend-test frontend-visual frontend-visual-update frontend-build frontend-validate test test-fast test-ci test-unit test-integration test-e2e test-architecture test-snapshot test-backup-integrity snapshot-test snapshot-accept coverage coverage-html coverage-ci mutants audit deny security udeps watch-test watch-unit watch-integration cache-stats cache-reset test-performance-report testcontainers-clean makosh-lab mail-domain-closure-audit mail-domain-closure-gate vault-backup vault-restore clean clean-dev clean-validate clean-build clean-data clean-vault

help:
	@printf '%s\n' 'Макошь development commands:'
	@printf '%s\n' '  make docker-env    Create docker/.env from docker/.env.example when missing'
	@printf '%s\n' '  make docker-app-up Start Макошь backend and frontend in Docker Compose'
	@printf '%s\n' '  make docker-app-stop Stop the Docker Compose backend and frontend'
	@printf '%s\n' '  make docker-app-logs Tail Docker Compose backend and frontend logs'
	@printf '%s\n' '  make dev           Start PostgreSQL, backend watcher, and Vite dev server'
	@printf '%s\n' '  make dev-desktop   Run make dev plus the Tauri desktop shell window'
	@printf '%s\n' '  make logs          Tail the active live development log'
	@printf '%s\n' '  make build         Build backend, frontend, and Tauri release artifacts'
	@printf '%s\n' '  make migrate       Start PostgreSQL if needed and run backend-managed migrations'
	@printf '%s\n' '  make validate      Run architecture, backend, and frontend validation'
	@printf '%s\n' '  make frontend-visual Run Storybook visual regression screenshots'
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
	@printf '%s\n' '  make testcontainers-clean Remove Макошь-owned testcontainers'
	@printf '%s\n' '  make makosh-lab ACTION=readiness PROVIDER=zulip Run Макошь Lab provider action'
	@printf '%s\n' '  make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 Run scenario against local Zulip fixture'
	@printf '%s\n' '  make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 BACKEND=1 Run provider scenario plus backend live evidence'
	@printf '%s\n' '  make makosh-lab ACTION=compliance PROVIDER=zulip Generate Zulip Communication Compliance Suite report'
	@printf '%s\n' '  make makosh-lab ACTION=compliance PROVIDER=zulip BACKEND=1 Run backend contract evidence before report'
	@printf '%s\n' '  make mail-domain-closure-audit Report Mail closure gaps from gap-analysis.md'
	@printf '%s\n' '  make mail-domain-closure-gate Fail until every Mail gap row is implemented or ADR-excluded'
	@printf '%s\n' '  make vault-backup  Create a timestamped PostgreSQL + vault + mail-blob backup'
	@printf '%s\n' '  make vault-restore Interactively restore PostgreSQL, vault and mail blobs from a backup'
	@printf '%s\n' '  make clean         Remove build artifacts, temporary files, and logs'
	@printf '%s\n' '  make clean-dev     Remove dev watcher Cargo artifacts and local dev logs'
	@printf '%s\n' '  make clean-validate  Remove validation Cargo artifacts'
	@printf '%s\n' '  make clean-build   Remove release/Tauri build artifacts'
	@printf '%s\n' '  make clean-data    Delete local PostgreSQL data after confirmation'
	@printf '%s\n' '  make clean-vault   Delete local vault data after confirmation'

docker-env:
	@bash -lc 'source scripts/lib/env.sh; ensure_docker_env_file'

docker-app-config: docker-env
	@docker compose --env-file docker/.env --project-directory docker -f docker/docker-compose.yml --profile app config >/dev/null

docker-app-up: docker-env
	@docker compose --env-file docker/.env --project-directory docker -f docker/docker-compose.yml --profile app up -d --build backend frontend

docker-app-stop: docker-env
	@docker compose --env-file docker/.env --project-directory docker -f docker/docker-compose.yml --profile app stop backend frontend

docker-app-logs: docker-env
	@docker compose --env-file docker/.env --project-directory docker -f docker/docker-compose.yml --profile app logs -f backend frontend

dev:
	@./scripts/dev.sh

dev-desktop:
	@./scripts/dev.sh --desktop

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
	@node scripts/check-cargo-architecture.test.mjs
	@node scripts/check-cargo-architecture.mjs
	@node scripts/check-architecture.mjs --self-test
	@node scripts/check-architecture.mjs

code-boundaries-check:
	@node scripts/check-code-boundaries.mjs

backend-fmt-check:
	@cargo fmt --check --manifest-path backend/Cargo.toml

backend-clippy:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_CLIPPY_TARGET_DIR)" CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings

backend-test:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh default --workspace --all-targets
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite backend-full --output reports/test-performance/backend-full

backend-validate: backend-fmt-check backend-clippy backend-test

test-unit:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; export CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-1}"; disable_sccache_for_incremental; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --workspace --lib --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS)'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite unit --output reports/test-performance/unit

test-integration:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh integration $(foreach target,$(BACKEND_INTEGRATION_TARGETS),--test $(target))
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/integration/junit.xml --suite integration --output reports/test-performance/integration

test-e2e:
	@CARGO_TARGET_DIR="$(CARGO_VALIDATE_TEST_TARGET_DIR)" ./scripts/test/run-nextest.sh integration $(foreach target,$(BACKEND_E2E_TARGETS),--test $(target))
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/integration/junit.xml --suite e2e --output reports/test-performance/e2e

test-architecture:
	@$(MAKE) architecture-check
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; export CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-1}"; disable_sccache_for_incremental; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_ARCHITECTURE_TARGETS),--test $(target))'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite architecture --output reports/test-performance/architecture

test-snapshot: snapshot-test

snapshot-test:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; export CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-1}"; disable_sccache_for_incremental; CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_SNAPSHOT_TARGETS),--test $(target))'
	@node scripts/test/analyze-nextest-junit.mjs --input target/nextest/default/junit.xml --suite snapshot --output reports/test-performance/snapshot

snapshot-accept:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand nextest "cargo install --locked cargo-nextest"; NEXTEST_SHOW_PROGRESS="$${NEXTEST_SHOW_PROGRESS:-bar}"; export CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-1}"; disable_sccache_for_incremental; INSTA_UPDATE=always CARGO_TARGET_DIR="$(CARGO_VALIDATE_TARGET_DIR)" cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress "$${NEXTEST_SHOW_PROGRESS}" --test-threads $(MAKOSH_NEXTEST_JOBS) $(foreach target,$(BACKEND_SNAPSHOT_TARGETS),--test $(target))'

test-fast: test-backup-integrity test-unit test-architecture test-snapshot frontend-test

test-backup-integrity:
	@./scripts/test/vault-backup-integrity.test.sh

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
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand audit "cargo install --locked cargo-audit"; cargo audit $(CARGO_AUDIT_IGNORE_FLAGS)'

deny:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand deny "cargo install --locked cargo-deny"; cargo deny check'

security: audit deny

udeps:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand udeps "cargo install --locked cargo-udeps"; cargo +nightly udeps --workspace --all-targets'

watch-test:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand watch "cargo install --locked cargo-watch"; cargo watch -w backend -w crates -w scripts -w .config/nextest.toml -s "make test-fast"'

watch-unit:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand watch "cargo install --locked cargo-watch"; cargo watch -w backend -w crates -w scripts -w .config/nextest.toml -s "make test-unit"'

watch-integration:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_cargo_subcommand watch "cargo install --locked cargo-watch"; cargo watch -w backend -w crates -w scripts -w .config/nextest.toml -s "make test-integration"'

cache-stats:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_binary sccache "brew install sccache or cargo install --locked sccache"; sccache --show-stats'

cache-reset:
	@bash -lc 'source scripts/lib/rust-tooling.sh; require_binary sccache "brew install sccache or cargo install --locked sccache"; sccache --zero-stats'

test-performance-report:
	@./scripts/test/collect-performance-reports.sh

testcontainers-clean:
	@./scripts/test/clean-testcontainers.sh --all

makosh-lab:
	@node scripts/makosh-lab.mjs --provider "$(MAKOSH_LAB_PROVIDER)" $(MAKOSH_LAB_EXTRA_ARGS) "$(MAKOSH_LAB_ACTION)"

mail-domain-closure-audit:
	@node scripts/mail-domain-closure-audit.mjs

mail-domain-closure-gate:
	@node scripts/mail-domain-closure-audit.mjs --require-closed

frontend-lint:
	@cd frontend && pnpm lint

frontend-test:
	@cd frontend && pnpm test:unit

frontend-visual:
	@cd frontend && pnpm test:visual

frontend-visual-update:
	@cd frontend && pnpm test:visual:update

frontend-build:
	@cd frontend && pnpm build

frontend-validate: frontend-lint frontend-test frontend-visual frontend-build

vault-backup:
	@./scripts/vault-backup.sh

vault-restore:
	@./scripts/vault-restore.sh

clean:
	@./scripts/clean.sh

clean-dev:
	@rm -rf "$(CARGO_DEV_TARGET_DIR)"
	@rm -rf ".local/dev-logs"
	@rm -rf "frontend/node_modules/.vite" "frontend/node_modules/.vite-temp"

clean-validate:
	@rm -rf "$(CARGO_VALIDATE_TARGET_DIR)"
	@rm -rf "$(CARGO_VALIDATE_CLIPPY_TARGET_DIR)"
	@rm -rf "$(CARGO_VALIDATE_TEST_TARGET_DIR)"

clean-build:
	@rm -rf "$(CARGO_BUILD_TARGET_DIR)"
	@rm -rf "frontend/src-tauri/target"
	@rm -rf "frontend/dist" "frontend/build"
	@rm -f frontend/src-tauri/binaries/makosh-backend-*

clean-data:
	@./scripts/clean-data.sh

clean-vault:
	@./scripts/clean-vault.sh
