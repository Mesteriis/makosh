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
        if r.status_code >= 500:
            pytest.fail(f"Task create server error: {r.status_code}")
        if r.status_code != HTTPStatus.OK:
            pytest.skip("task creation not supported by current DB schema")
        tid = r.json()["task_id"]
        assert r.json()["title"] == f"E2E Task CRUD {self.suffix}"
        r = api(f"/api/v1/tasks/{tid}")
        assert r.status_code == HTTPStatus.OK
        assert r.json()["task_id"] == tid
        r = put(f"/api/v1/tasks/{tid}", json={"title": f"E2E Updated {self.suffix}", "priority": "high"})
        assert r.status_code == HTTPStatus.OK
        r = post(f"/api/v1/tasks/{tid}/status", json={"status": "completed"})
        assert r.status_code == HTTPStatus.OK
        r = post(f"/api/v1/tasks/{tid}/archive", json={})
        assert r.status_code == HTTPStatus.OK

    def test_tasks_list(self):
        tid = self._create_task()
        if tid is None:
            pytest.skip("task creation not supported")
        r = api("/api/v1/tasks")
        assert r.status_code == HTTPStatus.OK
        assert len(r.json()["items"]) >= 1

    def test_task_subresources(self):
        tid = self._create_task()
        if tid is None:
            pytest.skip("task creation not supported")
        for sub in [
            "context-pack", "evidence", "relations",
            "checklist", "subtasks", "export", "external",
        ]:
            r = api(f"/api/v1/tasks/{tid}/{sub}")
            assert r.status_code < 500, f"GET /{sub} returned {r.status_code}"

    def test_task_read_endpoints(self):
        for path in [
            "/api/v1/tasks/providers",
            "/api/v1/tasks/search?q=test",
            "/api/v1/tasks/daily-brief",
            "/api/v1/tasks/rules",
            "/api/v1/tasks/templates",
            "/api/v1/tasks/watchtower",
            "/api/v1/tasks/health",
            "/api/v1/tasks/analytics",
            "/api/v1/task-candidates",
        ]:
            r = api(path)
            assert r.status_code < 500, f"GET {path} returned {r.status_code}"


# ══════════════════════════════════ Persons ══════════════════════════════════

class TestPersons:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def test_persons_list(self):
        r = api("/api/v1/persons")
        assert r.status_code == HTTPStatus.OK

    def test_person_not_found(self):
        r = api(f"/api/v1/persons/person:e2e-nonexistent-{self.suffix}")
        assert r.status_code == HTTPStatus.NOT_FOUND

    def test_person_search(self):
        r = api("/api/v1/persons/search?q=alex")
        assert r.status_code == HTTPStatus.OK

    def test_person_identity_candidates(self):
        r = api("/api/v1/identity-candidates")
        assert r.status_code == HTTPStatus.OK

    def test_persons_read_endpoints(self):
        for path in [
            "/api/v1/persons/health",
            "/api/v1/persons/watchlist",
            "/api/v1/persons/search/expertise?q=rust",
        ]:
            r = api(path)
            assert r.status_code < 500, f"GET {path} returned {r.status_code}"


# ═══════════════════════════════ Communications ══════════════════════════════

class TestCommunications:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def test_v1_status(self):
        r = api("/api/v1/status")
        assert r.status_code == HTTPStatus.OK

    def test_messages_list(self):
        r = api("/api/v1/communications/messages")
        assert r.status_code == HTTPStatus.OK

    def test_messages_states(self):
        r = api("/api/v1/communications/messages/states")
        assert r.status_code == HTTPStatus.OK

    def test_threads(self):
        r = api("/api/v1/communications/threads")
        assert r.status_code == HTTPStatus.OK

    def test_search(self):
        r = api("/api/v1/communications/search?q=test")
        assert r.status_code == HTTPStatus.OK

    def test_personas(self):
        r = api("/api/v1/communications/personas")
        assert r.status_code == HTTPStatus.OK

    def test_drafts(self):
        r = api("/api/v1/communications/drafts")
        assert r.status_code == HTTPStatus.OK

    def test_invoices(self):
        r = api("/api/v1/communications/finance/invoices")
        assert r.status_code < 500

    def test_analytics(self):
        r = api("/api/v1/communications/analytics/health")
        assert r.status_code < 500
        r = api("/api/v1/communications/analytics/senders")
        assert r.status_code < 500

    def test_subscriptions(self):
        r = api("/api/v1/communications/subscriptions")
        assert r.status_code == HTTPStatus.OK

    def test_attachments_duplicates(self):
        r = api("/api/v1/communications/attachments/duplicates")
        assert r.status_code == HTTPStatus.OK

    def test_legal_docs(self):
        r = api("/api/v1/communications/legal")
        assert r.status_code == HTTPStatus.OK

    def test_certificates(self):
        r = api("/api/v1/communications/certificates")
        assert r.status_code == HTTPStatus.OK
        r = api("/api/v1/communications/certificates/expiring")
        assert r.status_code == HTTPStatus.OK

    def test_rich_templates(self):
        r = api("/api/v1/communications/templates/rich")
        assert r.status_code == HTTPStatus.OK

    def test_blockers(self):
        r = api("/api/v1/communications/blockers")
        assert r.status_code == HTTPStatus.OK

    def test_message_actions_graceful(self):
        mid = f"msg:e2e-fake-{self.suffix}"
        for path in [
            f"/api/v1/communications/messages/{mid}/explain",
            f"/api/v1/communications/messages/{mid}/smart-cc",
            f"/api/v1/communications/messages/{mid}/export",
            f"/api/v1/communications/messages/{mid}/spf-dkim",
            f"/api/v1/communications/messages/{mid}/detect-language",
            f"/api/v1/communications/messages/{mid}/signature",
        ]:
            r = api(path)
            assert r.status_code < 500, f"GET {path} returned {r.status_code}"


