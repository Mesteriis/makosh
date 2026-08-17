export type TelegramAuthorizationStatus = {
	state: string
	qrLink?: string
	passwordHint?: string
}

import { getTelegramAuthorizationConnectClient } from './telegramAuthorizationClient'
import { withTelegramConfigurationRuntimeV1 } from '../setup/telegramConfigurationRuntimeRetry'

export async function getTelegramAuthorizationStatus(): Promise<TelegramAuthorizationStatus> {
	const response = await withTelegramConfigurationRuntimeV1(() =>
		getTelegramAuthorizationConnectClient().authorize({
			request: { case: 'authorizationStatus', value: {} },
		}),
	)
	if (response.response.case !== 'authorizationStatus') {
		throw new Error('Telegram authorization status is unavailable')
	}
	return {
		state: response.response.value.state,
		qrLink: response.response.value.qrLink,
		passwordHint: response.response.value.passwordHint,
	}
}

export async function submitTelegramAuthorizationPassword(password: string): Promise<void> {
	const normalizedPassword = password.trim()
	if (!normalizedPassword) {
		throw new RangeError('Telegram authorization password is required')
	}
	const response = await getTelegramAuthorizationConnectClient().authorize({
		request: {
			case: 'submitPassword',
			value: { password: normalizedPassword },
		},
	})
	if (response.response.case !== 'passwordAccepted') {
		throw new Error('Telegram authorization password was not accepted')
	}
}
