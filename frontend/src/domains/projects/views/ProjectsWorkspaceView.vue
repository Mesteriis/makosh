<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import {
  ProjectOutcomeStateV1,
  ProjectReferenceKindV1,
  ProjectStateV1,
  type ProjectV1
} from '../../../gen/makosh/projects/client/v1/projects_pb'
import { Button, Card, Input, Select } from '../../../shared/ui'
import { useProjectsPageSurface } from '../queries/useProjectsPageSurface'
import {
  dateTimestamp,
  formatTimestamp,
  hex,
  outcomeStateLabel,
  parsePublicId,
  projectStateLabel,
  referenceKindLabel
} from '../stores/projects'

const surface = useProjectsPageSurface()
const createDraft = reactive({ name: '', description: '', startAt: '', targetAt: '' })
const outcomeDraft = reactive({ title: '', description: '', targetAt: '' })
const referenceDraft = reactive({
  kind: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_PERSON),
  publicId: '',
  label: ''
})

const projectStateOptions = [
  { value: String(ProjectStateV1.PROJECT_STATE_PLANNING), label: 'Planning' },
  { value: String(ProjectStateV1.PROJECT_STATE_ACTIVE), label: 'Active' },
  { value: String(ProjectStateV1.PROJECT_STATE_ON_HOLD), label: 'On Hold' },
  { value: String(ProjectStateV1.PROJECT_STATE_COMPLETED), label: 'Completed' },
  { value: String(ProjectStateV1.PROJECT_STATE_ARCHIVED), label: 'Archived' }
]

const outcomeStateOptions = [
  { value: String(ProjectOutcomeStateV1.PROJECT_OUTCOME_STATE_PENDING), label: 'Pending' },
  { value: String(ProjectOutcomeStateV1.PROJECT_OUTCOME_STATE_ACHIEVED), label: 'Achieved' },
  { value: String(ProjectOutcomeStateV1.PROJECT_OUTCOME_STATE_MISSED), label: 'Missed' },
  { value: String(ProjectOutcomeStateV1.PROJECT_OUTCOME_STATE_CANCELLED), label: 'Cancelled' }
]

const referenceKindOptions = [
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_PERSON), label: 'Person' },
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_ORGANIZATION), label: 'Organization' },
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_RELATIONSHIP), label: 'Relationship' },
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_TASK), label: 'Task' },
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_DOCUMENT), label: 'Document' },
  { value: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_CALENDAR_EVENT), label: 'Calendar Event' }
]

onMounted(() => { void surface.loadProjects() })

async function createProject(): Promise<void> {
  if (!createDraft.name.trim()) return
  await surface.createProject({
    name: createDraft.name.trim(),
    description: createDraft.description.trim(),
    startAt: dateTimestamp(createDraft.startAt),
    targetAt: dateTimestamp(createDraft.targetAt)
  })
  Object.assign(createDraft, { name: '', description: '', startAt: '', targetAt: '' })
}

async function addOutcome(project: ProjectV1): Promise<void> {
  if (!outcomeDraft.title.trim()) return
  await surface.addOutcome(project, {
    title: outcomeDraft.title.trim(),
    description: outcomeDraft.description.trim(),
    targetAt: dateTimestamp(outcomeDraft.targetAt)
  })
  Object.assign(outcomeDraft, { title: '', description: '', targetAt: '' })
}

async function addReference(project: ProjectV1): Promise<void> {
  if (!referenceDraft.publicId.trim() || !referenceDraft.label.trim()) return
  await surface.addReference(project, {
    kind: Number(referenceDraft.kind) as ProjectReferenceKindV1,
    publicId: parsePublicId(referenceDraft.publicId),
    label: referenceDraft.label.trim()
  })
  Object.assign(referenceDraft, {
    kind: String(ProjectReferenceKindV1.PROJECT_REFERENCE_KIND_PERSON),
    publicId: '',
    label: ''
  })
}

function isMutating(project: ProjectV1): boolean {
  return surface.mutatingProjectId.value === hex(project.projectId)
}
</script>

