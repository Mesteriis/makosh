import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  ObligationStateV1,
  type ObligationEvidenceLinkV1,
  type ObligationSummaryV1
} from '../../../gen/makosh/obligations/client/v1/obligations_pb'

const clients = vi.hoisted(() => ({
  query: { list: vi.fn(), get: vi.fn(), listEvidence: vi.fn() },
  command: { update: vi.fn(), setState: vi.fn(), addEvidence: vi.fn(), removeEvidence: vi.fn() }
}))

vi.mock('../../../platform/connect/obligationsClient', () => ({
  getObligationsQueryClient: () => clients.query,
  getObligationsCommandClient: () => clients.command
}))

import { useObligationsStore } from './obligations'

describe('typed Obligations owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.query.listEvidence.mockResolvedValue({ evidenceLinks: [], nextAfterEvidenceLinkId: new Uint8Array() })
  })

  it('loads all obligations and their typed evidence without a manual create route', async () => {
    clients.query.list
      .mockResolvedValueOnce({ obligations: [obligation(1)], nextAfterObligationId: id(1) })
      .mockResolvedValueOnce({ obligations: [obligation(2)], nextAfterObligationId: new Uint8Array() })
    clients.query.listEvidence
      .mockResolvedValueOnce({ evidenceLinks: [evidence(7)], nextAfterEvidenceLinkId: new Uint8Array() })
      .mockResolvedValueOnce({ evidenceLinks: [], nextAfterEvidenceLinkId: new Uint8Array() })
    const store = useObligationsStore()

    await store.loadAll()

    expect(store.obligations.map((value) => value.obligationId)).toEqual([id(1), id(2)])
    expect(store.evidenceByObligation[hex(id(1))]).toEqual([evidence(7)])
    expect(clients.query.list.mock.calls[1]?.[0].afterObligationId).toEqual(id(1))
  })

  it('dispatches evidence and terminal lifecycle mutations with exact current revisions', async () => {
    const initial = obligation(1)
    clients.query.list.mockResolvedValue({ obligations: [initial], nextAfterObligationId: new Uint8Array() })
    clients.command.addEvidence.mockResolvedValue({ obligation: { ...initial, obligationRevision: 2n } })
    clients.command.setState.mockResolvedValue({ obligation: { ...initial, obligationRevision: 3n, state: ObligationStateV1.OBLIGATION_STATE_FULFILLED } })
    const store = useObligationsStore()
    await store.loadAll()

    await store.addEvidence(store.obligations[0]!, evidence(9))
    await store.setObligationState(store.obligations[0]!, ObligationStateV1.OBLIGATION_STATE_FULFILLED)

    expect(clients.command.addEvidence.mock.calls[0]?.[0]).toMatchObject({
      obligationId: id(1), expectedObligationRevision: 1n, evidence: evidence(9)
    })
    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      obligationId: id(1), expectedObligationRevision: 2n,
      state: ObligationStateV1.OBLIGATION_STATE_FULFILLED
    })
  })
})

function obligation(seed: number): ObligationSummaryV1 {
  return {
    $typeName: 'makosh.obligations.client.v1.ObligationSummaryV1',
    obligationId: id(seed),
    logicalOwnerId: 'owner-1',
    statement: `Obligation ${seed}`,
    state: ObligationStateV1.OBLIGATION_STATE_OPEN,
    obligationRevision: 1n,
    obligatedPartyId: id(3)
  }
}

function evidence(seed: number): ObligationEvidenceLinkV1 {
  return {
    $typeName: 'makosh.obligations.client.v1.ObligationEvidenceLinkV1',
    evidenceLinkId: id(seed),
    evidenceOwnerId: 'documents',
    evidenceRecordId: id(seed + 1),
    evidenceRevision: 1n,
    evidenceDigest: new Uint8Array(32).fill(seed)
  }
}

function id(seed: number): Uint8Array { return new Uint8Array(16).fill(seed) }
function hex(value: Uint8Array): string { return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('') }
