SHELL := /usr/bin/env bash

RUST_TOOLCHAIN ?= 1.97.0
DOCKER ?= docker
LINUX_CI_IMAGE ?= rust:1.97.0-bookworm
LINUX_CI_PLATFORM ?= linux/amd64
TARGET ?= $(shell rustc +$(RUST_TOOLCHAIN) -vV | sed -n 's/^host: //p')

.PHONY: build test dev docker tauri clean \
	architecture-policy-check srp-policy-check cargo-boundaries-check \
	architecture-evidence-check \
	test-architecture fmt-check clippy test-workspace test-integration \
	dependency-policy-check supply-chain-check check-target check-linux-container ci \
	persons-postgres-conformance \
	mail-persons-sync-postgres-conformance \
	review-person-match-candidate-postgres-conformance \
	mail-persons-sync-managed-conformance \
	telegram-admission-preflight \
	persons-admission-cutover-conformance \
	restore-postgres-backup verify-release-reproducibility

build:
	cargo +$(RUST_TOOLCHAIN) build --locked --workspace
	cd ../frontend && pnpm build

test:
	node scripts/run-impacted-tests.mjs

dev:
	@./scripts/dev-ensemble.sh

docker:
	docker compose -f development/compose.yaml up --detach --wait

tauri: build
	cd ../frontend && pnpm exec tauri build

# Removes only reproducible Макошь build and test output. It intentionally
# preserves runtime data, Control Store state, Docker volumes and releases.
clean:
	cargo clean
	rm -rf -- ../frontend/dist ../frontend/coverage ../frontend/test-results ../frontend/playwright-report ../frontend/storybook-static ../frontend/node_modules/.cache ../frontend/node_modules/.vite

architecture-policy-check:
	node scripts/check-architecture-policy.mjs

architecture-evidence-check:
	node scripts/check-architecture-evidence.mjs

srp-policy-check:
	node scripts/check-srp-policy.mjs

cargo-boundaries-check:
	node scripts/check-cargo-boundaries.mjs

test-architecture:
	node --test tests/architecture/*.test.mjs

fmt-check:
	cargo +$(RUST_TOOLCHAIN) fmt --check

clippy:
	cargo +$(RUST_TOOLCHAIN) clippy --locked --workspace --all-targets -- -D warnings

test-workspace:
	cargo nextest run --locked --workspace --profile ci

test-integration:
	cargo nextest run --locked --workspace --profile integration

telegram-admission-preflight:
	cargo +$(RUST_TOOLCHAIN) --config 'build.rustc-wrapper=""' test --locked \
		-p makosh-kernel-recovery-testkit \
		managed_telegram_real_provider_prerequisites_are_exact -- \
		--ignored --test-threads=1

persons-postgres-conformance:
	@MAKOSH_PERSONS_POSTGRES_TEST_FILTER=all node scripts/test-authenticated-storage.mjs $(RUST_TOOLCHAIN)

mail-persons-sync-postgres-conformance:
	@MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_TEST_FILTER=all node scripts/test-authenticated-storage.mjs $(RUST_TOOLCHAIN)

review-person-match-candidate-postgres-conformance:
	@MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_POSTGRES_TEST_FILTER=all node scripts/test-authenticated-storage.mjs $(RUST_TOOLCHAIN)

mail-persons-sync-managed-conformance:
	@MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_persons_sync_actual_binary_bootstraps_exact_private_contour node scripts/test-authenticated-storage.mjs $(RUST_TOOLCHAIN)

persons-admission-cutover-conformance:
	@MAKOSH_STORAGE_MANAGED_TEST_FILTER=empty_start_provider_resync_without_contacts_import node scripts/test-authenticated-storage.mjs $(RUST_TOOLCHAIN)

dependency-policy-check:
	node scripts/check-dependency-policy.mjs
	cargo deny check bans licenses sources

supply-chain-check:
	cargo audit
	mkdir -p target/evidence
	syft dir:. --exclude './target/**' --exclude './.git/**' --output cyclonedx-json > target/evidence/backend-sbom.cdx.json
	test -s target/evidence/backend-sbom.cdx.json

restore-postgres-backup:
	node scripts/restore-postgres-backup.mjs --pg-restore "$(PG_RESTORE)" --psql "$(PSQL)" --connection-url-file "$(POSTGRES_CONNECTION_URL_FILE)" --input "$(POSTGRES_BACKUP_INPUT)"

verify-release-reproducibility:
	node scripts/verify-release-reproducibility.mjs --first-input "$(FIRST_RELEASE_INPUT)" --second-input "$(SECOND_RELEASE_INPUT)"

check-target:
	@test "$(TARGET)" = "aarch64-apple-darwin" -o "$(TARGET)" = "x86_64-unknown-linux-gnu" || { echo "unsupported CI target: $(TARGET)" >&2; exit 2; }
	@if test "$(TARGET)" = "x86_64-unknown-linux-gnu"; then $(MAKE) check-linux-container; fi

check-linux-container:
	$(DOCKER) run --rm --platform $(LINUX_CI_PLATFORM) --mount type=bind,src="$(CURDIR)",dst=/workspace,readonly --workdir /workspace -e CARGO_TARGET_DIR=/tmp/makosh-target $(LINUX_CI_IMAGE) bash -lc 'apt-get update && apt-get install -y --no-install-recommends clang libclang-dev protobuf-compiler >/tmp/apt.log && /usr/local/cargo/bin/rustup component add rustfmt && PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/$(RUST_TOOLCHAIN)-x86_64-unknown-linux-gnu/bin:$$PATH PROTOC=/usr/bin/protoc /usr/local/cargo/bin/cargo +$(RUST_TOOLCHAIN) check --locked --workspace --target x86_64-unknown-linux-gnu'

ci: check-target architecture-policy-check architecture-evidence-check srp-policy-check cargo-boundaries-check test-architecture fmt-check clippy test-workspace test-integration dependency-policy-check supply-chain-check
