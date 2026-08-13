const IDENTIFIER_PART = '(?:"[^"]+"|[a-zA-Z_][a-zA-Z0-9_$]*)';
const QUALIFIED_IDENTIFIER = `${IDENTIFIER_PART}(?:\\s*\\.\\s*${IDENTIFIER_PART}){0,2}`;

const OBJECT_REFERENCE = new RegExp(
  `\\b(create\\s+(?:table|type|sequence|view|materialized\\s+view)|alter\\s+(?:table|type|sequence|view|materialized\\s+view|index)|drop\\s+(?:table|type|sequence|view|materialized\\s+view|index)|insert\\s+into|update|delete\\s+from|references|from|join|truncate(?:\\s+table)?|copy)\\s+(?:if\\s+(?:not\\s+)?exists\\s+)?(${QUALIFIED_IDENTIFIER})`,
  'giu',
);

const INDEX_TARGET = new RegExp(
  `\\bcreate\\s+(?:unique\\s+)?index(?:\\s+concurrently)?(?:\\s+if\\s+not\\s+exists)?\\s+(${QUALIFIED_IDENTIFIER})\\s+on\\s+(${QUALIFIED_IDENTIFIER})`,
  'giu',
);

const QUALIFIED_FUNCTION_CALL = new RegExp(
  `(${IDENTIFIER_PART}\\s*\\.\\s*${IDENTIFIER_PART})\\s*\\(`,
  'giu',
);

