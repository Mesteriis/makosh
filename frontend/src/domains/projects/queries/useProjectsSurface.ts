import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/projects/queries/useProjectsPageSurface.ts'

export function useProjectsSurface() {
  return createDomainSurface({
    surfaceId: 'projects',
    labelKey: 'Projects',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'projects-lifecycle',
        labelKey: 'Project lifecycle',
        descriptionKey: 'Owner projects with bounded planning, active, hold, completion and archive states.',
        icon: 'tabler:briefcase',
        status: 'active',
        kind: 'command',
        contract: 'projects.client.v1'
      },
      {
        id: 'projects-outcomes',
        labelKey: 'Expected outcomes',
        descriptionKey: 'Revision-bound expected outcomes with explicit terminal states.',
        icon: 'tabler:target-arrow',
        status: 'active',
        kind: 'command',
        contract: 'projects.client.v1'
      },
      {
        id: 'projects-references',
        labelKey: 'Public references',
        descriptionKey: 'Typed public references without copied foreign records.',
        icon: 'tabler:link',
        status: 'active',
        kind: 'query',
        contract: 'projects.client.v1'
      }
    ],
    childSurfaces: [
      { id: 'projects-overview', labelKey: 'Overview', status: 'active', surfacePath, capabilityIds: ['projects-lifecycle'] },
      { id: 'projects-outcomes', labelKey: 'Expected outcomes', status: 'active', surfacePath, capabilityIds: ['projects-outcomes'] },
      { id: 'projects-references', labelKey: 'Public references', status: 'active', surfacePath, capabilityIds: ['projects-references'] }
    ]
  })
}
