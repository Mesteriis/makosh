import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'
import { setLocale, type Locale } from '@/platform/i18n'
import PersonaDirectoryPanel from '@/domains/personas/components/PersonaDirectoryPanel.vue'
import PersonaNotesPanel from '@/domains/personas/components/PersonaNotesPanel.vue'
import PersonaOverviewPanel from '@/domains/personas/components/PersonaOverviewPanel.vue'
import PersonaProfileHero from '@/domains/personas/components/PersonaProfileHero.vue'
import PersonaRelationshipsPanel from '@/domains/personas/components/PersonaRelationshipsPanel.vue'
import PersonaSectionTabs from '@/domains/personas/components/PersonaSectionTabs.vue'
import PersonaUnavailablePanel from '@/domains/personas/components/PersonaUnavailablePanel.vue'
import '@/domains/personas/components/personaWorkspace.css'
import type {
	EnrichedPersona,
	PersonaDirectoryFilter,
	PersonaIdentity,
	PersonaIdentityCandidate,
	PersonaPanelProfile,
	PersonaWorkspaceSection,
	Relationship
} from '@/domains/personas/types/persona'
import { storybookLocaleFromGlobals } from '../ui/storybook-i18n'

const meta = {
	title: 'Макошь App/Personas/Components',
	component: PersonaOverviewPanel
} satisfies Meta<typeof PersonaOverviewPanel>

export default meta
type Story = StoryObj<typeof meta>

const now = '2026-07-09T07:30:00.000Z'
const ownerPersonaId = 'persona:owner'

const personas: readonly EnrichedPersona[] = [
	persona({
		id: 'persona:owner',
		name: 'Александр Мещеряков',
		email: 'avm@makosh.local',
		language: 'ru',
		tone: 'direct',
		trust: 96,
		channel: 'mail',
		interactions: 184,
		topics: ['AI Hub', 'почта', 'архитектура'],
		notes: 'Владелец workspace. Основной язык используется mail intelligence.',
		addressBook: true
	}),
	persona({
		id: 'persona:maya',
		name: 'Maya Chen',
		email: 'maya@northwind.example',
		language: 'en',
		tone: 'precise',
		trust: 88,
		channel: 'mail',
		interactions: 42,
		topics: ['vendor review', 'security', 'contracts'],
		notes: 'Often sends security review follow-ups with attachments.',
		addressBook: true
	}),
	persona({
		id: 'persona:alexey',
		name: 'Алексей Волков',
		email: null,
		language: 'ru',
		tone: 'short',
		trust: 78,
		channel: 'telegram',
		interactions: 31,
		topics: ['infra', 'deploy', 'logs'],
		notes: 'Operational persona. Usually linked to incident and deployment threads.',
		addressBook: false
	}),
	persona({
		id: 'persona:brand',
		name: 'Warp Updates',
		email: 'olivia@warp.dev',
		language: 'en',
		tone: 'marketing',
		trust: 54,
		channel: 'mail',
		interactions: 12,
		topics: ['newsletter', 'developer tools'],
		notes: 'Newsletter sender: candidate organization/persona may be promoted after review.',
		addressBook: false
	})
]

const ownerProfile = panelProfile(personas[0], ownerPersonaId)
const selectedProfile = panelProfile(personas[1], ownerPersonaId)

const identityCandidates: readonly PersonaIdentityCandidate[] = [
	{
		identity_candidate_id: 'identity-candidate:maya-alt-email',
		candidate_kind: 'attach_email_address',
		left_persona_id: 'persona:maya',
		right_persona_id: null,
		email_address: 'm.chen@northwind.example',
		evidence_summary: 'Seen in the same vendor review thread and signed as Maya Chen.',
		confidence: 0.86,
		review_state: 'suggested',
		generated_at: now,
		reviewed_at: null,
		updated_at: now
	},
	{
		identity_candidate_id: 'identity-candidate:maya-duplicate',
		candidate_kind: 'merge_personas',
		left_persona_id: 'persona:maya',
		right_persona_id: 'persona:maya-alt',
		email_address: null,
		evidence_summary: 'Same phone number appears in mail signature and imported address book entries.',
		confidence: 0.74,
		review_state: 'suggested',
		generated_at: now,
		reviewed_at: null,
		updated_at: now
	}
]

const identityTraces: readonly PersonaIdentity[] = [
	identityTrace('identity:phone:owner', 'phone', '+34 600 000 123', 'icloud-address-book', 0.92),
	identityTrace('identity:telegram:maya', 'telegram', '@maya_vendor', 'telegram-profile', 0.81),
	identityTrace('identity:email:brand', 'email', 'team@taskade.example', 'mail.ai.extraction', 0.68)
]

const relationships: readonly Relationship[] = [
	relationship('relationship:owner-maya', 'persona:owner', 'persona:maya', 'vendor_security_partner'),
	relationship('relationship:maya-brand', 'persona:maya', 'persona:brand', 'newsletter_sender')
]
const entityLabels = Object.fromEntries(
	personas.map((personaValue) => [personaValue.persona_id, personaValue.display_name])
)

