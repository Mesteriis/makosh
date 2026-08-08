# ADR-0173: Frontend Capability Surfaces and Thin Vue Components

Status: Accepted
Date: 2026-07-02

Clarifies:

- ADR-0093 - Frontend Platform Migration to Vue 3
- ADR-0172 - Макошь UI Kit на базе shadcn-vue / Reka UI
- ADR-architecture-communication-contract

## Context

Макошь frontend использует Vue 3, Pinia и TanStack Query. Backend постепенно
открывает больше доменных, workflow, AI, review, communications, provider
runtime и settings capabilities. Если UI будет подключать эти capabilities
напрямую из `.vue` файлов, бизнес-потоки снова расползутся по шаблонам,
watchers, event handlers и локальным helper-функциям.

В репозитории уже появился рабочий паттерн:

```text
frontend/src/domains/<domain>/api/*
frontend/src/domains/<domain>/queries/use*Query.ts
frontend/src/domains/<domain>/queries/use*Surface.ts
frontend/src/domains/<domain>/stores/*
frontend/src/app/queries/use*Surface.ts
frontend/src/integrations/<provider>/queries/use*Surface.ts
```

Также уже есть guard `frontend/scripts/check-vue-component-boundaries.mjs`,
который запрещает в Vue components прямой `fetch`, прямой `ApiClient`,
прямой `useQuery` / `useMutation`, runtime imports из business paths и тяжёлые
collection transforms в доменных компонентах.

Нужно зафиксировать этот паттерн как архитектурное правило, а не как набор
случайных тестов.

## Decision

Каждая backend capability, которая становится видимой во frontend, должна иметь
явную frontend capability surface до того, как Vue component начнёт её
использовать.

Surface - это Composition API module, который собирает capability для конкретной
операционной поверхности:

```text
use<DomainOrFeature><Capability>Surface()
use<DomainOrFeature>PageSurface()
use<Provider>RuntimeSurface()
```

Текущая директория для таких modules:

```text
frontend/src/app/queries/use*Surface.ts
frontend/src/domains/<domain>/queries/use*Surface.ts
frontend/src/integrations/<provider>/queries/use*Surface.ts
```

Название `queries/` сохраняется, потому что оно уже валидировано текущим кодом и
guard tests. Переезд в отдельную директорию `surfaces/` возможен только отдельной
миграцией.

## Surface Responsibilities

Surface owns frontend application orchestration for one UI capability:

- wires TanStack Query hooks, mutations and invalidation;
- reads and updates Pinia transient UI state;
- exposes explicit command handlers for owner actions;
- maps backend DTOs into presentation-ready view models;
- computes loading, empty, disabled, selected and error states;
- coordinates refresh/refetch after successful mutations;
- keeps capability availability visible to the UI;
- keeps provider runtime surfaces under `frontend/src/integrations/*`;
- keeps product/business surfaces under `frontend/src/domains/*` or
  `frontend/src/app/*`.

Surface does not own canonical business truth. Durable rules, validation,
provider authority, evidence semantics and state transitions remain backend
responsibilities. Frontend business logic here means client-side orchestration and
presentation policy, not domain authority.

## Vue Component Responsibilities

`.vue` files are thin rendering units. They may contain:

- template markup and component composition;
- props, emits and `v-model` get/set glue;
- local visual state such as open/closed, hover, focus and selected visual tab;
- small display-only computed values such as labels, classes and aria text;
- animation, transition and DOM interaction code;
- calls to actions returned by the owning surface.

`.vue` files must not contain:

- direct API calls;
- direct `ApiClient` usage;
- direct `fetch` / HTTP calls;
- direct `useQuery` / `useMutation` calls;
- query invalidation or cache patching;
- provider command construction;
- durable business state decisions;
- domain validation schemas;
- cross-domain orchestration;
- provider-specific business policy;
- non-trivial collection transforms for domain data.

When a component needs non-trivial data preparation, that preparation belongs in
the surface or a domain presentation module imported by the surface.

## Capability Module Contract

A user-visible backend capability should be represented by these frontend
surfaces, scaled to the capability size:

```text
frontend/src/domains/<domain>/
  api/<capability>.ts          # typed transport functions only
  queries/use<Capability>Query.ts
  queries/use<Capability>Surface.ts
  stores/<domain>.ts           # transient UI state only
  types/<domain>.ts
```

Provider setup/runtime capabilities use:

```text
frontend/src/integrations/<provider>/
  api/*
  queries/use<Provider>*Query.ts
  queries/use<Provider>*Surface.ts
  stores/*
  types/*
```

App shell and cross-domain operating surfaces use:

```text
frontend/src/app/queries/use*Surface.ts
```

If a backend capability has no surface, Vue code must not bind to it directly.

## Query And Cache Rules

TanStack Query owns server-derived state. Pinia owns transient UI state.

Allowed:

```ts
// useTasksPageSurface.ts
const tasksQuery = useTasksQuery()
const reviewMutation = useReviewTaskCandidateMutation()
```

```vue
<!-- TasksView.vue -->
<script setup lang="ts">
import { useTasksPageSurface } from '@/domains/tasks/queries/useTasksPageSurface'

const surface = useTasksPageSurface()
</script>
```

Forbidden:

```vue
<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { fetchTasks } from '../api/tasks'

const tasks = useQuery({ queryKey: ['tasks'], queryFn: fetchTasks })
</script>
```

Query keys must keep existing Макошь ownership:

- product/business data uses domain roots such as `['communications', ...]`;
- provider runtime/setup data uses integration roots such as
  `['integrations', provider, 'runtime', ...]`;
- provider business roots such as `['telegram', ...]`, `['whatsapp', ...]` and
  `['mail', ...]` remain forbidden.

## Testing And Guards

For new or changed capability surfaces, add or update a targeted boundary test
when the behavior is architecture-sensitive. Boundary tests should verify that:

- Vue components import the surface, not API or query internals;
- surface modules wire required queries/mutations;
- provider/runtime actions stay in integration surfaces;
- product/business actions stay in domain/app surfaces;
- direct API and TanStack usage does not leak into `.vue` files.

`frontend/scripts/check-vue-component-boundaries.mjs` remains the executable
guard for thin Vue components. Future guard extensions should enforce this ADR
structurally instead of using per-file exceptions or baseline files.

## Consequences

Positive:

- Each backend capability has a discoverable frontend entrypoint.
- Vue files stay readable and focused on rendering.
- Business orchestration becomes testable without rendering full UI trees.
- TanStack Query and Pinia usage stays consistent across domains.
- Provider runtime UI cannot accidentally become product business UI.

Negative:

- More modules are required for small capabilities.
- Surface modules can become new god files if they collect unrelated behavior.
- Some existing modules may need migration as they are touched.

Mitigation:

- Split surfaces by capability, not by arbitrary line count.
- Keep API modules transport-only.
- Keep stores transient.
- Keep `.vue` files thin.
- Strengthen boundary tests before or with migrations.

## Validation

The expected validation for implementation work affected by this ADR is:

```sh
make frontend-lint
make frontend-test
```

For architecture-sensitive changes, also run:

```sh
make architecture-check
make code-boundaries-check
```

Documentation-only changes to this ADR do not require frontend runtime tests, but
must not claim implementation validation unless the commands were run.

## References

- `frontend/scripts/check-vue-component-boundaries.mjs`
- `frontend/package.json`
- `docs/architecture/ui.md`
- `docs/architecture/frontend-ui-system.md`
- `docs/architecture/component-communication.md`
