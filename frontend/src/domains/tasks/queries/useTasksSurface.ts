import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/tasks/queries/useTasksPageSurface.ts'

export function useTasksSurface() {
  return createDomainSurface({
    surfaceId: 'tasks',
    labelKey: 'Tasks',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'tasks-worklist',
        labelKey: 'Task worklist',
        descriptionKey: 'Durable owner tasks, state transitions and due context.',
        icon: 'tabler:checkbox',
        status: 'active',
        kind: 'query',
        contract: 'useTasksPageSurface.tasks'
      },
      {
        id: 'tasks-checklist',
        labelKey: 'Task checklist',
        descriptionKey: 'Owner-managed checklist items and task dependencies.',
        icon: 'tabler:list-check',
        status: 'active',
        kind: 'command',
        contract: 'useTasksPageSurface.checklist'
      }
    ],
    childSurfaces: [
      {
        id: 'tasks-list',
        labelKey: 'Tasks',
        status: 'facade',
        surfacePath,
        capabilityIds: ['tasks-worklist']
      },
      {
        id: 'tasks-checklist',
        labelKey: 'Checklist',
        status: 'facade',
        surfacePath,
        capabilityIds: ['tasks-checklist']
      }
    ]
  })
}
