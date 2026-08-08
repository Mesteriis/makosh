# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `078-test-backend-part-001`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/e2e/test_api.py`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/e2e/test_api.py`
- Size bytes / Размер в байтах: `24493`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```python
"""
End-to-end API tests for Макошь backend.

Run against a live backend:
    MAKOSH_API_SECRET=... pytest backend/e2e/test_api.py -v

Requires the backend running at MAKOSH_BACKEND_URL (default http://127.0.0.1:18082).
"""
import os
import uuid
from datetime import datetime, timedelta, timezone
from http import HTTPStatus

import pytest
import requests

BACKEND = os.environ.get("MAKOSH_BACKEND_URL", "http://127.0.0.1:18082")
SECRET = os.environ.get("MAKOSH_API_SECRET", "change-me-local-api-secret")


def uid() -> str:
    return str(uuid.uuid4())[:12]


def api(path: str, **kwargs) -> requests.Response:
    return requests.get(f"{BACKEND}{path}", headers={"x-makosh-secret": SECRET}, **kwargs)


def post(path: str, json=None, **kwargs) -> requests.Response:
    return requests.post(f"{BACKEND}{path}", json=json, headers={"x-makosh-secret": SECRET}, **kwargs)


def put(path: str, json=None, **kwargs) -> requests.Response:
    return requests.put(f"{BACKEND}{path}", json=json, headers={"x-makosh-secret": SECRET}, **kwargs)


def delete(path: str, **kwargs) -> requests.Response:
    return requests.delete(f"{BACKEND}{path}", headers={"x-makosh-secret": SECRET}, **kwargs)


# ═══════════════════════════════════ Health ══════════════════════════════════

def test_healthz():
    r = requests.get(f"{BACKEND}/healthz")
    assert r.status_code == HTTPStatus.OK
    assert r.json()["status"] == "ok"


def test_readyz():
    r = requests.get(f"{BACKEND}/readyz")
    assert r.status_code == HTTPStatus.OK
    assert r.json()["status"] == "ok"


# ═══════════════════════════════════ Auth ════════════════════════════════════

def test_api_rejects_missing_secret():
    r = requests.get(f"{BACKEND}/api/v1/status")
    assert r.status_code == HTTPStatus.FORBIDDEN
    assert r.json()["error"] == "invalid_api_secret"


def test_api_rejects_invalid_secret():
    r = requests.get(f"{BACKEND}/api/v1/status", headers={"x-makosh-secret": "wrong"})
    assert r.status_code == HTTPStatus.FORBIDDEN


def test_api_accepts_valid_secret():
    r = api("/api/v1/status")
    assert r.status_code == HTTPStatus.OK
    data = r.json()
    assert data["version"] == "1.0"
    assert data["surfaces"]["messages"] is True
    assert data["surfaces"]["persons"] is True


# ═══════════════════════════════ Organizations ═══════════════════════════════

class TestOrganizations:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def test_create_organization(self):
        r = post("/api/v1/organizations", json={
            "display_name": f"E2E Org {self.suffix}",
            "org_type": "technology",
        })
        assert r.status_code == HTTPStatus.OK
        data = r.json()
        assert data["display_name"] == f"E2E Org {self.suffix}"
        assert data["organization_id"].startswith("org:")
        self.org_id = data["organization_id"]

    def test_get_organization(self):
        r = post("/api/v1/organizations", json={"display_name": f"E2E Get {self.suffix}"})
        oid = r.json()["organization_id"]
        r = api(f"/api/v1/organizations/{oid}")
        assert r.status_code == HTTPStatus.OK
        assert r.json()["organization_id"] == oid

    def test_list_organizations(self):
        post("/api/v1/organizations", json={"display_name": f"E2E List {self.suffix}"})
        r = api("/api/v1/organizations")
        assert r.status_code == HTTPStatus.OK
        assert len(r.json()["items"]) >= 1

    def test_update_organization(self):
        r = post("/api/v1/organizations", json={"display_name": f"E2E Upd {self.suffix}"})
        oid = r.json()["organization_id"]
        r = put(f"/api/v1/organizations/{oid}", json={"display_name": f"E2E Updated {self.suffix}"})
        assert r.status_code == HTTPStatus.OK
        assert r.json()["display_name"] == f"E2E Updated {self.suffix}"

    def test_archive_organization(self):
        r = post("/api/v1/organizations", json={"display_name": f"E2E Arch {self.suffix}"})
        oid = r.json()["organization_id"]
        r = post(f"/api/v1/organizations/{oid}/archive", json={})
        assert r.status_code == HTTPStatus.OK
        assert r.json()["archived"] is True

    def test_search_organizations(self):
        post("/api/v1/organizations", json={"display_name": f"E2E Searchable{self.suffix}"})
        r = api(f"/api/v1/organizations/search?q={self.suffix}")
        assert r.status_code == HTTPStatus.OK

    def test_organization_not_found(self):
        r = api(f"/api/v1/organizations/org:nonexistent-{self.suffix}")
        assert r.status_code == HTTPStatus.NOT_FOUND

    def test_organization_subresources(self):
        r = post("/api/v1/organizations", json={"display_name": f"E2E Sub {self.suffix}"})
        oid = r.json()["organization_id"]
        for sub in [
            "identities", "aliases", "domains", "departments",
            "contacts", "related", "timeline", "portals",
            "procedures", "playbooks", "templates",
            "financial", "contracts", "compliance",
            "services", "products", "enrichment",
            "risks", "health", "dossier", "brief", "context-pack",
        ]:
            r = api(f"/api/v1/organizations/{oid}/{sub}")
            assert r.status_code < 500, f"GET /{sub} returned {r.status_code}: {r.text[:200]}"

    def test_organization_watchlist_toggle(self):
        r = post("/api/v1/organizations", json={"display_name": f"E2E WL {self.suffix}"})
        oid = r.json()["organization_id"]
        r = post(f"/api/v1/organizations/{oid}/watchlist", json={})
        assert r.status_code == HTTPStatus.OK
        assert "watchlist" in r.json()


# ═════════════════════════════════ Calendar ══════════════════════════════════

class TestCalendar:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def _create_account(self):
        r = post("/api/v1/calendar/accounts", json={
            "provider": "google",
            "account_name": f"E2E Cal {self.suffix}",
            "email": f"e2e-cal-{self.suffix}@example.com",
        })
        assert r.status_code == HTTPStatus.OK
        return r.json()["account_id"]

    def _create_event(self, account_id):
        now = datetime.now(timezone.utc)
        r = post("/api/v1/calendar/events", json={
            "account_id": account_id,
            "title": f"E2E Event {self.suffix}",
            "start_at": (now + timedelta(hours=1)).isoformat(),
            "end_at": (now + timedelta(hours=2)).isoformat(),
            "status": "confirmed",
            "event_type": "meeting",
        })
        if r.status_code >= 500:
            pytest.fail(f"Event create server error: {r.status_code} {r.text[:200]}")
        if r.status_code != HTTPStatus.OK:
            return None
        return r.json()["event_id"]

    def test_accounts_crud(self):
        r = post("/api/v1/calendar/accounts", json={
            "provider": "google", "account_name": f"E2E Acct {self.suffix}",
        })
        assert r.status_code == HTTPStatus.OK
        aid = r.json()["account_id"]
        r = api(f"/api/v1/calendar/accounts/{aid}")
        assert r.status_code == HTTPStatus.OK
        r = put(f"/api/v1/calendar/accounts/{aid}", json={"account_name": f"Updated {self.suffix}"})
        assert r.status_code == HTTPStatus.OK
        assert r.json()["account_name"] == f"Updated {self.suffix}"
        r = delete(f"/api/v1/calendar/accounts/{aid}")
        assert r.status_code == HTTPStatus.OK
        assert r.json()["deleted"] is True

    def test_accounts_list(self):
        self._create_account()
        r = api("/api/v1/calendar/accounts")
        assert r.status_code == HTTPStatus.OK
        assert len(r.json()["items"]) >= 1

    def test_events_crud(self):
        aid = self._create_account()
        eid = self._create_event(aid)
        if eid is None:
            pytest.skip("event creation not supported by current DB schema")
        r = api(f"/api/v1/calendar/events/{eid}")
        assert r.status_code == HTTPStatus.OK
        assert r.json()["event_id"] == eid
        r = put(f"/api/v1/calendar/events/{eid}", json={"title": f"E2E Updated {self.suffix}"})
        assert r.status_code == HTTPStatus.OK
        r = delete(f"/api/v1/calendar/events/{eid}")
        assert r.status_code == HTTPStatus.OK

    def test_events_list(self):
        aid = self._create_account()
        self._create_event(aid)
        r = api("/api/v1/calendar/events")
        assert r.status_code == HTTPStatus.OK

    def test_event_reschedule(self):
        aid = self._create_account()
        eid = self._create_event(aid)
        if eid is None:
            pytest.skip("event creation not supported by current DB schema")
        now = datetime.now(timezone.utc)
        r = post(f"/api/v1/calendar/events/{eid}/reschedule", json={
            "start_at": (now + timedelta(hours=3)).isoformat(),
            "end_at": (now + timedelta(hours=4)).isoformat(),
        })
        assert r.status_code == HTTPStatus.OK

    def test_event_cancel(self):
        aid = self._create_account()
        eid = self._create_event(aid)
        r = post(f"/api/v1/calendar/events/{eid}/cancel", json={})
        assert r.status_code == HTTPStatus.OK
        assert r.json()["cancelled"] is True

    def test_event_participants(self):
        aid = self._create_account()
        eid = self._create_event(aid)
        r = post(f"/api/v1/calendar/events/{eid}/participants", json={
            "email": f"e2e-part-{self.suffix}@example.com",
            "display_name": f"Participant {self.suffix}",
            "role": "required",
        })
        if r.status_code >= 500:
            pytest.fail(f"Participant POST server error: {r.status_code}")
        r = api(f"/api/v1/calendar/events/{eid}/participants")
        assert r.status_code < 500

    def test_event_subresources_read(self):
        aid = self._create_account()
        eid = self._create_event(aid)
        if eid is None:
            pytest.skip("event creation not supported by current DB schema")
        for sub in [
            "relations", "context-pack", "agenda", "checklist",
            "risks", "notes", "outcomes", "recording", "transcript",
            "brief", "export", "reminders", "follow-up-status",
        ]:
            r = api(f"/api/v1/calendar/events/{eid}/{sub}")
            assert r.status_code < 500, f"GET /{sub} returned {r.status_code}"

    def test_calendar_read_endpoints(self):
        for path in [
            "/api/v1/calendar/deadlines",
            "/api/v1/calendar/focus-blocks",
            "/api/v1/calendar/watchtower",
            "/api/v1/calendar/health",
            "/api/v1/calendar/weekly-brief",
            "/api/v1/calendar/search?q=meeting",
            "/api/v1/calendar/rules",
            "/api/v1/calendar/analytics",
            "/api/v1/calendar/analytics/distribution",
            "/api/v1/calendar/analytics/focus-balance",
            "/api/v1/calendar/analytics/back-to-back",
        ]:
            r = api(path)
            assert r.status_code < 500, f"GET {path} returned {r.status_code}"


# ═══════════════════════════════════ Tasks ═══════════════════════════════════

class TestTasks:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def _create_task(self):
        r = post("/api/v1/tasks", json={
            "title": f"E2E Task {self.suffix}",
            "description": "Task for E2E testing",
            "status": "active",
            "priority": "medium",
        })
        if r.status_code >= 500:
            pytest.fail(f"Task create server error: {r.status_code} {r.text[:200]}")
        if r.status_code != HTTPStatus.OK:
            return None
        return r.json()["task_id"]

    def test_tasks_crud(self):
        r = post("/api/v1/tasks", json={
            "title": f"E2E Task CRUD {self.suffix}",
            "description": "CRUD test",
            "status": "active",
        })
        if r.status_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/fixtures/signal_hub/test_signals.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/fixtures/signal_hub/test_signals.toml`
