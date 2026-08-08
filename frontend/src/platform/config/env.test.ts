import { describe, expect, it } from 'vitest'
import { loadFrontendConfig } from './env'

describe('frontend env config', () => {
	it('uses Макошь env names and default backend URL', () => {
		const config = loadFrontendConfig({
			VITE_MAKOSH_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:8080')
		expect(config.apiSecret).toBe('dev-secret')
		expect(config.sseUrl).toBe('http://127.0.0.1:8080/api/realtime/v2/events')
	})

	it('falls back to the shared local development secret when env is missing', () => {
		const config = loadFrontendConfig({})

		expect(config.apiSecret).toBe('change-me-local-api-secret')
	})

	it('accepts explicit Макошь backend URL', () => {
		const config = loadFrontendConfig({
			VITE_MAKOSH_API_BASE_URL: 'http://127.0.0.1:9090/',
			VITE_MAKOSH_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:9090')
		expect(config.sseUrl).toBe('http://127.0.0.1:9090/api/realtime/v2/events')
	})
})
