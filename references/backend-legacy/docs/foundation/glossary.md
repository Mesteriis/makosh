# Макошь Glossary

This glossary is the canonical vocabulary for active Макошь documentation.
Historical ADR and implementation plans may contain older terms; new documents
should use the definitions below.

## Agent

A software actor that uses typed tools, policy checks and source-backed context
to help the owner. Agents are Personas of type `ai_agent` when represented in the
graph. Agents are not sources of truth.

## Communication

An interaction between participants through a channel such as email, Telegram,
WhatsApp, calls or meetings. Communication is the canonical domain concept;
Email, Telegram and WhatsApp are provider/channel shapes, not separate product
identities.

## Context

The assembled explanation around an entity or situation: source evidence,
relationships, timeline, relevant memory, decisions, obligations, risks and open
questions. Context is the primary product value.

## Decision

A durable record that a choice was made. A Decision must link to source evidence
and the entities it affects. Decisions are primary memory records, not AI
summaries.

## Dossier

A generated read model for a Persona, Organization, Project or other context
anchor. A Dossier summarizes identity, relationships, interests, projects,
organizations, skills, communication patterns, observations and source
references. A Dossier is derived state, not source of truth.

## Document

An imported or created artifact with versions, extracted content, metadata and
links to other entities. A Document is source evidence. Notes are lightweight
document-like artifacts unless a future ADR creates a separate Notes domain.

## Domain

A bounded context that owns source-of-truth entities, lifecycle rules and
invariants. Domains may use engines, but engines do not own domain entities.

## Engine

A reusable system mechanism that operates across domains. Examples: Memory
Engine, Timeline Engine, Trust Engine, Search Engine, Enrichment Engine,
Obligation Engine, Risk Engine and Consistency / Contradiction Engine. Engines
produce projections, observations or scores; they do not replace domain
ownership.

## Consistency / Contradiction Engine

A reusable engine that compares new evidence with accepted Memory and Knowledge.
It creates source-backed contradiction observations and review items when claims
conflict. User-facing alias: Polygraph.

## Enrichment

The process of proposing additional information from approved sources.
Enrichment output is a candidate or observation until reviewed or otherwise
accepted under domain rules.

## Event

A meaningful thing that happened. Events are append-only facts used to rebuild
projections, timelines, graph links and indexes. Calendar events are scheduled
events; canonical events are system facts in the event log.

## Follow-Up

A prompt to revisit something. A Follow-Up is not always a Task. It becomes a
Task only when it has a concrete action and lifecycle. It becomes an Obligation
only when there is a commitment or expected duty.

## Knowledge

Evidence-backed understanding stored by Макошь: facts, relationships,
decisions, observations and reviewed summaries. Knowledge is not a loose wiki.
It is built from domain records and provenance.

## Memory

Durable, source-backed information that Макошь keeps over time. Memory includes
events, relationships, facts, decisions, obligations, document evidence and
curated knowledge. Memory is not an LLM weight, cache or unverified summary.

## Note

A lightweight captured artifact. A Note is treated as a Document or memory input
unless a future ADR makes Notes a first-class domain. Notes are not a separate
source of truth in the current model.

## Obligation

A commitment, duty or expected action that arose from communication, a document,
a decision, a meeting or manual entry. An Obligation may generate Tasks or
Follow-Ups, but it is not identical to either.

## Organization

A durable entity representing a company, institution, agency, community or
similar collective actor. Organizations are not fields on Personas or Projects.

## Owner Persona

The single Persona with `is_self: true`. It represents the owner of the local
Макошь instance. There is no separate Self domain or User Profile.

## Persona

A durable digital representation of a subject in Макошь. A Persona is not a
contact, address-book entry or CRM profile. A Persona owns identity,
relationships and memory anchors; timeline and dossier are derived views built
from source-backed records and shared engines.

## Project

A bounded work context with goals, participants, documents, communications,
decisions, tasks, obligations and timeline. A Project is not an Organization and
is not a Task.

## Provenance

The source trail that explains where a record, relationship, score or conclusion
came from. Provenance is required for imported and AI-derived facts.

## Relationship

A first-class connection between entities, especially Personas, Organizations,
Projects, Documents, Communications, Tasks, Events and Decisions. Relationships
carry type, direction where relevant, confidence, provenance and validity.

## Risk

An evidence-backed condition that may harm an objective, relationship,
obligation, project or decision. Risk is an observation or domain record with
provenance, not a vague label.

## Source Record

Legacy term for imported provider records or local artifacts preserved before
canonical projections are built. New architecture uses Observation Platform as
the canonical append-only evidence store.

## Task

A concrete actionable unit with lifecycle, owner, status and evidence. Tasks can
come from Obligations, Communications, Documents, Projects, Events or manual
entry. A Task is not the same as an Obligation or Follow-Up.

## Timeline

A chronological view produced from Events and domain records. Timeline is an
engine output used by domains; it is not separately implemented inside every
domain as a source of truth.

## Trust

An evidence-backed assessment attached primarily to Relationships and source
reliability. Trust is not a generic field on every entity.
