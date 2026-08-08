# ADR-0004 Tauri Desktop Shell

Status: Proposed

## Context

Макошь is local-first and needs desktop integration for files, local services, secret storage, notifications and possible provider bridge workflows.

## Decision

Use Tauri as the desktop shell.

## Consequences

- Rust backend and desktop bridge can share technology.
- The app can remain lighter than Electron-based alternatives.
- Tauri command boundaries must be narrow and validated.
- OS-specific behavior must be isolated behind clear ports.
