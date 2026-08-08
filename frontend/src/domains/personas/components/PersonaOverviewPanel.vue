<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { Button, Icon, SkeletonPanel, Switch } from '../../../shared/ui'
import type {
  PersonaIdentity,
  PersonaIdentityCandidate,
  PersonaIdentityReviewState,
  PersonaPanelProfile,
  Relationship
} from '../types/persona'
import PersonaRelationshipGraph from './PersonaRelationshipGraph.vue'
import {
  canAssignIdentityTraceToOwner,
  canAssignIdentityTraceToPersona,
  isAssigningIdentityTrace,
  isReviewingIdentityCandidate,
} from './personaOverviewPresentation'
import {
  candidateKindLabel,
  candidateTitle,
  formatDateTime,
  identityConfidence,
  languageLabel,
  traceKindLabel,
  traceTitle,
  trustScoreLabel
} from './personaWorkspaceElements'

const props = withDefaults(defineProps<{
  ownerPersona: PersonaPanelProfile | null
  selectedPersona: PersonaPanelProfile
  entityLabels?: Readonly<Record<string, string>>
  selectedPersonaRelationships: readonly Relationship[]
  pendingReviewCount: number
  suggestedIdentityCandidates: readonly PersonaIdentityCandidate[]
  identityTraces: readonly PersonaIdentity[]
  reviewingCandidateId?: string | null
  assigningTraceId?: string | null
}>(), {
  entityLabels: () => ({}),
  reviewingCandidateId: null,
  assigningTraceId: null
})

const emit = defineEmits<{
  reviewCandidate: [candidate: PersonaIdentityCandidate, state: PersonaIdentityReviewState]
  assignTraceToOwner: [trace: PersonaIdentity]
  assignTraceToSelectedPersona: [trace: PersonaIdentity]
  toggleAddressBook: [persona: PersonaPanelProfile, value: boolean]
}>()

const { t } = useI18n()

function isReviewingCandidate(candidateId: string): boolean {
  return isReviewingIdentityCandidate(candidateId, props.reviewingCandidateId)
}

function isAssigningTrace(traceId: string): boolean {
  return isAssigningIdentityTrace(traceId, props.assigningTraceId)
}

function canAssignToSelected(trace: PersonaIdentity): boolean {
  return canAssignIdentityTraceToPersona(trace, props.selectedPersona.persona_id)
}

function canAssignToOwner(trace: PersonaIdentity): boolean {
  return canAssignIdentityTraceToOwner(trace, props.ownerPersona?.persona_id ?? null)
}

function candidateId(candidate: PersonaIdentityCandidate): string {
  return candidate.identity_candidate_id
}

function traceId(trace: PersonaIdentity): string {
  return trace.id
}

function handleAddressBookToggle(value: boolean): void {
  emit('toggleAddressBook', props.selectedPersona, value)
}
</script>

