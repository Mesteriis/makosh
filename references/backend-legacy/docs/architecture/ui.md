# Canonical UI Architecture

Status: Canonical architecture baseline for the 2026-06-18 documentation
consolidation.

Scope: UI architecture and operating-surface principles. This document does not
authorize frontend refactoring by itself.

## Purpose

The UI is the Personal Operating System surface over Макошь memory and context.
It is not a collection of provider clones.

## Responsibility

The UI is responsible for:

- exposing context-rich workspaces;
- making evidence and provenance visible;
- supporting dense desktop workflows;
- surfacing review queues and capability states;
- enabling fast navigation across Personas, Organizations, Communications,
  Projects, Documents, Tasks, Decisions, Obligations and Timeline;
- showing agent outputs as generated and cited;
- preventing unavailable or unsafe provider actions from appearing as working
  commands.

## Boundaries

The UI does not own:

- durable domain state;
- provider credentials;
- capability authority;
- source evidence truth;
- server-derived cache truth;
- AI conclusions.

Durable owner changes must go through backend APIs and owning domain commands.

## Platform Baseline

Current accepted frontend baseline:

- Vue 3;
- TypeScript;
- Vite;
- Tauri 2;
- Pinia for transient UI state;
- TanStack Query for server state;
- generated ConnectRPC/Protobuf clients through one transport factory;
- one replayable SSE stream per active client process;
- Tauri host bridge only for desktop/OS capabilities;
- planned Android first-party client using the same public contracts.

Android UI framework is not selected by this document. Desktop and Android may
use different presentation technology, but neither creates separate business
API or domain ownership.

SvelteKit-specific ADRs are historical and superseded.

## Surface Model

Primary UI surfaces:

- Home and operating overview;
- Communications workspace;
- Telegram channel workbench;
- WhatsApp channel workbench when implemented;
- Personas workspace;
- Organizations workspace;
- Projects workspace;
- Documents workspace;
- Tasks and Obligations views;
- Review workspace;
- Knowledge/Graph/Memory exploration;
- Timeline;
- Agents;
- Settings and capability control.

These are operating surfaces. They do not imply independent backend domains.

## State Model

| State kind | Owner |
|---|---|
| Durable domain state | Backend domain APIs. |
| Server-derived state | TanStack Query. |
| Transient UI state | Pinia or component-local state. |
| Draft state | UI plus backend draft APIs where durable. |
| Realtime patches | Shared platform bootstrap and query invalidation. |
| Capability state | Backend capability contract. |
| Agent workflow state | Agent run/proposal APIs. |

Direct component-level API calls for server-derived state remain prohibited by
the Vue architecture direction.

## Interaction Rules

- Desktop may use dense keyboard-first layouts. Android uses mobile-appropriate
  navigation and interaction without changing backend contracts.
- Keyboard-first workflows and command palette remain target UI patterns.
- Provider channel workbenches may look familiar, but they must show Макошь
  evidence, review, context and capability semantics.
- Message bodies, document contents and private data must not leak into audit,
  telemetry or unsafe logs.
- AI-generated text must be visually and semantically distinct from source
  evidence.
- Review surfaces must show target owner domain and promotion result.

## Reasons For Existence

Макошь needs an operating surface because memory without action is passive, and
action without evidence is unsafe. The UI ties context, review and owner
decisions together without letting provider surfaces redefine the product.
