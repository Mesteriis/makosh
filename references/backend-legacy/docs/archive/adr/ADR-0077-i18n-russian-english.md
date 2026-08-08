# ADR-0077: i18n — Russian and English Interface

Status: Accepted
Date: 2026-06-08
Deciders: Alex (makosh maintainer)

## Context

Макошь is a personal knowledge system. The primary user is Russian-speaking, but technical collaborators and future extensibility benefit from English as a secondary language. Hardcoding strings in one language creates maintenance debt and makes switching impractical.

## Decision

The Макошь desktop interface supports **two languages: Russian (ru) and English (en)** via a lightweight i18n system.

**Mechanism:**
- JSON translation dictionaries under `frontend/src/lib/i18n/` (`en.json`, `ru.json`)
- A Svelte writable store `currentLocale` (defaults to `en`)
- A pure `t(locale, key)` function for translations
- English strings serve as translation keys; `en.json` is an empty `{}`

**Language switch:**
- A toggle in the user menu (⌘ menu) allows switching between Russian and English
- The HTML `lang` attribute is set to `ru` (primary user language)

**Scope:**
- User-visible UI text only: navigation labels, widget titles, zone titles, page headings, form labels, buttons, wizard steps, status messages
- Non-user-visible identifiers (API keys, setting keys, CSS class names, TypeScript identifiers, code comments) remain in English

## Consequences

- All new UI text must go through the `t()` / `_()` translation function
- `ru.json` must be kept in sync when English strings are added or changed
- Translation coverage is progressive: unwrapped strings display as English (the fallback key)
- The i18n system is intentionally minimal — no external library dependency

## Alternatives Considered

**Russian-only UI (ADR-0077 draft).** Rejected — English fallback supports collaboration and future extensibility without blocking.

**Full i18n library (svelte-i18n, @sveltekit-i18n, typesafe-i18n).** Rejected — adds dependency weight without proportional benefit for a two-language personal system.

**No i18n, English only.** Rejected — primary user is Russian-speaking.

## References

- ADR-0026 — desktop-first responsive UI
- ADR-0031 — temporary desktop-only UI scope
