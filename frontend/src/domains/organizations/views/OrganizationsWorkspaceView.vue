<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import {
  OrganizationSourceStateV1,
  OrganizationStateV1,
  type OrganizationV1
} from '../../../gen/makosh/organizations/client/v1/organizations_pb'
import { Button, Card, Input, Select } from '../../../shared/ui'
import { useOrganizationsPageSurface } from '../queries/useOrganizationsPageSurface'
import { hex, parseDigest } from '../stores/organizations'

const surface = useOrganizationsPageSurface()
const query = ref('')
const createDraft = reactive({
  displayName: '', legalName: '', description: '', website: '', industry: '', countryCode: ''
})
const sourceDraft = reactive({
  sourceOwnerId: '', sourceRecordId: '', sourceRevision: '1', evidenceDigest: ''
})

const stateOptions = [
  { value: String(OrganizationStateV1.ORGANIZATION_STATE_ACTIVE), label: 'Active' },
  { value: String(OrganizationStateV1.ORGANIZATION_STATE_ARCHIVED), label: 'Archived' }
]

onMounted(() => { void surface.loadOrganizations() })

async function createOrganization(): Promise<void> {
  if (!createDraft.displayName.trim()) return
  await surface.createOrganization({
    displayName: createDraft.displayName.trim(),
    legalName: createDraft.legalName.trim(),
    description: createDraft.description.trim(),
    website: createDraft.website.trim(),
    industry: createDraft.industry.trim(),
    countryCode: createDraft.countryCode.trim().toUpperCase()
  })
  Object.assign(createDraft, {
    displayName: '', legalName: '', description: '', website: '', industry: '', countryCode: ''
  })
}

async function addSource(organization: OrganizationV1): Promise<void> {
  if (!sourceDraft.sourceOwnerId.trim() || !sourceDraft.sourceRecordId.trim()) return
  await surface.addSource(organization, {
    sourceOwnerId: sourceDraft.sourceOwnerId.trim(),
    sourceRecordId: sourceDraft.sourceRecordId.trim(),
    sourceRevision: BigInt(sourceDraft.sourceRevision),
    evidenceDigest: parseDigest(sourceDraft.evidenceDigest)
  })
  Object.assign(sourceDraft, {
    sourceOwnerId: '', sourceRecordId: '', sourceRevision: '1', evidenceDigest: ''
  })
}

function isMutating(organization: OrganizationV1): boolean {
  return surface.mutatingOrganizationId.value === hex(organization.organizationId)
}
</script>

