import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('OrganizationsPage boundary', () => {
	it('renders only the typed owner Organizations product boundary', () => {
		const viewSource = readFileSync(new URL('./OrganizationsWorkspaceView.vue', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useOrganizationsPageSurface.ts', import.meta.url), 'utf8')
		const storeSource = readFileSync(new URL('../stores/organizations.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./OrganizationsPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/OrganizationsList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/OrganizationsDetail.vue', import.meta.url))).toBe(false)

		expect(viewSource).toContain('OWNER ORGANIZATIONS')
		expect(viewSource).toContain('Public source provenance')
		expect(viewSource).not.toContain('Relationships')
		expect(viewSource).not.toContain('Review')
		expect(surfaceSource).toContain('useOrganizationsStore')
		expect(surfaceSource).toContain('selectedOrganization')
		expect(surfaceSource).toContain('sources')
		expect(storeSource).toContain('getOrganizationsQueryClient')
		expect(storeSource).toContain('getOrganizationsCommandClient')
		expect(storeSource).not.toContain('/api/v1/organizations')
	})
})