<template>
  <section class="personas-profile-dashboard">
    <aside class="personas-dashboard-column">
      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:address-book" />
          <h3>{{ t('Identity channels') }}</h3>
        </header>
        <div class="personas-address-book-toggle">
          <span>
            <strong>
              {{ selectedPersona.is_address_book ? t('In contacts') : t('Add to contacts') }}
            </strong>
            <small>{{ t('Contacts') }}</small>
          </span>
          <Switch
            :model-value="selectedPersona.is_address_book"
            :aria-label="t('Add to contacts')"
            @update:model-value="handleAddressBookToggle"
          />
        </div>
        <ul class="personas-address-book-list">
          <li>
            <Icon icon="tabler:mail" />
            <span>{{ selectedPersona.email_address || t('No primary email') }}</span>
            <small>{{ t('Work') }}</small>
          </li>
          <li>
            <Icon icon="tabler:message-circle" />
            <span>{{ selectedPersona.preferred_channel || t('Not set') }}</span>
            <small>{{ t('Primary') }}</small>
          </li>
          <li>
            <Icon icon="tabler:language" />
            <span>{{ languageLabel(selectedPersona.language, t) }}</span>
            <small>{{ t('Language') }}</small>
          </li>
        </ul>
      </section>

      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:building" />
          <h3>{{ t('Organizations') }}</h3>
        </header>
        <ul class="personas-compact-list">
          <li>
            <span>{{ selectedPersona.preferred_channel || t('Personal network') }}</span>
            <small>{{ selectedPersona.tone || t('Current') }}</small>
          </li>
          <li>
            <span>{{ t('Макошь context') }}</span>
            <small>{{ selectedPersonaRelationships.length }} {{ t('links') }}</small>
          </li>
        </ul>
      </section>

      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:tags" />
          <h3>{{ t('Skills') }}</h3>
        </header>
        <ul class="personas-skill-cloud">
          <li v-for="topic in selectedPersona.frequent_topics" :key="topic">{{ topic }}</li>
          <li v-if="selectedPersona.frequent_topics.length === 0">{{ t('No topics yet') }}</li>
        </ul>
      </section>

      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:language" />
          <h3>{{ t('Languages') }}</h3>
        </header>
        <dl class="personas-language-list">
          <div>
            <dt>{{ languageLabel(selectedPersona.language, t) }}</dt>
            <dd><span class="personas-language-progress--primary" /></dd>
          </div>
          <div>
            <dt>{{ t('English') }}</dt>
            <dd><span class="personas-language-progress--secondary" /></dd>
          </div>
        </dl>
      </section>
    </aside>

    <section class="personas-dashboard-main">
      <section class="personas-dashboard-panel personas-communications-panel">
        <header>
          <Icon icon="tabler:messages" />
          <h3>{{ t('Recent communications') }}</h3>
        </header>
        <SkeletonPanel
          :title="t('Communication projection is not connected to this persona surface yet.')"
          :description="t('This area is reserved for persona-scoped communication cards once the query is wired.')"
          :rows="4"
        />
      </section>

      <section class="personas-dashboard-split">
        <section class="personas-dashboard-panel personas-graph-panel">
          <PersonaRelationshipGraph
            :selected-persona="selectedPersona"
            :entity-labels="entityLabels"
            :relationships="selectedPersonaRelationships"
          />
        </section>

        <section class="personas-dashboard-panel">
          <header>
            <Icon icon="tabler:checkbox" />
            <h3>{{ t('Tasks and agreements') }}</h3>
          </header>
          <SkeletonPanel
            :title="t('Task candidates require review promotion wiring.')"
            :description="t('Durable tasks should appear only after candidate review or promotion.')"
            :rows="3"
          />
        </section>
      </section>

      <section class="personas-dashboard-split">
        <section class="personas-dashboard-panel">
          <header>
            <Icon icon="tabler:timeline" />
            <h3>{{ t('Relationship timeline') }}</h3>
          </header>
          <SkeletonPanel
            :title="t('Persona timeline endpoint exists, but this tab is not wired yet.')"
            :description="t('Timeline cards will use persona events when the frontend query is connected.')"
            :rows="3"
          />
        </section>

        <section class="personas-dashboard-panel">
          <header>
            <Icon icon="tabler:activity" />
            <h3>{{ t('Activity and metrics') }}</h3>
          </header>
          <SkeletonPanel
            :title="t('Persona analytics is backend-listed and not bound to this story surface.')"
            :description="t('The panel keeps the layout space while analytics contracts settle.')"
            :rows="3"
          />
        </section>
      </section>
    </section>

    <aside class="personas-dashboard-column">
      <section class="personas-dashboard-panel personas-ai-panel">
        <header>
          <Icon icon="tabler:sparkles" />
          <h3>{{ t('AI Summary') }}</h3>
        </header>
        <p>{{ selectedPersona.notes || t('No notes yet') }}</p>
        <ul>
          <li v-for="topic in selectedPersona.frequent_topics" :key="topic">{{ topic }}</li>
        </ul>
      </section>

      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:folder-open" />
          <h3>{{ t('Dossier') }}</h3>
        </header>
        <dl class="personas-dossier-list">
          <div>
            <dt>{{ t('Interactions') }}</dt>
            <dd>{{ selectedPersona.interaction_count }}</dd>
          </div>
          <div>
            <dt>{{ t('Relationships') }}</dt>
            <dd>{{ selectedPersonaRelationships.length }}</dd>
          </div>
          <div>
            <dt>{{ t('Trust score') }}</dt>
            <dd>{{ trustScoreLabel(selectedPersona.trust_score, t) }}</dd>
          </div>
          <div>
            <dt>{{ t('Last activity') }}</dt>
            <dd>{{ formatDateTime(selectedPersona.updated_at, t) }}</dd>
          </div>
        </dl>
      </section>

      <section class="personas-dashboard-panel">
        <header>
          <Icon icon="tabler:user-question" />
          <h3>{{ t('Identity review') }}</h3>
          <strong>{{ pendingReviewCount }}</strong>
        </header>
        <div
          v-if="suggestedIdentityCandidates.length === 0 && identityTraces.length === 0"
          class="personas-empty-compact"
        >
          <Icon icon="tabler:circle-check" />
          <span>{{ t('No pending identity candidates') }}</span>
        </div>
        <article
          v-for="candidate in suggestedIdentityCandidates"
          :key="candidateId(candidate)"
          class="personas-candidate-row"
        >
          <div>
            <span>{{ candidateKindLabel(candidate, t) }}</span>
            <strong>{{ candidateTitle(candidate, t) }}</strong>
            <p>{{ candidate.evidence_summary }}</p>
            <small>{{ identityConfidence(candidate) }}</small>
          </div>
          <footer>
            <Button
              type="button"
              size="sm"
              variant="outline"
              icon="tabler:x"
              :loading="isReviewingCandidate(candidateId(candidate))"
              @click="emit('reviewCandidate', candidate, 'user_rejected')"
            >
              {{ t('Reject') }}
            </Button>
            <Button
              type="button"
              size="sm"
              icon="tabler:check"
              :loading="isReviewingCandidate(candidateId(candidate))"
              @click="emit('reviewCandidate', candidate, 'user_confirmed')"
            >
              {{ t('Confirm') }}
            </Button>
          </footer>
        </article>
        <article
          v-for="trace in identityTraces"
          :key="traceId(trace)"
          class="personas-trace-row"
        >
          <div>
            <span>{{ traceKindLabel(trace) }}</span>
            <strong>{{ traceTitle(trace) }}</strong>
            <p>{{ trace.source }} · {{ identityConfidence(trace) }}</p>
          </div>
          <footer>
            <Button
              type="button"
              size="sm"
              variant="outline"
              icon="tabler:user"
              :disabled="!canAssignToSelected(trace)"
              :loading="isAssigningTrace(traceId(trace))"
              @click="emit('assignTraceToSelectedPersona', trace)"
            >
              {{ t('Attach to selected') }}
            </Button>
            <Button
              type="button"
              size="sm"
              icon="tabler:user-check"
              :disabled="!canAssignToOwner(trace)"
              :loading="isAssigningTrace(traceId(trace))"
              @click="emit('assignTraceToOwner', trace)"
            >
              {{ t('Add to me') }}
            </Button>
          </footer>
        </article>
      </section>
    </aside>
  </section>
</template>
