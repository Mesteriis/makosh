# Security Policy

Макошь handles personal communications, documents, credentials references
and local AI workflows. Please report security issues privately.

## Supported Versions

The project is pre-release. Security fixes are handled on the default branch.

## Reporting a Vulnerability

Do not open a public issue for security-sensitive reports.

Email the maintainer or use GitHub's private vulnerability reporting if it is
enabled for the repository. Include:

- affected commit or version;
- impact and affected data or capability;
- reproduction steps;
- whether secrets, private messages or documents may have been exposed.

## Security Expectations

- Never commit secrets, provider tokens, app passwords, private keys or local
  `.env` files.
- Never publish private message bodies, document contents or raw provider
  records in issues or test fixtures.
- Keep provider writes, destructive actions and automation behind explicit
  capability, confirmation and audit boundaries.
- Treat imported documents and messages as untrusted input.
