import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('PersonasPage boundary', () => {
  it('renders personas UI through the personas surface without direct API calls in the view', () => {
    const viewUrl = new URL('./PersonasWorkspaceView.vue', import.meta.url)
    const componentUrl = new URL('../components/PersonasWorkspace.vue', import.meta.url)
    const presentationalUrls = [
      new URL('../components/PersonaDirectoryPanel.vue', import.meta.url),
      new URL('../components/PersonaNotesPanel.vue', import.meta.url),
      new URL('../components/PersonaOverviewPanel.vue', import.meta.url),
      new URL('../components/PersonaProfileHero.vue', import.meta.url),
      new URL('../components/PersonaRelationshipEdge.vue', import.meta.url),
      new URL('../components/PersonaRelationshipGraph.vue', import.meta.url),
      new URL('../components/PersonaRelationshipsPanel.vue', import.meta.url),
      new URL('../components/PersonaSectionTabs.vue', import.meta.url),
      new URL('../components/PersonaUnavailablePanel.vue', import.meta.url),
      new URL('../components/personaWorkspaceElements.ts', import.meta.url),
      new URL('../components/personaWorkspace.css', import.meta.url)
    ]
    const surfaceUrl = new URL('../queries/usePersonasPageSurface.ts', import.meta.url)
    const queryUrl = new URL('../queries/usePersonasQuery.ts', import.meta.url)
    const apiUrl = new URL('../api/personas.ts', import.meta.url)
    const typeUrl = new URL('../types/persona.ts', import.meta.url)
    const storyUrl = new URL('../../../../stories/app/Personas.stories.ts', import.meta.url)

    expect(existsSync(viewUrl)).toBe(true)
    expect(existsSync(componentUrl)).toBe(true)
    for (const presentationalUrl of presentationalUrls) {
      expect(existsSync(presentationalUrl)).toBe(true)
    }

    const viewSource = readFileSync(viewUrl, 'utf8')
    const componentSource = readFileSync(componentUrl, 'utf8')
    const directoryPanelSource = readFileSync(
      new URL('../components/PersonaDirectoryPanel.vue', import.meta.url),
      'utf8'
    )
    const relationshipPanelSource = readFileSync(
      new URL('../components/PersonaRelationshipsPanel.vue', import.meta.url),
      'utf8'
    )
    const presentationalSource = presentationalUrls
      .map((presentationalUrl) => readFileSync(presentationalUrl, 'utf8'))
      .join('\n')
    const surfaceSource = readFileSync(surfaceUrl, 'utf8')
    const querySource = readFileSync(queryUrl, 'utf8')
    const apiSource = readFileSync(apiUrl, 'utf8')
    const typeSource = readFileSync(typeUrl, 'utf8')
    const storySource = readFileSync(storyUrl, 'utf8')

    expect(viewSource).toContain('PersonasPage')
    expect(viewSource).toContain("../presentation/PersonasPage.vue")
    expect(viewSource).toContain("../presentation/personasPageModel")
    expect(viewSource).toContain('usePersonasPageSurface')
    expect(viewSource).toContain('surface.ownerPersona')
    expect(viewSource).toContain('surface.setOwnerPersona')
    expect(viewSource).toContain('surface.setIdentityCandidateReview')
    expect(viewSource).toContain('surface.assignTraceToOwner')
    expect(viewSource).toContain('surface.directoryFilter')
    expect(viewSource).toContain('surface.activeSection')
    expect(viewSource).toContain('surface.toggleAddressBookMembership')
    expect(viewSource).toContain('const model = computed<PersonasPageModel>')
    expect(viewSource).toContain('const actions: PersonasPageActions')
    expect(viewSource).toContain('<PersonasPage :model="model" :actions="actions"')
    expect(viewSource).not.toContain('ApiClient')
    expect(viewSource).not.toContain('fetch(')

    expect(componentSource).toContain('defineProps')
    expect(componentSource).toContain('defineEmits')
    expect(componentSource).toContain('ownerPersona: PersonaPanelProfile | null')
    expect(componentSource).toContain('PersonaDirectoryPanel')
    expect(componentSource).toContain('PersonaProfileHero')
    expect(componentSource).toContain('PersonaSectionTabs')
    expect(componentSource).toContain('PersonaOverviewPanel')
    expect(componentSource).toContain('PersonaUnavailablePanel')
    expect(componentSource).toContain('reviewCandidate')
    expect(componentSource).toContain('toggleAddressBook')
    expect(componentSource).toContain('update:directoryFilter')
    expect(componentSource).not.toContain('personas-header')
    expect(componentSource).not.toContain('personas-metrics')
    expect(componentSource).not.toContain('personas-owner-panel')
    expect(componentSource).not.toContain('usePersonasPageSurface')
    expect(componentSource).not.toContain('ApiClient')
    expect(componentSource).not.toContain('useQuery')

    expect(presentationalSource).toContain('personas-profile-hero')
    expect(presentationalSource).toContain('personas-profile-dashboard')
    expect(presentationalSource).toContain('SkeletonPanel')
    expect(presentationalSource).toContain('PERSONA_WORKSPACE_SECTIONS')
    expect(presentationalSource).toContain('PersonaDirectoryFilter')
    expect(presentationalSource).toContain('personas-directory-filter')
    expect(presentationalSource).toContain('personas-directory-address-book-icon')
    expect(directoryPanelSource).toContain('SearchInput')
    expect(directoryPanelSource).not.toContain('<input')
    expect(presentationalSource).toContain('@vue-flow/core')
    expect(presentationalSource).toContain('personas-relationship-flow')
    expect(presentationalSource).toContain('personas-relationship-avatar-node')
    expect(presentationalSource).toContain('PersonaRelationshipEdge')
    expect(presentationalSource).toContain('personas-relationship-edge-action')
    expect(presentationalSource).toContain('personas-graph-controls')
    expect(presentationalSource).toContain('@edge-click')
    expect(presentationalSource).not.toContain('personas-relationship-list')
    expect(relationshipPanelSource).not.toContain('tabler:chart-dots')
    expect(relationshipPanelSource).not.toContain("<h3>{{ t('Relationship graph') }}</h3>")
    expect(presentationalSource).toContain('Add to contacts')
    expect(presentationalSource).not.toContain('merge_persons')
    expect(presentationalSource).not.toContain('split_person:')
    expect(presentationalSource).not.toContain('personas-header')
    expect(presentationalSource).not.toContain('personas-metrics')
    expect(presentationalSource).not.toContain('personas-owner-panel')
    expect(presentationalSource).not.toContain('usePersonasPageSurface')
    expect(presentationalSource).not.toContain('ApiClient')
    expect(presentationalSource).not.toContain('useQuery')
    expect(presentationalSource).not.toContain('fetch(')

    expect(surfaceSource).toContain('useOwnerPersonaQuery')
    expect(surfaceSource).toContain('useSetOwnerPersonaMutation')
    expect(surfaceSource).toContain('useUpdatePersonaAddressBookMembershipMutation')
    expect(surfaceSource).toContain('useReviewIdentityCandidateMutation')
    expect(surfaceSource).toContain('useAssignIdentityTraceMutation')
    expect(surfaceSource).toContain('ownerProfile')
    expect(surfaceSource).toContain('directoryFilter')
    expect(surfaceSource).toContain("activeSection = ref<PersonaWorkspaceSection>('overview')")
    expect(surfaceSource).toContain('toggleAddressBookMembership')
    expect(surfaceSource).toContain('is_address_book')
    expect(surfaceSource).toContain('suggestedIdentityCandidates')
    expect(surfaceSource).toContain('identityTraces')
    expect(surfaceSource).not.toContain('merge_persons')
    expect(surfaceSource).not.toContain('split_person:')
    expect(surfaceSource).not.toContain("from '../api/personas'")

    expect(querySource).toContain('personasQueryKeys')
    expect(querySource).toContain('useOwnerPersonaQuery')
    expect(querySource).toContain('invalidateQueries')
    expect(querySource).toContain('is_address_book')
    expect(apiSource).toContain('/api/v1/personas/owner')
    expect(apiSource).not.toContain('/api/v1/persons')
    expect(apiSource).not.toContain('person_id: personaId')
    expect(apiSource).toContain('/api/v1/personas/${encodeURIComponent(personaId)}/address-book')
    expect(apiSource).toContain('/api/v1/identity-traces/')
    expect(apiSource).toContain('/assignment')
    expect(apiSource).not.toContain('normalizeIdentityCandidate')
    expect(apiSource).not.toContain('normalizeIdentityTrace')
    expect(apiSource).not.toContain('RawPersonaIdentity')
    expect(typeSource).toContain('export interface PersonaProfile')
    expect(typeSource).toContain('export interface OwnerPersona')
    expect(typeSource).toContain('left_persona_id: string')
    expect(typeSource).toContain('right_persona_id: string | null')
    expect(typeSource).toContain('persona_id: string | null')
    expect(typeSource).not.toContain('left_person_id: string')
    expect(typeSource).not.toContain('right_person_id: string | null')
    expect(typeSource).not.toContain('person_id: string | null')
    expect(typeSource).not.toContain('Deprecated compatibility alias for older Personas API payloads')
    expect(storySource).not.toContain('person_id: input.id')
    expect(storySource).not.toContain('person:')
    expect(storySource).not.toContain('left_person_id')
    expect(storySource).not.toContain('right_person_id')
    expect(storySource).not.toContain('person_id: null')
    expect(storySource).toContain('PersonasWorkspaceComponent')
    expect(storySource).toContain("title: 'Макошь App/Personas/Workspace'")
    expect(storySource).not.toContain('createDomainScaffoldStory')
  })

  it('keeps legacy Persons frontend paths retired', () => {
    const retiredPaths = [
      '../../../app/queries/usePersonsViewSurface.ts',
      '../queries/usePersonsPageSurface.ts',
      '../queries/usePersonsSurface.ts',
      './PersonsPage.boundary.test.ts',
      '../../../../stories/app/Persons.stories.ts'
    ]

    for (const retiredPath of retiredPaths) {
      expect(existsSync(new URL(retiredPath, import.meta.url)), retiredPath).toBe(false)
    }
  })
})
