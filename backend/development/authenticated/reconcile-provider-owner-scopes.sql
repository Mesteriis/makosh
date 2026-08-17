-- Development-only reconciliation for owner-scoped provider databases that
-- survived a Storage instance identity cutover. The durable human owner must
-- remain the exact development owner; only the fenced runtime-role prefix is
-- advanced to the current Storage role ledger binding.
DO $reconcile_provider_owner_scopes$
DECLARE
    provider RECORD;
    current_prefix TEXT;
BEGIN
    FOR provider IN
        SELECT * FROM (VALUES
            ('telegram', 'telegram_owner_scope'),
            ('whatsapp', 'whatsapp_owner_scope'),
            ('zulip', 'zulip_owner_scope')
        ) AS providers(owner_id, scope_table)
    LOOP
        IF to_regclass(format('makosh_data.%I', provider.scope_table)) IS NULL THEN
            CONTINUE;
        END IF;

        SELECT regexp_replace(runtime_principal, '_[0-9]+$', '')
        INTO current_prefix
        FROM makosh_platform.storage_role_ledger
        WHERE owner_id = provider.owner_id
          AND runtime_principal ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$';

        IF current_prefix IS NULL THEN
            CONTINUE;
        END IF;

        EXECUTE format(
            'UPDATE makosh_data.%I
             SET runtime_principal_prefix = $1
             WHERE singleton = TRUE
               AND logical_owner_id = $2
               AND runtime_principal_prefix <> $1',
            provider.scope_table
        )
        USING current_prefix, 'development-owner';
    END LOOP;
END
$reconcile_provider_owner_scopes$;
