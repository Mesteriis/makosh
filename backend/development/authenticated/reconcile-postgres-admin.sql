-- Development-only reconciliation for a persisted PostgreSQL volume.
-- The password stays in the Docker secret mount and is never passed through
-- the host environment, a process argument, or command output.
DO $makosh_reconcile_postgres_admin$
DECLARE
    credential text;
BEGIN
    IF current_user <> 'makosh_postgres_admin' THEN
        RAISE EXCEPTION 'unexpected development PostgreSQL bootstrap role';
    END IF;

    credential := regexp_replace(
        pg_read_file('/run/secrets/storage_postgres_admin_password'),
        E'[\\r\\n]+$',
        ''
    );
    IF octet_length(credential) = 0 OR octet_length(credential) > 65536 THEN
        RAISE EXCEPTION 'development PostgreSQL bootstrap credential is invalid';
    END IF;

    EXECUTE format('ALTER ROLE %I PASSWORD %L', current_user, credential);
END
$makosh_reconcile_postgres_admin$;
