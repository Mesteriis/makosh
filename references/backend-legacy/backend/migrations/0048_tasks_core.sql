-- Phase 0: Extend tasks table with full domain model

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS priority_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS risk_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS readiness_score REAL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS source_type TEXT DEFAULT 'manual';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS area TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS why TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS outcome TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS due_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS makosh_status TEXT DEFAULT 'new';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS waiting_reason TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS energy_type TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS confidentiality TEXT DEFAULT 'private_local';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS tags JSONB DEFAULT '[]';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS task_metadata JSONB DEFAULT '{}';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS linked_person_id TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS linked_organization_id TEXT;

-- Drop old constraint, add new
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS tasks_status_check;
ALTER TABLE tasks ADD CONSTRAINT tasks_makosh_status_check CHECK (makosh_status IN ('new','triaged','ready','in_progress','waiting','blocked','review','done','cancelled','archived'));
ALTER TABLE tasks ADD CONSTRAINT tasks_source_type_check CHECK (source_type IN ('manual','email','telegram','whatsapp','calendar','meeting','document','note','jira','youtrack','github','gitlab','linear','todoist','apple_reminders','ms_todo','ai_rule','workflow','import'));
ALTER TABLE tasks ADD CONSTRAINT tasks_confidentiality_check CHECK (confidentiality IN ('public_to_provider','private_local','sensitive','confidential'));

CREATE INDEX IF NOT EXISTS tasks_makosh_status_idx ON tasks (makosh_status);
CREATE INDEX IF NOT EXISTS tasks_due_at_idx ON tasks (due_at);
CREATE INDEX IF NOT EXISTS tasks_priority_idx ON tasks (priority_score DESC NULLS LAST);
CREATE INDEX IF NOT EXISTS tasks_person_idx ON tasks (linked_person_id);
CREATE INDEX IF NOT EXISTS tasks_org_idx ON tasks (linked_organization_id);