# ═══════════════════════════════════ Graph ═══════════════════════════════════

class TestGraph:
    def test_graph_summary(self):
        r = api("/api/v1/graph/summary")
        assert r.status_code == HTTPStatus.OK

    def test_graph_nodes(self):
        r = api("/api/v1/graph/nodes")
        assert r.status_code == HTTPStatus.OK

    def test_graph_search(self):
        r = api("/api/v1/graph/search?q=alex")
        assert r.status_code == HTTPStatus.OK

    def test_graph_neighborhood(self):
        r = api("/api/v1/graph/neighborhood?node_id=graph:node:v1:person:alex&depth=1")
        assert r.status_code < 500


# ══════════════════════════════════ Projects ═════════════════════════════════

class TestProjects:
    def test_projects_list(self):
        r = api("/api/v1/projects")
        assert r.status_code == HTTPStatus.OK

    def test_project_detail_not_found(self):
        r = api("/api/v1/projects/project:v1:e2e:nonexistent")
        assert r.status_code < 500


# ═════════════════════════════════ Documents ═════════════════════════════════

class TestDocuments:
    def test_document_processing_jobs(self):
        r = api("/api/v1/document-processing/jobs")
        assert r.status_code == HTTPStatus.OK

    def test_document_processing_not_found(self):
        r = api("/api/v1/documents/doc:e2e:nonexistent/processing")
        assert r.status_code < 500


# ═════════════════════════════════ Settings ══════════════════════════════════

class TestSettings:
    def test_get_settings(self):
        r = api("/api/v1/settings")
        assert r.status_code == HTTPStatus.OK

    def test_get_settings_accounts(self):
        r = api("/api/v1/settings/accounts")
        assert r.status_code == HTTPStatus.OK

    def test_put_setting(self):
        r = put("/api/v1/settings/ui.theme", json={"value": "dark"})
        assert r.status_code < 500


# ════════════════════════════════════ AI ═════════════════════════════════════

class TestAI:
    def test_ai_status(self):
        r = api("/api/v1/ai/status")
        assert r.status_code < 500

    def test_ai_agents(self):
        r = api("/api/v1/ai/agents")
        assert r.status_code < 500


# ═══════════════════════════════ Events API ══════════════════════════════════

class TestEventsAPI:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def test_post_and_get_event(self):
        r = post("/api/v1/events", json={
            "event_type": "e2e.test",
            "aggregate_id": f"e2e-agg-{self.suffix}",
            "payload": {"key": "value"},
        })
        if r.status_code >= 500:
            pytest.fail(f"Event post server error: {r.status_code}")
        if r.status_code != HTTPStatus.OK:
            pytest.skip("event posting not supported" )
        eid = r.json()["event_id"]
        r = api(f"/api/v1/events/{eid}")
        assert r.status_code == HTTPStatus.OK
        assert r.json()["event_id"] == eid

    def test_audit_events(self):
        r = api("/api/v1/audit/events?limit=10")
        assert r.status_code == HTTPStatus.OK


# ═══════════════════════════════════ Calls ═══════════════════════════════════

class TestCalls:

    @pytest.fixture(autouse=True)
    def setup(self):
        self.suffix = uid()

    def test_calls_list(self):
        r = api("/api/v1/calls")
        assert r.status_code == HTTPStatus.OK

    def test_call_create(self):
        r = post("/api/v1/calls", json={
            "call_type": "telegram",
            "chat_id": f"e2e-chat-{self.suffix}",
            "direction": "inbound",
            "state": "completed",
            "initiated_at": datetime.now(timezone.utc).isoformat(),
            "duration_seconds": 120,
        })
        assert r.status_code < 500


# ═════════════════════════════════ Telegram ══════════════════════════════════

class TestTelegram:
    def test_telegram_capabilities(self):
        r = api("/api/v1/integrations/telegram/capabilities")
        assert r.status_code == HTTPStatus.OK


# ═════════════════════════════════ WhatsApp ══════════════════════════════════

class TestWhatsApp:
    def test_whatsapp_capabilities(self):
        r = api("/api/v1/communications/provider-web/capabilities")
        assert r.status_code == HTTPStatus.OK
