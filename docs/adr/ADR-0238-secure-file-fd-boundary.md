# ADR-0238: Secure-file FD boundary

Статус: Принято
Дата: 2026-07-20
Состояние реализации: foundation реализован в `makosh-secure-file`; Vault
wrapping key и Gateway TLS material переведены. Остальные Vault recovery и
release readers должны мигрировать до admission первого owner.

Зависит от:

- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md).

## Контекст

Проверка path metadata перед `File::open` не закрывает TOCTOU: атакующий может
заменить terminal path symlink между проверкой и чтением. Это особенно опасно
для private key, Vault/recovery artifacts, TLS material и release trust inputs.
Локальные повторные проверки в каждом owner приводят к drift и не дают одного
исполняемого policy surface.

## Решение

`makosh-secure-file` — узкий public platform contract для bounded Unix reads.
Он открывает final pathname descriptor с `O_NOFOLLOW | O_CLOEXEC`, проверяет
уже открытый descriptor как non-empty regular file и читает не больше exact
лимита. Проверка не доверяет предварительному `metadata(path)`.

Для secret/key material используется `SecureReadPolicy::owner_private`: файл
должен принадлежать effective UID процесса и не иметь group/other permission.
Для public signed artifacts допускается `regular` policy, но terminal symlink,
directory, non-regular file, zero-byte input или size overflow всё равно
отклоняются. Ошибки typed, stable и не включают path или содержимое.

Потребители не создают собственные `File::open` + `symlink_metadata` read
sequences. Write/atomic replace, directory traversal и inherited FD имеют
отдельные contracts; этот ADR не объявляет их безопасными автоматически.

## Проверка

- regression доказывает bounded owner-private read и rejection symlink;
- Cargo boundary допускает dependency только на public contract;
- Vault wrapping key и Gateway TLS certificate/private key используют boundary;
- до first-owner admission должны быть добавлены negative tests для всех
  remaining Vault recovery, Control Store/release trust и signing readers.

## Последствия

Единый policy убирает terminal-symlink race из новых readers, но не заменяет
проверку parent-directory ownership там, где операция создаёт или заменяет
файл. Компиляция остаётся изолированной: package не знает Vault, Kernel,
Gateway или owner state.
