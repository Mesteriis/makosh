ALTER TABLE persons
    ALTER COLUMN email_address DROP NOT NULL;

UPDATE persons
SET email_address = NULL,
    updated_at = now()
WHERE email_address LIKE '%@makosh.invalid';

UPDATE person_identities
SET status = 'outdated',
    updated_at = now()
WHERE identity_type = 'email'
  AND identity_value LIKE '%@makosh.invalid'
  AND status = 'active';
