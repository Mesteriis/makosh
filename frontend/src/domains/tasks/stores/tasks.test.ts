import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  TaskPriorityV1,
  TaskStateV1,
  type TaskSummaryV1
} from '../../../gen/makosh/tasks/client/v1/tasks_pb'

const clients = vi.hoisted(() => ({
  query: { list: vi.fn(), get: vi.fn() },
  command: {
    create: vi.fn(), update: vi.fn(), setState: vi.fn(), setPriority: vi.fn(),
    addDependency: vi.fn(), removeDependency: vi.fn(), addChecklistItem: vi.fn(),
    updateChecklistItem: vi.fn(), removeChecklistItem: vi.fn()
  }
}))

vi.mock('../../../platform/connect/tasksClient', () => ({
  getTasksQueryClient: () => clients.query,
  getTasksCommandClient: () => clients.command
}))

import { useTasksStore } from './tasks'

describe('typed Tasks owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('loads all bounded pages with the exclusive last-returned cursor', async () => {
    clients.query.list
      .mockResolvedValueOnce({ tasks: [task(1)], nextAfterTaskId: id(1) })
      .mockResolvedValueOnce({ tasks: [task(2)], nextAfterTaskId: new Uint8Array() })
    const store = useTasksStore()

    await store.loadAll()

    expect(store.tasks.map((value) => value.taskId)).toEqual([id(1), id(2)])
    expect(clients.query.list.mock.calls).toEqual([
      [{ logicalOwnerId: '', afterTaskId: new Uint8Array(), limit: 50 }],
      [{ logicalOwnerId: '', afterTaskId: id(1), limit: 50 }]
    ])
  })

  it('dispatches state priority and checklist mutations with exact current revisions', async () => {
    const initial = task(1)
    clients.query.list.mockResolvedValue({ tasks: [initial], nextAfterTaskId: new Uint8Array() })
    clients.command.setState.mockResolvedValue({ task: { ...initial, taskRevision: 2n, state: TaskStateV1.TASK_STATE_IN_PROGRESS } })
    clients.command.setPriority.mockResolvedValue({ task: { ...initial, taskRevision: 3n, priority: TaskPriorityV1.TASK_PRIORITY_HIGH } })
    clients.command.addChecklistItem.mockResolvedValue({ task: { ...initial, taskRevision: 4n, checklist: [{ checklistItemId: id(9), label: 'Proof', completed: false, position: 0, updatedAtTaskRevision: 4n }] } })
    const store = useTasksStore()
    await store.loadAll()

    await store.setTaskState(store.tasks[0]!, TaskStateV1.TASK_STATE_IN_PROGRESS)
    await store.setTaskPriority(store.tasks[0]!, TaskPriorityV1.TASK_PRIORITY_HIGH)
    await store.addChecklistItem(store.tasks[0]!, 'Proof')

    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      taskId: id(1), expectedTaskRevision: 1n, state: TaskStateV1.TASK_STATE_IN_PROGRESS
    })
    expect(clients.command.setPriority.mock.calls[0]?.[0]).toMatchObject({
      taskId: id(1), expectedTaskRevision: 2n, priority: TaskPriorityV1.TASK_PRIORITY_HIGH
    })
    expect(clients.command.addChecklistItem.mock.calls[0]?.[0]).toMatchObject({
      taskId: id(1), expectedTaskRevision: 3n, label: 'Proof'
    })
    expect(store.tasks[0]?.taskRevision).toBe(4n)
  })
})

function task(seed: number): TaskSummaryV1 {
  return {
    $typeName: 'makosh.tasks.client.v1.TaskSummaryV1',
    taskId: id(seed),
    logicalOwnerId: 'owner-1',
    title: `Task ${seed}`,
    state: TaskStateV1.TASK_STATE_OPEN,
    priority: TaskPriorityV1.TASK_PRIORITY_NORMAL,
    taskRevision: 1n,
    dependencies: [],
    checklist: []
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