const FORBIDDEN_MIGRATION_PATTERNS = [
  ['drop', /\bdrop\b/iu],
  ['truncate', /\btruncate\b/iu],
  ['rename', /\balter\s+(?:table|type|sequence|view|materialized\s+view|index)\b[^;]*\brename\b/iu],
  ['destructive_alter', /\balter\s+table\b[^;]*\balter\s+(?:column\s+)?\b/iu],
  ['role_database_schema_or_extension', /\b(?:create|alter|set|reset)\s+(?:role|user|database|schema|extension)\b/iu],
  ['grant_or_revoke', /\b(?:grant|revoke)\b/iu],
  ['do_block', /(?:^|;)\s*do\b/imu],
  ['dynamic_or_prepared_sql', /\b(?:prepare|execute|deallocate)\b/iu],
  ['function_or_procedure', /\b(?:create(?:\s+or\s+replace)?|alter)\s+(?:function|procedure)\b/iu],
  ['trigger', /\bcreate\s+(?:constraint\s+)?trigger\b/iu],
  ['trigger_state', /\balter\s+table\b[^;]*\b(?:enable|disable)\s+(?:(?:always|replica)\s+)?trigger\b/iu],
  ['alter_trigger', /\balter\s+trigger\b/iu],
  ['session_replication_role', /\b(?:set|reset)\s+(?:(?:local|session)\s+)?session_replication_role\b/iu],
  ['set_config', /(?:\bset_config|"set_config")\s*\(/iu],
  ['foreign_data_wrapper', /\b(?:foreign\s+data\s+wrapper|foreign\s+(?:table|server)|create\s+server|user\s+mapping|import\s+foreign\s+schema)\b/iu],
  ['copy_program', /\bcopy\b[^;]*\bprogram\b/iu],
  ['alter_system', /\balter\s+system\b/iu],
  ['concurrently', /\bconcurrently\b/iu],
  ['tablespace', /\btablespace\b/iu],
  ['load', /(?:^|;)\s*load\b/imu],
  ['nontransactional_maintenance', /(?:^|;)\s*(?:vacuum|reindex|cluster)\b/imu],
  ['transaction_control', /(?:^|;)\s*(?:begin|start\s+transaction|commit|end|rollback|abort|savepoint|release\s+savepoint)\b/imu],
];

function blankRange(output, source, start, end) {
  for (let index = start; index < end; index += 1) {
    output[index] = source[index] === '\n' ? '\n' : ' ';
  }
}

// This deliberately small scanner removes comments and literal payloads before
// the heuristic checks below. It is not a PostgreSQL parser and must not be
// treated as proof that arbitrary SQL is safe.
function sqlForHeuristicInspection(content) {
  const output = [...content];
  let index = 0;

  while (index < content.length) {
    if (content.startsWith('--', index)) {
      const end = content.indexOf('\n', index + 2);
      const boundary = end === -1 ? content.length : end;
      blankRange(output, content, index, boundary);
      index = boundary;
      continue;
    }

    if (content.startsWith('/*', index)) {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < content.length && depth > 0) {
        if (content.startsWith('/*', cursor)) {
          depth += 1;
          cursor += 2;
        } else if (content.startsWith('*/', cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      blankRange(output, content, index, cursor);
      index = cursor;
      continue;
    }

    if (content[index] === "'") {
      let cursor = index + 1;
      while (cursor < content.length) {
        if (content[cursor] !== "'") {
          cursor += 1;
          continue;
        }
        if (content[cursor + 1] === "'") {
          cursor += 2;
          continue;
        }
        cursor += 1;
        break;
      }
      blankRange(output, content, index, cursor);
      index = cursor;
      continue;
    }

    if (content[index] === '$') {
      const delimiter = content.slice(index).match(/^\$[a-zA-Z_0-9]*\$/u)?.[0];
      if (delimiter) {
        const close = content.indexOf(delimiter, index + delimiter.length);
        const boundary = close === -1 ? content.length : close + delimiter.length;
        blankRange(output, content, index, boundary);
        index = boundary;
        continue;
      }
    }

    index += 1;
  }

  return output.join('');
}

function normalizeIdentifier(identifier) {
  return identifier.replace(/\s*\.\s*/gu, '.');
}

export function sqlReferencedObjects(content) {
  const sql = sqlForHeuristicInspection(content);
  const references = new Map();

  for (const match of sql.matchAll(OBJECT_REFERENCE)) {
    const operation = match[1].toLowerCase().replace(/\s+/gu, ' ');
    const prefix = sql.slice(Math.max(0, (match.index ?? 0) - 24), match.index ?? 0);
    if (operation === 'update' && /\bon\s*$/iu.test(prefix)) continue;
    const identifier = normalizeIdentifier(match[2]);
    const remainder = sql.slice((match.index ?? 0) + match[0].length).trimStart();
    const kind = ['from', 'join'].includes(operation) && remainder.startsWith('(')
      ? 'function'
      : 'object';
    references.set(`${kind}:${identifier.toLowerCase()}`, { identifier, kind });
  }

  for (const match of sql.matchAll(INDEX_TARGET)) {
    const indexIdentifier = normalizeIdentifier(match[1]);
    references.set(`index:${indexIdentifier.toLowerCase()}`, {
      identifier: indexIdentifier,
      kind: 'index',
    });
    const targetIdentifier = normalizeIdentifier(match[2]);
    references.set(`object:${targetIdentifier.toLowerCase()}`, {
      identifier: targetIdentifier,
      kind: 'object',
    });
  }

  for (const match of sql.matchAll(QUALIFIED_FUNCTION_CALL)) {
    const identifier = normalizeIdentifier(match[1]);
    const objectKey = `object:${identifier.toLowerCase()}`;
    if (references.has(objectKey)) continue;
    references.set(`function:${identifier.toLowerCase()}`, { identifier, kind: 'function' });
  }

  return [...references.values()];
}

export function sqlReferencedIdentifiers(content) {
  return sqlReferencedObjects(content).map(({ identifier }) => identifier);
}

export function forbiddenMigrationConstructs(content) {
  const sql = sqlForHeuristicInspection(content);
  return FORBIDDEN_MIGRATION_PATTERNS
    .filter(([, pattern]) => pattern.test(sql))
    .map(([name]) => name);
}