<template>
  <main class="projects-workspace" aria-label="Projects">
    <header class="projects-workspace__header">
      <div>
        <p class="projects-workspace__eyebrow">OWNER PROJECTS</p>
        <h1>Projects</h1>
        <p>{{ surface.activeProjects.value.length }} active · {{ surface.completedProjects.value.length }} completed or archived</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadProjects">Refresh</Button>
    </header>

    <form class="projects-workspace__create" @submit.prevent="createProject">
      <Input v-model="createDraft.name" aria-label="Project name" placeholder="Project name" />
      <Input v-model="createDraft.description" aria-label="Project description" placeholder="Description" />
      <Input v-model="createDraft.startAt" aria-label="Project start date" type="date" />
      <Input v-model="createDraft.targetAt" aria-label="Project target date" type="date" />
      <Button type="submit" icon="tabler:briefcase" :disabled="!createDraft.name.trim()">Create project</Button>
    </form>

    <p v-if="surface.error.value" class="projects-workspace__error" role="alert">{{ surface.error.value }}</p>
    <p v-if="surface.isLoading.value && surface.projects.value.length === 0" aria-live="polite">Loading Projects…</p>
    <p v-else-if="surface.projects.value.length === 0" class="projects-workspace__empty">No projects yet.</p>

    <section v-else class="projects-workspace__layout">
      <div class="projects-workspace__list" aria-label="Projects directory">
        <Card
          v-for="project in surface.projects.value"
          :key="hex(project.projectId)"
          class="projects-workspace__project"
          :selected="surface.selectedProject.value !== undefined && hex(surface.selectedProject.value.projectId) === hex(project.projectId)"
          @click="surface.select(project)"
        >
          <h2>{{ project.name }}</h2>
          <p>{{ project.description || 'No description.' }}</p>
          <small>{{ formatTimestamp(project.startAt) }} → {{ formatTimestamp(project.targetAt) }} · revision {{ project.projectRevision }}</small>
          <Select
            :model-value="String(project.state)"
            :options="projectStateOptions"
            aria-label="Project lifecycle state"
            :disabled="isMutating(project)"
            @click.stop
            @update:model-value="surface.setProjectState(project, Number($event) as ProjectStateV1)"
          />
        </Card>
      </div>

      <Card v-if="surface.selectedProject.value" class="projects-workspace__detail">
        <header>
          <div>
            <p class="projects-workspace__eyebrow">{{ projectStateLabel(surface.selectedProject.value.state) }}</p>
            <h2>{{ surface.selectedProject.value.name }}</h2>
          </div>
          <small>Revision {{ surface.selectedProject.value.projectRevision }}</small>
        </header>
        <p>{{ surface.selectedProject.value.description || 'No description.' }}</p>
        <dl>
          <div><dt>Starts</dt><dd>{{ formatTimestamp(surface.selectedProject.value.startAt) }}</dd></div>
          <div><dt>Target</dt><dd>{{ formatTimestamp(surface.selectedProject.value.targetAt) }}</dd></div>
        </dl>

        <section>
          <h3>Expected outcomes</h3>
          <p v-if="surface.outcomes.value.length === 0" class="projects-workspace__muted">Add at least one outcome before completing this project.</p>
          <ul>
            <li v-for="outcome in surface.outcomes.value" :key="hex(outcome.outcomeId)">
              <div>
                <strong>{{ outcome.title }}</strong>
                <span>{{ outcome.description || 'No description.' }} · {{ formatTimestamp(outcome.targetAt) }}</span>
              </div>
              <Select
                :model-value="String(outcome.state)"
                :options="outcomeStateOptions"
                aria-label="Expected outcome state"
                @update:model-value="surface.setOutcomeState(surface.selectedProject.value!, outcome, Number($event) as ProjectOutcomeStateV1)"
              />
              <Button size="sm" variant="ghost" @click="surface.removeOutcome(surface.selectedProject.value!, outcome)">Remove</Button>
            </li>
          </ul>
          <form @submit.prevent="addOutcome(surface.selectedProject.value)">
            <Input v-model="outcomeDraft.title" aria-label="Expected outcome title" placeholder="Expected outcome" />
            <Input v-model="outcomeDraft.description" aria-label="Expected outcome description" placeholder="Description" />
            <Input v-model="outcomeDraft.targetAt" aria-label="Expected outcome target date" type="date" />
            <Button type="submit" variant="outline" size="sm">Add outcome</Button>
          </form>
        </section>

        <section>
          <h3>Typed public references</h3>
          <p v-if="surface.references.value.length === 0" class="projects-workspace__muted">No public owner references.</p>
          <ul>
            <li v-for="reference in surface.references.value" :key="hex(reference.referenceId)">
              <span>{{ referenceKindLabel(reference.kind) }} · {{ reference.label }}</span>
              <Button size="sm" variant="ghost" @click="surface.removeReference(surface.selectedProject.value!, reference)">Remove</Button>
            </li>
          </ul>
          <form @submit.prevent="addReference(surface.selectedProject.value)">
            <Select v-model="referenceDraft.kind" :options="referenceKindOptions" aria-label="Reference kind" />
            <Input v-model="referenceDraft.publicId" aria-label="Public reference ID" placeholder="32-character public ID" />
            <Input v-model="referenceDraft.label" aria-label="Reference label" placeholder="Label" />
            <Button type="submit" variant="outline" size="sm">Add reference</Button>
          </form>
        </section>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.projects-workspace { display: grid; gap: 1.25rem; width: min(78rem, 100%); margin: 0 auto; padding: 1.5rem; }
