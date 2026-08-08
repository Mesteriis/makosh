# Макошь legacy documentation reference

В этой директории сохранена документация предыдущей реализации Макошь. Она не
является active policy, текущим implementation status или шаблоном структуры
clean-room backend.

Активная документация находится в
[`docs/README.md`](../../../docs/README.md).

## Содержимое snapshot

- [`archive/`](archive/) — прежние ADR и исторические решения;
- [`architecture/`](architecture/) — предыдущие architecture descriptions;
- [`product/`](product/), [`foundation/`](foundation/) и [`domains/`](domains/)
  — прежняя product/domain model;
- [`integrations/`](integrations/) — прежние provider specifications;
- [`development/`](development/), [`roadmap/`](roadmap/) и [`refactoring/`](refactoring/)
  — старые команды, планы и status documents;
- [`wiki/`](wiki/) и [`site/`](site/) — generated documentation и прежний
  Pages portal;
- [`INDEX-LEGACY.md`](INDEX-LEGACY.md) — исходный documentation index на момент
  переноса. Его ссылки могут указывать на вынесенные active clean-room файлы.

Legacy documentation читается только для восстановления конкретного behavior,
fixture, event name, security invariant или product evidence. Любое решение
для новой системы заново фиксируется в active ADR.
