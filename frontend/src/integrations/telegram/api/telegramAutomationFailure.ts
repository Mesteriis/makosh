import {
	AutomationFailureCodeV1,
	type AutomationFailureV1,
} from '../../../gen/makosh/telegram/automation/v1/automation_pb'

export function telegramAutomationFailure(failure: AutomationFailureV1): Error {
	const field = failure.field ? ` (${failure.field})` : ''
	switch (failure.code) {
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_INVALID_REQUEST:
			return new RangeError(`Telegram automation request is invalid${field}`)
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_NOT_FOUND:
			return new Error(`Telegram automation object was not found${field}`)
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_REVISION_CONFLICT:
			return new Error('Telegram automation revision changed; refresh before saving')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_IDEMPOTENCY_CONFLICT:
			return new Error('Telegram automation operation ID was already used')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_POLICY_DISABLED:
			return new Error('Telegram automation policy is disabled')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_POLICY_EXPIRED:
			return new Error('Telegram automation policy is expired')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_SCOPE_DENIED:
			return new Error('Telegram automation policy does not allow this account and chat')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_VARIABLE_MISSING:
			return new Error('Telegram automation preview is missing a required variable')
		case AutomationFailureCodeV1.AUTOMATION_FAILURE_CODE_VARIABLE_UNDECLARED:
			return new Error('Telegram automation preview contains an undeclared variable')
		default:
			return new Error('Telegram automation is unavailable')
	}
}
