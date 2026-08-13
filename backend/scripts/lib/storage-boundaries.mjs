import { createHash } from 'node:crypto';

import {
  ownerAliases,
  pathTokens,
  violation,
} from './validation-diagnostics.mjs';
import {
  forbiddenMigrationConstructs,
  sqlReferencedObjects,
} from './sql-inspection.mjs';

const MODULE_ROLES = new Set(['domain', 'integration', 'workflow', 'engine']);

function tableName(identifier) {
  return identifier.replaceAll('"', '').toLowerCase().split('.').at(-1);
}

function hasOwnerPrefix(identifier, owner) {
  const name = tableName(identifier);
  return [...ownerAliases(owner)].some((alias) => name === alias || name.startsWith(`${alias}_`));
}

function knownOwners(policy) {
  return new Set([
    ...policy.domains.registered,
    ...policy.projections.blockedOwners,
    ...policy.owners.platform,
    ...policy.owners.api,
    policy.owners.core,
  ]);
}

function referencedOwner(policy, identifier) {
  const firstToken = pathTokens(tableName(identifier))[0];
  return [...knownOwners(policy)].find((owner) => ownerAliases(owner).has(firstToken));
}

function identifierParts(identifier) {
  return identifier
    .replaceAll('"', '')
    .toLowerCase()
    .split('.')
    .filter(Boolean);
}

function expectedPostgresSchema(role) {
  if (MODULE_ROLES.has(role)) return 'makosh_data';
  if (role === 'platform') return 'makosh_platform';
  return null;
}

function isVersionedPlatformFunction(policy, reference) {
  if (reference.kind !== 'function') return false;
  return (policy.storage.sharedTechnicalFunctions ?? []).includes(
    reference.identifier.toLowerCase(),
  );
}

function isMigration(path) {
  return path.toLowerCase().split(/[\\/]+/u).includes('migrations');
}

function isDownMigration(path) {
  const lower = path.toLowerCase();
  const segments = lower.split(/[\\/]+/u);
  const migrationIndex = segments.indexOf('migrations');
  if (migrationIndex === -1) return false;
  const migrationPath = segments.slice(migrationIndex + 1);
  if (migrationPath.slice(0, -1).includes('down')) return true;
  const fileName = migrationPath.at(-1) ?? '';
  return /(?:^|[._-])down(?:[._-]|$)/u.test(fileName);
}

function isExactPersonsProfileImmutabilityGuard(entry, construct) {
  if (
    entry.path !== 'src/persons-persistence/migrations/0001_persons.sql'
    || entry.packageName !== 'makosh-persons-persistence'
    || entry.role !== 'domain'
    || entry.owner !== 'persons'
    || entry.surface !== 'persistence'
    || !new Set(['function_or_procedure', 'trigger', 'dynamic_or_prepared_sql']).has(construct)
  ) return false;
  const functions = entry.content.match(
    /\b(?:CREATE(?:\s+OR\s+REPLACE)?|ALTER)\s+(?:FUNCTION|PROCEDURE)\b/giu,
  ) ?? [];
  const triggers = entry.content.match(
    /\b(?:CREATE\s+(?:CONSTRAINT\s+)?|ALTER\s+)TRIGGER\b/giu,
  ) ?? [];
  const dynamicTokens = entry.content.match(/\b(?:PREPARE|EXECUTE|DEALLOCATE)\b/giu) ?? [];
  const exactFunction = entry.content.match(
    /CREATE\s+FUNCTION\s+makosh_data\.persons_reject_profile_history_mutation\s*\(\s*\)\s+RETURNS\s+TRIGGER\s+LANGUAGE\s+plpgsql\s+AS\s+\$\$([\s\S]*?)\$\$\s*;/iu,
  );
  const exactBody = exactFunction?.[1]?.trim().replace(/\s+/gu, ' ');
  const exactTrigger = /CREATE\s+TRIGGER\s+persons_profiles_immutable\s+BEFORE\s+UPDATE\s+OR\s+DELETE\s+ON\s+makosh_data\.persons_profiles\s+FOR\s+EACH\s+ROW\s+EXECUTE\s+FUNCTION\s+makosh_data\.persons_reject_profile_history_mutation\s*\(\s*\)\s*;/iu;
  return functions.length === 1
    && triggers.length === 1
    && dynamicTokens.length === 1
    && dynamicTokens[0].toUpperCase() === 'EXECUTE'
    && exactBody === "BEGIN RAISE EXCEPTION 'persons profile history is immutable' USING ERRCODE = '55000'; END;"
    && exactTrigger.test(entry.content);
}

