# ADR-0180: Bounded DOCX Content Disarm and Reconstruction

Status: Accepted
Date: 2026-07-12

Extends ADR-0179 and supersedes its DOCX non-goal.

## Decision

For a clean DOCX attachment Макошь may create a separate content-disarmed PDF
artifact. The isolated worker opens the DOCX only through bounded ZIP/XML text
extraction, removes non-printable control content, limits the rendered text to
four fixed-size pages and reconstructs those pages as an image-only PDF with a
fixed local font.

The artifact copies no DOCX package member, relationship, hyperlink, embedded
file, macro, XML metadata, style, field or active Office content. It is bounded
by source bytes, ZIP entry count, uncompressed XML bytes, rendered page/pixel
limits, wall-clock time and final artifact size. Failure produces no artifact
and leaves the immutable original untouched.

The same clean-verdict and source-SHA revalidation contract from ADR-0179
applies before Макошь serves the derived PDF. RAR, 7z, nested archives and
legacy binary Office formats remain unsupported.

## Consequences

DOCX CDR is a deliberately lossy readable representation rather than a
document-preserving conversion. Users can inspect safe text-derived content
without the browser receiving the original Office container.
