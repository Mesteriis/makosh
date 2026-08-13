import { computed } from 'vue'
import { useTasksStore } from '../stores/tasks'

export function useTasksPageSurface() {
  const store = useTasksStore()

  return {
    tasks: computed(() => store.tasks),
    openTasks: computed(() => store.openTasks),
    completedTasks: computed(() => store.completedTasks),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingTaskId: computed(() => store.mutatingTaskId),
    loadTasks: store.loadAll,
    createTask: store.createTask,
    updateTask: store.updateTask,
    setTaskState: store.setTaskState,
    setTaskPriority: store.setTaskPriority,
    addDependency: store.addDependency,
    removeDependency: store.removeDependency,
    addChecklistItem: store.addChecklistItem,
    updateChecklistItem: store.updateChecklistItem,
    removeChecklistItem: store.removeChecklistItem,
    store
  }
}