- Size bytes / Размер в байтах: `509`
- Included characters / Включено символов: `509`
- Truncated / Обрезано: `no`

```toml
schema_version = 1

[[fixtures]]
fixture_id = "fixture_basic_message"
source = "fixture"
event_type = "signal.raw.fixture.message.observed"
source_id = "fixture-message-001"
subject_kind = "signal"
subject_entity_id = "fixture-message-001"
occurred_at = "2026-01-01T00:00:00Z"
correlation_id = "fixture-basic-message"

[fixtures.payload]
message_key = "fixture-message-001"
summary = "Fixture message"
text = "Representative fixture signal"

[fixtures.provenance]
catalog = "signal_hub"
kind = "test_fixture"
```

### `backend/src/integrations/telegram/runtime/manager/chat_events/tests/archive_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_events/tests/archive_reconciliation.rs`
- Size bytes / Размер в байтах: `12166`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::TelegramChat;
use crate::integrations::telegram::tdjson::TelegramTdlibChatPositionSnapshot;
use crate::platform::events::EventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_position_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_position_event_reconciles_archive_command_when_provider_chat_is_archived() {
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-archive-reconcile-1";
    let provider_chat_id = "chat-archive-reconcile-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed archive chat");
    let command_id = "cmd-archive-reconcile-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "archive",
        "archive:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed archive command");

    let event_bus = EventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "archive".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("archive command status");
    assert_eq!(row, ("completed".to_owned(), "observed".to_owned()));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("archive command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_reconciles_unarchive_command_when_provider_chat_is_unarchived()
{
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unarchive-reconcile-1";
    let provider_chat_id = "chat-unarchive-reconcile-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed unarchive chat");
    let command_id = "cmd-unarchive-reconcile-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unarchive",
        "unarchive:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": false,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed unarchive command");

    let event_bus = EventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "main".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("unarchive command status");
    assert_eq!(row, ("completed".to_owned(), "observed".to_owned()));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("unarchive command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_marks_archive_command_as_mismatch_when_provider_disagrees() {
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-archive-mismatch-1";
    let provider_chat_id = "chat-archive-mismatch-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed archive mismatch chat");
    let command_id = "cmd-archive-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "archive",
        "archive-mismatch:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed archive mismatch command");

    let event_bus = EventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "main".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("archive mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different archive state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_archived"], json!(true));
    assert_eq!(row.3["observed_is_archived"], json!(false));
    assert_eq!(row.4["observed_via"], json!("tdlib.updateChatPosition"));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("archive mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_marks_unarchive_command_as_mismatch_when_provider_disagrees() {
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unarchive-mismatch-1";
    let provider_chat_id = "chat-unarchive-mismatch-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed unarchive mismatch chat");
    let command_id = "cmd-unarchive-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unarchive",
        "unarchive-mismatch:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": false,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed unarchive mismatch command");

    let event_bus = EventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "archive".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("unarchive mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different archive state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_archived"], json!(false));
    assert_eq!(row.3["observed_is_archived"], json!(true));
    assert_eq!(row.4["observed_via"], json!("tdlib.updateChatPosition"));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("unarchive mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, tel
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/chat_events/tests/mark_unread_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_events/tests/mark_unread_reconciliation.rs`
- Size bytes / Размер в байтах: `6382`
- Included characters / Включено символов: `6382`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use testkit::context::TestContext;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::TelegramChat;
use crate::integrations::telegram::tdjson::TelegramTdlibChatMarkedAsUnreadSnapshot;
use crate::platform::events::EventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_marked_as_unread_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_marked_as_unread_event_reconciles_mark_unread_command_and_emits_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-mark-unread-reconcile";
    let provider_chat_id = "chat-mark-unread-reconcile";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-mark-unread-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "mark_unread",
        "mark_unread:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_marked_as_unread": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed mark_unread command");

    let event_bus = EventBus::new();
    publish_chat_marked_as_unread_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatMarkedAsUnreadSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            is_marked_as_unread: true,
            source_event: "updateChatIsMarkedAsUnread".to_owned(),
        },
    )
    .await;

    let row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("mark_unread command status");
    assert_eq!(row, ("completed".to_owned(), "observed".to_owned()));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("mark_unread command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_marked_as_unread_event_marks_mark_unread_as_mismatch_when_provider_disagrees()
{
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-mark-unread-mismatch";
    let provider_chat_id = "chat-mark-unread-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-mark-unread-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "mark_unread",
        "mark_unread:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_marked_as_unread": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed mark_unread command");

    let event_bus = EventBus::new();
    publish_chat_marked_as_unread_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatMarkedAsUnreadSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            is_marked_as_unread: false,
            source_event: "updateChatIsMarkedAsUnread".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("mark_unread mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different unread state than requested".to_owned())
    );
    assert_eq!(row.3["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(row.3["expected_is_marked_as_unread"], json!(true));
    assert_eq!(row.3["observed_is_marked_as_unread"], json!(false));
    assert_eq!(
        row.3["observed_via"],
        json!("tdlib.updateChatIsMarkedAsUnread")
    );
    assert_eq!(row.4["mismatch"], json!(true));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("mark_unread mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["result_payload"]["mismatch"], json!(true));
}
```

### `backend/src/integrations/telegram/runtime/manager/chat_events/tests/pin_mute_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_events/tests/pin_mute_reconciliation.rs`
- Size bytes / Размер в байтах: `6837`
- Included characters / Включено символов: `6837`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::TelegramChat;
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatNotificationSettingsSnapshot, TelegramTdlibChatPositionSnapshot,
};
use crate::platform::events::EventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_notification_settings_event;
use super::super::publish_chat_position_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_position_event_marks_pin_command_as_mismatch_when_provider_disagrees() {
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-pin-mismatch";
    let provider_chat_id = "chat-pin-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-pin-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "pin",
        "pin:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_pinned": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed pin command");

    let event_bus = EventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "main".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("pin mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different dialog pin state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_pinned"], json!(true));
    assert_eq!(row.3["observed_is_pinned"], json!(false));
    assert_eq!(row.4["observed_via"], json!("tdlib.updateChatPosition"));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("pin mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_notification_settings_event_marks_unmute_command_as_mismatch_when_provider_disagrees()
 {
    let ctx = testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unmute-mismatch";
    let provider_chat_id = "chat-unmute-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-unmute-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unmute",
        "unmute:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_muted": false,
            "use_default_mute_for": true,
            "mute_for": 0,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed unmute command");

    let event_bus = EventBus::new();
    publish_chat_notification_settings_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatNotificationSettingsSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            use_default_mute_for: false,
            mute_for: 31_708_800,
            source_event: "updateChatNotificationSettings".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("unmute mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different mute state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_muted"], json!(false));
    assert_eq!(row.3["observed_is_muted"], json!(true));
    assert_eq!(
        row.4["observed_via"],
        json!("tdlib.updateChatNotificationSettings")
    );

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("unmute mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}
```

### `backend/src/integrations/telegram/tdjson/tests/environment.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/tests/environment.rs`
- Size bytes / Размер в байтах: `1348`
- Included characters / Включено символов: `1348`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;

#[cfg(target_os = "macos")]
#[test]
fn macos_tdjson_candidates_prefer_bundled_tauri_resources() {
    let exe_dir = Path::new("/Applications/Макошь.app/Contents/MacOS");
    let cwd = Path::new("/workspace/makosh");
    let candidates =
        super::super::tdjson_library_candidates_with_context(None, Some(exe_dir), Some(cwd));
    let bundled_resource = Path::new("/Applications/Макошь.app/Contents/Resources")
        .join("tdlib")
        .join(super::super::tdjson_platform_dir())
        .join("libtdjson.dylib");
    let dev_resource = cwd
        .join("frontend/src-tauri/resources/tdlib")
        .join(super::super::tdjson_platform_dir())
        .join("libtdjson.dylib");

    assert_eq!(candidates.first(), Some(&bundled_resource));
    assert!(candidates.contains(&dev_resource));
    assert!(
        candidates
            .iter()
            .position(|candidate| candidate == &bundled_resource)
            < candidates
                .iter()
                .position(|candidate| candidate == Path::new("/opt/homebrew/lib/libtdjson.dylib"))
    );
}

#[test]
fn renders_tdlib_qr_link_as_svg() {
    let svg = super::super::render_qr_svg("tg://login?token=test-token").expect("QR SVG");

    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.len() > 100);
}
```

### `backend/src/integrations/telegram/tdjson/tests/parsing_snapshots.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/tests/parsing_snapshots.rs`
- Size bytes / Размер в байтах: `15847`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;

#[test]
fn parses_tdlib_file_snapshot_from_download_file_response() {
    let file = super::super::parse_tdlib_file_snapshot(&json!({
        "@type": "file",
        "id": 42,
        "size": 2048,
        "expected_size": 4096,
        "local": {
            "@type": "localFile",
            "path": "docker/data/telegram/account/files/document.pdf",
            "can_be_downloaded": true,
            "is_downloading_active": false,
            "is_downloading_completed": true,
            "downloaded_size": 2048
        },
        "remote": {
            "@type": "remoteFile",
            "id": "remote-file-id",
            "unique_id": "remote-unique-id",
            "is_uploading_active": false,
            "is_uploading_completed": false,
            "uploaded_size": 0
        }
    }))
    .expect("file snapshot");

    assert_eq!(file.file_id, 42);
    assert_eq!(file.size_bytes, Some(2048));
    assert_eq!(file.expected_size_bytes, Some(4096));
    assert_eq!(
        file.local_path.as_deref(),
        Some("docker/data/telegram/account/files/document.pdf")
    );
    assert!(file.is_downloading_completed);
    assert!(!file.is_downloading_active);
    assert_eq!(file.remote_unique_id.as_deref(), Some("remote-unique-id"));
    assert_eq!(file.downloaded_size_bytes, Some(2048));
}

#[test]
fn parses_tdlib_chat_snapshot_from_chat_object() {
    let chat = super::super::parse_tdlib_chat_snapshot(&json!({
        "@type": "chat",
        "id": 123456789,
        "type": {
            "@type": "chatTypeSupergroup",
            "supergroup_id": 555,
            "is_channel": true
        },
        "title": "Release Channel",
        "last_message": {
            "@type": "message",
            "id": 42,
            "date": 1781352000
        },
        "metadata": {"ignored": true}
    }))
    .expect("chat snapshot");

    assert_eq!(chat.provider_chat_id, "123456789");
    assert_eq!(chat.chat_kind.as_str(), "channel");
    assert_eq!(chat.title, "Release Channel");
    assert_eq!(chat.username, None);
    assert_eq!(
        chat.last_message_at.expect("last message").to_rfc3339(),
        "2026-06-13T12:00:00+00:00"
    );
    assert_eq!(chat.raw["@type"], "chat");
}

#[test]
fn parses_tdlib_chat_members_with_roles_and_permissions() {
    let members = super::super::parse_tdlib_chat_member_list(&json!({
        "@type": "chatMembers",
        "total_count": 2,
        "members": [
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 42},
                "status": {
                    "@type": "chatMemberStatusCreator",
                    "is_member": true,
                    "custom_title": "Owner"
                }
            },
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 43},
                "status": {
                    "@type": "chatMemberStatusAdministrator",
                    "can_be_edited": false,
                    "rights": {
                        "@type": "chatAdministratorRights",
                        "can_invite_users": true,
                        "can_delete_messages": true
                    }
                }
            }
        ]
    }))
    .expect("chat member list");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].provider_member_id, "user:42");
    assert_eq!(members[0].role, "owner");
    assert!(members[0].is_owner);
    assert!(members[0].is_admin);
    assert_eq!(members[0].permissions["custom_title"], "Owner");
    assert_eq!(members[1].provider_member_id, "user:43");
    assert_eq!(members[1].role, "admin");
    assert!(members[1].is_admin);
    assert!(!members[1].is_owner);
    assert_eq!(members[1].permissions["rights"]["can_invite_users"], true);
}

