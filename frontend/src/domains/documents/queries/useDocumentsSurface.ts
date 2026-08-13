import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/documents/queries/useDocumentsPageSurface.ts'

export function useDocumentsSurface() {
  return createDomainSurface({
    surfaceId: 'documents',
    labelKey: 'Documents',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'documents-library',
        labelKey: 'Document library',
        descriptionKey: 'Canonical owner-local document metadata and lifecycle state.',
        icon: 'tabler:file-text',
        status: 'active',
        kind: 'query',
        contract: 'useDocumentsPageSurface.documents'
      },
      {
        id: 'documents-custody',
        labelKey: 'Blob custody',
        descriptionKey: 'Sanitized Blob custody state without private references or proofs.',
        icon: 'tabler:lock',
        status: 'active',
        kind: 'query',
        contract: 'useDocumentsPageSurface.selectedDocument'
      },
      {
        id: 'documents-evidence',
        labelKey: 'Evidence',
        descriptionKey: 'Public source provenance without source-private payloads.',
        icon: 'tabler:shield-check',
        status: 'active',
        kind: 'evidence',
        contract: 'useDocumentsPageSurface.sources'
      }
    ],
    childSurfaces: [
      {
        id: 'documents-library',
        labelKey: 'Library',
        status: 'active',
        surfacePath,
        capabilityIds: ['documents-library']
      },
      {
        id: 'documents-custody',
        labelKey: 'Custody',
        status: 'active',
        surfacePath,
        capabilityIds: ['documents-custody']
      },
      {
        id: 'documents-evidence',
        labelKey: 'Evidence',
        status: 'active',
        surfacePath,
        capabilityIds: ['documents-evidence']
      }
    ]
  })
}
