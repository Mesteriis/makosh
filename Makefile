SHELL := /usr/bin/env bash

.DEFAULT_GOAL := help

.PHONY: help build test dev docker tauri clean pre-commit pre-push

help:
	@printf '%s\n' 'Макошь development commands:'
	@printf '%s\n' '  make build   Build the clean-room backend and browser client'
	@printf '%s\n' '  make test    Run tests impacted by the current changes'
	@printf '%s\n' '  make dev     Start the full local stack at http://127.0.0.1:5173'
	@printf '%s\n' '  make docker  Start the local PostgreSQL, PgBouncer and NATS contour'
	@printf '%s\n' '  make tauri   Build the desktop application'
	@printf '%s\n' '  make clean   Remove reproducible Макошь build and test output'
	@printf '%s\n' '  make pre-commit  Run the fast local commit gate'
	@printf '%s\n' '  make pre-push    Run the full backend and frontend gate'

build test dev docker tauri clean:
	@$(MAKE) -C backend $@

pre-commit:
	@$(MAKE) -C backend architecture-policy-check architecture-evidence-check srp-policy-check cargo-boundaries-check test-architecture fmt-check
	@cd frontend && pnpm lint
	@cd frontend && pnpm typecheck

pre-push:
	@$(MAKE) -C backend ci
	@cd frontend && MAKOSH_STORYBOOK_PORT=6007 pnpm validate
