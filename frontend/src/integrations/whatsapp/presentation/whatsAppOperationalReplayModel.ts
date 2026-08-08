import type { WhatsAppOperationalReplayFrameV1 } from '../../../gen/makosh/whatsapp/operational/realtime/v1/client_pb'
import type { WhatsAppOperationalAccount } from '../queries/whatsAppOperationalAccounts'
import {
	providerEventLabel,
	type WhatsAppAccountOption,
	type WhatsAppOperationalReadState,
} from './whatsAppOperationalReadModel'

export type WhatsAppOperationalReplayModel = {
	canReplay: boolean
	state: WhatsAppOperationalReadState
	statusMessage: string
	accounts: readonly WhatsAppAccountOption[]
	selectedAccountId: string
	earliestSequence: string
	latestSequence: string
	nextSequence: string
	resetRequired: boolean
	frames: readonly WhatsAppReplayRow[]
	hasMore: boolean
}

export type WhatsAppReplayRow = {
	sequence: string
	kind: string
}

export function buildWhatsAppOperationalReplayModel(input: {
	canReplay: boolean
	state: WhatsAppOperationalReadState
	statusMessage: string
	accounts: readonly WhatsAppOperationalAccount[]
	selectedAccountId: string
	earliestSequence: bigint | undefined
	latestSequence: bigint | undefined
	nextSequence: bigint
	resetRequired: boolean
	frames: readonly WhatsAppOperationalReplayFrameV1[]
}): WhatsAppOperationalReplayModel {
	return {
		canReplay: input.canReplay,
		state: input.state,
		statusMessage: input.statusMessage,
		accounts: input.accounts.map(({ accountId }) => ({ id: accountId, label: accountId })),
		selectedAccountId: input.selectedAccountId,
		earliestSequence: optionalSequence(input.earliestSequence),
		latestSequence: optionalSequence(input.latestSequence),
		nextSequence: `${input.nextSequence}`,
		resetRequired: input.resetRequired,
		frames: input.frames.map((frame) => ({
			sequence: `${frame.sequence}`,
			kind: frame.event ? providerEventLabel(frame.event) : 'Missing event',
		})),
		hasMore: !input.resetRequired
			&& input.latestSequence !== undefined
			&& input.nextSequence < input.latestSequence,
	}
}

function optionalSequence(value: bigint | undefined): string {
	return value === undefined ? 'Not available' : `${value}`
}
