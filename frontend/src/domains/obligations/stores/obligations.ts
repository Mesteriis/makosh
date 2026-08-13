import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  ObligationStateV1,
  type ObligationEvidenceLinkV1,
  type ObligationSummaryV1,
  type TimestampV1
} from '../../../gen/makosh/obligations/client/v1/obligations_pb'
import {
  getObligationsCommandClient,
  getObligationsQueryClient
} from '../../../platform/connect/obligationsClient'

const PAGE_LIMIT = 50

export const useObligationsStore = defineStore('obligations', () => {
  const obligations = ref<ObligationSummaryV1[]>([])
  const evidenceByObligation = ref<Record<string, ObligationEvidenceLinkV1[]>>({})
  const error = ref('')
  const mutatingObligationId = ref<string | null>(null)
  const isLoading = ref(false)

  const openObligations = computed(() => obligations.value.filter((obligation) =>
    obligation.state === ObligationStateV1.OBLIGATION_STATE_OPEN
  ))
  const completedObligations = computed(() => obligations.value.filter((obligation) =>
    obligation.state !== ObligationStateV1.OBLIGATION_STATE_OPEN
  ))

  async function loadAll(): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const loaded: ObligationSummaryV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      do {
        const page = await getObligationsQueryClient().list({
          logicalOwnerId: '',
          afterObligationId: cursor,
          limit: PAGE_LIMIT
        })
        loaded.push(...page.obligations)
        cursor = page.nextAfterObligationId
      } while (cursor.length > 0)
      obligations.value = loaded
      await Promise.all(loaded.map(loadEvidence))
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function loadEvidence(obligation: ObligationSummaryV1): Promise<void> {
    const loaded: ObligationEvidenceLinkV1[] = []
    let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    do {
      const page = await getObligationsQueryClient().listEvidence({
        logicalOwnerId: '',
        obligationId: obligation.obligationId,
        afterEvidenceLinkId: cursor,
        limit: PAGE_LIMIT
      })
      loaded.push(...page.evidenceLinks)
      cursor = page.nextAfterEvidenceLinkId
    } while (cursor.length > 0)
    evidenceByObligation.value[hex(obligation.obligationId)] = loaded
  }

  async function updateObligation(
    obligation: ObligationSummaryV1,
    values: {
      statement?: string
      condition?: string
      clearCondition?: boolean
      dueAt?: Date
      clearDueAt?: boolean
      obligatedPartyId?: Uint8Array
      beneficiaryPartyId?: Uint8Array
      clearBeneficiaryPartyId?: boolean
    }
  ): Promise<void> {
    await run(obligation, async () => {
      const result = await getObligationsCommandClient().update({
        operationId: randomId16(),
        obligationId: obligation.obligationId,
        logicalOwnerId: '',
        expectedObligationRevision: obligation.obligationRevision,
        statement: values.statement,
        condition: values.condition,
        clearCondition: values.clearCondition ?? false,
        dueAt: values.dueAt ? timestamp(values.dueAt) : undefined,
        clearDueAt: values.clearDueAt ?? false,
        obligatedPartyId: values.obligatedPartyId,
        beneficiaryPartyId: values.beneficiaryPartyId,
        clearBeneficiaryPartyId: values.clearBeneficiaryPartyId ?? false,
        updatedAt: timestamp(new Date())
      })
      replaceResult(result.obligation)
    })
  }

  async function setObligationState(obligation: ObligationSummaryV1, state: ObligationStateV1): Promise<void> {
    await run(obligation, async () => {
      const result = await getObligationsCommandClient().setState({
        operationId: randomId16(),
        obligationId: obligation.obligationId,
        logicalOwnerId: '',
        expectedObligationRevision: obligation.obligationRevision,
        state,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.obligation)
    })
  }

  async function addEvidence(obligation: ObligationSummaryV1, evidence: ObligationEvidenceLinkV1): Promise<void> {
    await run(obligation, async () => {
      const result = await getObligationsCommandClient().addEvidence({
        operationId: randomId16(),
        obligationId: obligation.obligationId,
        logicalOwnerId: '',
        expectedObligationRevision: obligation.obligationRevision,
        evidence,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.obligation)
      await loadEvidence(result.obligation ?? obligation)
    })
  }

  async function removeEvidence(obligation: ObligationSummaryV1, evidenceLinkId: Uint8Array): Promise<void> {
    await run(obligation, async () => {
      const result = await getObligationsCommandClient().removeEvidence({
        operationId: randomId16(),
        obligationId: obligation.obligationId,
        logicalOwnerId: '',
        expectedObligationRevision: obligation.obligationRevision,
        evidenceLinkId,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.obligation)
      await loadEvidence(result.obligation ?? obligation)
    })
  }

  async function run(obligation: ObligationSummaryV1, operation: () => Promise<void>): Promise<void> {
    mutatingObligationId.value = hex(obligation.obligationId)
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingObligationId.value = null
    }
  }

  function replaceResult(obligation: ObligationSummaryV1 | undefined): void {
    if (!obligation) throw new Error('obligations_invalid_response')
    const index = obligations.value.findIndex((value) => sameBytes(value.obligationId, obligation.obligationId))
    if (index === -1) obligations.value.push(obligation)
    else obligations.value[index] = obligation
    obligations.value.sort((left, right) => compareBytes(left.obligationId, right.obligationId))
  }

  return {
    obligations,
    evidenceByObligation,
    error,
    mutatingObligationId,
    isLoading,
    openObligations,
    completedObligations,
    loadAll,
    loadEvidence,
    updateObligation,
    setObligationState,
    addEvidence,
    removeEvidence
  }
})

function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.obligations.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
}

function randomId16(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(16))
}

function sameBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index])
}

function compareBytes(left: Uint8Array, right: Uint8Array): number {
  const length = Math.min(left.length, right.length)
  for (let index = 0; index < length; index += 1) {
    const comparison = (left[index] ?? 0) - (right[index] ?? 0)
    if (comparison !== 0) return comparison
  }
  return left.length - right.length
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'obligations_unavailable'
}
