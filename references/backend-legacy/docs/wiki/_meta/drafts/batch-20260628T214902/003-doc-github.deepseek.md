### Summary / Резюме

Необходимо создать (или обновить) русскоязычную страницу wiki `operations/documentation-map.md`, задокументировав в ней файл `.github/pull_request_template.md` – шаблон запроса на включение, присутствующий в репозитории `makosh`. Страница должна отражать структуру шаблона, перечислять его секции и дословные формулировки чеклистов, не добавляя неподтверждённой интерпретации.

### Proposed pages / Предлагаемые страницы

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
## Summary

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

### Source coverage / Покрытие источников

- **`.github/pull_request_template.md`**
  - Полный путь к файлу.
  - Наличие секций `Summary`, `Validation`, `Security and Privacy`.
  - Содержимое секции `Summary` – пустой маркированный список.
  - Чекбокс `make validate` и свободное поле `Other:`.
  - Текст чекбокса «No secrets, private messages, private documents or local \`.env\` values are included.»
  - Текст чекбокса «Provider writes, destructive actions and automation changes preserve capability and audit boundaries.»

### Drift candidates / Кандидаты на drift

Из предоставленного контекста возможные расхождения не видны. Наличие команды `make validate` в репозитории данным чанком не подтверждено, однако сам факт её упоминания в шаблоне задокументирован буквально.
