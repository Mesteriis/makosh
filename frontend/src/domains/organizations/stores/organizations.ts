import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  OrganizationStateV1,
  type OrganizationSourceV1,
  type OrganizationV1,
  type TimestampV1
} from '../../../gen/makosh/organizations/client/v1/organizations_pb'
import {
  getOrganizationsCommandClient,
  getOrganizationsQueryClient
} from '../../../platform/connect/organizationsClient'

const PAGE_LIMIT = 50

export const useOrganizationsStore = defineStore('organizations-owner', () => {
  const organizations = ref<OrganizationV1[]>([])
  const selectedOrganization = ref<OrganizationV1>()
  const sources = ref<OrganizationSourceV1[]>([])
  const searchQuery = ref('')
  const error = ref('')
  const isLoading = ref(false)
  const mutatingOrganizationId = ref<string | null>(null)

  const activeOrganizations = computed(() => organizations.value.filter((organization) =>
    organization.state === OrganizationStateV1.ORGANIZATION_STATE_ACTIVE
  ))
  const archivedOrganizations = computed(() => organizations.value.filter((organization) =>
    organization.state === OrganizationStateV1.ORGANIZATION_STATE_ARCHIVED
  ))

  async function loadAll(): Promise<void> {
    await loadPages('')
  }

  async function search(query: string): Promise<void> {
    await loadPages(query.trim())
  }

  async function select(organization: OrganizationV1): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      selectedOrganization.value = await getOrganizationsQueryClient().get({
        logicalOwnerId: '',
        organizationId: organization.organizationId
      })
      await loadSources(selectedOrganization.value)
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function createOrganization(input: {
    displayName: string
    legalName: string
    description: string
    website: string
    industry: string
    countryCode: string
  }): Promise<void> {
    await run(null, async () => {
      const result = await getOrganizationsCommandClient().create({
        operationId: randomId16(),
        logicalOwnerId: '',
        displayName: input.displayName,
        legalName: input.legalName,
        description: input.description,
        website: input.website,
        industry: input.industry,
        countryCode: input.countryCode,
        createdAt: timestamp(new Date())
      })
      replaceResult(result.organization)
    })
  }

  async function updateOrganization(organization: OrganizationV1, input: {
    displayName?: string
    legalName?: string
    description?: string
    website?: string
    industry?: string
    countryCode?: string
  }): Promise<void> {
    await mutate(organization, () => getOrganizationsCommandClient().update({
      operationId: randomId16(),
      organizationId: organization.organizationId,
      logicalOwnerId: '',
      expectedOrganizationRevision: organization.organizationRevision,
      displayName: input.displayName,
      legalName: input.legalName,
      description: input.description,
      website: input.website,
      industry: input.industry,
      countryCode: input.countryCode,
      updatedAt: timestamp(new Date())
    }))
  }

  async function setOrganizationState(
    organization: OrganizationV1,
    state: OrganizationStateV1
  ): Promise<void> {
    await mutate(organization, () => getOrganizationsCommandClient().setState({
      operationId: randomId16(),
      organizationId: organization.organizationId,
      logicalOwnerId: '',
      expectedOrganizationRevision: organization.organizationRevision,
      state,
      changedAt: timestamp(new Date())
    }))
  }

  async function addSource(organization: OrganizationV1, input: {
    sourceOwnerId: string
    sourceRecordId: string
    sourceRevision: bigint
    evidenceDigest: Uint8Array
  }): Promise<void> {
    await mutate(organization, () => getOrganizationsCommandClient().addSource({
      operationId: randomId16(),
      organizationId: organization.organizationId,
      logicalOwnerId: '',
      expectedOrganizationRevision: organization.organizationRevision,
      sourceOwnerId: input.sourceOwnerId,
      sourceRecordId: input.sourceRecordId,
      sourceRevision: input.sourceRevision,
      evidenceDigest: input.evidenceDigest,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function removeSource(organization: OrganizationV1, source: OrganizationSourceV1): Promise<void> {
    await mutate(organization, () => getOrganizationsCommandClient().removeSource({
      operationId: randomId16(),
      organizationId: organization.organizationId,
      logicalOwnerId: '',
      expectedOrganizationRevision: organization.organizationRevision,
      sourceId: source.sourceId,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function loadPages(query: string): Promise<void> {
    isLoading.value = true
    error.value = ''
    searchQuery.value = query
    try {
      const next: OrganizationV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      for (let page = 0; page < 100; page += 1) {
        const result = query
          ? await getOrganizationsQueryClient().search({
              logicalOwnerId: '', query, afterOrganizationId: cursor, limit: PAGE_LIMIT
            })
          : await getOrganizationsQueryClient().list({
              logicalOwnerId: '', afterOrganizationId: cursor, limit: PAGE_LIMIT
            })
        next.push(...result.organizations)
        if (result.nextAfterOrganizationId.length === 0) break
        cursor = result.nextAfterOrganizationId
      }
      organizations.value = next
      if (selectedOrganization.value) {
        const refreshed = next.find((organization) => sameBytes(
          organization.organizationId,
          selectedOrganization.value!.organizationId
        ))
        if (refreshed) selectedOrganization.value = refreshed
      }
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function loadSources(organization: OrganizationV1): Promise<void> {
    const next: OrganizationSourceV1[] = []
    let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    for (let page = 0; page < 100; page += 1) {
      const result = await getOrganizationsQueryClient().listSources({
        logicalOwnerId: '',
        organizationId: organization.organizationId,
        afterSourceId: cursor,
        limit: PAGE_LIMIT
      })
      next.push(...result.sources)
      if (result.nextAfterSourceId.length === 0) break
      cursor = result.nextAfterSourceId
    }
    sources.value = next
  }

  async function mutate(
    organization: OrganizationV1,
    operation: () => Promise<{ organization?: OrganizationV1 }>,
    reloadSources = false
  ): Promise<void> {
    await run(hex(organization.organizationId), async () => {
      const result = await operation()
      const updated = replaceResult(result.organization)
      if (reloadSources) await loadSources(updated)
    })
  }

  async function run(organizationId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingOrganizationId.value = organizationId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingOrganizationId.value = null
    }
  }

  function replaceResult(organization: OrganizationV1 | undefined): OrganizationV1 {
    if (!organization) throw new Error('organizations_invalid_response')
    const index = organizations.value.findIndex((value) => sameBytes(
      value.organizationId,
      organization.organizationId
    ))
    if (index === -1) organizations.value.push(organization)
    else organizations.value[index] = organization
    organizations.value.sort((left, right) => compareBytes(left.organizationId, right.organizationId))
    if (selectedOrganization.value && sameBytes(
      selectedOrganization.value.organizationId,
      organization.organizationId
    )) selectedOrganization.value = organization
    return organization
  }

  return {
    organizations,
    selectedOrganization,
    sources,
    searchQuery,
    error,
    isLoading,
    mutatingOrganizationId,
    activeOrganizations,
    archivedOrganizations,
    loadAll,
    search,
    select,
    createOrganization,
    updateOrganization,
    setOrganizationState,
    addSource,
    removeSource
  }
})

export function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.organizations.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
}

export function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

export function parseDigest(value: string): Uint8Array {
  const normalized = value.trim().toLowerCase()
  if (!/^[0-9a-f]{64}$/.test(normalized)) throw new Error('organizations_invalid_evidence_digest')
  return Uint8Array.from(normalized.match(/../g) ?? [], (byte) => Number.parseInt(byte, 16))
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

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'organizations_unavailable'
}