export const DirectoryPanel: Story = {
	render: (_args, context) => ({
		components: { PersonaDirectoryPanel },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const searchQuery = ref('')
			const directoryFilter = ref<PersonaDirectoryFilter>('all')
			const filteredPersonas = computed(() => {
				const query = searchQuery.value.trim().toLowerCase()
				return personas.filter((personaValue) =>
					(directoryFilter.value !== 'address_book' || personaValue.is_address_book) &&
					(!query ||
						[
							personaValue.display_name,
							personaValue.email_address,
							personaValue.language,
							personaValue.tone,
							personaValue.preferred_channel
						].some((value) => value?.toLowerCase().includes(query)))
				)
			})
			return {
				directoryFilter,
				filteredPersonas,
				ownerProfile,
				personas,
				searchQuery,
				selectedProfile
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="personas-workspace-view">
					<div class="personas-workbench">
						<PersonaDirectoryPanel
							:owner-persona="ownerProfile"
							:selected-persona="selectedProfile"
							:filtered-personas="filteredPersonas"
							:directory-count="personas.length"
							:search-query="searchQuery"
							:directory-filter="directoryFilter"
							@update:directory-filter="directoryFilter = $event"
							@update:search-query="searchQuery = $event"
						/>
					</div>
				</div>
			</section>
		`
	})
}

export const ProfileHero: Story = {
	render: (_args, context) => ({
		components: { PersonaProfileHero },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { relationships, selectedProfile }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaProfileHero
					:selected-persona="selectedProfile"
					:relationship-count="relationships.length"
				/>
			</section>
		`
	})
}

export const SectionTabs: Story = {
	render: (_args, context) => ({
		components: { PersonaSectionTabs },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const activeSection = ref<PersonaWorkspaceSection>('overview')
			return { activeSection }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaSectionTabs
					:active-section="activeSection"
					@update:active-section="activeSection = $event"
				/>
			</section>
		`
	})
}

export const OverviewPanel: Story = {
	render: (_args, context) => ({
		components: { PersonaOverviewPanel },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const candidates = ref([...identityCandidates])
			const traces = ref([...identityTraces])
			const selected = ref(selectedProfile)
			function toggleAddressBook(personaValue: PersonaPanelProfile, value: boolean): void {
				selected.value = { ...personaValue, is_address_book: value }
			}
			return {
				candidates,
				entityLabels,
				ownerProfile,
				relationships,
				selected,
				toggleAddressBook,
				traces
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaOverviewPanel
					:owner-persona="ownerProfile"
					:selected-persona="selected"
					:entity-labels="entityLabels"
					:selected-persona-relationships="relationships"
					:pending-review-count="candidates.length + traces.length"
					:suggested-identity-candidates="candidates"
					:identity-traces="traces"
					@review-candidate="candidates = candidates.filter((item) => item.identity_candidate_id !== $event.identity_candidate_id)"
					@assign-trace-to-owner="traces = traces.filter((item) => item.id !== $event.id)"
					@assign-trace-to-selected-persona="traces = traces.filter((item) => item.id !== $event.id)"
					@toggle-address-book="toggleAddressBook"
				/>
			</section>
		`
	})
}

export const RelationshipsPanel: Story = {
	render: (_args, context) => ({
		components: { PersonaRelationshipsPanel },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { entityLabels, relationships, selectedProfile }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaRelationshipsPanel
					:selected-persona="selectedProfile"
					:entity-labels="entityLabels"
					:selected-persona-relationships="relationships"
				/>
			</section>
		`
	})
}

export const NotesPanel: Story = {
	render: (_args, context) => ({
		components: { PersonaNotesPanel },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { selectedProfile }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaNotesPanel :selected-persona="selectedProfile" />
			</section>
		`
	})
}

export const UnavailableSkeletonPanel: Story = {
	render: (_args, context) => ({
		components: { PersonaUnavailablePanel },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const activeSection = ref<PersonaWorkspaceSection>('documents')
			return { activeSection }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide personas-workspace-view">
				<PersonaUnavailablePanel :active-section="activeSection" />
			</section>
		`
	})
}

function syncAppLocaleFromStorybook(globals: Record<string, unknown>): Locale {
	const storybookLocale = storybookLocaleFromGlobals(globals)
	const locale: Locale = storybookLocale === 'ru' ? 'ru' : 'en'
	setLocale(locale)
	return locale
}

function panelProfile(item: EnrichedPersona, currentOwnerPersonaId: string): PersonaPanelProfile {
	return {
		...item,
		is_owner: item.persona_id === currentOwnerPersonaId
	}
}

function persona(input: {
	id: string
	name: string
	email: string | null
	language: string
	tone: string
	trust: number
	channel: string
	interactions: number
	topics: readonly string[]
	notes: string
	addressBook?: boolean
}): EnrichedPersona {
	return {
		persona_id: input.id,
		display_name: input.name,
		email_address: input.email,
		language: input.language,
		tone: input.tone,
		trust_score: input.trust,
		avg_response_hours: 4.5,
		preferred_channel: input.channel,
		last_interaction_at: now,
		interaction_count: input.interactions,
		frequent_topics: [...input.topics],
		writing_style: input.tone,
		persona_metadata: {},
		is_favorite: input.trust > 85,
		is_address_book: input.addressBook ?? false,
		notes: input.notes,
		linked_projects: [],
		linked_documents: [],
		created_at: '2026-06-20T08:00:00.000Z',
		updated_at: now
	}
}

function identityTrace(
	id: string,
	identityType: string,
	value: string,
	source: string,
	confidence: number
): PersonaIdentity {
	return {
		id,
		persona_id: null,
		identity_type: identityType,
		identity_value: value,
		source,
		confidence,
		last_verified_at: null,
		status: 'active',
		metadata: {},
		created_at: now,
		updated_at: now
	}
}

function relationship(
	id: string,
	sourceId: string,
	targetId: string,
	type: string
): Relationship {
	return {
		relationship_id: id,
		source_entity_id: sourceId,
		source_entity_kind: 'persona',
		target_entity_id: targetId,
		target_entity_kind: 'persona',
		relationship_type: type,
		trust_score: 0.84,
		strength_score: 0.72,
		confidence: 0.8,
		review_state: 'suggested'
	}
}
