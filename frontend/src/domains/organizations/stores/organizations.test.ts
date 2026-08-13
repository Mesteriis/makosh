import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  OrganizationStateV1,
  type OrganizationV1
} from '../../../gen/makosh/organizations/client/v1/organizations_pb'

const clients = vi.hoisted(() => ({
  query: {
    list: vi.fn(), search: vi.fn(), get: vi.fn(), listSources: vi.fn()
  },
  command: {
    create: vi.fn(), update: vi.fn(), setState: vi.fn(), addSource: vi.fn(), removeSource: vi.fn()
  }
}))

vi.mock('../../../platform/connect/organizationsClient', () => ({
  getOrganizationsQueryClient: () => clients.query,
  getOrganizationsCommandClient: () => clients.command
}))

import { useOrganizationsStore } from './organizations'

describe('typed Organizations owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.query.listSources.mockResolvedValue({ sources: [], nextAfterSourceId: new Uint8Array() })
  })

  it('loads all bounded pages using the exclusive last-returned cursor', async () => {
    clients.query.list
      .mockResolvedValueOnce({ organizations: [organization(1)], nextAfterOrganizationId: id(1) })
      .mockResolvedValueOnce({ organizations: [organization(2)], nextAfterOrganizationId: new Uint8Array() })
    const store = useOrganizationsStore()

    await store.loadAll()

    expect(store.organizations.map((value) => value.organizationId)).toEqual([id(1), id(2)])
    expect(clients.query.list.mock.calls).toEqual([
      [{ logicalOwnerId: '', afterOrganizationId: new Uint8Array(), limit: 50 }],
      [{ logicalOwnerId: '', afterOrganizationId: id(1), limit: 50 }]
    ])
  })

  it('dispatches lifecycle and provenance commands with exact current revisions', async () => {
    const initial = organization(1)
    const revised = { ...initial, organizationRevision: 2n }
    clients.query.list.mockResolvedValue({ organizations: [initial], nextAfterOrganizationId: new Uint8Array() })
    clients.command.setState.mockResolvedValue({ organization: revised })
    clients.command.addSource.mockResolvedValue({ organization: { ...revised, organizationRevision: 3n } })
    const store = useOrganizationsStore()
    await store.loadAll()

    await store.setOrganizationState(store.organizations[0]!, OrganizationStateV1.ORGANIZATION_STATE_ARCHIVED)
    await store.addSource(store.organizations[0]!, {
      sourceOwnerId: 'public-source', sourceRecordId: 'record-1', sourceRevision: 7n,
      evidenceDigest: new Uint8Array(32).fill(9)
    })

    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      organizationId: id(1), expectedOrganizationRevision: 1n,
      state: OrganizationStateV1.ORGANIZATION_STATE_ARCHIVED
    })
    expect(clients.command.addSource.mock.calls[0]?.[0]).toMatchObject({
      organizationId: id(1), expectedOrganizationRevision: 2n,
      sourceOwnerId: 'public-source', sourceRecordId: 'record-1', sourceRevision: 7n
    })
    expect(store.organizations[0]?.organizationRevision).toBe(3n)
  })
})

function organization(seed: number): OrganizationV1 {
  return {
    $typeName: 'makosh.organizations.client.v1.OrganizationV1',
    organizationId: id(seed),
    logicalOwnerId: 'owner-1',
    displayName: `Organization ${seed}`,
    legalName: '', description: '', website: '', industry: '', countryCode: '',
    state: OrganizationStateV1.ORGANIZATION_STATE_ACTIVE,
    organizationRevision: 1n
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
