import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  ProjectOutcomeStateV1,
  ProjectReferenceKindV1,
  ProjectStateV1,
  type ProjectOutcomeV1,
  type ProjectReferenceV1,
  type ProjectV1,
  type TimestampV1
} from '../../../gen/makosh/projects/client/v1/projects_pb'
import {
  getProjectsCommandClient,
  getProjectsQueryClient
} from '../api/projects'
import type { ProjectDraft, ProjectOutcomeDraft, ProjectReferenceDraft } from '../types/project'

const PAGE_LIMIT = 50

export const useProjectsStore = defineStore('projects-owner', () => {
  const projects = ref<ProjectV1[]>([])
  const selectedProject = ref<ProjectV1>()
  const outcomes = ref<ProjectOutcomeV1[]>([])
  const references = ref<ProjectReferenceV1[]>([])
  const error = ref('')
  const isLoading = ref(false)
  const mutatingProjectId = ref<string | null>(null)

  const activeProjects = computed(() => projects.value.filter((project) =>
    project.state === ProjectStateV1.PROJECT_STATE_ACTIVE
  ))
  const completedProjects = computed(() => projects.value.filter((project) =>
    project.state === ProjectStateV1.PROJECT_STATE_COMPLETED
      || project.state === ProjectStateV1.PROJECT_STATE_ARCHIVED
  ))

  async function loadAll(): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const next: ProjectV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      for (let page = 0; page < 100; page += 1) {
        const result = await getProjectsQueryClient().list({
          logicalOwnerId: '',
          afterProjectId: cursor,
          limit: PAGE_LIMIT
        })
        next.push(...result.projects)
        if (result.nextAfterProjectId.length === 0) break
        cursor = result.nextAfterProjectId
      }
      projects.value = next
      if (selectedProject.value) {
        const refreshed = next.find((project) => sameBytes(
          project.projectId,
          selectedProject.value!.projectId
        ))
        if (refreshed) selectedProject.value = refreshed
      }
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function select(project: ProjectV1): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      selectedProject.value = await getProjectsQueryClient().get({
        logicalOwnerId: '',
        projectId: project.projectId
      })
      await loadChildren(selectedProject.value)
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function createProject(input: ProjectDraft): Promise<void> {
    await run(null, async () => {
      const result = await getProjectsCommandClient().create({
        operationId: randomId16(),
        logicalOwnerId: '',
        name: input.name,
        description: input.description,
        startAt: input.startAt,
        targetAt: input.targetAt,
        createdAt: timestamp(new Date())
      })
      const project = replaceResult(result.project)
      selectedProject.value = project
      outcomes.value = []
      references.value = []
    })
  }

  async function updateProject(project: ProjectV1, input: Partial<ProjectDraft>): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().update({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      name: input.name,
      description: input.description,
      startAt: input.startAt,
      targetAt: input.targetAt,
      changedAt: timestamp(new Date())
    }))
  }

  async function setProjectState(project: ProjectV1, state: ProjectStateV1): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().setState({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      state,
      changedAt: timestamp(new Date())
    }))
  }

  async function addOutcome(project: ProjectV1, input: ProjectOutcomeDraft): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().addOutcome({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      title: input.title,
      description: input.description,
      targetAt: input.targetAt,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function updateOutcome(
    project: ProjectV1,
    outcome: ProjectOutcomeV1,
    input: Partial<ProjectOutcomeDraft>
  ): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().updateOutcome({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      outcomeId: outcome.outcomeId,
      expectedOutcomeRevision: outcome.outcomeRevision,
      title: input.title,
      description: input.description,
      targetAt: input.targetAt,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function setOutcomeState(
    project: ProjectV1,
    outcome: ProjectOutcomeV1,
    state: ProjectOutcomeStateV1
  ): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().setOutcomeState({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      outcomeId: outcome.outcomeId,
      expectedOutcomeRevision: outcome.outcomeRevision,
      state,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function removeOutcome(project: ProjectV1, outcome: ProjectOutcomeV1): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().removeOutcome({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      outcomeId: outcome.outcomeId,
      expectedOutcomeRevision: outcome.outcomeRevision,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function addReference(project: ProjectV1, input: ProjectReferenceDraft): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().addReference({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      kind: input.kind,
      publicId: input.publicId,
      label: input.label,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function removeReference(project: ProjectV1, reference: ProjectReferenceV1): Promise<void> {
    await mutate(project, () => getProjectsCommandClient().removeReference({
      operationId: randomId16(),
      projectId: project.projectId,
      logicalOwnerId: '',
      expectedProjectRevision: project.projectRevision,
      referenceId: reference.referenceId,
      changedAt: timestamp(new Date())
    }), true)
  }

  async function loadChildren(project: ProjectV1): Promise<void> {
    const nextOutcomes: ProjectOutcomeV1[] = []
    let outcomeCursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    for (let page = 0; page < 100; page += 1) {
      const result = await getProjectsQueryClient().listOutcomes({
        logicalOwnerId: '', projectId: project.projectId,
        afterOutcomeId: outcomeCursor, limit: PAGE_LIMIT
      })
      nextOutcomes.push(...result.outcomes)
      if (result.nextAfterOutcomeId.length === 0) break
      outcomeCursor = result.nextAfterOutcomeId
    }
    const nextReferences: ProjectReferenceV1[] = []
    let referenceCursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    for (let page = 0; page < 100; page += 1) {
      const result = await getProjectsQueryClient().listReferences({
        logicalOwnerId: '', projectId: project.projectId,
        afterReferenceId: referenceCursor, limit: PAGE_LIMIT
      })
      nextReferences.push(...result.references)
      if (result.nextAfterReferenceId.length === 0) break
      referenceCursor = result.nextAfterReferenceId
    }
    outcomes.value = nextOutcomes
    references.value = nextReferences
  }

  async function mutate(
    project: ProjectV1,
    operation: () => Promise<{ project?: ProjectV1 }>,
    reloadChildren = false
  ): Promise<void> {
    await run(hex(project.projectId), async () => {
      const result = await operation()
      const updated = replaceResult(result.project)
      if (reloadChildren) await loadChildren(updated)
    })
  }

  async function run(projectId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingProjectId.value = projectId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingProjectId.value = null
    }
  }

  function replaceResult(project: ProjectV1 | undefined): ProjectV1 {
    if (!project) throw new Error('projects_invalid_response')
    const index = projects.value.findIndex((value) => sameBytes(value.projectId, project.projectId))
    if (index === -1) projects.value.push(project)
    else projects.value[index] = project
    projects.value.sort((left, right) => compareBytes(left.projectId, right.projectId))
    if (selectedProject.value && sameBytes(selectedProject.value.projectId, project.projectId)) {
      selectedProject.value = project
    }
    return project
  }

  return {
    projects, selectedProject, outcomes, references, error, isLoading, mutatingProjectId,
    activeProjects, completedProjects, loadAll, select, createProject, updateProject,
    setProjectState, addOutcome, updateOutcome, setOutcomeState, removeOutcome,
    addReference, removeReference
  }
})

export function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.projects.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
}

export function dateTimestamp(value: string): TimestampV1 | undefined {
  if (!value) return undefined
  const date = new Date(`${value}T00:00:00`)
  if (Number.isNaN(date.getTime())) throw new Error('projects_invalid_date')
  return timestamp(date)
}

export function formatTimestamp(value: TimestampV1 | undefined): string {
  if (!value) return 'Not set'
  const date = new Date(Number(value.unixSeconds) * 1_000 + Math.trunc(value.nanos / 1_000_000))
  return new Intl.DateTimeFormat('en', { dateStyle: 'medium' }).format(date)
}

export function projectStateLabel(value: ProjectStateV1): string {
  return enumLabel(ProjectStateV1[value]?.replace('PROJECT_STATE_', '') ?? 'Unknown')
}

export function outcomeStateLabel(value: ProjectOutcomeStateV1): string {
  return enumLabel(ProjectOutcomeStateV1[value]?.replace('PROJECT_OUTCOME_STATE_', '') ?? 'Unknown')
}

export function referenceKindLabel(value: ProjectReferenceKindV1): string {
  return enumLabel(ProjectReferenceKindV1[value]?.replace('PROJECT_REFERENCE_KIND_', '') ?? 'Unknown')
}

export function parsePublicId(value: string): Uint8Array {
  const normalized = value.trim().toLowerCase()
  if (!/^[0-9a-f]{32}$/.test(normalized)) throw new Error('projects_invalid_public_id')
  return Uint8Array.from(normalized.match(/../g) ?? [], (byte) => Number.parseInt(byte, 16))
}

export function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function enumLabel(value: string): string {
  return value.toLowerCase().split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ')
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
  return cause instanceof Error ? cause.message : 'projects_unavailable'
}
