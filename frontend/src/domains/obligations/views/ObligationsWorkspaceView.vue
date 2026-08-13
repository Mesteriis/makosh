<script setup lang="ts">
import { onMounted } from 'vue'
import {
  ObligationStateV1,
  type ObligationSummaryV1
} from '../../../gen/makosh/obligations/client/v1/obligations_pb'
import { Button, Card, Select } from '../../../shared/ui'
import { useObligationsPageSurface } from '../queries/useObligationsPageSurface'

const surface = useObligationsPageSurface()
const terminalStateOptions = [
  { value: String(ObligationStateV1.OBLIGATION_STATE_FULFILLED), label: 'Fulfilled' },
  { value: String(ObligationStateV1.OBLIGATION_STATE_WAIVED), label: 'Waived' },
  { value: String(ObligationStateV1.OBLIGATION_STATE_BREACHED), label: 'Breached' },
  { value: String(ObligationStateV1.OBLIGATION_STATE_CANCELLED), label: 'Cancelled' }
]

onMounted(() => { void surface.loadObligations() })

function isMutating(obligation: ObligationSummaryV1): boolean {
  return surface.mutatingObligationId.value === hex(obligation.obligationId)
}

function evidenceCount(obligation: ObligationSummaryV1): number {
  return surface.evidenceByObligation.value[hex(obligation.obligationId)]?.length ?? 0
}

function partyLabel(value: Uint8Array | undefined): string {
  return value?.length ? hex(value).slice(0, 12) : 'none'
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}
</script>

<template>
  <main class="obligations-workspace" aria-label="Obligations">
    <header class="obligations-workspace__header">
      <div>
        <p class="obligations-workspace__eyebrow">CONFIRMED OWNER TRUTH</p>
        <h1>Obligations</h1>
        <p>{{ surface.openObligations.value.length }} open · {{ surface.completedObligations.value.length }} terminal</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadObligations">
        Refresh
      </Button>
    </header>

    <p class="obligations-workspace__notice">
      New obligations enter this owner only after an explicit Review approval. Manual creation is unavailable.
    </p>
    <p v-if="surface.error.value" class="obligations-workspace__error" role="alert">
      {{ surface.error.value }}
    </p>
    <p v-if="surface.isLoading.value && surface.obligations.value.length === 0" aria-live="polite">
      Loading obligations…
    </p>
    <p v-else-if="surface.obligations.value.length === 0" class="obligations-workspace__empty">
      No confirmed obligations yet. Review candidates remain in the Review workspace until approved.
    </p>

    <section v-else class="obligations-workspace__list" aria-label="Confirmed obligations">
      <Card v-for="obligation in surface.obligations.value" :key="hex(obligation.obligationId)" class="obligations-workspace__obligation">
        <div class="obligations-workspace__obligation-header">
          <div>
            <h2>{{ obligation.statement }}</h2>
            <p v-if="obligation.condition">Condition: {{ obligation.condition }}</p>
            <small>
              Revision {{ obligation.obligationRevision }} · obligated {{ partyLabel(obligation.obligatedPartyId) }}
              · beneficiary {{ partyLabel(obligation.beneficiaryPartyId) }} · {{ evidenceCount(obligation) }} evidence links
            </small>
          </div>
          <Select
            v-if="obligation.state === ObligationStateV1.OBLIGATION_STATE_OPEN"
            :model-value="''"
            :options="terminalStateOptions"
            aria-label="Close obligation as"
            placeholder="Set terminal state"
            :disabled="isMutating(obligation)"
            @update:model-value="surface.setObligationState(obligation, Number($event) as ObligationStateV1)"
          />
          <span v-else class="obligations-workspace__terminal">Terminal state {{ obligation.state }}</span>
        </div>

        <ul v-if="evidenceCount(obligation)" class="obligations-workspace__evidence" aria-label="Evidence links">
          <li
            v-for="evidence in surface.evidenceByObligation.value[hex(obligation.obligationId)]"
            :key="hex(evidence.evidenceLinkId)"
          >
            <span>{{ evidence.evidenceOwnerId }} · revision {{ evidence.evidenceRevision }}</span>
            <Button
              v-if="obligation.state === ObligationStateV1.OBLIGATION_STATE_OPEN"
              variant="ghost"
              size="sm"
              icon="tabler:unlink"
              aria-label="Remove evidence link"
              :disabled="isMutating(obligation)"
              @click="surface.removeEvidence(obligation, evidence.evidenceLinkId)"
            />
          </li>
        </ul>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.obligations-workspace { display: grid; gap: 1.25rem; width: min(72rem, 100%); margin: 0 auto; padding: 1.5rem; }
.obligations-workspace__header, .obligations-workspace__obligation-header { display: flex; justify-content: space-between; align-items: flex-start; gap: .75rem; }
.obligations-workspace__header h1, .obligations-workspace__obligation h2 { margin: 0; }
.obligations-workspace__header p, .obligations-workspace__obligation p, .obligations-workspace__obligation small { color: var(--text-secondary); }
.obligations-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.obligations-workspace__notice { padding: .875rem 1rem; color: var(--text-secondary); background: var(--surface-secondary); border-radius: .75rem; }
.obligations-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.obligations-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
.obligations-workspace__list { display: grid; gap: 1rem; }
.obligations-workspace__obligation { display: grid; gap: 1rem; padding: 1rem; }
.obligations-workspace__terminal { color: var(--text-secondary); white-space: nowrap; }
.obligations-workspace__evidence { display: grid; gap: .5rem; margin: 0; padding: 0; list-style: none; }
.obligations-workspace__evidence li { display: flex; align-items: center; justify-content: space-between; gap: .75rem; }
@media (max-width: 720px) { .obligations-workspace__header, .obligations-workspace__obligation-header { flex-direction: column; } }
</style>
