# Telegram Product Research And Next Bets

Status: research snapshot, 2026-06-23.

Scope: exploratory product research for the next Telegram-adjacent work after
the base channel capability set. This document is not an ADR, implementation
plan or committed roadmap. Treat it as input for future specs, ADRs and
provider-neutral Communications work.

## Local Context

Telegram in Макошь is an integration channel, not a product domain. ADR-0097
keeps durable communication state inside the Communications domain. ADR-0099
keeps source control, pause/resume/replay and source health inside Signal Hub.

The useful next product work should therefore avoid Telegram-client parity.
Telegram can be the first rich evidence source for provider-neutral
Communications, Radar, Memory, Persona, Relationship, Obligation and Decision
workflows.

Observed local surfaces at the time of this research:

- backend Telegram integration covers account setup and lifecycle, chat
  metadata and reconciliation, commands, evidence, messages, attachments,
  manual send, reaction metadata, references, search, topics, participants,
  TDLib parsing, runtime actors, sync, media download and realtime events;
- frontend Telegram work exists inside the Communications workbench through
  Telegram panels, business queries and realtime patches for messages, media,
  topics and participants;
- base channel status and gap documents mark the core Telegram capability set
  as complete for daily desktop work.

## External Product Patterns

### Telegram

Sources:

- <https://telegram.org/blog/folders>
- <https://telegram.org/blog/shareable-folders-custom-wallpapers>
- <https://telegram.org/blog/new-saved-messages-and-9-more>
- <https://telegram.org/blog/reply-revolution>
- <https://telegram.org/blog/ultimate-privacy-topics-2-0>

Useful patterns:

- Chat folders separate noisy chat lists into work, unread, channel and other
  focused views.
- Shareable folders package a set of groups and channels behind an invite link.
- Saved Messages behaves like personal storage for links, media, bookmarks and
  tagged saved items.
- Replies can quote precise fragments and move context across chats.
- Topics organize high-volume groups, while privacy and deletion controls make
  provider history mutable.

Макошь interpretation:

- folders and topics are useful as local view and workflow inputs, not as a
  reason to build a Telegram clone;
- saved messages map well to a source-backed evidence shelf with tags,
  provenance and later Memory or Document promotion;
- quote fragments are a strong primitive for stable evidence capture across
  noisy conversations;
- provider edit/delete behavior should become local evidence, tombstones and
  history, not silent loss of truth.

### Beeper

Source:

- <https://www.beeper.com/faq>

Useful patterns:

- unified inbox across many chat networks;
- cross-device sync and a strong "one app for chats" posture;
- emphasis on trust, encryption and local/on-device connection where possible.

Макошь interpretation:

- unified inbox is useful, but Макошь should differentiate on local-first
  evidence, auditability and provider-neutral context rather than chat-client
  aggregation alone.

### Missive

Source:

- <https://missiveapp.com/docs/core-features/team-inboxes>

Useful patterns:

- shared inbox triage;
- assignment, transfer, close and reopen workflows;
- conversation ownership and status as first-class working state.

Макошь interpretation:

- the team-inbox model can be adapted to solo/operator workflows: assign to
  self, defer, close, reopen, track owner state and surface unresolved
  obligations without turning Макошь into a help desk.

### Superhuman

Source:

- <https://superhuman.com/products/mail>

Useful patterns:

- split inbox for priority lanes;
- reminders and follow-up workflows;
- keyboard-first triage;
- AI drafting and live sharing/commenting.

Макошь interpretation:

- priority lanes and follow-up obligations are higher-value than more raw
  Telegram UI parity;
- reply drafts can be useful, but they must stay behind explicit user
  confirmation and capability gates.

### Shortwave

Sources:

- <https://www.shortwave.com/>
- <https://www.shortwave.com/changelog/>

Useful patterns:

- AI filters written in plain English;
- AI-powered search over messages and attachments;
- automated tasklets for drafts, labels, comments and todo extraction.

Макошь interpretation:

- start with local suggestions, drafts, classifications and candidate
  obligations;
- do not allow automated provider writes or destructive provider-side actions
  without explicit confirmation.

## Research Themes

1. Unified Telegram inbox work should not imitate Telegram. It should expose
   workable conversations across accounts with local overlays: priority, owner
   state, obligation, trust, source evidence and runtime posture.
