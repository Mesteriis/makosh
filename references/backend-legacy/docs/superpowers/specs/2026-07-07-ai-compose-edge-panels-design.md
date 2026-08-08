# AI Compose Edge Panels Design

Date: 2026-07-07
Status: Approved visual direction, awaiting implementation plan approval

## Goal

Redesign the mail compose modal into an intelligent writing surface without turning
the editor into a settings screen or chat-first interface.

The compose editor remains the primary object. AI, templates, signatures,
recipient checks, and message context appear as side tools that can be opened on
demand.

## Approved Interaction Model

The compose modal uses two side panels:

- Left panel: AI writing actions.
- Right panel: contextual tools.

Both panels behave like edge handles:

- In the closed state, only a narrow vertical handle with icons is visible.
- Icons belong to the panel, not to the central compose card.
- When a panel opens, its handle moves together with the panel.
- The panel slides out from behind the compose window, visually like a background
  wing.
- The central compose card stays above the panels and remains the main focus.
- Open panels do not cover the editor, account selector, address fields, toolbar,
  or footer buttons.

This matches the selected “Samsung edge-style” mental model: a side surface with
its own handle, not an overlay that steals the writing area.

## Layout

The modal consists of three layers:

1. Modal overlay and backdrop.
2. Background side panels.
3. Foreground compose card.

The foreground card contains:

- sender account selector;
- recipients: To, Cc, Bcc;
- subject;
- rich text toolbar;
- rich text editor;
- footer actions: save draft and send.

The side handles are attached to the side panels and sit just outside the compose
card edge. They are always reachable while the modal is open.

## Left Panel: AI Actions

The left panel is the writing assistant surface.

Initial actions:

- prompt-to-email;
- rewrite selected draft;
- improve tone;
- translate;
- autocorrect;
- generate reply variants when compose was opened from a message;
- apply generated subject/body explicitly.

AI output must never silently mutate the draft. Every AI result needs an explicit
user action:

- replace draft;
- insert at cursor;
- append below;
- apply subject only;
- discard.

## Right Panel: Context Tools

The right panel is the contextual support surface.

Initial sections:

- templates;
- signatures;
- recipient review;
- security/authentication checks;
- thread context when composing a reply;
- attachments and evidence hints when available.

This panel should not display internal ids as user-facing labels. Account ids,
raw provider ids, and backend refs are diagnostic data and should stay out of the
normal compose surface.

## Account Selection

Compose must allow choosing among all mail-capable accounts available to Макошь.

The sender selector should display user-facing account labels and addresses, not
technical account ids. If multiple accounts are available, the selector is shown
by default. If only one account is available, the account is still visible but
does not dominate the form.

Sending and draft saving must fail early in the UI if no sender account is
selected.

## Responsiveness

Desktop is the primary target for this interaction.

On wide viewports:

- both side handles may be visible at once;
- one or both panels may be open if there is enough horizontal space;
- the compose card should not resize when panels open.

On constrained widths:

- only one panel should be open at a time;
- panel width may shrink to a compact variant;
- the compose card remains usable with a stable editor width;
- if the viewport is too narrow for side wings, the panel may become a side sheet,
  but it still uses the same handle ownership model.

## Data And API Boundaries

The first implementation should reuse existing Communications compose state and
rich text editor behavior.

Expected existing surfaces to integrate with:

- compose form model and send/draft request mapping;
- rich text editor component;
- mail account list/query;
- existing AI reply and reply variant actions where already available;
- existing templates and signature APIs where already available.

No provider-specific business cache root should be introduced. Compose remains a
Communications surface.

## Error Handling

User-facing failures should use the global notification mechanism.

Inline compose errors are acceptable only when they point to a field the user can
fix immediately, such as missing sender, missing recipient, invalid address, or
empty body.

Backend/internal messages should be translated into user-facing text before they
reach the compose modal.

## Accessibility

Minimum behavior:

- side handles are keyboard-focusable buttons;
- each icon has a tooltip and accessible label;
- opening a panel preserves focus intentionally;
- Escape closes the active panel first, then closes the modal only when no panel
  is open;
- Tab order stays inside the modal while it is open;
- AI apply/discard actions are reachable by keyboard.

## Non-Goals For The First Slice

- No new provider send pipeline.
- No silent AI edits.
- No persistent automation rules from compose.
- No new durable business entities from drafted email content.
- No redesign of the message reader.

## Validation Expectations

Before claiming implementation complete:

- frontend typecheck/build must pass or failures must be reported exactly;
- the compose modal must be manually checked with one and multiple mail accounts;
- panel closed/open states must be checked visually;
- sender account id must be present in send/draft requests;
- missing-account errors must be handled before backend submission where possible.
