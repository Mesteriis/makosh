---
chunk_id: 003-doc-github
batch_id: batch-20260628T214902
group: .github
role: doc
source_status: pending
source_count: 1
generated_by: code-wiki-ru
---

# 003-doc-github — .github/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `1`

## Резюме

-
```

#### Validation (Проверка)

Чеклист действий, которые необходимо выполнить перед созданием PR:

- `- [ ] make validate` – запуск команды `make validate`.
- `- [ ] Other:` – дополнительная проверка (заполняется автором).

#### Security and Privacy (Безопасность и конфиденциальность)

Чеклист требований безопасности:

- `- [ ] No secrets, private messages, private documents or local .env values are included.`
- `- [ ] Provider writes, destructive actions and automation changes preserve capability and audit boundaries.`

> Примечание: точное значение пункта «Provider writes...» не подтверждено данным контекстом; приведено как буквальный текст из шаблона.
```

## Предложенные страницы

**Путь в wiki:** `operations/documentation-map.md`

```markdown
# Карта документации

## `.github/pull_request_template.md`

Файл шаблона запроса на включение (Pull Request) в репозитории `makosh`.

**Путь:** `.github/pull_request_template.md`

### Секции шаблона

#### Summary (Краткое описание)

Секция, в которой автор PR кратко описывает изменения. Представляет собой маркированный список с одним пустым пунктом:

```markdown

## Покрытие источников

- **`.github/pull_request_template.md`**
  - Полный путь к файлу.
  - Наличие секций `Summary`, `Validation`, `Security and Privacy`.
  - Содержимое секции `Summary` – пустой маркированный список.
  - Чекбокс `make validate` и свободное поле `Other:`.
  - Текст чекбокса «No secrets, private messages, private documents or local \`.env\` values are included.»
  - Текст чекбокса «Provider writes, destructive actions and automation changes preserve capability and audit boundaries.»

## Исходные файлы

- [`.github/pull_request_template.md`](../../../../.github/pull_request_template.md)

## Кандидаты на drift

Из предоставленного контекста возможные расхождения не видны. Наличие команды `make validate` в репозитории данным чанком не подтверждено, однако сам факт её упоминания в шаблоне задокументирован буквально.
