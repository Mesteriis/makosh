# Plugin Architecture

## Purpose

Plugins allow Макошь to extend providers, tools, document processors, UI panels and AI capabilities without turning the core system into an unbounded integration layer.

## Plugin Types

- provider adapters
- document processors
- search enrichers
- agent tools
- UI extensions
- export/import connectors
- automation workflows

## Capability Manifest

Each plugin must declare:

- name and version
- plugin type
- required permissions
- data classes accessed
- outbound network requirements
- commands exposed
- events emitted
- compatibility range

## Runtime Rules

- Plugins do not write directly to canonical tables.
- Plugins emit commands or candidate events through application boundaries.
- Plugins receive scoped data views.
- Plugins cannot access secrets except through named secret references.
- Plugin failures must be isolated and observable.

## Versioning

Plugin APIs require semantic versioning and compatibility checks. Breaking changes must be represented by ADR or migration notes when they affect durable data.

## Security

Plugins are untrusted by default. The system must prefer least privilege and make plugin permissions visible to the user before activation.
