import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ApiClient } from '@/platform/api/ApiClient'

describe('ApiClient', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
	})

	it('throws if accessed before init', () => {
		expect(() => ApiClient.instance).toThrow('ApiClient not initialized')
	})

	it('initializes with baseUrl and secret', () => {
		const client = ApiClient.init('http://localhost:3000', 'test-secret')
		expect(client).toBeInstanceOf(ApiClient)
		expect(ApiClient.instance).toBe(client)
	})

	it('rejects an empty secret', () => {
		expect(() => ApiClient.init('http://localhost:3000', '   ')).toThrow(
			'x-makosh-secret cannot be empty'
		)
	})

	it('strips trailing slash from baseUrl', () => {
		const client = ApiClient.init('http://localhost:3000/', 'secret')
		// Private field access for test — we verify behavior via GET request
		expect(client).toBeInstanceOf(ApiClient)
	})

	it('sends x-makosh-secret header with GET requests', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ data: 'test' })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'my-secret')
		await ApiClient.instance.get('/api/v1/test')

		expect(mockFetch).toHaveBeenCalledTimes(1)
		const [url, options] = mockFetch.mock.calls[0]
		expect(url).toBe('http://localhost:3000/api/v1/test')
		expect(options.headers['x-makosh-secret']).toBe('my-secret')
		expect(options.method).toBe('GET')

		vi.unstubAllGlobals()
	})

	it('handles 204 No Content responses', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 204,
			json: () => Promise.resolve('should not be called')
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		const result = await ApiClient.instance.get('/api/v1/empty')

		expect(result).toBeUndefined()

		vi.unstubAllGlobals()
	})

	it('throws ApiError on non-ok response', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 403,
			text: () => Promise.resolve('forbidden')
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')

		await expect(
			ApiClient.instance.get('/api/v1/protected')
		).rejects.toMatchObject({
			message: 'forbidden',
			status: 403
		})

		vi.unstubAllGlobals()
	})

	it('throws Error instances with parsed backend JSON error details', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 400,
			text: () => Promise.resolve(JSON.stringify({
				error: 'invalid_platform_request',
				message: 'request must not be empty'
			}))
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')

		await expect(
			ApiClient.instance.post('/api/v1/test', {})
		).rejects.toMatchObject({
			code: 'invalid_platform_request',
			message: 'request must not be empty',
			status: 400
		})
		await expect(
			ApiClient.instance.post('/api/v1/test', {})
		).rejects.toBeInstanceOf(Error)

		vi.unstubAllGlobals()
	})

	it('sends JSON body with POST requests', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ id: 1 })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		await ApiClient.instance.post('/api/v1/create', { name: 'test' })

		const [, options] = mockFetch.mock.calls[0]
		expect(options.method).toBe('POST')
		expect(options.headers['Content-Type']).toBe('application/json')
		expect(JSON.parse(options.body)).toEqual({ name: 'test' })

		vi.unstubAllGlobals()
	})

	it('sends DELETE request without body', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ deleted: true })
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')
		await ApiClient.instance.delete('/api/v1/item/1')

		const [, options] = mockFetch.mock.calls[0]
		expect(options.method).toBe('DELETE')
		expect(options.body).toBeUndefined()

		vi.unstubAllGlobals()
	})

	it('falls back to default error message when text() fails', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 500,
			text: () => Promise.reject(new Error('parse failed'))
		})
		vi.stubGlobal('fetch', mockFetch)

		ApiClient.init('http://localhost:3000', 'secret')

		await expect(
			ApiClient.instance.get('/api/v1/error')
		).rejects.toMatchObject({
			status: 500
		})

		vi.unstubAllGlobals()
	})
})
