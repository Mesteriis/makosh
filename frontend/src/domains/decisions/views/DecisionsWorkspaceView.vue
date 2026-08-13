<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { DecisionStateV1 } from '../../../gen/makosh/decisions/client/v1/decisions_pb'
import { Button, Card, Input } from '../../../shared/ui'
import { useDecisionsPageSurface } from '../queries/useDecisionsPageSurface'

const surface = useDecisionsPageSurface()
const title = ref('')
const question = ref('')
const rationale = ref('')
const alternativeTitle = ref('')
const alternativeDescription = ref('')

onMounted(() => { void surface.loadAll() })

async function create(): Promise<void> {
  await surface.createDecision(title.value, question.value)
  title.value = ''
  question.value = ''
}

async function addAlternative(): Promise<void> {
  const decision = surface.selectedDecision.value
  if (!decision) return
  await surface.addAlternative(decision, alternativeTitle.value, alternativeDescription.value)
  alternativeTitle.value = ''
  alternativeDescription.value = ''
}

function isSelected(decisionId: Uint8Array): boolean {
  const selected = surface.selectedDecision.value?.decisionId
  return selected !== undefined
    && selected.length === decisionId.length
    && selected.every((value, index) => value === decisionId[index])
}
</script>

<template>
  <main class="decisions-workspace" aria-label="Decisions">
    <header>
      <div><p>OWNER TRUTH</p><h1>Decisions</h1></div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadAll">Refresh</Button>
    </header>
    <Card class="decisions-workspace__composer">
      <Input v-model="title" label="Title" />
      <Input v-model="question" label="Decision question" />
      <Button :disabled="!title.trim() || !question.trim()" @click="create">Create draft</Button>
    </Card>
    <p v-if="surface.error.value" role="alert">{{ surface.error.value }}</p>
    <section class="decisions-workspace__list">
      <Card v-for="decision in surface.decisions.value" :key="String(decision.decisionId)" @click="surface.select(decision)">
        <h2>{{ decision.title }}</h2>
        <p>{{ decision.question }}</p>
        <small>Revision {{ decision.decisionRevision }} · state {{ decision.state }}</small>
        <template v-if="isSelected(decision.decisionId) && decision.state === DecisionStateV1.DECISION_STATE_DRAFT">
          <Input v-model="alternativeTitle" label="Alternative" />
          <Input v-model="alternativeDescription" label="Alternative notes" />
          <Button variant="secondary" :disabled="!alternativeTitle.trim()" @click.stop="addAlternative">Add alternative</Button>
          <Input v-model="rationale" label="Rationale" />
          <div class="decisions-workspace__alternatives">
            <Button v-for="alternative in surface.alternatives.value" :key="String(alternative.alternativeId)" variant="secondary" :disabled="surface.alternatives.value.length < 2 || !rationale.trim()" @click.stop="surface.decide(decision, alternative.alternativeId, rationale)">
              Select {{ alternative.title }}
            </Button>
          </div>
          <Button variant="ghost" @click.stop="surface.cancel(decision)">Cancel draft</Button>
        </template>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.decisions-workspace { display: grid; gap: 1rem; width: min(72rem, 100%); margin: 0 auto; padding: 1.5rem; }
.decisions-workspace header { display: flex; justify-content: space-between; align-items: flex-start; }
.decisions-workspace h1, .decisions-workspace h2, .decisions-workspace p { margin: 0; }
.decisions-workspace__composer, .decisions-workspace__list { display: grid; gap: .75rem; }
.decisions-workspace__alternatives { display: flex; flex-wrap: wrap; gap: .5rem; }
</style>
