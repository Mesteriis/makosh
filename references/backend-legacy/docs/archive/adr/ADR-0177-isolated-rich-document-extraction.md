# ADR-0177 Isolated Rich Document Extraction

Status: Accepted
Date: 2026-07-12

## Decision

PDF, DOCX and OCR extraction runs only in a separate local worker. The worker
has no network egress, receives one bounded blob through a private local
transport, writes only derived artifacts, and is subject to CPU, memory, time
and temporary-storage limits. The main Макошь API process never invokes office
or PDF parsers directly.

The local Compose environment provisions the worker as `attachment-extractor`.
The worker is attached only to its own Docker `internal` network, so it has no
external egress. A tiny loopback-only gateway, with no document parser and a
bounded JSON frame, bridges the host API to that internal network. The worker
also keeps a private Unix socket for in-container diagnostics, sees
`docker/data/mail` read-only, and runs with a read-only root filesystem,
dropped capabilities, process/memory/CPU limits and a bounded `noexec`
temporary filesystem. It supports bounded PDF text, DOCX XML and local English
OCR extraction and bounded first-page PDF-to-PNG rendering. The Макошь command
client is introduced separately; until it
is enabled, rich documents remain quarantined for preview and extraction.
Text-like attachments continue to use the bounded local UTF-8 extractor.

## Consequences

Extraction commands must be durable and record only blob references, verdicts,
limits and sanitized failure classes. Derived text or previews may be promoted
only after the worker succeeds and the source hash and clean scan verdict still
match. Worker failures do not expose document bytes or parser diagnostics.
