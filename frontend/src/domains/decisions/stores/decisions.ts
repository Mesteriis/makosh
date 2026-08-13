import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  DecisionStateV1,
  type DecisionAlternativeV1,
  type DecisionEvidenceLinkV1,
  type DecisionV1,
  type TimestampV1
} from '../../../gen/makosh/decisions/client/v1/decisions_pb'
import {
  getDecisionsCommandClient,
  getDecisionsQueryClient
} from '../api/decisions'

const PAGE_LIMIT = 50

export const useDecisionsStore = defineStore('decisions-owner', () => {
  const decisions = ref<DecisionV1[]>([])
  const selectedDecision = ref<DecisionV1>()
  const alternatives = ref<DecisionAlternativeV1[]>([])
  const evidence = ref<DecisionEvidenceLinkV1[]>([])
  const error = ref('')
  const isLoading = ref(false)
  const mutatingDecisionId = ref<string | null>(null)

  const drafts = computed(() => decisions.value.filter((value) =>
    value.state === DecisionStateV1.DECISION_STATE_DRAFT
  ))
  const terminal = computed(() => decisions.value.filter((value) =>
    value.state !== DecisionStateV1.DECISION_STATE_DRAFT
  ))

  async function loadAll(): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const rows: DecisionV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      do {
        const page = await getDecisionsQueryClient().list({
          logicalOwnerId: '', afterDecisionId: cursor, limit: PAGE_LIMIT
        })
        rows.push(...page.decisions)
        cursor = page.nextAfterDecisionId
      } while (cursor.length > 0)
      decisions.value = rows
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function select(decision: DecisionV1): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      selectedDecision.value = await getDecisionsQueryClient().get({
        logicalOwnerId: '', decisionId: decision.decisionId
      })
      await loadChildren(selectedDecision.value)
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function createDecision(title: string, question: string): Promise<void> {
    await run(undefined, async () => {
      const result = await getDecisionsCommandClient().create({
        operationId: randomId16(), logicalOwnerId: '', title, question, createdAt: timestamp()
      })
      replaceResult(result.decision)
      alternatives.value = []
      evidence.value = []
    })
  }

  async function addAlternative(decision: DecisionV1, title: string, description: string): Promise<void> {
    await run(decision, async () => {
      const result = await getDecisionsCommandClient().addAlternative({
        operationId: randomId16(), decisionId: decision.decisionId, logicalOwnerId: '',
        expectedDecisionRevision: decision.decisionRevision, title, description, changedAt: timestamp()
      })
      await loadChildren(replaceResult(result.decision))
    })
  }

  async function decide(decision: DecisionV1, alternativeId: Uint8Array, rationale: string): Promise<void> {
    await run(decision, async () => {
      const result = await getDecisionsCommandClient().decide({
        operationId: randomId16(), decisionId: decision.decisionId, logicalOwnerId: '',
        expectedDecisionRevision: decision.decisionRevision,
        selectedAlternativeId: alternativeId, rationale, decidedAt: timestamp()
      })
      await loadChildren(replaceResult(result.decision))
    })
  }

  async function cancel(decision: DecisionV1): Promise<void> {
    await run(decision, async () => {
      const result = await getDecisionsCommandClient().cancel({
        operationId: randomId16(), decisionId: decision.decisionId, logicalOwnerId: '',
        expectedDecisionRevision: decision.decisionRevision, changedAt: timestamp()
      })
      replaceResult(result.decision)
    })
  }

  async function loadChildren(decision: DecisionV1): Promise<void> {
    const [alternativeRows, evidenceRows] = await Promise.all([
      loadAlternativePages(decision.decisionId),
      loadEvidencePages(decision.decisionId)
    ])
    alternatives.value = alternativeRows
    evidence.value = evidenceRows
  }

  async function run(decision: DecisionV1 | undefined, action: () => Promise<void>): Promise<void> {
    mutatingDecisionId.value = decision ? hex(decision.decisionId) : 'new'
    error.value = ''
    try {
      await action()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingDecisionId.value = null
    }
  }

  function replaceResult(decision: DecisionV1 | undefined): DecisionV1 {
    if (!decision) throw new Error('Decision response is incomplete')
    const index = decisions.value.findIndex((value) => sameBytes(value.decisionId, decision.decisionId))
    if (index < 0) decisions.value.push(decision)
    else decisions.value.splice(index, 1, decision)
    selectedDecision.value = decision
    return decision
  }

  return {
    decisions, selectedDecision, alternatives, evidence, drafts, terminal,
    error, isLoading, mutatingDecisionId, loadAll, select, createDecision,
    addAlternative, decide, cancel
  }
})

async function loadAlternativePages(decisionId: Uint8Array): Promise<DecisionAlternativeV1[]> {
  const rows: DecisionAlternativeV1[] = []
  let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
  do {
    const page = await getDecisionsQueryClient().listAlternatives({ logicalOwnerId: '', decisionId, afterAlternativeId: cursor, limit: PAGE_LIMIT })
    rows.push(...page.alternatives)
    cursor = page.nextAfterAlternativeId
  } while (cursor.length > 0)
  return rows
}

async function loadEvidencePages(decisionId: Uint8Array): Promise<DecisionEvidenceLinkV1[]> {
  const rows: DecisionEvidenceLinkV1[] = []
  let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
  do {
    const page = await getDecisionsQueryClient().listEvidence({ logicalOwnerId: '', decisionId, afterEvidenceLinkId: cursor, limit: PAGE_LIMIT })
    rows.push(...page.evidenceLinks)
    cursor = page.nextAfterEvidenceLinkId
  } while (cursor.length > 0)
  return rows
}

function timestamp(): TimestampV1 {
  const millis = Date.now()
  return {
    $typeName: 'makosh.decisions.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(millis / 1000)),
    nanos: (millis % 1000) * 1_000_000
  }
}
function randomId16(): Uint8Array { const value = new Uint8Array(16); crypto.getRandomValues(value); return value }
function sameBytes(left: Uint8Array, right: Uint8Array): boolean { return left.length === right.length && left.every((value, index) => value === right[index]) }
function hex(value: Uint8Array): string { return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('') }
function message(value: unknown): string { return value instanceof Error ? value.message : 'Decision request failed' }
