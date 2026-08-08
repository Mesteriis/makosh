import type { ZulipOperationalReplayFrameV1 } from '../../../gen/makosh/zulip/operational/realtime/v1/client_pb'
import type { ZulipOperationalAccount } from '../queries/zulipOperationalAccounts'
import {
	eventKindLabel,
	type ZulipAccountOption,
	type ZulipOperationalReadState,
} from './zulipOperationalReadModel'

export type ZulipOperationalReplayModel = {
	canReplay: boolean
	state: ZulipOperationalReadState
	statusMessage: string
	accounts: readonly ZulipAccountOption[]
	selectedAccountId: string
	earliestSequence: string
	latestSequence: string
	nextSequence: string
	resetRequired: boolean
	frames: readonly ZulipReplayRow[]
	hasMore: boolean
}

export type ZulipReplayRow = {
	sequence: string
	kind: string
	messageId: string
}

export function buildZulipOperationalReplayModel(input: {
	canReplay: boolean
	state: ZulipOperationalReadState
	statusMessage: string
	accounts: readonly ZulipOperationalAccount[]
	selectedAccountId: string
	earliestSequence: bigint | undefined
	latestSequence: bigint | undefined
	nextSequence: bigint
	resetRequired: boolean
	frames: readonly ZulipOperationalReplayFrameV1[]
}): ZulipOperationalReplayModel {
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
			kind: frame.event ? eventKindLabel(frame.event.kind) : 'Missing event',
			messageId: frame.event?.providerMessageId ?? 'Not available',
		})),
		hasMore: !input.resetRequired
			&& input.latestSequence !== undefined
			&& input.nextSequence < input.latestSequence,
	}
}

function optionalSequence(value: bigint | undefined): string {
	return value === undefined ? 'Not available' : `${value}`
}
