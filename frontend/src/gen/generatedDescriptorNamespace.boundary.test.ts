import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { MailAccountCatalogService } from './makosh/mail/account/v1/client_pb'

const frontendRoot = fileURLToPath(new URL('../../', import.meta.url))

function generatedProtoFiles(): string[] {
	const root = join(frontendRoot, 'src/gen')
	return readdirSync(root, { recursive: true, withFileTypes: true })
		.filter((entry) => entry.isFile() && entry.name.endsWith('_pb.ts'))
		.map((entry) => relative(frontendRoot, join(entry.parentPath, entry.name)))
		.sort()
}

describe('generated protobuf descriptor namespace boundary', () => {
	it('derives the canonical Mail account catalog service name', () => {
		expect(MailAccountCatalogService.typeName).toBe(
			'makosh.mail.account.v1.MailAccountCatalogService'
		)
	})

	it('keeps every tracked generated descriptor in the canonical Makosh namespace', () => {
		const generatedFiles = generatedProtoFiles()
		const missingDescriptors: string[] = []
		const legacyDescriptors: string[] = []
		const nonCanonicalDescriptors: string[] = []

		expect(generatedFiles.length).toBeGreaterThan(0)

		for (const relativePath of generatedFiles) {
			const source = readFileSync(join(frontendRoot, relativePath), 'utf8')
			const descriptorPayload = source.match(/fileDesc\(\s*"([^"]+)"/)?.[1]

			if (descriptorPayload === undefined) {
				missingDescriptors.push(relativePath)
				continue
			}

			const decodedDescriptor = Buffer.from(descriptorPayload, 'base64').toString('utf8')
			if (/hermes[/.]/.test(decodedDescriptor)) {
				legacyDescriptors.push(relativePath)
			}
			if (!/makosh[/.]/.test(decodedDescriptor)) {
				nonCanonicalDescriptors.push(relativePath)
			}
		}

		expect(missingDescriptors).toEqual([])
		expect(legacyDescriptors).toEqual([])
		expect(nonCanonicalDescriptors).toEqual([])
	})
})
