import {
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	type ClientSystemComponentStatusV1,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
export {
	publicModuleSettingRows,
	type PublicModuleSettingRow,
} from '../gateway/publicModuleSettings'

type SystemComponentDefinition = {
	id: ClientSystemComponentIdV1
	label: string
	icon: string
}

export type SystemControlComponentRow = SystemComponentDefinition & {
	state: ClientSystemComponentStateV1
	stateLabel: string
	reasonCode: string
	disabled: boolean
}

export const schedulerComponents: readonly SystemComponentDefinition[] = [
	{ id: ClientSystemComponentIdV1.SCHEDULER, label: 'Scheduler runtime', icon: 'tabler:calendar-time' },
	{ id: ClientSystemComponentIdV1.CLOCK, label: 'Clock', icon: 'tabler:clock' },
	{ id: ClientSystemComponentIdV1.STORAGE_CONTROL, label: 'Storage Control', icon: 'tabler:database-cog' },
	{ id: ClientSystemComponentIdV1.POSTGRESQL, label: 'PostgreSQL', icon: 'tabler:database' },
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub', icon: 'tabler:route' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS', icon: 'tabler:arrows-exchange' },
]

export const eventComponents: readonly SystemComponentDefinition[] = [
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub', icon: 'tabler:route' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS', icon: 'tabler:arrows-exchange' },
	{ id: ClientSystemComponentIdV1.TELEMETRY, label: 'Telemetry', icon: 'tabler:chart-dots' },
	{ id: ClientSystemComponentIdV1.SSE, label: 'Client SSE', icon: 'tabler:activity-heartbeat' },
]

export const architectureComponents: readonly SystemComponentDefinition[] = [
	{ id: ClientSystemComponentIdV1.KERNEL, label: 'Kernel', icon: 'tabler:server' },
	{ id: ClientSystemComponentIdV1.CONTROL_STORE, label: 'Control Store', icon: 'tabler:shield-lock' },
	{ id: ClientSystemComponentIdV1.MODULE_CONTROL_PLANE, label: 'Module Control Plane', icon: 'tabler:package' },
	{ id: ClientSystemComponentIdV1.GATEWAY, label: 'Gateway', icon: 'tabler:plug-connected' },
	{ id: ClientSystemComponentIdV1.VAULT, label: 'Vault', icon: 'tabler:key' },
	{ id: ClientSystemComponentIdV1.STORAGE_CONTROL, label: 'Storage Control', icon: 'tabler:database-cog' },
	{ id: ClientSystemComponentIdV1.POSTGRESQL, label: 'PostgreSQL', icon: 'tabler:database' },
	{ id: ClientSystemComponentIdV1.PGBOUNCER, label: 'PgBouncer', icon: 'tabler:route-alt-left' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS', icon: 'tabler:arrows-exchange' },
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub', icon: 'tabler:route' },
	{ id: ClientSystemComponentIdV1.SCHEDULER, label: 'Scheduler', icon: 'tabler:calendar-time' },
	{ id: ClientSystemComponentIdV1.CLOCK, label: 'Clock', icon: 'tabler:clock' },
	{ id: ClientSystemComponentIdV1.BLOB, label: 'Blob', icon: 'tabler:database-export' },
	{ id: ClientSystemComponentIdV1.TELEMETRY, label: 'Telemetry', icon: 'tabler:chart-dots' },
	{ id: ClientSystemComponentIdV1.SSE, label: 'Client SSE', icon: 'tabler:activity-heartbeat' },
]

export function systemControlComponentRows(
	definitions: readonly SystemComponentDefinition[],
	statuses: readonly ClientSystemComponentStatusV1[],
): readonly SystemControlComponentRow[] {
	const statusByComponent = new Map(statuses.map((status) => [status.componentId, status]))
	return definitions.map((definition) => {
		const status = statusByComponent.get(definition.id)
		const state = status?.state ?? ClientSystemComponentStateV1.UNAVAILABLE
		return {
			...definition,
			state,
			stateLabel: systemComponentStateLabel(state),
			reasonCode: status?.sanitizedReasonCode || 'status_unavailable',
			disabled: state === ClientSystemComponentStateV1.UNAVAILABLE
				|| state === ClientSystemComponentStateV1.NOT_ADMITTED,
		}
	})
}

function systemComponentStateLabel(state: ClientSystemComponentStateV1): string {
	if (state === ClientSystemComponentStateV1.HEALTHY) return 'Healthy'
	if (state === ClientSystemComponentStateV1.DEGRADED) return 'Degraded'
	if (state === ClientSystemComponentStateV1.NOT_ADMITTED) return 'Not admitted'
	return 'Unavailable'
}
