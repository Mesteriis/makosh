import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	ClientSettingsApplyStateV1,
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
	ClientSystemComponentStatusV1Schema,
} from '../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	eventComponents,
	publicModuleSettingRows,
	schedulerComponents,
	systemControlComponentRows,
} from './systemControlComponents'

describe('system control component rows', () => {
	it('projects scheduler dependencies from the typed bootstrap status', () => {
		const rows = systemControlComponentRows(schedulerComponents, [status(
			ClientSystemComponentIdV1.SCHEDULER,
			ClientSystemComponentStateV1.DEGRADED,
			'runtime_liveness_not_observed',
		)])

		expect(rows[0]).toMatchObject({
			label: 'Scheduler runtime',
			stateLabel: 'Degraded',
			reasonCode: 'runtime_liveness_not_observed',
			disabled: false,
		})
		expect(rows[1]).toMatchObject({ stateLabel: 'Unavailable', disabled: true })
	})

	it('keeps Event Hub, NATS, and SSE in the events section', () => {
		expect(eventComponents.map(({ id }) => id)).toEqual([
			ClientSystemComponentIdV1.EVENT_HUB,
			ClientSystemComponentIdV1.NATS,
			ClientSystemComponentIdV1.TELEMETRY,
			ClientSystemComponentIdV1.SSE,
		])
	})

	it('projects only typed public module values with their compiled metadata', () => {
		const rows = publicModuleSettingRows([create(ClientModuleBootstrapV1Schema, {
			registrationId: 'scheduler.local',
			moduleId: 'platform.scheduler',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				applyState: ClientSettingsApplyStateV1.CURRENT,
				values: [create(ClientSettingValueEntryV1Schema, {
					settingId: 'tick_interval',
					displayName: 'Tick interval',
					editable: false,
					value: create(ClientSettingValueV1Schema, { value: { case: 'durationMillis', value: 15000n } }),
				})],
			}),
		})])

		expect(rows).toEqual([expect.objectContaining({
			label: 'Tick interval',
			value: '15000 ms',
			editable: false,
			applyState: 'Current',
		})])
	})
})

function status(
	componentId: ClientSystemComponentIdV1,
	state: ClientSystemComponentStateV1,
	sanitizedReasonCode: string,
) {
	return create(ClientSystemComponentStatusV1Schema, { componentId, state, sanitizedReasonCode })
}
