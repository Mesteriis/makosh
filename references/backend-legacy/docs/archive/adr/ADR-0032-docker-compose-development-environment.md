# ADR-0032 Docker Compose Development Environment

Status: Accepted

## Context

Макошь targets a local-first desktop product with PostgreSQL, Rust, Vue 3 + Vite and Tauri. Development needs a repeatable local environment without introducing production deployment semantics or scattering Docker files across the repository.

## Decision

Use Docker Compose for local development infrastructure. Keep Docker-specific files under `docker/`, including `docker/docker-compose.yml`, `docker/Dockerfile`, environment examples and bind-mounted development data under `docker/data/`.

The default Compose surface runs local dependencies. The `app` profile also runs the Макошь backend and Vite frontend in containers, sharing the existing local vault and mail-blob bind mounts while retaining build and dependency caches in named Docker volumes. This profile is for local development only; it is not a production deployment.

Expose developer commands through the root `Makefile`.

## Consequences

- Local development can start from a consistent Compose entry point.
- `make docker-app-up` starts the backend and frontend alongside PostgreSQL, NATS, ClamAV and the isolated attachment extractor.
- Persistent dev data stays under `docker/data/` and is ignored by Git.
- Docker files do not leak into backend or frontend implementation directories.
- This setup is not a production deployment model.
- Any future production/self-hosted deployment design requires a separate ADR.
