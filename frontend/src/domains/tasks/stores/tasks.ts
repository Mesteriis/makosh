import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  TaskPriorityV1,
  TaskStateV1,
  type TaskSummaryV1,
  type TimestampV1
} from '../../../gen/makosh/tasks/client/v1/tasks_pb'
import {
  getTasksCommandClient,
  getTasksQueryClient
} from '../../../platform/connect/tasksClient'

const PAGE_LIMIT = 50

export const useTasksStore = defineStore('tasks', () => {
  const tasks = ref<TaskSummaryV1[]>([])
  const error = ref('')
  const mutatingTaskId = ref<string | null>(null)
  const isLoading = ref(false)

  const openTasks = computed(() => tasks.value.filter((task) =>
    task.state === TaskStateV1.TASK_STATE_OPEN
    || task.state === TaskStateV1.TASK_STATE_IN_PROGRESS
  ))
  const completedTasks = computed(() => tasks.value.filter((task) =>
    task.state === TaskStateV1.TASK_STATE_COMPLETED
    || task.state === TaskStateV1.TASK_STATE_CANCELLED
  ))

  async function loadAll(): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const loaded: TaskSummaryV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      do {
        const page = await getTasksQueryClient().list({
          logicalOwnerId: '',
          afterTaskId: cursor,
          limit: PAGE_LIMIT
        })
        loaded.push(...page.tasks)
        cursor = page.nextAfterTaskId
      } while (cursor.length > 0)
      tasks.value = loaded
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function createTask(title: string, description?: string, dueAt?: Date): Promise<void> {
    await run(null, async () => {
      const result = await getTasksCommandClient().create({
        operationId: randomId16(),
        taskId: new Uint8Array(),
        logicalOwnerId: '',
        title,
        description: description?.trim() || undefined,
        dueAt: dueAt ? timestamp(dueAt) : undefined,
        priority: TaskPriorityV1.TASK_PRIORITY_NORMAL,
        createdAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function updateTask(
    task: TaskSummaryV1,
    values: { title?: string; description?: string; clearDescription?: boolean; dueAt?: Date; clearDueAt?: boolean }
  ): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().update({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        title: values.title,
        description: values.description,
        clearDescription: values.clearDescription ?? false,
        dueAt: values.dueAt ? timestamp(values.dueAt) : undefined,
        clearDueAt: values.clearDueAt ?? false,
        updatedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function setTaskState(task: TaskSummaryV1, state: TaskStateV1): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().setState({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        state,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function setTaskPriority(task: TaskSummaryV1, priority: TaskPriorityV1): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().setPriority({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        priority,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function addDependency(task: TaskSummaryV1, dependsOnTaskId: Uint8Array): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().addDependency({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        dependencyId: randomId16(),
        dependsOnTaskId,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function removeDependency(task: TaskSummaryV1, dependencyId: Uint8Array): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().removeDependency({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        dependencyId,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function addChecklistItem(task: TaskSummaryV1, label: string): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().addChecklistItem({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        checklistItemId: randomId16(),
        label,
        position: task.checklist.length,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function updateChecklistItem(
    task: TaskSummaryV1,
    checklistItemId: Uint8Array,
    values: { label?: string; completed?: boolean; position?: number }
  ): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().updateChecklistItem({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        checklistItemId,
        label: values.label,
        completed: values.completed,
        position: values.position,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function removeChecklistItem(task: TaskSummaryV1, checklistItemId: Uint8Array): Promise<void> {
    await run(hex(task.taskId), async () => {
      const result = await getTasksCommandClient().removeChecklistItem({
        operationId: randomId16(),
        taskId: task.taskId,
        logicalOwnerId: '',
        expectedTaskRevision: task.taskRevision,
        checklistItemId,
        changedAt: timestamp(new Date())
      })
      replaceResult(result.task)
    })
  }

  async function run(taskId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingTaskId.value = taskId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingTaskId.value = null
    }
  }

  function replaceResult(task: TaskSummaryV1 | undefined): void {
    if (!task) throw new Error('tasks_invalid_response')
    const index = tasks.value.findIndex((value) => sameBytes(value.taskId, task.taskId))
    if (index === -1) tasks.value.push(task)
    else tasks.value[index] = task
    tasks.value.sort((left, right) => compareBytes(left.taskId, right.taskId))
  }

  return {
    tasks,
    error,
    mutatingTaskId,
    isLoading,
    openTasks,
    completedTasks,
    loadAll,
    createTask,
    updateTask,
    setTaskState,
    setTaskPriority,
    addDependency,
    removeDependency,
    addChecklistItem,
    updateChecklistItem,
    removeChecklistItem
  }
})

function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.tasks.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
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

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'tasks_unavailable'
}
