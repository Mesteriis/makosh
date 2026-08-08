#!/usr/bin/env node

import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const args = new Set(process.argv.slice(2))
const requireClosed =
  args.has('--require-closed') || process.env.MAKOSH_MAIL_REQUIRE_DOMAIN_CLOSED === '1'
const gapAnalysisPath = 'docs/integrations/mail/gap-analysis.md'
const supportedStatuses = new Set([
  'IMPLEMENTED',
  'PARTIAL',
  'BROKEN',
  'MISSING',
  'REGRESSION',
  'EXCLUDED',
])

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function parseGapRows(markdown) {
  return markdown
    .split('\n')
    .map((line, lineIndex) => ({ line: line.trim(), lineNumber: lineIndex + 1 }))
    .filter(({ line }) => line.startsWith('|') && line.endsWith('|'))
    .map(({ line, lineNumber }) => ({
      cells: line
        .slice(1, -1)
        .split('|')
        .map((cell) => cell.trim()),
      lineNumber,
    }))
    .filter(({ cells }) => {
      const status = cells[1]
      return (
        cells.length >= 3 &&
        cells[0] !== 'Capability' &&
        cells[0] !== 'Feature' &&
        status !== 'Status' &&
        status !== '---' &&
        status !== ''
      )
    })
    .map(({ cells, lineNumber }) => ({
      capability: cells[0],
      status: supportedStatuses.has(cells[1]) ? cells[1] : 'INVALID',
      reportedStatus: cells[1],
      evidence: cells.slice(2).join('|').trim(),
      lineNumber,
    }))
}

function acceptedAdrIds(evidence, adrDirectoryEntries) {
  const ids = Array.from(evidence.matchAll(/\bADR-(\d{4})\b/gu), (match) => match[1])
  return ids.filter((id) => {
    const filename = adrDirectoryEntries.find((entry) => entry.startsWith(`ADR-${id}-`))
    if (!filename) return false
    return /^Status:\s*Accepted\s*$/mu.test(readText(path.join('docs/adr', filename)))
  })
}

function auditGapRows(rows, adrDirectoryEntries) {
  const checks = []
  const blockers = []

  for (const row of rows) {
    if (row.status === 'INVALID') {
      blockers.push({
        id: row.capability,
        status: 'invalid_status',
        line: row.lineNumber,
        evidence: `Unknown status: ${row.reportedStatus}`,
      })
      continue
    }
    if (row.status === 'IMPLEMENTED') {
      checks.push({ id: row.capability, status: 'implemented', line: row.lineNumber })
      continue
    }
    if (row.status === 'EXCLUDED') {
      const acceptedAdrs = acceptedAdrIds(row.evidence, adrDirectoryEntries)
      if (acceptedAdrs.length > 0) {
        checks.push({
          id: row.capability,
          status: 'excluded',
          line: row.lineNumber,
          accepted_adrs: acceptedAdrs,
        })
      } else {
        blockers.push({
          id: row.capability,
          status: 'invalid_exclusion',
          line: row.lineNumber,
          evidence: 'EXCLUDED rows must reference an accepted ADR-####.',
        })
      }
      continue
    }
    blockers.push({ id: row.capability, status: row.status.toLowerCase(), line: row.lineNumber })
  }

  return { checks, blockers }
}

function runSelfTest() {
  const rows = parseGapRows([
    '| Capability | Status | Evidence |',
    '|---|---|---|',
    '| Local state | IMPLEMENTED | Verified |',
    '| Delete | EXCLUDED | ADR-0176 documents the scope |',
    '| CDR | PARTIAL | Renderer pending |',
    '| Incorrect | COMPLETE | Unknown status |',
  ].join('\n'))
  assert.deepEqual(rows.map((row) => row.status), ['IMPLEMENTED', 'EXCLUDED', 'PARTIAL', 'INVALID'])

  const audit = auditGapRows(rows, ['ADR-0176-mail-closure-local-first-reconciliation.md'])
  assert.equal(audit.checks.length, 2)
  assert.equal(audit.blockers.length, 2)
  assert.equal(audit.blockers[0].id, 'CDR')
  assert.equal(audit.blockers[1].status, 'invalid_status')
  console.log(JSON.stringify({ ok: true, mode: 'self-test' }))
}

if (args.has('--self-test')) {
  runSelfTest()
  process.exit(0)
}

const rows = parseGapRows(readText(gapAnalysisPath))
const adrDirectoryEntries = fs.readdirSync(path.join(repoRoot, 'docs/adr'))
const { checks, blockers } = auditGapRows(rows, adrDirectoryEntries)
const closureAchieved = rows.length > 0 && blockers.length === 0
const result = {
  ok: !requireClosed || closureAchieved,
  require_closed: requireClosed,
  closure_achieved: closureAchieved,
  gap_analysis: gapAnalysisPath,
  checked_rows: rows.length,
  implemented: checks.filter((check) => check.status === 'implemented').length,
  excluded: checks.filter((check) => check.status === 'excluded').length,
  blockers,
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (!result.ok) {
  process.exitCode = 1
}
