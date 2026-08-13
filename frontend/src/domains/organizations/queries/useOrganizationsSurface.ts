import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/organizations/queries/useOrganizationsPageSurface.ts'

export function useOrganizationsSurface() {
  return createDomainSurface({
    surfaceId: 'organizations',
    labelKey: 'Organizations',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'organizations-directory',
        labelKey: 'Organization directory',
        descriptionKey: 'Canonical owner-local organizations and public source provenance.',
        icon: 'tabler:building',
        status: 'active',
        kind: 'query',
        contract: 'useOrganizationsPageSurface.organizations'
      },
      {
        id: 'organizations-lifecycle',
        labelKey: 'Organization lifecycle',
        descriptionKey: 'Typed owner organization creation, profile updates and lifecycle state.',
        icon: 'tabler:building-plus',
        status: 'active',
        kind: 'command',
        contract: 'OrganizationsCommandService.Create/Update/SetState'
      },
      {
        id: 'organizations-provenance',
        labelKey: 'Public source provenance',
        descriptionKey: 'Bounded public source evidence attached to an exact Organization revision.',
        icon: 'tabler:link',
        status: 'active',
        kind: 'command',
        contract: 'OrganizationsCommandService.AddSource/RemoveSource'
      }
    ],
    childSurfaces: [
      {
        id: 'organizations-directory',
        labelKey: 'Directory',
        status: 'active',
        surfacePath,
        capabilityIds: ['organizations-directory']
      },
      {
        id: 'organizations-lifecycle',
        labelKey: 'Lifecycle',
        status: 'active',
        surfacePath,
        capabilityIds: ['organizations-lifecycle']
      },
      {
        id: 'organizations-provenance',
        labelKey: 'Sources',
        status: 'active',
        surfacePath,
        capabilityIds: ['organizations-provenance']
      }
    ]
  })
}
