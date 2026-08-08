import { describe, expect, it } from 'vitest'

import {
	automationDigestHex,
	parseAutomationIdentifiers,
	parseAutomationVariables,
} from './telegramAutomationModel'

describe('Telegram automation presentation model', () => {
	it('parses exact unique identifiers and typed variable lines', () => {
		expect(parseAutomationIdentifiers('chat-1, chat-2\nchat-3')).toEqual([
			'chat-1',
			'chat-2',
			'chat-3',
		])
		expect(parseAutomationVariables('name=Ada\nteam=Макошь')).toEqual([
			{ name: 'name', value: 'Ada' },
			{ name: 'team', value: 'Макошь' },
		])
	})

	it('rejects ambiguous duplicate inputs', () => {
		expect(() => parseAutomationIdentifiers('chat-1 chat-1')).toThrow('must be unique')
		expect(() => parseAutomationVariables('name=Ada\nname=Grace')).toThrow('must be unique')
		expect(() => parseAutomationVariables('name')).toThrow('name=value')
	})

	it('formats the complete provider digest', () => {
		expect(automationDigestHex(new Uint8Array([0, 15, 255]))).toBe('000fff')
	})
})
