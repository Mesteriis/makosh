# ADR-0309: Loopback browser Owner Vault provisioning host

Статус: Принято
Дата: 2026-07-28
Состояние реализации: Implemented. Отдельный feature-gated native host,
same-origin Vite proxy, exact Origin/private-proof admission, frontend adapter,
root `make dev` orchestration и server/client tests реализованы. Живой
loopback start/cancel через Vite подтверждён, direct request без proof получает
`403`. Реальный provider credential и Telegram QR не создавались без
пользовательских Telegram API ID/hash; fake provisioning не используется.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0223: encrypted Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0295: Owner write-only Vault provisioning](ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md);
- [ADR-0300: loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md);
- [ADR-0303: provider-owned QR account linking](ADR-0303-provider-owned-qr-account-linking-and-transient-artifact-custody.md).

## Контекст

`make dev` поднимает authenticated loopback Core Gateway и Vue client в
браузере. Integration-owned Mail, Telegram и Zulip setup используют
`OwnerVaultProvisioningClientV1`, но его HPKE host adapter существует только
как Tauri command surface. Поэтому browser development показывает account
forms, но не может сохранить provider credential и не достигает следующего
provider lifecycle:

```text
Telegram account setup
  -> Vault credential revisions
  -> effective Telegram settings
  -> TDLib authorization
  -> real provider QR
```

Передавать credential plaintext в Gateway, Kernel, development assembly,
integration Settings или provider lifecycle command запрещено ADR-0295.
Fake account, fixture credential, fake QR и client assertion
`qr_authorized=true` также запрещены ADR-0303.

## Решение

Вводится development-only gate:

```text
loopback_browser_owner_vault_host_v1
```

### Отдельная unit

Provider-neutral native crate `makosh-owner-vault-provisioning-host` получает
отдельный development server binary. Он переиспользует ту же реализацию:

- bounded ephemeral X25519 response recipient;
- opening exact one-action Vault lease;
- sealing exact Vault provisioning command;
- opening only sanitized provisioning receipt;
- session TTL, capacity bound and zeroization.

Binary является app/platform host adapter. Он не импортирует Mail, Telegram,
WhatsApp, Zulip, Communications или Kernel implementation и не владеет
provider semantics.

### Loopback transport

Development host:

- слушает только exact `127.0.0.1` address;
- запускается только root `make dev`;
- принимает только bounded JSON POST на versioned host paths;
- требует exact browser Origin;
- требует private development proxy proof в отдельном header;
- не поддерживает CORS, cookies, redirects, file access или arbitrary paths;
- не пишет request bodies, credentials, ciphertext или receipts в logs;
- останавливается вместе с development ensemble.

Browser не получает proxy proof. Vite читает existing private `0600` proof
file и добавляет header только при proxying exact host path. Direct request к
host без proof отклоняется.

Этот transport является development equivalent Tauri invoke, а не business
HTTP API. Production Vite build и Tauri bundle не содержат server binary и не
объявляют его availability.

### Credential custody

```text
integration-owned setup form
  -> same-origin Vite host proxy
  -> separate native provisioning host
  -> HPKE ciphertext
  -> generated OwnerVaultProvisioningService through Core Gateway
  -> Kernel opaque relay
  -> Vault
```

Credential plaintext может находиться только в active provider form и
development host request memory. Оно очищается после seal attempt. Vite,
host, Gateway и Kernel не логируют body. Gateway и Kernel по-прежнему получают
только ciphertext и non-secret fences.

Development host не ослабляет fresh P-256 owner-device proof, exact
registration/GrantSet/capability admission, Vault generation fence,
idempotency или CAS.

### Telegram QR consequence

После успешного account setup Telegram integration получает только sanitized
credential revisions и effective non-secret settings. Затем Telegram runtime
сам запускает TDLib authorization, запрашивает
`requestQrCodeAuthentication`, возвращает transient provider `tg://login`
через `telegram.authorization.v1`, а Telegram frontend локально рендерит QR.

Development host не создаёт QR, не читает TDLib state и не становится generic
account service.

## Failure semantics

- missing/invalid proxy proof, wrong Origin, non-loopback bind или unknown
  path: reject before body dispatch;
- unavailable host: account setup remains unavailable and no Settings or
  lifecycle mutation occurs;
- expired/unknown host session: restart provisioning from `Prepare`;
- failed seal/commit/open: secret buffer is cleared and no success is shown;
- lost response after Vault commit: ADR-0295 idempotent operation retry
  applies;
- development ensemble stop: all ephemeral host sessions disappear.

## Units of assembly

```text
native provisioning host library   HPKE/session semantics
development host binary            bounded loopback host transport
Vite development proxy             proof injection and exact routing
Core Gateway/Kernel/Vault           unchanged provisioning authorities
Telegram integration               account binding and TDLib QR lifecycle
Communications domain               no dependency
```

## Gate `loopback_browser_owner_vault_host_v1`

Gate закрывается только при наличии:

1. separate feature-gated development host binary;
2. exact loopback bind, Origin and private proxy-proof admission;
3. bounded request size and versioned paths;
4. no request-body or secret logging;
5. frontend host selection that keeps production browser fail-closed;
6. server and client tests for start/seal/open/cancel and negative admission;
7. live `make dev` account provisioning through Vault;
8. live Telegram TDLib status reaching real QR or a truthful provider/config
   failure without fake artifact;
9. architecture, SRP, Rust, frontend type and unit gates.
