import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'
import { expect, userEvent, within } from 'storybook/test'
import { setLocale, type Locale } from '@/platform/i18n'
import PersonasWorkspaceComponent from '@/domains/personas/components/PersonasWorkspace.vue'
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
	title: 'Макошь App/Personas/Workspace',
	component: PersonasWorkspaceComponent
} satisfies Meta<typeof PersonasWorkspaceComponent>

export default meta
type Story = StoryObj<typeof meta>

const now = '2026-07-09T07:30:00.000Z'

const personaFixtures: readonly EnrichedPersona[] = [
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
		notes: 'Владелец workspace. Основной язык нужен для переводов и mail intelligence.',
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
		id: 'persona:newsletter-brand',
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
	}),
	persona({
		id: 'persona:finance',
		name: 'Nadia Petrova',
		email: 'nadia@finance.example',
		language: 'ru',
		tone: 'formal',
		trust: 82,
		channel: 'mail',
		interactions: 27,
		topics: ['invoices', 'payments', 'reconciliation'],
		notes: 'Finance persona with recurring invoice evidence.',
		addressBook: true
	})
]

const identityCandidateFixtures: readonly PersonaIdentityCandidate[] = [
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
		identity_candidate_id: 'identity-candidate:alexey-duplicate',
		candidate_kind: 'merge_personas',
		left_persona_id: 'persona:alexey',
		right_persona_id: 'persona:alexey-alt',
		email_address: null,
		evidence_summary: 'Same phone number appears in Telegram profile and mail signature.',
		confidence: 0.74,
		review_state: 'suggested',
		generated_at: now,
		reviewed_at: null,
		updated_at: now
	}
]

const identityTraceFixtures: readonly PersonaIdentity[] = [
	identityTrace('identity:phone:owner', 'phone', '+34 600 000 123', 'icloud-address-book', 0.92),
	identityTrace('identity:telegram:alexey', 'telegram', '@volkov_ops', 'telegram-profile', 0.81),
	identityTrace('identity:email:brand', 'email', 'team@taskade.example', 'mail.ai.extraction', 0.68)
]

const relationshipFixtures: readonly Relationship[] = [
	relationship('relationship:owner-maya', 'persona:owner', 'persona:maya', 'vendor_security_partner'),
	relationship('relationship:owner-alexey', 'persona:owner', 'persona:alexey', 'operations_peer')
]

