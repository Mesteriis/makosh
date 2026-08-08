# Signal Hub Domain

Signal Hub is the domain that owns the durable registry and control state for
all signal sources in Макошь.

It exists because Макошь is a memory system that receives evidence from many
places, not a collection of provider apps. Email, Telegram, WhatsApp, GitHub,
Browser capture, RSS, Calendar, Filesystem, Home Assistant and fixtures are all
sources of signals.

## Owns

Signal Hub owns:

- SignalSource;
- SignalConnection;
- SignalCapability;
- SignalRuntime;
- SignalHealth;
- SignalPolicy;
- SignalProfile;
- SignalReplayRequest;
- system recovery fixture definitions;
- fixture source catalog metadata.

## Does Not Own

Signal Hub does not own:

- provider protocol code;
- provider secrets;
- raw private message bodies;
- Communication messages or conversations;
- Tasks;
- Personas;
- Documents;
- Calendar event source-of-truth state;
- Radar review lifecycle;
- Knowledge Graph relationships.

## Flow

```text
Signal Source
  -> Signal Hub policy/runtime control
  -> Event Backbone
  -> Owning domain consumer
  -> Projection / Memory / Knowledge
```

Communication sources follow:

```text
signal.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
  -> review.item.promoted
  -> owning domain command
```

## Domain Rules

- Signal Hub emits events; it does not mutate other domains.
- Integration adapters ask Signal Hub for control state, but remain protocol
  owners.
- Signal Hub state is non-secret and policy-oriented.
- Recovery fixtures are canonical-code based and schema-agnostic.
- Fixture sources are first-class test sources.
- Mute, pause, resume and replay are core domain behavior, not testing hacks.

## Canonical Documentation

Detailed implementation docs live in [Signal Hub](README.md).
