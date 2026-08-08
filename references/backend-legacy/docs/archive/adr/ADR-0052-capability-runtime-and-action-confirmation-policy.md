# ADR-0052 Capability Runtime and Action Confirmation Policy

Status: Proposed

## Context

ADR-0027 selects a capability-based permission model, but intentionally leaves the runtime shape open. ADR-0038, ADR-0039 and ADR-0040 add temporary local API protection, access audit and actor identity for the current single-user desktop implementation.

V3, V4 and V5 now expose source-backed AI, Telegram automation dry-runs and WhatsApp Web companion capability reporting. Live provider writes, destructive actions, plugin execution and secret access still need a concrete policy boundary before they can move from blocked capability states to available runtime behavior.

## Decision

Implement the long-term capability runtime around a backend application-layer policy boundary.

Rules:

- Capability checks are centralized in the backend application boundary before privileged reads, local writes, provider writes, destructive actions, exports, secret resolution, automation execution or plugin tool calls.
- UI, agent and plugin clients may present intent, but they do not authorize their own actions.
- The temporary `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>` and `X-Макошь-Actor-Id` headers remain valid only as local-development and desktop bootstrap guards until the capability runtime replaces them with authenticated actor and capability identifiers.
- Capability decisions classify requested actions as `read`, `local_write`, `provider_write`, `destructive`, `export`, `secret_access` or `automation`.
- Capability grants are scoped. Scopes may include actor, provider account, channel/chat/thread, project, document, data class, command, template, automation policy, time window, rate limit and expiry.
- Message sends, provider mutations, deletes, destructive local changes, sensitive exports and direct secret access require explicit confirmation unless an enabled scoped automation policy authorizes the action.
- Automation policies are never open-ended. They must bind account, destination scope, template, trigger, rate limit, quiet hours and expiry.
- AI may fill declared template variables inside an authorized policy, but it cannot choose destination, account, template, policy authority or send scope from retrieved content.
- Allowed and rejected high-risk actions write audit metadata with actor, capability, action class, target scope, policy/template identifiers where relevant, decision, reason and correlation ID.
- Audit metadata must not store API tokens, provider credentials, private message bodies, document contents, pairing codes or local browser profile paths containing private state.
- For external side effects and destructive actions, audit insertion is fail-closed: if the audit record cannot be written, the action is not executed.
- Plugins are untrusted by default. Plugin activation requires a declarative capability manifest, user-visible permissions and scoped data views. Plugins cannot access raw secrets or canonical tables directly.

## Consequences

Positive:

- Live Telegram, WhatsApp, plugin and future provider actions get one policy model instead of separate ad hoc confirmation checks.
- The UI can expose clear capability and confirmation states without being the source of authority.
- Audit records can explain why high-risk actions were allowed or rejected without leaking private content.
- Temporary local token and actor headers remain usable during development while their replacement boundary is defined.

Negative:

- Capability evaluation becomes security-critical application infrastructure.
- Provider adapters and agent tools must route through the policy boundary before side effects.
- Existing V4/V5 blocked live capabilities remain blocked until the runtime, persistence, UI and validation are implemented.

Risk handling:

- Do not enable live outbound sends, deletes, provider mutations, sensitive exports or plugin side effects until capability checks and high-risk audit records are covered by regression tests.
- Treat confirmation text, retrieved content and plugin manifests as untrusted input.
- Keep secret values out of capability grants, audit records, event payloads and test fixtures.

## Non-Goals

- Multi-user remote access.
- Cloud identity provider integration.
- Third-party plugin code execution.
- Replacing ADR-0038, ADR-0039 or ADR-0040 in the current local bootstrap implementation.