#[test]
fn parses_tdlib_basic_group_full_info_members() {
    let members = super::super::parse_tdlib_basic_group_member_list(&json!({
        "@type": "basicGroupFullInfo",
        "creator_user_id": 42,
        "members": [
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 42},
                "status": {
                    "@type": "chatMemberStatusCreator",
                    "is_member": true
                }
            },
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 77},
                "status": {
                    "@type": "chatMemberStatusMember",
                    "member_until_date": 0
                }
            }
        ]
    }))
    .expect("basic group member list");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].provider_member_id, "user:42");
    assert_eq!(members[0].role, "owner");
    assert_eq!(members[1].provider_member_id, "user:77");
    assert_eq!(members[1].role, "member");
    assert_eq!(members[1].status, "member");
}

#[test]
fn parses_tdlib_typing_update_from_user_chat_action() {
    let typing = super::super::parse_tdlib_typing_snapshot(&json!({
        "@type": "updateUserChatAction",
        "chat_id": -1001234567890_i64,
        "message_thread_id": 42,
        "sender_id": {
            "@type": "messageSenderUser",
            "user_id": 777
        },
        "action": {
            "@type": "chatActionTyping"
        }
    }))
    .expect("typing snapshot");

    assert_eq!(typing.provider_chat_id, "-1001234567890");
    assert_eq!(typing.provider_thread_id.as_deref(), Some("42"));
    assert_eq!(typing.sender_id, "user:777");
    assert_eq!(typing.action, "chatActionTyping");
    assert!(typing.is_active);
}

#[test]
fn parses_tdlib_typing_cancel_as_inactive() {
    let typing = super::super::parse_tdlib_typing_snapshot(&json!({
        "@type": "updateUserChatAction",
        "chat_id": -1001234567890_i64,
        "sender_id": {
            "@type": "messageSenderChat",
            "chat_id": -1009876543210_i64
        },
        "action": {
            "@type": "chatActionCancel"
        }
    }))
    .expect("typing snapshot");

    assert_eq!(typing.sender_id, "chat:-1009876543210");
    assert_eq!(typing.provider_thread_id, None);
    assert_eq!(typing.action, "chatActionCancel");
    assert!(!typing.is_active);
}

