# ADR-0322: Loopback native Owner device proof host

Статус: Принято
Дата: 2026-07-28
Состояние реализации: implemented. Separate native signer, exact loopback
route, shared frontend proof factory и `make dev` orchestration реализованы;
live owner Settings и Owner Vault mutations проходят с fresh device proof без
выдачи private key браузеру.

Уточняет:

- [ADR-0218: Owner/device identity](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0232: browser client identity](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0295: Owner Vault provisioning](ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md);
- [ADR-0296: Owner Module Settings Gateway](ADR-0296-owner-authorized-public-module-settings-gateway.md);
- [ADR-0300: loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md);
- [ADR-0309: loopback Owner Vault host](ADR-0309-loopback-browser-owner-vault-provisioning-host.md).

## Контекст

`loopback_full_stack_dev_assembly_v1` создаёт initial file-backed ES256
development device и автоматическую Gateway session:

```text
owner_id  = initial owner
device_id = initial owner device
session   = loopback-development
```

Public owner Settings и Vault routes правильно требуют:

```text
active owner/device principal
  -> operation-bound challenge
  -> fresh P-256 signature
  -> single-use commit
```

Однако текущая проверка принимает только paired browser-device identity.
Loopback development не создаёт WebAuthn credential и не должен выдавать
private owner key browser JavaScript. Поэтому автоматическая session проходит
Gateway transport admission, но `Prepare` отклоняется до challenge. Наличие
provider account wizard или native Vault seal host не делает mutation
работоспособной.

Нельзя:

- отключать fresh proof для development;
- считать private proxy proof owner/device identity;
- копировать initial owner private key в browser storage;
- автоматически создавать fake WebAuthn identity;
- переносить Settings/Vault mutation в integration, Communications или Vite.

## Решение

Вводится development-only gate:

```text
loopback_native_owner_device_proof_host_v1
```

### Kernel validation

Kernel сохраняет один proof verifier и различает authority только по
server-issued principal:

- paired browser session требует current active browser-device identity и
  проверяет browser-bound P-256 public key;
- exact `loopback-development` session требует exact initial owner/device и
  проверяет initial owner ES256 public key;
- LAN development остаётся запрещённым для owner mutations;
- любой другой session/device mismatch отклоняется.

Browser не выбирает session ID и не может повысить paired session до
loopback. Gateway создаёт exact development principal только после admission
ephemeral proxy proof, loopback address, Host и Origin.

### Отдельная native signing unit

Provider-neutral app crate `makosh-owner-device-proof-host`:

- открывает только explicit absolute initial device key file;
- требует regular non-symlink file, exact 32 bytes и owner-only permissions;
- подписывает только exact 32-byte owner challenge;
- возвращает fixed-width 64-byte ES256 signature;
- не принимает arbitrary message, operation payload или provider data;
- не импортирует Kernel implementation, domain или integration;
- не логирует key, challenge или signature.

Existing feature-gated development host server предоставляет exact endpoint:

```text
POST /__makosh/owner-device-proof/v1/sign
```

Endpoint использует тот же exact loopback bind, Origin, private Vite
proxy-proof и bounded request admission, что Owner Vault host. Vite удаляет
одноимённые browser headers и добавляет proof только server-side.

### Frontend selection

`OwnerDeviceProofV1` остаётся узким port:

- paired/Tauri browser использует existing browser/WebAuthn signer;
- `make dev` при explicit build-time host availability использует native
  development adapter;
- production browser без approved signer fail closed.

Owner Settings и Owner Vault используют одну factory. Provider workflows не
знают, какой signer обслужил challenge.

## Units of assembly

```text
Kernel owner proof verifier
  authority and signature validation

native owner-device proof host
  initial file-backed device signer adapter

development host server
  bounded loopback transport composition

Vite development proxy
  server-side ephemeral proof injection

Owner Settings / Owner Vault clients
  operation-specific prepare/sign/commit

Mail / Telegram integrations
  no owner identity implementation dependency
```

## Failure semantics

- missing/mismatched key file: host startup fails before Vite readiness;
- wrong Origin/proof/path/body length: request rejected before signing;
- unknown or non-loopback session: Kernel rejects initial owner proof path;
- invalid signature: single-use challenge is consumed according to existing
  owner route semantics and no mutation succeeds;
- host unavailable: setup/recovery remains fail closed;
- restart: ephemeral proxy proof changes; durable owner key and Control Store
  identity are not replaced.

## Gate `loopback_native_owner_device_proof_host_v1`

Gate закрывается только при наличии:

1. separate native signer crate and exact 32-byte challenge contract;
2. Kernel tests for exact loopback initial owner proof plus paired-browser
   regression and mismatch rejection;
3. exact host route with loopback Origin/proof/body admission;
4. frontend signer factory shared by Settings and Vault;
5. Vite/root `make dev` orchestration without private key exposure;
6. native, frontend and architecture tests;
7. live owner Settings prepare/commit and Vault prepare/commit through
   `make dev`;
8. no LAN mutation authority, fake identity or proof bypass.
