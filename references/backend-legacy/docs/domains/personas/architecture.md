# Personas — Intelligence Architecture

This document describes the target architecture for the `personas` domain after
the Persona refactoring. It is a domain architecture document, not a statement
that every backend table or route has already been migrated.

## Architectural Position

Макошь is a local-first Personal Memory System. The Personas domain owns Persona
Intelligence: the structures that allow Макошь to understand subjects, remember
relationships and build context over time.

The domain sits between raw evidence and user-facing memory:

```text
provider records / documents / messages / calendar events
  -> canonical events and append-only source records
  -> identity resolution
  -> Personas and Relationships
  -> Memory records, Timeline Engine views and Dossier read models
  -> UI, agents, search and graph queries
```

## Core Boundaries

| Boundary | Owns | Does not own |
|---|---|---|
| Persona | Subject identity, type and lifecycle | Raw provider payloads |
| Identity Resolution | Digital trace matching, merge/split candidates | Silent ambiguous merges |
| Relationships | Persona-to-Persona edges, trust, strength and provenance | Relationship fields embedded on Persona |
| Memory | Facts, knowledge, preferences, memory cards and conflicts | Uncited AI claims |
| Timeline Engine use | Time-ordered views connected to Personas and Relationships | A separate Persona-owned timeline engine |
| Dossier read model | Generated context and preparation brief | New source-of-truth facts |
| Persona Intelligence | Pattern extraction, observations and scoring | CRM pipeline logic |

## Persona

`Persona` is the root aggregate for a subject in Макошь.

```yaml
Persona:
  id:
  is_self:
  persona_type:
    - human
    - ai_agent
    - organization_proxy
    - system
  identity:
  communication:
  memory:
  timeline_view:
  relationships:
  dossier_read_model:
```

The current implementation uses `/personas` routes and Persona-native read
payloads over `personas` / `persona_*` storage with `persona_id` identifier
columns. New documentation and future schema work must use Persona as the domain
concept.

## Self Persona

There is one and only one `Persona` with `is_self: true`. It represents the owner
of the local Макошь instance.

Consequences:

- no separate `UserProfile` domain;
- no separate Self domain;
- local agents act through the Owner Persona;
- audit and provenance records can attribute user-owned actions to the Owner
  Persona when appropriate;
- generated memory about the owner is stored as Persona memory, not as app
  settings.

## Identity Resolution

Identity Resolution merges digital traces into a Persona candidate, not into a
legacy address-book row.

Supported traces include:

- email addresses;
- phone numbers;
- Telegram identities;
- WhatsApp identities;
- GitHub accounts;
- LinkedIn profiles;
- document mentions;
- message participants;
- future provider-specific handles.

Ambiguous matches create reviewable candidates. They must not be collapsed
silently. This preserves the ADR-0019 safety property while replacing the old
legacy address-book-row framing.

## Relationship First

Relationships are primary records:

```yaml
Relationship:
  id:
  source_persona:
  target_persona:
  type:
  trust_score:
  strength_score:
  provenance:
  confidence:
  valid_from:
  valid_to:
```

Relationship examples:

- Owner Persona collaborates with a human Persona.
- Human Persona works with an organization proxy Persona.
- AI agent Persona assists the Owner Persona.
- System Persona produced an automated observation.

Do not model relationships as `primary_role`, `organization_reference` or other
Persona-root fields. `watchlist` and `health_status` may exist only as
temporary UI/risk compatibility projections until the schema/API migration
retires those names.

## Memory First

Persona memory contains structured, cited records:

```yaml
PersonaMemory:
  facts:
  knowledge:
  preferences:
  memory_cards:
  conflicts:
```

Memory records must carry source, confidence and verification metadata. AI can
propose memory, detect conflicts or produce observations, but AI output remains
derived state unless reviewed and stored as evidence-backed memory.

## Timeline Engine Use

The Timeline Engine explains how a Persona or Relationship changed over time.
The Personas domain may contribute dated records, but it does not own a separate
Timeline subsystem.

```yaml
TimelineEvent:
  id:
  persona_id:
  relationship_id:
  event_type:
  occurred_at:
  summary:
  source_refs:
  confidence:
```

