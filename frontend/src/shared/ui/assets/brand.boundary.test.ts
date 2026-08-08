import { describe, expect, it } from 'vitest'
import { makoshBrandAssets, makoshShellBackgroundAssetPaths } from './brand'

describe('Макошь UI local asset inventory', () => {
	it('keeps shell assets compiled and fixed to local public paths', () => {
		expect(makoshBrandAssets.logoMarkDark).toBe('/assets/makosh-logo-mark-dark.png')
		expect(makoshBrandAssets.logoMarkLight).toBe('/assets/makosh-logo-mark-light.png')
		expect(makoshShellBackgroundAssetPaths).toHaveLength(10)
		for (const assetPath of makoshShellBackgroundAssetPaths) {
			expect(assetPath).toMatch(/^\/assets\/shell-backgrounds\/[a-z-]+\.webp$/)
		}
	})
})
