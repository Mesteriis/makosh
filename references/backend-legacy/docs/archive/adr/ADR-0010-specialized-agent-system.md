# ADR-0010 Specialized Agent System

Status: Proposed

## Context

A single generic assistant would blur responsibility and make permissions difficult to reason about.

## Decision

Use specialized agents: HESTIA, MAKOSH, MNEMOSYNE, ATHENA and HEPHAESTUS.

## Consequences

- Agent responsibilities are easier to explain and constrain.
- Tool permissions can be scoped by role.
- HESTIA must coordinate without becoming a hidden god object.
- Agent interactions require audit events.