export const Workspace: Story = {
	render: (_args, context) => ({
		components: { PersonasWorkspaceComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)

			const personas = ref([...personaFixtures])
			const candidates = ref([...identityCandidateFixtures])
			const identityTraces = ref([...identityTraceFixtures])
			const ownerPersonaId = ref('persona:owner')
			const selectedPersonaIndex = ref(1)
			const activeSection = ref<PersonaWorkspaceSection>('overview')
			const directoryFilter = ref<PersonaDirectoryFilter>('all')
			const searchQuery = ref('')
			const actionLog = ref<string[]>([])

			const filteredPersonas = computed(() => {
				const query = searchQuery.value.trim().toLowerCase()

				return personas.value.filter((item) =>
					(directoryFilter.value !== 'address_book' || item.is_address_book) &&
					(!query ||
					[item.display_name, item.email_address, item.language, item.tone, item.preferred_channel]
						.some((value) => value?.toLowerCase().includes(query)))
				)
			})
			const ownerPersona = computed(() => {
				const owner = personas.value.find((item) => item.persona_id === ownerPersonaId.value)
				return owner ? panelProfile(owner, ownerPersonaId.value) : null
			})
			const selectedPersona = computed(() => {
				const selected = filteredPersonas.value[selectedPersonaIndex.value] ?? filteredPersonas.value[0]
				return selected ? panelProfile(selected, ownerPersonaId.value) : null
			})
			const selectedPersonaRelationships = computed(() => {
				const selectedId = selectedPersona.value?.persona_id
				if (!selectedId) return []

				return relationshipFixtures.filter((item) =>
					item.source_entity_id === selectedId || item.target_entity_id === selectedId
				)
			})
			const pendingReviewCount = computed(() => candidates.value.length + identityTraces.value.length)

			function selectPersona(index: number): void {
				selectedPersonaIndex.value = index
			}

			function setOwner(personaValue: EnrichedPersona): void {
				ownerPersonaId.value = personaValue.persona_id
				actionLog.value = [`owner:${personaValue.display_name}`, ...actionLog.value].slice(0, 4)
			}

			function reviewCandidate(candidate: PersonaIdentityCandidate, state: 'user_confirmed' | 'user_rejected'): void {
				candidates.value = candidates.value.filter((item) => item.identity_candidate_id !== candidate.identity_candidate_id)
				actionLog.value = [`${state}:${candidate.identity_candidate_id}`, ...actionLog.value].slice(0, 4)
			}

			function assignTraceToOwner(trace: PersonaIdentity): void {
				identityTraces.value = identityTraces.value.filter((item) => item.id !== trace.id)
				actionLog.value = [`owner-trace:${trace.identity_value}`, ...actionLog.value].slice(0, 4)
			}

			function assignTraceToSelectedPersona(trace: PersonaIdentity): void {
				identityTraces.value = identityTraces.value.filter((item) => item.id !== trace.id)
				actionLog.value = [`selected-trace:${trace.identity_value}`, ...actionLog.value].slice(0, 4)
			}

			function refresh(): void {
				actionLog.value = ['refresh', ...actionLog.value].slice(0, 4)
			}

			function toggleAddressBook(personaValue: EnrichedPersona, value: boolean): void {
				personas.value = personas.value.map((item) =>
					item.persona_id === personaValue.persona_id
						? { ...item, is_address_book: value }
						: item
				)
				actionLog.value = [`addressBook:${personaValue.display_name}:${value}`, ...actionLog.value].slice(0, 4)
			}

			return {
				actionLog,
				activeSection,
				assignTraceToOwner,
				assignTraceToSelectedPersona,
				candidates,
				directoryFilter,
				filteredPersonas,
				identityTraces,
				ownerPersona,
				pendingReviewCount,
				personas,
				refresh,
				reviewCandidate,
				searchQuery,
				selectedPersona,
				selectedPersonaRelationships,
				selectPersona,
				setOwner,
				toggleAddressBook
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<PersonasWorkspaceComponent
					:owner-persona="ownerPersona"
					:selected-persona="selectedPersona"
					:filtered-personas="filteredPersonas"
					:directory-count="personas.length"
					:pending-review-count="pendingReviewCount"
					:directory-filter="directoryFilter"
					:active-section="activeSection"
					:search-query="searchQuery"
					:suggested-identity-candidates="candidates"
					:identity-traces="identityTraces"
					:selected-persona-relationships="selectedPersonaRelationships"
					@refresh="refresh"
					@select-persona="selectPersona"
					@set-owner="setOwner"
					@review-candidate="reviewCandidate"
					@assign-trace-to-owner="assignTraceToOwner"
					@assign-trace-to-selected-persona="assignTraceToSelectedPersona"
					@toggle-address-book="toggleAddressBook"
					@update:directory-filter="directoryFilter = $event"
					@update:active-section="activeSection = $event"
					@update:search-query="searchQuery = $event"
				/>
				<p v-if="actionLog.length" class="makosh-sr-only" aria-live="polite">{{ actionLog.join(', ') }}</p>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: /Personas|Персоны|Люди/ })).toBeVisible()
		await expect(canvas.getAllByText(/Maya Chen/)[0]).toBeVisible()
		await expect(canvas.getByText(/AI Summary|ИИ-сводка/)).toBeVisible()

		await userEvent.click(canvas.getByRole('button', { name: /Tasks|Задачи/ }))
		await expect(await canvas.findByText(/Task candidates must arrive|Кандидаты задач должны/)).toBeVisible()

		await userEvent.click(canvas.getByRole('button', { name: /Relationships|Связи/ }))
		await expect(canvas.getAllByRole('button', { name: /Александр Мещеряков/ })[0]).toBeVisible()
		await expect(canvas.getAllByText(/Александр Мещеряков/)[0]).toBeVisible()

		await userEvent.type(canvas.getByRole('searchbox'), 'Алексей')
		await expect(canvas.getAllByText('Алексей Волков')[0]).toBeVisible()
	}
}

export const EmptyReview: Story = {
	render: (_args, context) => ({
		components: { PersonasWorkspaceComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const owner = panelProfile(personaFixtures[0], 'persona:owner')
			const activeSection = ref<PersonaWorkspaceSection>('overview')
			const directoryFilter = ref<PersonaDirectoryFilter>('all')
			return {
				activeSection,
				directoryFilter,
				owner,
				personas: [personaFixtures[0]],
				selectedPersona: owner
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<PersonasWorkspaceComponent
					:owner-persona="owner"
					:selected-persona="selectedPersona"
					:filtered-personas="personas"
					:directory-count="personas.length"
					:pending-review-count="0"
					:directory-filter="directoryFilter"
					:active-section="activeSection"
					search-query=""
					:suggested-identity-candidates="[]"
					:identity-traces="[]"
					:selected-persona-relationships="[]"
					@update:directory-filter="directoryFilter = $event"
					@update:active-section="activeSection = $event"
				/>
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

function panelProfile(item: EnrichedPersona, ownerPersonaId: string): PersonaPanelProfile {
	return {
		...item,
		is_owner: item.persona_id === ownerPersonaId
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
