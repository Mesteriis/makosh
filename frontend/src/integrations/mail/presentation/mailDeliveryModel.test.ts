import { describe, expect, it } from 'vitest'

import { MailDeliveryOutcomeV1 } from '../../../gen/makosh/mail/v1/client_pb'
import { buildMailDeliveryStatusCard } from './mailDeliveryModel'

describe('Mail delivery presentation model', () => {
	it('maps the generated delivery outcome and timestamps', () => {
		expect(buildMailDeliveryStatusCard({
			operationId: 'delivery-1',
			connectionId: 'gmail-primary',
			outcome: MailDeliveryOutcomeV1.MAIL_DELIVERY_OUTCOME_ACCEPTED,
			requestedAtUnixSeconds: 1_753_520_400n,
			completedAtUnixSeconds: 1_753_520_401n,
			responseCode: 202,
		} as never)).toMatchObject({
			operationId: 'delivery-1',
			connectionId: 'gmail-primary',
			outcome: 'accepted',
			responseCode: '202',
		})
	})
})
