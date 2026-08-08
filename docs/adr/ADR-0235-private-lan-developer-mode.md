# ADR-0235: Private-LAN developer mode

Статус: Заменено
Дата: 2026-07-20
Состояние реализации: historical evidence only. Durable LAN setting и
cookie-free owner principal удалены migration 35→36; действующее решение —
[ADR-0237](ADR-0237-temporary-private-lan-development-without-owner-authority.md).

Зависит от:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0232: Browser client identity and same-origin Gateway session](ADR-0232-browser-client-device-identity-and-same-origin-session.md).

## Контекст

Локальная разработка UI и Gateway требует повторяемого запуска без WebAuthn
ceremony. Environment flag или browser-side bypass неприемлемы: они легко
попадают в public deployment и не являются authoritative Kernel state.
Проверка только peer IP также недостаточна, потому что public reverse proxy
может подключаться к Kernel из private network.

## Решение

Developer mode является Kernel-owned operator setting в private Control Store.
Default — `disabled`. Его можно изменить только локальной CLI-командой при
остановленном Kernel; runtime lock не допускает параллельную mutation.

При `enabled` browser Gateway запускается только при одновременном выполнении
всех условий:

- listener bind — конкретный RFC1918/link-local IPv4 либо ULA/link-local IPv6,
  но не wildcard, loopback или public address;
- exact HTTP origin содержит тот же literal IP и port, что listener;
- RP ID равен этому literal IP;
- paired-remote profile не выбран;
- каждый application request имеет exact `Host`, не имеет чужого `Origin`,
  cross-site fetch metadata или proxy/forwarding headers.

В этом профиле Gateway создаёт memory-only developer principal из уже
enrolled initial owner identity. Cookie и WebAuthn не требуются. Pairing и
authentication routes не монтируются. Client bootstrap, realtime и session
status используют тот же owner-scoped authority, что paired mode; bypass не
выдаёт новые grants и не включает business surfaces.

Plain HTTP разрешён только этому direct private-LAN developer listener. Он не
принимает TLS inputs и не является transport profile для paired/public clients.

Developer mode ограничивает только входящий доступ к Макошь. Исходящие
соединения самого Kernel, platform runtimes и provider integrations не
фильтруются этим режимом: они сохраняют обычный доступ к provider/API.

Публичный `paired_remote` profile остаётся неизменным и всегда требует
browser-device authentication. Public hostname, reverse proxy и
`makosh.sh-inc.ru` не могут быть developer origin.

Developer console logging включает lifecycle, method, закрытый route class,
HTTP status и admission outcome. Он никогда не пишет body, query, cookie,
authorization metadata, pairing/authentication ID или private content.

`BrowserSessionService/GetStatus` возвращает закрытый access-mode enum, чтобы
compiled client показывал заметное состояние в System Control. Interface
language является отдельной client-owned non-secret preference и не входит в
Kernel developer setting.

## Проверка

Executable checks обязаны доказывать:

- default-disabled и durable round-trip setting;
- rejection HTTPS, wildcard/public/mismatched origins;
- cookie-free session/bootstrap только с exact direct-LAN headers;
- rejection forwarded, wrong-host и cross-origin requests;
- paired mode по-прежнему требует cookie;
- unknown access-mode enum fails closed in client;
- logs не содержат request bodies, cookies или dynamic path identifiers.

## Последствия

- Developer mode удобен для локального frontend цикла, но намеренно не
  доступен через public domain или reverse proxy.
- Смена режима требует остановки и повторного запуска Kernel, поскольку она
  меняет listener и authentication boundary.
- LAN peers получают owner-level browser surface на время режима; UI обязан
  показывать этот риск, а оператор — выключать режим после разработки.
