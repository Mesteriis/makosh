# ADR-0179: Bounded PDF Content Disarm and Reconstruction

Status: Accepted
Date: 2026-07-12

Clarifies ADR-0177. Its former DOCX non-goal is superseded in part by ADR-0180.

## Decision

Макошь may create a derived content-disarmed artifact only for an attachment
with a current `clean` scan verdict and a verified PDF type. The isolated
attachment worker rasterizes a bounded number of pages with Poppler and writes
a new image-only PDF with Pillow. It never copies objects, scripts, forms,
embedded files, annotations, links, metadata or JavaScript from the source.

The output is bounded in source size, page count, rendered pixels, wall-clock
time and artifact size. A limit breach or parser failure produces no artifact.
The original remains immutable in blob storage and is never replaced.

The domain stores the derived artifact separately with its source SHA-256,
renderer version, status and safe content type. Reads must revalidate the
current source SHA-256 and clean verdict; a changed or quarantined source makes
the artifact unavailable. CDR status changes emit the existing privacy-safe
attachment processing event.

## Non-goals

RAR, 7z, encrypted PDFs and arbitrary office/archive formats are not
CDR-supported by this decision. DOCX support is defined separately by ADR-0180.
The remaining formats stay quarantined or unsupported until each has a bounded,
isolated implementation and its own security corpus.