#[test]
fn parses_tdlib_topic_update_from_forum_topic_info() {
    let update = super::super::parse_tdlib_topic_update_snapshot(&json!({
        "@type": "updateForumTopicInfo",
        "chat_id": -1001234567890_i64,
        "info": {
            "@type": "forumTopicInfo",
            "message_thread_id": 42,
            "name": "Release notes",
            "icon": {
                "@type": "forumTopicIcon",
                "custom_emoji_id": "5368324170671202286"
            },
            "is_pinned": true,
            "is_closed": false
        }
    }))
    .expect("parse result")
    .expect("topic update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.topic.provider_topic_id, 42);
    assert_eq!(update.topic.title, "Release notes");
    assert_eq!(
        update.topic.icon_emoji.as_deref(),
        Some("5368324170671202286")
    );
    assert!(update.topic.is_pinned);
    assert!(!update.topic.is_closed);
    assert_eq!(update.topic.unread_count, 0);
    assert_eq!(update.topic.last_message_at, None);
}

#[test]
fn parses_tdlib_chat_read_inbox_update() {
    let update = super::super::parse_tdlib_chat_unread_snapshot(&json!({
        "@type": "updateChatReadInbox",
        "chat_id": -1001234567890_i64,
        "last_read_inbox_message_id": 777,
        "unread_count": 3
    }))
    .expect("parse result")
    .expect("unread update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.unread_count, Some(3));
    assert_eq!(update.unread_mention_count, None);
    assert_eq!(update.last_read_inbox_message_id.as_deref(), Some("777"));
    assert_eq!(update.source_event, "updateChatReadInbox");
}

#[test]
fn parses_tdlib_unread_mention_count_update() {
    let update = super::super::parse_tdlib_chat_unread_snapshot(&json!({
        "@type": "updateChatUnreadMentionCount",
        "chat_id": -1001234567890_i64,
        "unread_mention_count": 2
    }))
    .expect("parse result")
    .expect("unread mention update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.unread_count, None);
    assert_eq!(update.unread_mention_count, Some(2));
    assert_eq!(update.last_read_inbox_message_id, None);
    assert_eq!(update.source_event, "updateChatUnreadMentionCount");
}

#[test]
fn parses_tdlib_marked_as_unread_update() {
    let update = super::super::parse_tdlib_chat_marked_as_unread_snapshot(&json!({
        "@type": "updateChatIsMarkedAsUnread",
        "chat_id": -1001234567890_i64,
        "is_marked_as_unread": true
    }))
    .expect("parse result")
    .expect("marked unread update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert!(update.is_marked_as_unread);
    assert_eq!(update.source_event, "updateChatIsMarkedAsUnread");
}

#[test]
fn parses_tdlib_chat_notification_settings_update() {
    let update = super::super::parse_tdlib_chat_notification_settings_snapshot(&json!({
        "@type": "updateChatNotificationSettings",
        "chat_id": -1001234567890_i64,
        "notification_settings": {
            "@type": "chatNotificationSettings",
            "use_default_mute_for": false,
            "mute_for": 31708800
        }
    }))
    .expect("parse result")
    .expect("notification settings update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert!(!update.use_default_mute_for);
    assert_eq!(update.mute_for, 31_708_800);
    assert_eq!(update.source_event, "updateChatNotificationSettings");
}

#[test]
fn parses_tdlib_chat_position_update() {
    let update = super::super::parse_tdlib_chat_position_snapshot(&json!({
        "@type": "updateChatPosition",
        "chat_id": -1001234567890_i64,
        "position": {
            "@type": "chatPosition",
            "list": {
                "@type": "chatListArchive"
            },
            "order": 42,
            "is_pinned": false,
            "source": null
        }
    }))
    .expect("parse result")
    .expect("chat position update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.list_kind, "archive");
    assert_eq!(update.provider_folder_id, None);
    assert_eq!(update.order, 42);
    assert!(!update.is_pinned);
    assert_eq!(update.source_event, "updateChatPosition");
}

#[test]
fn parses_tdlib_chat_removed_from_list_snapshot() {
    let update = super::super::parse_tdlib_chat_removed_from_list_snapshot(&json!({
        "@type": "updateChatRemovedFromList",
        "chat_id": -1001234567890_i64,
        "chat_list": {
            "@type": "chatListFolder",
            "chat_folder_id": 7
        }
    }))
    .expect("parse result")
    .expect("chat removed from list update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.list_kind, "folder");
    assert_eq!(update.provider_folder_id, Some(7));
    assert_eq!(update.source_event, "updateChatRemovedFromList");
    assert_eq!(update.raw["@type"], "updateChatRemovedFromList");
}

#[test]
fn parses_tdlib_chat_folder_snapshot() {
    let snapshot = super::super::parse_tdlib_chat_folder_snapshot(&json!({
        "@type": "chatFolder",
        "id": 7,
        "name": {
            "@type": "chatFolderName",
            "text": "Projects"
        },
        "icon": {
            "@type": "chatFolderIcon",
            "name": "Custom"
        },
        "color_id": 3
    }))
    .expect("parse result")
    .expect("chat folder snapshot");

    assert_eq!(snapshot.provider_folder_id, 7);
    assert_eq!(snapshot.title, "Projects");
    assert_eq!(snapshot.icon_name.as_deref(), Some("Custom"));
    assert_eq!(snapshot
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/tdjson/tests/qr_login_flows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/tests/qr_login_flows.rs`
- Size bytes / Размер в байтах: `7434`
- Included characters / Включено символов: `7434`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStatus,
};

#[test]
fn wait_password_state_is_not_a_qr_request_state() {
    assert!(!super::super::state_allows_qr_request(
        "authorizationStateWaitPassword"
    ));
}

#[test]
fn password_waiting_response_does_not_expose_stale_qr_token() {
    let response = super::super::password_waiting_response(
        "setup-id",
        "telegram-account",
        "Telegram requires your 2-step verification password.",
    );

    assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(response.poll_after_ms, 2_000);
}

#[test]
fn ready_response_for_existing_tdlib_session_does_not_expose_qr_token() {
    let identity = super::super::TelegramQrLoginIdentity {
        user_id: "123456789".to_owned(),
        username: Some("Test_User".to_owned()),
        suggested_account_id: "123456789_account_test_user".to_owned(),
        suggested_display_name: "@Test_User".to_owned(),
        suggested_external_account_id: "telegram:123456789".to_owned(),
    };

    let response = super::super::ready_response(
        "setup-id",
        "telegram-account",
        "Telegram TDLib session is already authorized.",
        Some(&identity),
    );

    assert_eq!(response.status, TelegramQrLoginStatus::Ready);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(
        response.suggested_account_id.as_deref(),
        Some("123456789_account_test_user")
    );
    assert_eq!(
        response.suggested_display_name.as_deref(),
        Some("@Test_User")
    );
    assert_eq!(
        response.suggested_external_account_id.as_deref(),
        Some("telegram:123456789")
    );
}

#[test]
fn qr_preparing_response_is_cancellable_without_exposing_qr_token() {
    let response = super::super::qr_preparing_response("setup-id", "telegram-account");

    assert_eq!(response.setup_id, "setup-id");
    assert_eq!(response.status, TelegramQrLoginStatus::WaitingQrScan);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(response.poll_after_ms, 1_000);
}

#[test]
fn qr_password_submission_sends_command_to_pending_session() {
    let (command_tx, command_rx) = mpsc::channel();
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingPassword),
            command_tx,
            worker_completion: super::super::new_worker_completion(),
        },
    )])));

    let login_check_value = "tdlib-check-value".to_owned();

    let response = super::super::submit_qr_login_password(
        pending,
        "setup-id",
        TelegramQrLoginPasswordRequest {
            password: login_check_value.clone(),
        },
    )
    .expect("password accepted");

    assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
    assert_eq!(
        response.message.as_deref(),
        Some("Checking Telegram password.")
    );
    assert_eq!(
        command_rx.try_recv().expect("password command"),
        super::super::TelegramQrLoginCommand::CheckPassword(login_check_value)
    );
}

#[test]
fn qr_password_submission_requires_waiting_password_status() {
    let (command_tx, command_rx) = mpsc::channel();
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
            command_tx,
            worker_completion: super::super::new_worker_completion(),
        },
    )])));

    let login_check_value = "tdlib-check-value".to_owned();

    let error = super::super::submit_qr_login_password(
        pending,
        "setup-id",
        TelegramQrLoginPasswordRequest {
            password: login_check_value,
        },
    )
    .expect_err("password must not be accepted before TDLib asks for it");

    assert!(matches!(error, TelegramError::InvalidRequest(_)));
    assert!(command_rx.try_recv().is_err());
}

#[test]
fn qr_login_cancel_removes_pending_session_and_notifies_worker() {
    let (command_tx, command_rx) = mpsc::channel();
    let worker_completion = super::super::new_worker_completion();
    super::super::mark_worker_complete(&worker_completion);
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
            command_tx,
            worker_completion,
        },
    )])));

    super::super::cancel_qr_login(Arc::clone(&pending), "setup-id").expect("QR login cancelled");

    assert!(
        !pending
            .lock()
            .expect("pending lock")
            .contains_key("setup-id")
    );
    assert_eq!(
        command_rx.try_recv().expect("cancel command"),
        super::super::TelegramQrLoginCommand::Cancel
    );
}

#[test]
fn qr_login_cancel_unknown_setup_returns_not_found() {
    let pending = Arc::new(Mutex::new(HashMap::new()));

    let error = super::super::cancel_qr_login(pending, "missing-setup")
        .expect_err("unknown QR setup must not be cancelled");

    assert!(matches!(error, TelegramError::QrLoginNotFound));
}

#[test]
fn qr_login_start_cancels_existing_sessions_for_same_account() {
    let (same_account_tx, same_account_rx) = mpsc::channel();
    let (other_account_tx, other_account_rx) = mpsc::channel();
    let same_account_completion = super::super::new_worker_completion();
    let other_account_completion = super::super::new_worker_completion();
    super::super::mark_worker_complete(&same_account_completion);
    super::super::mark_worker_complete(&other_account_completion);
    let mut other_response = super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan);
    other_response.setup_id = "other-setup-id".to_owned();
    other_response.account_id = "other-account".to_owned();
    let pending = Arc::new(Mutex::new(HashMap::from([
        (
            "setup-id".to_owned(),
            super::super::TelegramQrLoginSession {
                response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                command_tx: same_account_tx,
                worker_completion: same_account_completion,
            },
        ),
        (
            "other-setup-id".to_owned(),
            super::super::TelegramQrLoginSession {
                response: other_response,
                command_tx: other_account_tx,
                worker_completion: other_account_completion,
            },
        ),
    ])));

    super::super::cancel_existing_qr_logins_for_account(&pending, "telegram-account")
        .expect("same-account sessions cancelled");

    let pending = pending.lock().expect("pending lock");
    assert!(!pending.contains_key("setup-id"));
    assert!(pending.contains_key("other-setup-id"));
    assert_eq!(
        same_account_rx.try_recv().expect("same account cancel"),
        super::super::TelegramQrLoginCommand::Cancel
    );
    assert!(other_account_rx.try_recv().is_err());
}
```

### `backend/src/integrations/telegram/tdjson/tests/request_builders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/tests/request_builders.rs`
- Size bytes / Размер в байтах: `18590`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;

use crate::integrations::telegram::client::TelegramQrLoginStartRequest;
use crate::integrations::telegram::runtime::TelegramMediaSendType;

#[test]
fn tdlib_parameters_use_legacy_nested_shape_for_tdlib_1_8_runtime() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::set_tdlib_parameters_request(
        &request,
        Path::new("docker/data/telegram/telegram-qr"),
    )
    .expect("TDLib parameters");

    assert_eq!(command["@type"], "setTdlibParameters");
    assert_eq!(command["parameters"]["api_id"], 12345);
    assert_eq!(command["parameters"]["api_hash"], "telegram-api-hash");
    assert_eq!(command["parameters"]["enable_storage_optimizer"], true);
    assert_eq!(command["parameters"]["ignore_file_names"], false);
    assert_eq!(
        command["parameters"]["database_encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_eq!(command["database_encryption_key"], serde_json::Value::Null);
}

#[test]
fn tdlib_database_key_check_uses_same_base64_key_without_plaintext_secret() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::check_database_encryption_key_request(&request);

    assert_eq!(command["@type"], "checkDatabaseEncryptionKey");
    assert_eq!(
        command["encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_ne!(command["encryption_key"], "telegram-session-key");
}

#[test]
fn tdlib_send_text_message_request_uses_formatted_text_content() {
    let command = super::super::tdlib_send_text_message_request(
        123456789,
        "Hello from Макошь",
        "makosh-send-message-1",
    )
    .expect("send message request");

    assert_eq!(command["@type"], "sendMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "makosh-send-message-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["@type"],
        "formattedText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["text"],
        "Hello from Макошь"
    );
    assert_eq!(
        command["input_message_content"]["text"]["entities"],
        json!([])
    );
    assert_eq!(command["input_message_content"]["clear_draft"], true);
}

#[test]
fn tdlib_send_media_message_request_uses_local_document_content() {
    let command = super::super::tdlib_send_media_message_request(
        123456789,
        TelegramMediaSendType::Document,
        "/tmp/makosh/upload.pdf",
        Some("Document caption"),
        Some("upload.pdf"),
        "makosh-send-media-1",
    )
    .expect("send media request");

    assert_eq!(command["@type"], "sendMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "makosh-send-media-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageDocument"
    );
    assert_eq!(
        command["input_message_content"]["document"]["@type"],
        "inputFileLocal"
    );
    assert_eq!(
        command["input_message_content"]["document"]["path"],
        "/tmp/makosh/upload.pdf"
    );
    assert_eq!(
        command["input_message_content"]["caption"]["text"],
        "Document caption"
    );
}

#[test]
fn tdlib_send_media_message_request_rejects_empty_local_path() {
    let result = super::super::tdlib_send_media_message_request(
        123456789,
        TelegramMediaSendType::Photo,
        "   ",
        None,
        None,
        "makosh-send-media-empty",
    );

    assert!(result.is_err());
}

#[test]
fn tdlib_get_chat_history_request_caps_limit_to_tdlib_page_size() {
    let command = super::super::tdlib_get_chat_history_request(
        123456789,
        Some(98765),
        500,
        true,
        "makosh-history-1",
    );

    assert_eq!(command["@type"], "getChatHistory");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["from_message_id"], 98765);
    assert_eq!(command["offset"], 0);
    assert_eq!(command["limit"], 100);
    assert_eq!(command["only_local"], true);
    assert_eq!(command["@extra"], "makosh-history-1");
}

#[test]
fn tdlib_download_file_request_uses_synchronous_on_demand_download() {
    let command = super::super::tdlib_download_file_request(42, 16, "makosh-download-file-42");

    assert_eq!(command["@type"], "downloadFile");
    assert_eq!(command["file_id"], 42);
    assert_eq!(command["priority"], 16);
    assert_eq!(command["offset"], 0);
    assert_eq!(command["limit"], 0);
    assert_eq!(command["synchronous"], true);
    assert_eq!(command["@extra"], "makosh-download-file-42");
}

#[test]
fn tdlib_create_forum_topic_request_uses_expected_shape() {
    let command = super::super::tdlib_create_forum_topic_request(
        123456789,
        "Release planning",
        "makosh-topic-create-1",
    )
    .expect("topic create request");

    assert_eq!(command["@type"], "createForumTopic");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["name"], "Release planning");
    assert_eq!(command["icon_custom_emoji_id"], 0);
    assert_eq!(command["@extra"], "makosh-topic-create-1");
}

#[test]
fn tdlib_create_forum_topic_request_rejects_empty_title() {
    let result =
        super::super::tdlib_create_forum_topic_request(123456789, "   ", "makosh-topic-create-2");

    assert!(result.is_err());
}

#[test]
fn tdlib_edit_chat_folder_remove_chat_request_preserves_shape_and_excludes_chat() {
    let command = super::super::tdlib_edit_chat_folder_remove_chat_request(
        7,
        222,
        &json!({
            "@type": "chatFolder",
            "name": {
                "@type": "chatFolderName",
                "text": "Projects",
                "animate_custom_emoji": false
            },
            "icon": {
                "@type": "chatFolderIcon",
                "name": "Custom"
            },
            "color_id": 3,
            "is_shareable": false,
            "pinned_chat_ids": [111, 222],
            "included_chat_ids": [222, 333],
            "excluded_chat_ids": [444],
            "exclude_muted": false,
            "exclude_read": true,
            "exclude_archived": false,
            "include_contacts": true,
            "include_non_contacts": false,
            "include_bots": false,
            "include_groups": true,
            "include_channels": true
        }),
        "makosh-folder-remove-1",
    )
    .expect("folder remove request");

    assert_eq!(command["@type"], "editChatFolder");
    assert_eq!(command["chat_folder_id"], 7);
    assert_eq!(command["@extra"], "makosh-folder-remove-1");
    assert_eq!(command["folder"]["name"]["text"], "Projects");
    assert_eq!(command["folder"]["icon"]["name"], "Custom");
    assert_eq!(command["folder"]["pinned_chat_ids"], json!([111]));
    assert_eq!(command["folder"]["included_chat_ids"], json!([333]));
    assert_eq!(command["folder"]["excluded_chat_ids"], json!([444, 222]));
    assert_eq!(command["folder"]["exclude_read"], true);
    assert_eq!(command["folder"]["include_channels"], true);
}

#[test]
fn tdlib_toggle_forum_topic_is_closed_request_uses_expected_shape() {
    let command = super::super::tdlib_toggle_forum_topic_is_closed_request(
        123456789,
        555,
        true,
        "makosh-topic-close-1",
    );

    assert_eq!(command["@type"], "toggleForumTopicIsClosed");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_thread_id"], 555);
    assert_eq!(command["is_closed"], true);
    assert_eq!(command["@extra"], "makosh-topic-close-1");
}

#[test]
fn tdlib_edit_message_text_request_uses_edit_message_text_type() {
    let command = super::super::tdlib_edit_message_text_request(
        123456789,
        987654321,
        "Updated text",
        "makosh-edit-cmd-1",
    )
    .expect("edit message request");

    assert_eq!(command["@type"], "editMessageText");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["@extra"], "makosh-edit-cmd-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["text"],
        "Updated text"
    );
}

#[test]
fn tdlib_edit_message_text_request_rejects_empty_text() {
    let result = super::super::tdlib_edit_message_text_request(123, 456, "   ", "makosh-edit-1");
    assert!(result.is_err());
}

#[test]
fn tdlib_delete_messages_request_uses_delete_messages_type() {
    let command = super::super::tdlib_delete_messages_request(
        123456789,
        &[111, 222],
        true,
        "makosh-delete-1",
    );

    assert_eq!(command["@type"], "deleteMessages");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_ids"], json!([111, 222]));
    assert_eq!(command["revoke"], true);
    assert_eq!(command["@extra"], "makosh-delete-1");
}

#[test]
fn tdlib_add_message_reaction_request_uses_add_message_reaction_type() {
    let command = super::super::tdlib_add_message_reaction_request(
        123456789,
        987654321,
        "👍",
        "makosh-react-1",
    );

    assert_eq!(command["@type"], "addMessageReaction");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["reaction_type"]["@type"], "reactionTypeEmoji");
    assert_eq!(command["reaction_type"]["emoji"], "👍");
    assert_eq!(command["is_big"], false);
    assert_eq!(command["@extra"], "makosh-react-1");
}

#[test]
fn tdlib_remove_message_reaction_request_uses_remove_message_reaction_type() {
    let command = super::super::tdlib_remove_message_reaction_request(
        123456789,
        987654321,
        "👍",
        "makosh-unreact-1",
    );

    assert_eq!(command["@type"], "removeMessageReaction");
    assert_eq!(command["reaction_type"]["emoji"], "👍");
    assert_eq!(command["@extra"], "makosh-unreact-1");
}

#[test]
fn tdlib_pin_chat_message_request_uses_pin_chat_message_type() {
    let command =
        super::super::tdlib_pin_chat_message_request(123456789, 987654321, false, "makosh-pin-1");

    assert_eq!(command["@type"], "pinChatMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["disable_notification"], false);
    assert_eq!(command["only_for_self"], false);
    assert_eq!(command["@extra"], "makosh-pin-1");
}

#[test]
fn tdlib_unpin_chat_message_request_uses_unpin_chat_message_type() {
    let command =
        super::super::tdlib_unpin_chat_message_request(123456789, 987654321, "makosh-unpin-1");

    assert_eq!(command["@type"], "unpinChatMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["@extra"], "makosh-unpin-1");
}

#[test]
fn tdlib_toggle_chat_marked_as_unread_re
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/config/app_config/test_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/test_support.rs`
- Size bytes / Размер в байтах: `2408`
- Included characters / Включено символов: `2408`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use crate::platform::config::ConfigError;

use super::AppConfig;
use super::env::apply_config_pair;

impl AppConfig {
    pub fn test_with_api_secret(api_secret: impl Into<String>) -> Self {
        Self {
            local_api_secret: Some(api_secret.into()),
            zoom_token_maintenance_scheduler_enabled: false,
            zoom_recording_sync_scheduler_enabled: false,
            zoom_retention_cleanup_scheduler_enabled: false,
            ..Self::default()
        }
    }

    pub fn test_with_api_secret_and_database_url(
        api_secret: impl Into<String>,
        database_url: impl Into<String>,
    ) -> Self {
        let mut config = Self::test_with_api_secret(api_secret);
        config.database_url = Some(database_url.into());
        config
    }

    pub fn with_test_database_url(mut self, database_url: impl Into<String>) -> Self {
        self.database_url = Some(database_url.into());
        self
    }

    pub fn with_test_api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.local_api_secret = Some(api_secret.into());
        self
    }

    pub fn with_test_pairs<I, K, V>(mut self, pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in pairs {
            apply_config_pair(&mut self, key.as_ref(), value.as_ref())?;
        }

        Ok(self)
    }

    pub fn with_test_dev_mode(mut self) -> Self {
        self.dev_mode = true;
        self
    }

    pub fn with_test_dev_vault_paths(
        mut self,
        vault_home: impl Into<PathBuf>,
        dev_key_path: impl Into<PathBuf>,
    ) -> Self {
        self.dev_mode = true;
        self.vault_home = vault_home.into();
        self.dev_key_path = dev_key_path.into();
        self
    }

    pub fn with_test_tdjson_path(mut self, tdjson_path: impl Into<PathBuf>) -> Self {
        self.tdjson_path = Some(tdjson_path.into());
        self
    }

    pub fn with_test_telegram_app_credentials(
        mut self,
        api_id: i64,
        api_hash: impl AsRef<str>,
    ) -> Self {
        self.telegram_api_id = Some(api_id);
        self.telegram_api_hash = Some(
            crate::platform::secrets::ResolvedSecret::new(api_hash.as_ref())
                .expect("test Telegram API hash must be valid"),
        );
        self
    }
}
```

### `backend/src/test_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/test_support.rs`
- Size bytes / Размер в байтах: `3400`
- Included characters / Включено символов: `3400`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use serde_json::Value;
use sqlx::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::domains::signal_hub::{SignalHubStore, SignalRuntimeStateUpdate};
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::whatsapp::client::WhatsappWebStore;
use crate::platform::communications::StoredRawCommunicationRecord;
use crate::platform::communications::{EmailProviderKind, NewProviderAccount, ProviderAccount};

pub fn communication_provider_account_store(pool: &PgPool) -> CommunicationProviderAccountStore {
    CommunicationProviderAccountStore::new(pool.clone())
}

pub fn communication_provider_secret_binding_store(
    pool: &PgPool,
) -> CommunicationProviderSecretBindingStore {
    CommunicationProviderSecretBindingStore::new(pool.clone())
}

pub fn telegram_store(pool: &PgPool) -> TelegramStore {
    TelegramStore::new(
        pool.clone(),
        Arc::new(communication_provider_account_store(pool)),
        Arc::new(communication_provider_secret_binding_store(pool)),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone()),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    )
}

pub fn whatsapp_web_store(pool: &PgPool) -> WhatsappWebStore {
    WhatsappWebStore::new(
        pool.clone(),
        Arc::new(communication_provider_account_store(pool)),
        Arc::new(communication_provider_secret_binding_store(pool)),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
    )
}

pub async fn upsert_telegram_runtime_account(
    pool: &PgPool,
    account_id: &str,
    display_name: &str,
    external_account_id: &str,
) -> ProviderAccount {
    communication_provider_account_store(pool)
        .upsert(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::TelegramUser,
            display_name,
            external_account_id,
        ))
        .await
        .expect("seed Telegram provider account")
}

pub async fn restore_signal_hub_system_sources(pool: &PgPool) {
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore Signal Hub system sources");
}

pub async fn set_signal_runtime_state(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    state: &str,
    metadata: Value,
) {
    SignalHubStore::new(pool.clone())
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: source_code.to_owned(),
            runtime_kind: runtime_kind.to_owned(),
            state: state.to_owned(),
            metadata,
        })
        .await
        .expect("set Signal Hub runtime state");
}

pub async fn load_communication_raw_record(
    pool: &PgPool,
    raw_record_id: &str,
) -> StoredRawCommunicationRecord {
    crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await
        .expect("load communication raw record")
        .expect("stored communication raw record")
}
```

### `backend/tests/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai.rs`
- Size bytes / Размер в байтах: `168`
- Included characters / Включено символов: `168`
- Truncated / Обрезано: `no`

```rust
#[path = "ai/agents.rs"]
mod agents;
#[path = "ai/answers.rs"]
mod answers;
#[path = "ai/semantic_store.rs"]
mod semantic_store;
#[path = "ai/support.rs"]
mod support;
```

### `backend/tests/ai/agents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai/agents.rs`
- Size bytes / Размер в байтах: `8885`
- Included characters / Включено символов: `8885`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;
use testkit::context::TestContext;

#[tokio::test]
async fn ai_meeting_prep_returns_briefing_without_calendar_dependency() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let project_id = format!("project:v1:ai-meeting:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("AI Meeting Project {suffix}"),
                "Product Development",
                "Meeting prep project",
                "Alex Morgan",
                vec![format!("MeetingPrep{suffix}")],
            )
            .progress(42),
        )
        .await
        .expect("project");
    seed_document(
        &pool,
        &format!("ai_meeting_doc_{suffix}"),
        &format!("MeetingPrep{suffix} notes"),
        "Discuss V3 AI risks and validation.",
    )
    .await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str())])
            .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/meeting-prep",
            json!({
                "command_id": format!("meeting-prep-{suffix}"),
                "topic": "V3 AI implementation review",
                "project_id": project_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["agent_id"], json!("HESTIA"));
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(
        body["briefing"],
        json!("Discuss V3 risks and validation evidence.")
    );
    assert!(!body["citations"].as_array().expect("citations").is_empty());
    let run_id = body["run_id"].as_str().expect("run id");

    let raw_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
            'signal.raw.ai.run_requested.observed',
            'signal.raw.ai.run_completed.observed',
            'signal.accepted.ai.run_requested',
            'signal.accepted.ai.run_completed'
          )
          AND subject->>'run_id' = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("ai meeting prep signal hub event count");
    assert_eq!(raw_signal_count, 4);
}

#[tokio::test]
async fn ai_status_and_agents_are_protected() {
    let app = build_router(config_with_api_token());

    let missing_token = app
        .clone()
        .oneshot(get_request("/api/v1/ai/status"))
        .await
        .expect("response");
    assert_eq!(missing_token.status(), StatusCode::FORBIDDEN);

    let agents = app
        .oneshot(get_request_with_token("/api/v1/ai/agents", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(agents.status(), StatusCode::OK);
    let body = json_body(agents).await;
    let items = body["items"].as_array().expect("agents");
    assert_eq!(items.len(), 5);
    assert_eq!(items[0]["agent_id"], json!("HESTIA"));
    assert_eq!(items[0]["display_name"], json!("hestia@sh-inc.ru"));
    assert!(
        items
            .iter()
            .any(|item| item["agent_id"] == json!("HEPHAESTUS")),
        "HEPHAESTUS must be part of the initial AI agent registry"
    );
}

#[tokio::test]
async fn ai_agents_api_materializes_agent_personas_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token("/api/v1/ai/agents", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("agents");
    let hestia = items
        .iter()
        .find(|item| item["agent_id"] == "HESTIA")
        .expect("HESTIA descriptor");
    assert_eq!(hestia["persona_id"], "persona:v1:ai_agent:HESTIA");
    assert_eq!(hestia["persona_type"], "ai_agent");
    assert_eq!(hestia["persona_email"], "hestia@sh-inc.ru");
    let hephaestus = items
        .iter()
        .find(|item| item["agent_id"] == "HEPHAESTUS")
        .expect("HEPHAESTUS descriptor");
    assert_eq!(hephaestus["persona_id"], "persona:v1:ai_agent:HEPHAESTUS");
    assert_eq!(hephaestus["persona_type"], "ai_agent");
    assert_eq!(hephaestus["persona_email"], "hephaestus@sh-inc.ru");

    let row = sqlx::query(
        r#"
        SELECT display_name, person_type, email_address, is_self
        FROM persons
        WHERE person_id = 'persona:v1:ai_agent:HESTIA'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA Persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "hestia@sh-inc.ru"
    );
    assert_eq!(row.try_get::<String, _>("person_type").unwrap(), "ai_agent");
    assert_eq!(
        row.try_get::<String, _>("email_address").unwrap(),
        "hestia@sh-inc.ru"
    );
    assert!(!row.try_get::<bool, _>("is_self").unwrap());

    let identity_value: String = sqlx::query_scalar(
        r#"
        SELECT identity_value
        FROM person_identities
        WHERE person_id = 'persona:v1:ai_agent:HESTIA'
          AND identity_type = 'email'
          AND source = 'ai_agent_registry'
          AND status = 'active'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA email identity");
    assert_eq!(identity_value, "hestia@sh-inc.ru");

    let graph_row = sqlx::query(
        r#"
        SELECT label, properties
        FROM graph_nodes
        WHERE node_kind = 'person'
          AND stable_key = 'persona:v1:ai_agent:HESTIA'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HESTIA graph node");
    assert_eq!(
        graph_row.try_get::<String, _>("label").unwrap(),
        "hestia@sh-inc.ru"
    );
    let properties = graph_row
        .try_get::<serde_json::Value, _>("properties")
        .unwrap();
    assert_eq!(properties["persona_type"], "ai_agent");
    assert_eq!(properties["agent_id"], "HESTIA");

    let row = sqlx::query(
        r#"
        SELECT display_name, person_type, email_address, is_self
        FROM persons
        WHERE person_id = 'persona:v1:ai_agent:HEPHAESTUS'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS Persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    assert_eq!(row.try_get::<String, _>("person_type").unwrap(), "ai_agent");
    assert_eq!(
        row.try_get::<String, _>("email_address").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    assert!(!row.try_get::<bool, _>("is_self").unwrap());

    let identity_value: String = sqlx::query_scalar(
        r#"
        SELECT identity_value
        FROM person_identities
        WHERE person_id = 'persona:v1:ai_agent:HEPHAESTUS'
          AND identity_type = 'email'
          AND source = 'ai_agent_registry'
          AND status = 'active'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS email identity");
    assert_eq!(identity_value, "hephaestus@sh-inc.ru");

    let graph_row = sqlx::query(
        r#"
        SELECT label, properties
        FROM graph_nodes
        WHERE node_kind = 'person'
          AND stable_key = 'persona:v1:ai_agent:HEPHAESTUS'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("HEPHAESTUS graph node");
    assert_eq!(
        graph_row.try_get::<String, _>("label").unwrap(),
        "hephaestus@sh-inc.ru"
    );
    let properties = graph_row
        .try_get::<serde_json::Value, _>("properties")
        .unwrap();
    assert_eq!(properties["persona_type"], "ai_agent");
    assert_eq!(properties["agent_id"], "HEPHAESTUS");
}
```

### `backend/tests/ai/answers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai/answers.rs`
- Size bytes / Размер в байтах: `12680`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use crate::support::*;
use makosh_hub_backend::domains::signal_hub::{
    SignalHubStore, SignalPolicy, SignalPolicyMode, SignalPolicyScope,
};
use testkit::context::TestContext;

#[tokio::test]
async fn ai_answer_api_returns_source_backed_answer_and_persists_run() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let person_store = PersonProjectionStore::new(pool.clone());
    let owner = person_store
        .upsert_email_person(&format!("ai-owner-{suffix}@example.com"))
        .await
        .expect("owner persona candidate");
    let owner = person_store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");
    let retrieval_token = format!("V3AIAnswer{suffix}");
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-answer-{suffix}@example.com"),
        &[format!("ai-recipient-{suffix}@example.com")],
        &format!("provider-ai-answer-{suffix}"),
        &format!("Макошь AI roadmap {retrieval_token}"),
        &format!("The V3 AI plan for {retrieval_token} uses Ollama and source-backed citations."),
    )
    .await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str()),
                ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
                ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
            ])
            .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-{suffix}"),
                "query": format!("V3 AI plan for {retrieval_token}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["agent_id"], json!("MNEMOSYNE"));
    assert_eq!(
        body["agent_persona_id"],
        json!("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(body["owner_persona_id"], json!(owner.person_id));
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(body["model"], json!("qwen3:4b"));
    assert_eq!(body["embedding_model"], json!("qwen3-embedding:4b"));
    assert_eq!(body["answer"], json!("Макошь V3 is source-backed."));
    assert!(body["duration_ms"].as_i64().expect("duration") >= 0);

    let citations = body["citations"].as_array().expect("citations");
    assert!(!citations.is_empty());
    assert!(citations.iter().any(|citation| {
        citation["source_kind"] == json!("message") && citation["source_id"] == json!(message_id)
    }));

    let run_id = body["run_id"].as_str().expect("run id");
    let stored = AiRunStore::new(pool.clone())
        .get_run(run_id)
        .await
        .expect("load run")
        .expect("stored run");
    assert_eq!(
        stored.answer.as_deref(),
        Some("Макошь V3 is source-backed.")
    );
    assert_eq!(stored.status, "completed");

    let run_observations: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'agent_run'
          AND link.entity_id = $1
          AND kind.code IN ('AI_AGENT_RUN', 'AI_AGENT_RUN_STATUS')
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run observations");
    assert!(
        run_observations >= 2,
        "expected run requested + completed observations"
    );

    let raw_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
            'signal.raw.ai.run_requested.observed',
            'signal.raw.ai.run_completed.observed',
            'signal.accepted.ai.run_requested',
            'signal.accepted.ai.run_completed'
          )
          AND subject->>'run_id' = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("ai signal hub event count");
    assert_eq!(raw_signal_count, 4);

    let run_attribution = sqlx::query(
        r#"
        SELECT agent_persona_id, owner_persona_id
        FROM ai_agent_runs
        WHERE run_id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("run attribution");
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("agent_persona_id")
            .unwrap()
            .as_deref(),
        Some("persona:v1:ai_agent:MNEMOSYNE")
    );
    assert_eq!(
        run_attribution
            .try_get::<Option<String>, _>("owner_persona_id")
            .unwrap()
            .as_deref(),
        Some(owner.person_id.as_str())
    );
}

#[tokio::test]
async fn ai_answer_api_is_blocked_when_ai_source_is_muted_by_signal_hub() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    SignalHubStore::new(pool.clone())
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("ai".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "mute AI while debugging signal controls".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create ai mute policy");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str()),
                ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
                ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
            ])
            .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/answers",
            json!({
                "command_id": format!("answer-blocked-{suffix}"),
                "query": format!("Blocked AI query {suffix}"),
                "agent_id": "MNEMOSYNE"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED, "body={body}");
    assert_eq!(body["error"], json!("failed_precondition"));
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|message| message.contains("AI runtime is disabled by Signal Hub policy"))
    );

    let run_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM ai_agent_runs")
        .fetch_one(&pool)
        .await
        .expect("run count");
    assert_eq!(run_count, 0);
}

