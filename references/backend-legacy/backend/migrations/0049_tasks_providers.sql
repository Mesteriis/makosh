-- Phase 1: Provider accounts, external identities, status mappings

CREATE TABLE IF NOT EXISTS task_provider_accounts (
    account_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    account_name TEXT NOT NULL,
    credentials_reference TEXT,
    sync_mode TEXT DEFAULT 'manual',
    capabilities JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT task_providers_provider_check CHECK (provider IN ('jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','trello','local')),
    CONSTRAINT task_providers_sync_check CHECK (sync_mode IN ('manual','read_only','two_way'))
);

CREATE TABLE IF NOT EXISTS external_task_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    account_id TEXT,
    external_project_id TEXT,
    external_task_id TEXT,
    external_url TEXT,
    external_status TEXT,
    sync_status TEXT DEFAULT 'synced',
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ext_task_sync_status_check CHECK (sync_status IN ('local_only','syncing','synced','conflict','error'))
);

CREATE INDEX IF NOT EXISTS ext_task_identities_task_idx ON external_task_identities (task_id);
CREATE UNIQUE INDEX IF NOT EXISTS ext_task_identities_unique_idx ON external_task_identities (provider, account_id, external_task_id) WHERE external_task_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS provider_status_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    external_status TEXT NOT NULL,
    makosh_status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(provider, external_status)
);
