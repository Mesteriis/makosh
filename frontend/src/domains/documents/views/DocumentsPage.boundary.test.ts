import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('DocumentsPage boundary', () => {
  it('renders only the typed owner Documents product boundary', () => {
    const viewSource = readFileSync(new URL('./DocumentsWorkspaceView.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useDocumentsPageSurface.ts', import.meta.url), 'utf8')
    const storeSource = readFileSync(new URL('../stores/documents.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('../api/documents.ts', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../queries/useDocumentsQuery.ts', import.meta.url))).toBe(false)
    expect(viewSource).toContain('OWNER DOCUMENTS')
    expect(viewSource).toContain('Public source provenance')
    expect(viewSource).toContain('custodyLabel')
    expect(surfaceSource).toContain('useDocumentsStore')
    expect(surfaceSource).toContain('selectedDocument')
    expect(surfaceSource).toContain('sources')
    expect(storeSource).toContain('getDocumentsQueryClient')
    expect(storeSource).toContain('getDocumentsCommandClient')
    expect(storeSource).not.toContain('/api/v1/documents')
    expect(storeSource).not.toContain('document-processing')
    expect(storeSource).not.toContain('custodyTransferSourceProof')
  })
})
