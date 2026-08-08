# ADR-0011 Plugin Architecture

Status: Proposed

## Context

Макошь will need new providers, processors, tools and UI extensions over time. Hardcoding all integrations into core will not scale.

## Decision

Introduce a plugin architecture with manifests, explicit capabilities and bounded runtime access.

## Consequences

- Integrations can evolve outside the core.
- Permissions become visible and enforceable.
- Plugin sandboxing is a security-critical design area.
- The core must expose stable extension points.
