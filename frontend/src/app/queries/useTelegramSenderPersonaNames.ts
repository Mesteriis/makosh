import { shallowRef } from 'vue'
import type {
	PersonDirectoryEntryV1,
	PersonSourceLinkV1,
} from '../../gen/makosh/persons/v1/persons_pb'
import { getPersonsQueryClient } from '../../platform/connect/personsClient'
import { providerSourceIdentityKey } from '../../shared/identity/providerSourceIdentity'

const PAGE_SIZE = 200
const PROFILE_BATCH_SIZE = 16

export function useTelegramSenderPersonaNames() {
	const names = shallowRef<ReadonlyMap<string, string>>(new Map())
	let refreshGeneration = 0

	async function refresh(): Promise<void> {
		const generation = ++refreshGeneration
		const client = getPersonsQueryClient()
		const persons = await loadAllPersons(client)
		const nextNames = new Map<string, string>()

		for (let offset = 0; offset < persons.length; offset += PROFILE_BATCH_SIZE) {
			const batch = persons.slice(offset, offset + PROFILE_BATCH_SIZE)
			const profiles = await Promise.all(batch.map(async (person) => ({
				person,
				profile: await client.getProfile({ logicalOwnerId: '', personId: person.personId }),
				sourceLinks: await loadAllSourceLinks(client, person.personId),
			})))
			if (generation !== refreshGeneration) return

			for (const { person, profile, sourceLinks } of profiles) {
				const displayName = profile.ownerProfile?.displayName?.trim()
					|| person.displayName?.trim()
				if (!displayName) continue
				for (const sourceLink of sourceLinks) {
					const key = providerSourceIdentityKey(sourceLink.source)
					if (key) nextNames.set(key, displayName)
				}
			}
		}

		if (generation === refreshGeneration) names.value = nextNames
	}

	return { names, refresh }
}

type PersonsQueryClient = ReturnType<typeof getPersonsQueryClient>

async function loadAllPersons(client: PersonsQueryClient): Promise<readonly PersonDirectoryEntryV1[]> {
	const persons: PersonDirectoryEntryV1[] = []
	let afterPersonId = new Uint8Array()
	let previousCursor = ''
	while (true) {
		const page = await client.listDirectory({ logicalOwnerId: '', afterPersonId, limit: PAGE_SIZE })
		persons.push(...page.persons)
		const nextCursor = bytesCursor(page.nextAfterPersonId)
		if (!nextCursor || nextCursor === previousCursor) return persons
		previousCursor = nextCursor
		afterPersonId = new Uint8Array(page.nextAfterPersonId)
	}
}

async function loadAllSourceLinks(
	client: PersonsQueryClient,
	personId: Uint8Array,
): Promise<readonly PersonSourceLinkV1[]> {
	const sourceLinks: PersonSourceLinkV1[] = []
	let afterSourceLinkId = new Uint8Array()
	let previousCursor = ''
	while (true) {
		const page = await client.listSourceLinks({
			logicalOwnerId: '',
			personId,
			afterSourceLinkId,
			limit: PAGE_SIZE,
		})
		sourceLinks.push(...page.sourceLinks)
		const nextCursor = bytesCursor(page.nextAfterSourceLinkId)
		if (!nextCursor || nextCursor === previousCursor) return sourceLinks
		previousCursor = nextCursor
		afterSourceLinkId = new Uint8Array(page.nextAfterSourceLinkId)
	}
}

function bytesCursor(value: Uint8Array): string {
	return Array.from(value, byte => byte.toString(16).padStart(2, '0')).join('')
}
