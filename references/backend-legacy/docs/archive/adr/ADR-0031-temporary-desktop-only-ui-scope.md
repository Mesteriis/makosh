# ADR-0031 Temporary Desktop Only UI Scope

Status: Temporary

## Context

Макошь is currently defined as a desktop-first personal productivity system for PC and laptop use. Mobile UI introduces a separate product surface with different constraints: small-screen navigation, touch-first interactions, mobile OS permissions, background sync, notification behavior and mobile QA breakpoints.

ADR-0026 keeps responsive behavior for desktop window resizing and future web optionality, but it does not require mobile product design.

## Decision

Until this ADR is superseded, Макошь will not design, implement or validate a mobile UI.

Product, UX and frontend architecture work target PC and laptop layouts only. Responsive behavior means usable desktop resizing, not phone or tablet workflows.

## Consequences

- Mobile viewports may be incomplete or unusable; this is accepted temporary scope.
- No mobile navigation model, touch-first workflow, mobile breakpoint matrix or mobile packaging is required.
- UI documentation and future implementation must not claim mobile support.
- Future mobile support requires a new ADR defining goals, target devices, interaction model, data sync assumptions and validation requirements.
