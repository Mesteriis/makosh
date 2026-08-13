# ADR-0411: Zoom, Telemost и OmniRoute provider boundaries

Статус: Принято
Дата: 2026-08-13

## Контекст

После восстановления canonical owners и производных projections остаются три
исторически названные, но отсутствующие в clean-room inventory интеграции.
Zoom и Yandex Telemost являются внешними meeting providers; OmniRoute является
явно выбираемым внешним AI provider. Ни одна из них не является доменом
продуктовой истины и ни одна не может быть восстановлена копированием legacy
HTTP handlers, demo call cards или environment-secret fallback.

Communications уже владеет provider-neutral `call_evidence_observed` contract,
а AI engine — provider-neutral reply/summary/translation/explanation contracts.
Поэтому интеграции должны заканчиваться на этих ingress boundaries, а не читать
чужие таблицы или создавать новые generic façades.

## Решение

Создать три независимых integration contours. Каждый содержит ровно
API/core/persistence/runtime/assembly package и один managed module.

- `zoom` и `telemost` хранят только public account identity, opaque provider
  cursors, lifecycle/revision, exact inbox и sanitized call-evidence outbox.
  OAuth/application credentials доступны только через configuration-scoped
  Vault purpose. Runtime не хранит token, webhook body, join URL, participant
  address или raw provider response.
- `omniroute` реализует существующие typed AI provider contracts. API key
  разрешается только через Vault; endpoint/model routing задаются bounded
  settings. Persistence хранит exact request fingerprint, public model receipt,
  terminal outcome и replay bytes, но не prompt, provider response или key.
- Zoom/Telemost публикуют только canonical Communications call evidence с
  `Zoom`/`YandexTelemost` provenance. OmniRoute не получает generic Context и
  не имеет direct browser/client route.

Provider network is contacted only after exact managed admission, Storage,
Vault credential lease and output permit are ready. Missing, stale or revoked
credentials fail before Ready or yield a bounded unavailable outcome; they are
never read from environment variables or UI forms.

## Storage и privacy

All provider-owned tables use ENABLE+FORCE RLS and transaction-local owner
context. Exact input replay returns the stored result without a second provider
call; changed bytes under the same id conflict. Provider payloads are decoded
into closed typed structures and discarded. Durable envelopes, client replies,
typed errors and supervised diagnostics are covered by private sentinel tests.

## Admission

Task 26 adds fifteen production-role packages, three integrations, three
managed modules and fifteen new capability identifiers. Release inventory adds
runtime and storage artifacts for each integration. Zoom and Telemost expose a
small generated account/status client; OmniRoute exposes only provider RequestRpc
surfaces.

Admission requires actual provider success with separately provisioned
credentials. When credentials are absent, implementation, negative bootstrap,
RLS, release and privacy evidence may complete, but the policy slice must remain
unadvanced and the live provider checks must be reported as blocked.

## Отклонено

- historical demo call façade or Telemost recording commands;
- manual OAuth/API-key entry in frontend or plaintext settings;
- environment-variable credential fallback;
- storing webhook/provider response bodies or join URLs;
- direct Communications/AI persistence access;
- generic OmniRoute chat endpoint or generic Context read-all API;
- claiming provider readiness from a mock when real authorization is absent.