The existing `relationship_events` table is a transitional projection. The target
model separates Relationship records from dated events and uses the shared
Timeline Engine to present them.

## Dossier

The dossier is generated, not manually maintained.

```yaml
Dossier:
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
  source_refs:
  generated_at:
```

The dossier may be cached as a read model, but the source of truth remains
Persona identity, relationships, memory, timeline, graph evidence and provider
records.

## Persona Intelligence

Persona Intelligence replaces the fragmented legacy terms:

| Old term | New concept |
|---|---|
| communication fingerprint | communication patterns |
| communication profile | Persona communication intelligence |
| trust analytics | relationship intelligence |
| health status | relationship attention or Risk Engine signal |
| watchlist | UI attention preference |
| investigator | dossier and context assembly |
| analytics | Persona Intelligence read models |

The Persona Intelligence layer should be implemented through domain services and
shared engines:

- `IdentityResolutionService`
- `RelationshipIntelligenceService`
- `PersonaMemoryService`
- `DossierAssembler`
- `CommunicationPatternService`
- `PersonaObservationService`

Names above are architectural roles, not a mandate to create these exact Rust
modules in one migration.

## Compatibility With Current Backend

The current Rust backend already contains useful implementation pieces, but they
carry legacy names and CRM-shaped fields.

| Current artifact | Target interpretation |
|---|---|
| `personas` table | Persona projection table |
| `persona_identities` | Persona digital traces |
| `persona_identity_candidates` | Identity resolution review candidates |
| `persona_roles` | Deprecated; replace with Relationships |
| `persona_interaction_contexts` | Compatibility interaction-context storage renamed from legacy `person_personas`; Persona remains the root entity |
| `relationship_events` | Transitional dated event projection consumed by Timeline Engine |
| `persona_facts` | Persona facts |
| `persona_memory_cards` | Persona memory cards |
| `persona_preferences` | Persona preferences |
| `persona_knowledge_conflicts` | Persona memory conflicts |
| `persona_expertise` | Persona skills and knowledge signals |
| `persona_promises` | Commitment facts or timeline events |
| `persona_risks` | Persona/relationship observations requiring evidence |
| `health_status` | Deprecated Risk/attention cache; risk writes materialize this projection |
| `watchlist` | Deprecated UI/read-model cache; writes materialize Persona Preferences |
| `/api/v1/persons` | Retired legacy API; active Persona profile surfaces use `/api/v1/personas/*` |

Any implementation migration must preserve event sourcing, graph provenance and
reviewed identity resolution. Identifier/schema compatibility changes still
require a separate migration plan.

## Source of Truth

Source-of-truth order inside the domain:

1. Canonical events and append-only provider records.
2. Persona, Identity and Relationship records with provenance.
3. Memory records and dated events derived from reviewed evidence.
4. Dossier and Persona Intelligence read models.
5. Search indexes, embeddings and UI projections.

AI output, embeddings, dossiers and analytics are derived state. They are useful
for context, but they cannot become the source of truth for private memory.

## ADR

Relevant ADR:

- [ADR-0084 Persona Intelligence System](../../archive/adr/ADR-0084-persona-intelligence-system.md)
- [ADR-0001 Event Sourcing as System Spine](../../archive/adr/ADR-0001-event-sourcing-as-system-spine.md)
- [ADR-0008 Knowledge Graph First](../../archive/adr/ADR-0008-knowledge-graph-first.md)
- [ADR-0022 No Fine Tuning on Private Data](../../archive/adr/ADR-0022-no-fine-tuning-on-private-data.md)
- [ADR-0057 Person Memory and Provenance](../../archive/adr/ADR-0057-person-memory-and-provenance.md)
- [ADR-0058 Person Enrichment Engine](../../archive/adr/ADR-0058-person-enrichment-engine.md)
- [ADR-0060 Person Timeline and Graph Integration](../../archive/adr/ADR-0060-person-timeline-and-graph-integration.md)
- [ADR-0074 Persona Multi-Channel Identity Compatibility](../../archive/adr/ADR-0074-persona-multi-channel-identity-compatibility.md)
