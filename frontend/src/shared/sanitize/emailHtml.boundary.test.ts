import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('emailHtml remote image privacy boundary', () => {
  it('exposes pure helpers for blocking and proxying remote images', () => {
    const source = readFileSync(new URL('./emailHtml.ts', import.meta.url), 'utf8')

    expect(source).toContain('remoteImageUrlsFromHtml')
    expect(source).toContain('rewriteRemoteImageSources')
    expect(source).toContain('data-makosh-remote-src')
    expect(source).toContain('isRemoteImageUrl')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