<template>
  <main class="organizations-workspace" aria-label="Organizations">
    <header class="organizations-workspace__header">
      <div>
        <p class="organizations-workspace__eyebrow">OWNER ORGANIZATIONS</p>
        <h1>Organizations</h1>
        <p>{{ surface.activeOrganizations.value.length }} active · {{ surface.archivedOrganizations.value.length }} archived</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadOrganizations">Refresh</Button>
    </header>

    <form class="organizations-workspace__search" @submit.prevent="surface.search(query)">
      <Input v-model="query" aria-label="Search Organizations" placeholder="Search owner organizations…" />
      <Button type="submit" variant="outline" icon="tabler:search">Search</Button>
      <Button v-if="surface.searchQuery.value" type="button" variant="ghost" @click="query = ''; surface.loadOrganizations()">Clear</Button>
    </form>

    <form class="organizations-workspace__create" @submit.prevent="createOrganization">
      <Input v-model="createDraft.displayName" aria-label="Organization display name" placeholder="Display name" />
      <Input v-model="createDraft.legalName" aria-label="Organization legal name" placeholder="Legal name (optional)" />
      <Input v-model="createDraft.description" aria-label="Organization description" placeholder="Description" />
      <Input v-model="createDraft.website" aria-label="Organization website" placeholder="Website" />
      <Input v-model="createDraft.industry" aria-label="Organization industry" placeholder="Industry" />
      <Input v-model="createDraft.countryCode" aria-label="Organization country code" placeholder="Country code" />
      <Button type="submit" icon="tabler:building-plus" :disabled="!createDraft.displayName.trim()">Create organization</Button>
    </form>

    <p v-if="surface.error.value" class="organizations-workspace__error" role="alert">{{ surface.error.value }}</p>
    <p v-if="surface.isLoading.value && surface.organizations.value.length === 0" aria-live="polite">Loading Organizations…</p>
    <p v-else-if="surface.organizations.value.length === 0" class="organizations-workspace__empty">No matching organizations.</p>

    <section v-else class="organizations-workspace__layout">
      <div class="organizations-workspace__list" aria-label="Organizations directory">
        <Card
          v-for="organization in surface.organizations.value"
          :key="hex(organization.organizationId)"
          class="organizations-workspace__organization"
          :selected="surface.selectedOrganization.value !== undefined && hex(surface.selectedOrganization.value.organizationId) === hex(organization.organizationId)"
          @click="surface.select(organization)"
        >
          <h2>{{ organization.displayName }}</h2>
          <p>{{ organization.industry || 'Industry not set' }} · {{ organization.countryCode || 'Country not set' }}</p>
          <small>Revision {{ organization.organizationRevision }}</small>
          <Select
            :model-value="String(organization.state)"
            :options="stateOptions"
            aria-label="Organization lifecycle state"
            :disabled="isMutating(organization)"
            @click.stop
            @update:model-value="surface.setOrganizationState(organization, Number($event) as OrganizationStateV1)"
          />
        </Card>
      </div>

      <Card v-if="surface.selectedOrganization.value" class="organizations-workspace__detail">
        <h2>{{ surface.selectedOrganization.value.displayName }}</h2>
        <p>{{ surface.selectedOrganization.value.legalName || 'No legal name.' }}</p>
        <p>{{ surface.selectedOrganization.value.description || 'No description.' }}</p>
        <dl>
          <div><dt>Website</dt><dd>{{ surface.selectedOrganization.value.website || 'Not set' }}</dd></div>
          <div><dt>Industry</dt><dd>{{ surface.selectedOrganization.value.industry || 'Not set' }}</dd></div>
          <div><dt>Country</dt><dd>{{ surface.selectedOrganization.value.countryCode || 'Not set' }}</dd></div>
        </dl>

        <section>
          <h3>Public source provenance</h3>
          <ul>
            <li v-for="source in surface.sources.value" :key="hex(source.sourceId)">
              <span>{{ source.sourceOwnerId }} · {{ source.sourceRecordId }} · revision {{ source.sourceRevision }}</span>
              <Button
                v-if="source.state === OrganizationSourceStateV1.ORGANIZATION_SOURCE_STATE_ACTIVE"
                size="sm"
                variant="ghost"
                @click="surface.removeSource(surface.selectedOrganization.value!, source)"
              >Remove</Button>
            </li>
          </ul>
          <form @submit.prevent="addSource(surface.selectedOrganization.value)">
            <Input v-model="sourceDraft.sourceOwnerId" aria-label="Public source owner" placeholder="Source owner" />
            <Input v-model="sourceDraft.sourceRecordId" aria-label="Public source record" placeholder="Source record" />
            <Input v-model="sourceDraft.sourceRevision" aria-label="Public source revision" type="number" min="1" placeholder="Revision" />
            <Input v-model="sourceDraft.evidenceDigest" aria-label="Public evidence digest" placeholder="64-character evidence digest" />
            <Button type="submit" variant="outline" size="sm">Add source</Button>
          </form>
        </section>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.organizations-workspace { display: grid; gap: 1.25rem; width: min(78rem, 100%); margin: 0 auto; padding: 1.5rem; }
.organizations-workspace__header, .organizations-workspace__search, .organizations-workspace__create, .organizations-workspace__detail form { display: flex; gap: .75rem; align-items: center; }
.organizations-workspace__header { justify-content: space-between; align-items: flex-start; }
.organizations-workspace__header h1, .organizations-workspace__organization h2, .organizations-workspace__detail h2, .organizations-workspace__detail h3 { margin: 0; }
.organizations-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.organizations-workspace__header p, .organizations-workspace__organization p, .organizations-workspace__organization small { color: var(--text-secondary); }
.organizations-workspace__search :deep(.makosh-input-wrapper), .organizations-workspace__create :deep(.makosh-input-wrapper), .organizations-workspace__detail :deep(.makosh-input-wrapper) { flex: 1; }
.organizations-workspace__create { flex-wrap: wrap; padding: 1rem; border: 1px solid var(--border-subtle); border-radius: .75rem; }
.organizations-workspace__layout { display: grid; grid-template-columns: minmax(18rem, 2fr) minmax(22rem, 3fr); gap: 1rem; align-items: start; }
.organizations-workspace__list, .organizations-workspace__detail { display: grid; gap: .75rem; }
.organizations-workspace__organization { display: grid; gap: .5rem; padding: 1rem; cursor: pointer; }
.organizations-workspace__detail { padding: 1rem; }
.organizations-workspace__detail dl { display: grid; gap: .5rem; }
.organizations-workspace__detail dl div { display: grid; grid-template-columns: 6rem 1fr; gap: .5rem; }
.organizations-workspace__detail dt { color: var(--text-secondary); }
.organizations-workspace__detail dd { margin: 0; }
.organizations-workspace__detail section { display: grid; gap: .5rem; padding-top: .75rem; border-top: 1px solid var(--border-subtle); }
.organizations-workspace__detail ul { display: grid; gap: .375rem; margin: 0; padding: 0; list-style: none; }
.organizations-workspace__detail li { display: flex; justify-content: space-between; gap: .75rem; align-items: center; }
.organizations-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.organizations-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
@media (max-width: 820px) { .organizations-workspace__layout { grid-template-columns: 1fr; } .organizations-workspace__header, .organizations-workspace__search, .organizations-workspace__create, .organizations-workspace__detail form { align-items: stretch; flex-direction: column; } }
</style>