function isExactTasksLifecycleMigration(entry, construct) {
  return entry.path === 'src/tasks-persistence/migrations/0002_tasks_lifecycle_owner_rls.sql'
    && entry.packageName === 'makosh-tasks-persistence'
    && entry.role === 'domain'
    && entry.owner === 'tasks'
    && entry.surface === 'persistence'
    && new Set(['drop', 'destructive_alter']).has(construct)
    && createHash('sha256').update(entry.content, 'utf8').digest('hex')
      === 'b089988877ff2837e894fd47411a21a7ff4a00e917085fee296647c4ffbc7ade';
}

function isExactKnowledgeLifecycleMigration(entry, construct) {
  return entry.path === 'src/knowledge-persistence/migrations/0002_knowledge_lifecycle_owner_rls.sql'
    && entry.packageName === 'makosh-knowledge-persistence'
    && entry.role === 'domain'
    && entry.owner === 'knowledge'
    && entry.surface === 'persistence'
    && new Set(['drop', 'destructive_alter']).has(construct)
    && createHash('sha256').update(entry.content, 'utf8').digest('hex')
      === '67d64e39f8150e9c889b6ac3544ea889f91010ab1b7d58005be7375cb09e4d6e';
}

export function validateStorageEntries(policy, entries) {
  const violations = [];
  const emitted = new Set();
  const sqlitePackages = new Set(policy.storage.sqlitePackages);

  function emit(code, location, message) {
    const key = `${code}\0${location}\0${message}`;
    if (!emitted.has(key)) violations.push(violation(code, location, message));
    emitted.add(key);
  }

  for (const entry of entries) {
    if (!entry.packageName || !entry.role || !entry.owner || !entry.surface) {
      emit('orphan_sql', entry.path, 'SQL must belong to a registered workspace package');
      continue;
    }
    if (entry.surface !== 'persistence') {
      emit(
        'sql_outside_persistence',
        entry.path,
        `SQL is forbidden in ${entry.packageName} surface=${entry.surface}`,
      );
      continue;
    }

    if (isMigration(entry.path)) {
      if (isDownMigration(entry.path)) {
        emit('down_migration', entry.path, 'V1 storage accepts forward-only migration files');
      }
      for (const construct of forbiddenMigrationConstructs(entry.content)) {
        if (isExactPersonsProfileImmutabilityGuard(entry, construct)) continue;
        if (isExactTasksLifecycleMigration(entry, construct)) continue;
        if (isExactKnowledgeLifecycleMigration(entry, construct)) continue;
        emit(
          'forbidden_migration_construct',
          entry.path,
          `${construct} is rejected by the heuristic V1 migration guard`,
        );
      }
    }

    const sqlitePackage = sqlitePackages.has(entry.packageName);
    const expectedSchema = sqlitePackage ? null : expectedPostgresSchema(entry.role);
    if (!sqlitePackage && expectedSchema === null) {
      emit(
        'unsupported_sql_role',
        entry.path,
        `role=${entry.role} has no PostgreSQL persistence schema`,
      );
    }

    for (const reference of sqlReferencedObjects(entry.content)) {
      const { identifier } = reference;
      if (
        identifier.toLowerCase() === 'or'
        && isExactPersonsProfileImmutabilityGuard(entry, 'trigger')
      ) continue;
      const parts = identifierParts(identifier);

      if (!sqlitePackage && reference.kind === 'index') {
        if (parts.length !== 1) {
          emit(
            'forbidden_sql_schema',
            entry.path,
            `${identifier} index name must be unqualified; PostgreSQL derives its schema from the target table`,
          );
        }
      } else if (!sqlitePackage && reference.kind === 'function' && parts[0] === 'makosh_platform') {
        if (isVersionedPlatformFunction(policy, reference)) continue;
        emit(
          'invalid_platform_function',
          entry.path,
          `${identifier} is not an exact allowlisted makosh_platform technical function`,
        );
        continue;
      } else if (!sqlitePackage && parts.length < 2) {
        emit(
          'unqualified_sql_identifier',
          entry.path,
          `${identifier} must be qualified with ${expectedSchema ?? 'an allowed'} schema`,
        );
      } else if (!sqlitePackage && (parts.length !== 2 || parts[0] !== expectedSchema)) {
        emit(
          'forbidden_sql_schema',
          entry.path,
          `${identifier} is outside role=${entry.role} schema ${expectedSchema ?? '<none>'}`,
        );
      }

      if (hasOwnerPrefix(identifier, entry.owner)) continue;
      const otherOwner = referencedOwner(policy, identifier);
      if (otherOwner && otherOwner !== entry.owner) {
        emit(
          'cross_owner_sql',
          entry.path,
          `${entry.owner} persistence cannot access ${identifier} owned by ${otherOwner}`,
        );
      } else {
        emit(
          'unowned_sql_identifier',
          entry.path,
          `${identifier} must use the ${entry.owner}_ owner prefix`,
        );
      }
    }
  }

  return violations;
}