.projects-workspace__header, .projects-workspace__create, .projects-workspace__detail header, .projects-workspace__detail form { display: flex; gap: .75rem; align-items: center; }
.projects-workspace__header, .projects-workspace__detail header { justify-content: space-between; align-items: flex-start; }
.projects-workspace__header h1, .projects-workspace__project h2, .projects-workspace__detail h2, .projects-workspace__detail h3 { margin: 0; }
.projects-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.projects-workspace__header p, .projects-workspace__project p, .projects-workspace__project small, .projects-workspace__muted { color: var(--text-secondary); }
.projects-workspace__create { flex-wrap: wrap; padding: 1rem; border: 1px solid var(--border-subtle); border-radius: .75rem; }
.projects-workspace__create :deep(.makosh-input-wrapper), .projects-workspace__detail :deep(.makosh-input-wrapper) { flex: 1; }
.projects-workspace__layout { display: grid; grid-template-columns: minmax(18rem, 2fr) minmax(25rem, 3fr); gap: 1rem; align-items: start; }
.projects-workspace__list, .projects-workspace__detail { display: grid; gap: .75rem; }
.projects-workspace__project { display: grid; gap: .5rem; padding: 1rem; cursor: pointer; }
.projects-workspace__detail { padding: 1rem; }
.projects-workspace__detail dl { display: grid; gap: .5rem; }
.projects-workspace__detail dl div { display: grid; grid-template-columns: 5rem 1fr; gap: .5rem; }
.projects-workspace__detail dt { color: var(--text-secondary); }
.projects-workspace__detail dd { margin: 0; }
.projects-workspace__detail section { display: grid; gap: .75rem; padding-top: .75rem; border-top: 1px solid var(--border-subtle); }
.projects-workspace__detail ul { display: grid; gap: .5rem; margin: 0; padding: 0; list-style: none; }
.projects-workspace__detail li { display: flex; justify-content: space-between; gap: .75rem; align-items: center; }
.projects-workspace__detail li > div { display: grid; gap: .25rem; flex: 1; }
.projects-workspace__detail li span { color: var(--text-secondary); }
.projects-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.projects-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
@media (max-width: 880px) { .projects-workspace__layout { grid-template-columns: 1fr; } .projects-workspace__header, .projects-workspace__create, .projects-workspace__detail form { align-items: stretch; flex-direction: column; } }
</style>
