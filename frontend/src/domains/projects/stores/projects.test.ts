import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  ProjectOutcomeStateV1,
  ProjectStateV1,
  type ProjectV1
} from '../../../gen/makosh/projects/client/v1/projects_pb'

const clients = vi.hoisted(() => ({
  query: {
    list: vi.fn(), get: vi.fn(), listOutcomes: vi.fn(), listReferences: vi.fn()
  },
  command: {
    create: vi.fn(), update: vi.fn(), setState: vi.fn(), addOutcome: vi.fn(),
    updateOutcome: vi.fn(), setOutcomeState: vi.fn(), removeOutcome: vi.fn(),
    addReference: vi.fn(), removeReference: vi.fn()
  }
}))

vi.mock('../api/projects', () => ({
  getProjectsQueryClient: () => clients.query,
  getProjectsCommandClient: () => clients.command
}))

import { useProjectsStore } from './projects'

describe('typed Projects owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.query.listOutcomes.mockResolvedValue({ outcomes: [], nextAfterOutcomeId: new Uint8Array() })
    clients.query.listReferences.mockResolvedValue({ references: [], nextAfterReferenceId: new Uint8Array() })
  })

  it('loads every page using the exclusive last-returned cursor', async () => {
    clients.query.list
      .mockResolvedValueOnce({ projects: [project(1)], nextAfterProjectId: id(1) })
      .mockResolvedValueOnce({ projects: [project(2)], nextAfterProjectId: new Uint8Array() })
    const store = useProjectsStore()

    await store.loadAll()

    expect(store.projects.map((value) => value.projectId)).toEqual([id(1), id(2)])
    expect(clients.query.list.mock.calls).toEqual([
      [{ logicalOwnerId: '', afterProjectId: new Uint8Array(), limit: 50 }],
      [{ logicalOwnerId: '', afterProjectId: id(1), limit: 50 }]
    ])
  })

  it('dispatches lifecycle and outcome commands with exact current revisions', async () => {
    const initial = project(1)
    const active = { ...initial, state: ProjectStateV1.PROJECT_STATE_ACTIVE, projectRevision: 2n }
    const withOutcome = { ...active, projectRevision: 3n }
    clients.query.list.mockResolvedValue({ projects: [initial], nextAfterProjectId: new Uint8Array() })
    clients.command.setState.mockResolvedValue({ project: active })
    clients.command.addOutcome.mockResolvedValue({ project: withOutcome })
    const store = useProjectsStore()
    await store.loadAll()

    await store.setProjectState(store.projects[0]!, ProjectStateV1.PROJECT_STATE_ACTIVE)
    await store.addOutcome(store.projects[0]!, { title: 'Release', description: 'Ship safely' })

    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      projectId: id(1), expectedProjectRevision: 1n,
      state: ProjectStateV1.PROJECT_STATE_ACTIVE
    })
    expect(clients.command.addOutcome.mock.calls[0]?.[0]).toMatchObject({
      projectId: id(1), expectedProjectRevision: 2n,
      title: 'Release', description: 'Ship safely'
    })
    expect(store.projects[0]?.projectRevision).toBe(3n)
    expect(ProjectOutcomeStateV1.PROJECT_OUTCOME_STATE_PENDING).toBe(1)
  })
})

function project(seed: number): ProjectV1 {
  return {
    $typeName: 'makosh.projects.client.v1.ProjectV1',
    projectId: id(seed),
    logicalOwnerId: 'owner-1',
    name: `Project ${seed}`,
    description: '',
    state: ProjectStateV1.PROJECT_STATE_PLANNING,
    projectRevision: 1n
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