2. Folders, topics and saved messages are not only UI categories. In Макошь
   they can become local saved/search/task/memory surfaces with provenance.
3. Follow-up and reminder workflows fit Макошь better than generic chat parity.
4. AI should draft, summarize, classify and suggest actions locally first.
   Provider writes stay manually confirmed.
5. Team-inbox ideas can be reduced to solo/operator workflows: assign to self,
   defer, close, reopen, "needs decision" and "waiting".
6. Privacy and evidence boundaries are product differentiators: local
   tombstones, edit/version history, no hidden provider mutation and clear
   capability states.

## Consensus Next Bets

### 1. Context Inbox Lanes

Build provider-neutral lanes such as:

- Now;
- Waiting;
- Needs reply;
- Decision needed;
- Snoozed;
- High-trust;
- Noise.

Start with local owner-state overlays. Add Persona and Relationship-derived
lanes after identity quality is good enough.

Validation ideas:

- dogfood whether "Now" replaces scanning "All";
- fixture tests for snooze, reopen and realtime wake behavior.

### 2. Obligation And Decision Radar

Generate evidence-backed candidates from Telegram messages:

- "I owe";
- "they owe";
- "follow up";
- "decision pending".

The owner reviews and promotes candidates. Макошь must not automatically create
tasks, obligations or decisions from private messages without review.

Validation ideas:

- candidate acceptance rate;
- false-positive review cost;
- regression fixtures that separate casual chat from explicit commitments.

### 3. Source-Backed Evidence Capture

Allow saving a message, quote fragment, link, media item or file into a local
evidence shelf with:

- source citation;
- version or content hash where available;
- tags;
- Persona or project context;
- optional Memory or Document promotion.

Validation ideas:

- saved item retrieval through search;
- quote stability across edited or deleted messages;
- scanner-state and storage-boundary tests for media.

### 4. Forensic Conversation Timeline

Surface local evidence that normal chat clients hide:

- observed edit versions;
- tombstones;
- provider deletions;
- local delete reasons;
- source citations;
- scoped export.

This is a Макошь-native differentiator, but it needs careful privacy language
and UI framing.

Validation ideas:

- edit/delete fixture matrix;
- export scope tests;
- UX review for "valuable evidence" versus surprising retention.

### 5. Signal Hub Control And Runtime Posture

Expose source posture as enabling infrastructure:

- enabled;
- paused;
- muted;
- degraded;
- replaying;
- unhealthy;
- fixture mode.

Telegram account runtime health should appear here, but source policy belongs
to Signal Hub, not to a Telegram product domain.

Validation ideas:

- fixture source pause/resume/replay;
- disabled Telegram source stops projections without losing raw evidence;
- degraded source state is visible without implying data loss.

## Preserved Disagreements

- Persona Inbox is strategically useful, but it belongs to provider-neutral
  Communications plus Persona and Relationship systems, not to a Telegram
  module.
- Context Inbox Lanes are the better first slice because local owner state can
  ship before fuzzy identity resolution is perfect.
- AI reply drafts are useful but are not the primary Telegram bet. Keep them
  behind AI/runtime capability gates and explicit confirmation.
- Topics, media gallery, search, folders and reactions remain important
  integration polish. They are not the headline next product direction if the
  base Telegram status documents are accurate.

## Explicit Rejects

- Telegram clone parity as a product goal;
- automated live sends;
- provider-side destructive automation;
- mobile UI before ADR-0031 is superseded;
- calls, recording and screen sharing as part of this next slice;
- auto-download-all-media behavior;
- cloud indexing of private messages;
- physical delete by default;
- Telegram-owned task, decision or memory state.

## Open Questions

- What is the canonical provider-neutral schema for lane and workflow state?
- How should Persona resolution represent groups, channels, bots and
  organization proxies?
- What evidence snapshot is required for quote fragments across edited or
  deleted messages?
- What retention and privacy language makes forensic history understandable
  without surprising the owner?

## Research Provenance

This snapshot combines:

- repository context from Telegram status and gap documentation;
- ADR constraints from ADR-0097 and ADR-0099;
- external product research into Telegram, Beeper, Missive, Superhuman and
  Shortwave;
- a dual-model brainstorm and debate pass using Codex and Claude Code.
