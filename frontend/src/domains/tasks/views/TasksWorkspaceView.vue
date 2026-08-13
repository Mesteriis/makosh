<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import {
  TaskPriorityV1,
  TaskStateV1,
  type TaskSummaryV1
} from '../../../gen/makosh/tasks/client/v1/tasks_pb'
import { Button, Card, Input, Select } from '../../../shared/ui'
import { useTasksPageSurface } from '../queries/useTasksPageSurface'

const surface = useTasksPageSurface()
const newTaskTitle = ref('')
const checklistDrafts = reactive<Record<string, string>>({})

const stateOptions = [
  { value: String(TaskStateV1.TASK_STATE_OPEN), label: 'Open' },
  { value: String(TaskStateV1.TASK_STATE_IN_PROGRESS), label: 'In progress' },
  { value: String(TaskStateV1.TASK_STATE_COMPLETED), label: 'Completed' },
  { value: String(TaskStateV1.TASK_STATE_CANCELLED), label: 'Cancelled' }
]
const priorityOptions = [
  { value: String(TaskPriorityV1.TASK_PRIORITY_LOW), label: 'Low' },
  { value: String(TaskPriorityV1.TASK_PRIORITY_NORMAL), label: 'Normal' },
  { value: String(TaskPriorityV1.TASK_PRIORITY_HIGH), label: 'High' },
  { value: String(TaskPriorityV1.TASK_PRIORITY_URGENT), label: 'Urgent' }
]

onMounted(() => { void surface.loadTasks() })

async function createTask(): Promise<void> {
  const title = newTaskTitle.value.trim()
  if (!title) return
  await surface.createTask(title)
  newTaskTitle.value = ''
}

async function addChecklistItem(task: TaskSummaryV1): Promise<void> {
  const key = hex(task.taskId)
  const label = checklistDrafts[key]?.trim() ?? ''
  if (!label) return
  await surface.addChecklistItem(task, label)
  checklistDrafts[key] = ''
}

function isMutating(task: TaskSummaryV1): boolean {
  return surface.mutatingTaskId.value === hex(task.taskId)
}

function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}
</script>

<template>
  <main class="tasks-workspace" aria-label="Tasks">
    <header class="tasks-workspace__header">
      <div>
        <p class="tasks-workspace__eyebrow">OWNER WORKLIST</p>
        <h1>Tasks</h1>
        <p>{{ surface.openTasks.value.length }} active · {{ surface.completedTasks.value.length }} closed</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadTasks">
        Refresh
      </Button>
    </header>

    <form class="tasks-workspace__create" @submit.prevent="createTask">
      <Input v-model="newTaskTitle" aria-label="New task title" placeholder="Add a task…" />
      <Button type="submit" icon="tabler:plus" :disabled="!newTaskTitle.trim()">Create task</Button>
    </form>

    <p v-if="surface.error.value" class="tasks-workspace__error" role="alert">
      {{ surface.error.value }}
    </p>
    <p v-if="surface.isLoading.value && surface.tasks.value.length === 0" aria-live="polite">
      Loading Tasks…
    </p>
    <p v-else-if="surface.tasks.value.length === 0" class="tasks-workspace__empty">
      No tasks yet. Create the first owner task above.
    </p>

    <section v-else class="tasks-workspace__list" aria-label="Task worklist">
      <Card v-for="task in surface.tasks.value" :key="hex(task.taskId)" class="tasks-workspace__task">
        <div class="tasks-workspace__task-header">
          <div>
            <h2>{{ task.title }}</h2>
            <p v-if="task.description">{{ task.description }}</p>
            <small>Revision {{ task.taskRevision }} · {{ task.dependencies.length }} dependencies</small>
          </div>
          <div class="tasks-workspace__controls">
            <Select
              :model-value="String(task.state)"
              :options="stateOptions"
              aria-label="Task state"
              :disabled="isMutating(task)"
              @update:model-value="surface.setTaskState(task, Number($event) as TaskStateV1)"
            />
            <Select
              :model-value="String(task.priority)"
              :options="priorityOptions"
              aria-label="Task priority"
              :disabled="isMutating(task)"
              @update:model-value="surface.setTaskPriority(task, Number($event) as TaskPriorityV1)"
            />
          </div>
        </div>

        <ul v-if="task.checklist.length" class="tasks-workspace__checklist">
          <li v-for="item in task.checklist" :key="hex(item.checklistItemId)">
            <label>
              <input
                type="checkbox"
                :checked="item.completed"
                :disabled="isMutating(task)"
                @change="surface.updateChecklistItem(task, item.checklistItemId, { completed: !item.completed })"
              />
              <span :class="{ 'tasks-workspace__checked': item.completed }">{{ item.label }}</span>
            </label>
            <Button
              variant="ghost"
              size="sm"
              icon="tabler:x"
              aria-label="Remove checklist item"
              :disabled="isMutating(task)"
              @click="surface.removeChecklistItem(task, item.checklistItemId)"
            />
          </li>
        </ul>

        <form class="tasks-workspace__checklist-add" @submit.prevent="addChecklistItem(task)">
          <Input
            v-model="checklistDrafts[hex(task.taskId)]"
            aria-label="Checklist item label"
            placeholder="Add checklist item…"
            :disabled="isMutating(task)"
          />
          <Button type="submit" variant="outline" size="sm" :disabled="isMutating(task)">Add</Button>
        </form>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.tasks-workspace { display: grid; gap: 1.25rem; width: min(72rem, 100%); margin: 0 auto; padding: 1.5rem; }
.tasks-workspace__header, .tasks-workspace__task-header, .tasks-workspace__controls, .tasks-workspace__create, .tasks-workspace__checklist-add { display: flex; align-items: center; gap: .75rem; }
.tasks-workspace__header, .tasks-workspace__task-header { justify-content: space-between; align-items: flex-start; }
.tasks-workspace__header h1, .tasks-workspace__task h2 { margin: 0; }
.tasks-workspace__header p, .tasks-workspace__task p, .tasks-workspace__task small { color: var(--text-secondary); }
.tasks-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.tasks-workspace__create :deep(.makosh-input-wrapper), .tasks-workspace__checklist-add :deep(.makosh-input-wrapper) { flex: 1; }
.tasks-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.tasks-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
.tasks-workspace__list { display: grid; gap: 1rem; }
.tasks-workspace__task { display: grid; gap: 1rem; padding: 1rem; }
.tasks-workspace__controls { flex-wrap: wrap; }
.tasks-workspace__checklist { display: grid; gap: .5rem; margin: 0; padding: 0; list-style: none; }
.tasks-workspace__checklist li, .tasks-workspace__checklist label { display: flex; align-items: center; gap: .625rem; }
.tasks-workspace__checklist li { justify-content: space-between; }
.tasks-workspace__checked { text-decoration: line-through; color: var(--text-secondary); }
@media (max-width: 720px) { .tasks-workspace__header, .tasks-workspace__task-header { flex-direction: column; } .tasks-workspace__controls { width: 100%; } }
</style>
