CREATE TABLE IF NOT EXISTS projects (
    project_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    description TEXT NOT NULL,
    owner_display_name TEXT NOT NULL,
    progress_percent INTEGER NOT NULL DEFAULT 0,
    start_date DATE,
    target_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT projects_id_not_empty CHECK (length(trim(project_id)) > 0),
    CONSTRAINT projects_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT projects_kind_not_empty CHECK (length(trim(kind)) > 0),
    CONSTRAINT projects_status CHECK (status IN ('planning', 'active', 'on_hold', 'completed', 'archived')),
    CONSTRAINT projects_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT projects_owner_not_empty CHECK (length(trim(owner_display_name)) > 0),
    CONSTRAINT projects_progress_range CHECK (progress_percent >= 0 AND progress_percent <= 100)
);

CREATE TABLE IF NOT EXISTS project_keywords (
    project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
    keyword TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT project_keywords_keyword_not_empty CHECK (length(trim(keyword)) > 0),
    PRIMARY KEY (project_id, keyword)
);

ALTER TABLE graph_nodes DROP CONSTRAINT IF EXISTS graph_nodes_kind;
ALTER TABLE graph_nodes
ADD CONSTRAINT graph_nodes_kind CHECK (
    node_kind IN ('person', 'email_address', 'message', 'document', 'project')
);

ALTER TABLE graph_edges DROP CONSTRAINT IF EXISTS graph_edges_relationship_type;
ALTER TABLE graph_edges
ADD CONSTRAINT graph_edges_relationship_type CHECK (
    relationship_type IN (
        'person_has_email_address',
        'person_sent_message',
        'person_received_message',
        'email_address_sent_message',
        'email_address_received_message',
        'project_has_message',
        'project_has_document',
        'project_involves_person',
        'project_involves_email_address'
    )
);

CREATE INDEX IF NOT EXISTS projects_status_idx ON projects (status);
CREATE INDEX IF NOT EXISTS project_keywords_project_idx ON project_keywords (project_id);
CREATE INDEX IF NOT EXISTS project_keywords_keyword_idx ON project_keywords (keyword);

INSERT INTO projects (
    project_id,
    name,
    kind,
    status,
    description,
    owner_display_name,
    progress_percent,
    start_date,
    target_date
)
VALUES (
    'project:v1:makosh',
    'Макошь',
    'Product Development',
    'active',
    'Personal knowledge system for local-first communications, documents, graph memory and workflows.',
    'Alex Morgan',
    75,
    DATE '2024-01-15',
    DATE '2024-12-20'
)
ON CONFLICT (project_id) DO NOTHING;

INSERT INTO project_keywords (project_id, keyword)
VALUES
    ('project:v1:makosh', 'Макошь'),
    ('project:v1:makosh', 'Макошь Project'),
    ('project:v1:makosh', 'makosh')
ON CONFLICT (project_id, keyword) DO NOTHING;