#[tokio::test]
async fn ai_task_refresh_creates_suggested_candidates_without_active_tasks() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let _guard = AI_RUNTIME_TEST_LOCK.lock().await;
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    configure_fake_ollama_setting(&pool, &ollama_base_url).await;
    let suffix = unique_suffix();
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("ai-task-{suffix}@example.com"),
        &[format!("ai-task-recipient-{suffix}@example.com")],
        &format!("provider-ai-task-{suffix}"),
        "AI task source",
        &format!("Please review the V3 implementation checklist {suffix}."),
    )
    .await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", ollama_base_url.as_str())])
            .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/ai/task-candidates/refresh",
            json!({
                "command_id": format!("task-refresh-{suffix}"),
                "query": format!("Please review the V3 implementation checklist {suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["status"], json!("completed"));
    assert_eq!(body["created_count"], json!(1));
    let run_id = body["run_id"].as_str().expect("run id");

    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let candidate = sqlx::query(
        r#"
        SELECT task_candidate_id, review_state, agent_run_id, observation_id
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("candidate");
    assert_eq!(candidate.get::<String, _>("review_state"), "suggested");
    assert!(candidate.get::<Option<String>, _>("agent_run_id").is_some());
    assert_eq!(
        candidate
            .get::<Option<String>, _>("observation_id")
            .as_deref(),
        Some(message_observation_id.as_str())
    );

    let active_task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE source_id = $1")
            .bind(&message_observation_id)
            .fetch_one(&pool)
            .await
            .expect("active task count");
    assert_eq!(active_task_count, 0);

    let review_item_row = sqlx::query(
        r#"
        SELECT item_kind, status, metadata
        FROM review_items
        WHERE review_item_id IN (
            SELECT review_item_id
            FROM review_item_evidence
            WHERE observation_id = $1
        )
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("review item row");
    assert_eq!(
        review_item_row.get::<String, _>("item_kind"),
        "potential_task"
    );
    assert_eq!(review_item_row.get::<String, _>("status"), "new");
    let metadata = review_item_row.get::<serde_json::Value, _>("metadata");
    assert_eq!(metadata["mirrored_from"], json!("task_candidates"));

    let raw_signal_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/ai/semantic_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai/semantic_store.rs`
- Size bytes / Размер в байтах: `5119`
- Included characters / Включено символов: `5119`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;
use chrono::Utc;
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use sqlx::Row;
use testkit::context::TestContext;

#[tokio::test]
async fn pgvector_semantic_store_indexes_and_searches_sources_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = SemanticEmbeddingStore::new(pool.clone());
    let observation_store = ObservationStore::new(pool.clone());
    let embedding_model = format!("qwen3-embedding:4b-semantic-{suffix}");
    let message_observation = observation_store
        .capture(&NewObservation::new(
            "COMMUNICATION_MESSAGE",
            ObservationOriginKind::TestFixture,
            Utc::now(),
            json!({"source": "semantic_store_test", "suffix": suffix.to_string()}),
            format!("semantic-store-test://message/{suffix}"),
        ))
        .await
        .expect("message observation");

    let extension_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(&pool)
            .await
            .expect("vector extension");
    assert!(extension_exists);

    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            observation_id: Some(message_observation.observation_id.as_str()),
            title: "Roadmap planning",
            source_text: "Discussed Макошь AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("upsert first embedding");
    store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Document,
            source_id: &format!("document-semantic-{suffix}"),
            observation_id: None,
            title: "Garden notes",
            source_text: "Tomatoes need watering this weekend.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(8),
            graph_node_id: None,
        })
        .await
        .expect("upsert second embedding");

    let indexed = store
        .upsert_embedding(NewSemanticEmbedding {
            source_kind: SemanticSourceKind::Message,
            source_id: &format!("message-semantic-{suffix}"),
            observation_id: Some(message_observation.observation_id.as_str()),
            title: "Roadmap planning",
            source_text: "Discussed Макошь AI roadmap and local retrieval.",
            embedding_model: &embedding_model,
            embedding: &unit_embedding(0),
            graph_node_id: Some(&format!("graph:message:{suffix}")),
        })
        .await
        .expect("idempotent upsert");
    assert_eq!(indexed.source_kind, "message");
    assert_eq!(
        indexed.observation_id.as_deref(),
        Some(message_observation.observation_id.as_str())
    );
    assert_eq!(indexed.embedding_dimension, 2560);
    let semantic_observations = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'semantic_embedding'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&indexed.semantic_embedding_id)
    .fetch_all(&pool)
    .await
    .expect("semantic embedding observations");
    assert!(
        semantic_observations.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_SEMANTIC_EMBEDDING"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<serde_json::Value, _>("payload")["observation_id"]
                    == serde_json::Value::String(message_observation.observation_id.clone())
        }),
        "semantic embedding upsert observation must exist"
    );

    let results = store
        .search(&embedding_model, &unit_embedding(0), 5)
        .await
        .expect("search");
    assert_eq!(results[0].source_kind, "message");
    assert_eq!(results[0].source_id, format!("message-semantic-{suffix}"));
    assert_eq!(
        results[0].observation_id.as_deref(),
        Some(message_observation.observation_id.as_str())
    );
    assert!(results[0].score > results[1].score);
    assert_eq!(
        results[0].graph_node_id,
        Some(format!("graph:message:{suffix}"))
    );
}
```

### `backend/tests/ai/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai/support.rs`
- Size bytes / Размер в байтах: `8022`
- Included characters / Включено символов: `8022`
- Truncated / Обрезано: `no`

```rust
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Request, StatusCode, header};
pub(crate) use axum::routing::{get, post};
pub(crate) use axum::{Json, Router};
pub(crate) use makosh_hub_backend::ai::core::{
    AiRunStore, NewSemanticEmbedding, SemanticEmbeddingStore, SemanticSourceKind,
};
pub(crate) use makosh_hub_backend::app::{build_router, build_router_with_database};
pub(crate) use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
pub(crate) use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
pub(crate) use makosh_hub_backend::domains::documents::core::{
    DocumentImportStore, NewDocumentImport,
};
pub(crate) use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
pub(crate) use makosh_hub_backend::domains::projects::core::{NewProject, ProjectStore};
pub(crate) use makosh_hub_backend::platform::config::AppConfig;
pub(crate) use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
pub(crate) use makosh_hub_backend::platform::storage::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use sqlx::Row;
pub(crate) use sqlx::postgres::PgPool;
pub(crate) use tokio::net::TcpListener;
pub(crate) use tower::ServiceExt;

pub(crate) const LOCAL_API_TOKEN: &str = "ai-api-test-token";
pub(crate) static AI_RUNTIME_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

pub(crate) async fn spawn_fake_ollama() -> String {
    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            get(|| async {
                Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/embed",
            post(|Json(_body): Json<Value>| async {
                Json(json!({
                    "model": "qwen3-embedding:4b",
                    "embeddings": [unit_embedding(0)],
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 8u32
                }))
            }),
        )
        .route(
            "/api/chat",
            post(|Json(body): Json<Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Return JSON task candidates") {
                    r#"[{"source_kind":"message","source_id":"__first__","title":"Review the V3 implementation checklist","evidence_excerpt":"Please review the V3 implementation checklist.","confidence":0.82}]"#
                } else if text.contains("meeting briefing") {
                    "Discuss V3 risks and validation evidence."
                } else {
                    "Макошь V3 is source-backed."
                };

                Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": content },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

pub(crate) async fn configure_fake_ollama_setting(pool: &PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama setting");
}

pub(crate) fn unit_embedding(active_index: usize) -> Vec<f32> {
    let mut embedding = vec![0.0; 2560];
    embedding[active_index] = 1.0;
    embedding
}

pub(crate) async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let account_id = format!("ai-account-{suffix}");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id.clone(),
                EmailProviderKind::Imap,
                format!("AI Account {suffix}"),
                format!("ai-external-{suffix}"),
            )
            .config(json!({
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": format!("ai-{suffix}@example.com")
            })),
        )
        .await
        .expect("provider account");

    let raw_record = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_ai_{suffix}_{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("fingerprint-ai-{suffix}-{provider_record_id}"),
                format!("batch-ai-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body
                }),
            )
            .provenance(json!({"source":"ai_test"})),
        )
        .await
        .expect("raw record");

    let projected = project_raw_email_message(&message_store, &raw_record)
        .await
        .expect("project raw message");

    projected.message_id
}

pub(crate) async fn seed_document(
    pool: &PgPool,
    fingerprint: &str,
    title: &str,
    text: &str,
) -> String {
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(fingerprint, title, text))
        .await
        .expect("document")
        .document_id
}

pub(crate) fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) fn get_request(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
```

### `backend/tests/ai_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai_architecture.rs`
- Size bytes / Размер в байтах: `1872`
- Included characters / Включено символов: `1872`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn ai_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_ai_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "ai test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_ai_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_ai_test_violations(&path, violations);
            continue;
        }
        if !is_ai_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_ai_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "ai.rs" || file_name == "ai_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "ai")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```
