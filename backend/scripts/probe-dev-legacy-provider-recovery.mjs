#!/usr/bin/env node

import { lstatSync, readFileSync } from 'node:fs'
import { isAbsolute } from 'node:path'

const proofFile = process.argv[2]
if (!proofFile || !isAbsolute(proofFile)) process.exit(1)

try {
	const metadata = lstatSync(proofFile)
	if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o077) !== 0) {
		process.exit(1)
	}
	const proof = readFileSync(proofFile, 'utf8')
	if (!/^[0-9a-fA-F]{64}$/.test(proof)) process.exit(1)
	const headers = {
		'content-type': 'application/json',
		'origin': 'http://127.0.0.1:5173',
		'x-makosh-development-host-proof': proof,
	}
	const startedResponse = await fetch(
		'http://127.0.0.1:9445/__makosh/legacy-provider-recovery/v1/start',
		{
			method: 'POST',
			headers,
			body: '{}',
			signal: AbortSignal.timeout(2_000),
		},
	)
	if (!startedResponse.ok) process.exit(1)
	const started = await startedResponse.json()
	if (started?.schemaRevision !== 1
		|| typeof started.recoverySessionId !== 'string'
		|| !/^[0-9a-f]{32}$/.test(started.recoverySessionId)
		|| typeof started.bundleFingerprintSha256 !== 'string'
		|| !/^[0-9a-f]{64}$/.test(started.bundleFingerprintSha256)
		|| started.counts?.gmailActive !== 1
		|| started.counts?.icloudActive !== 1
		|| started.counts?.telegramUserActive !== 1
		|| started.counts?.gmailDeleted !== 2
		|| !Array.isArray(started.candidates)
		|| started.candidates.length !== 3) {
		process.exit(1)
	}
	const cancelResponse = await fetch(
		'http://127.0.0.1:9445/__makosh/legacy-provider-recovery/v1/cancel',
		{
			method: 'POST',
			headers,
			body: JSON.stringify({ recoverySessionId: started.recoverySessionId }),
			signal: AbortSignal.timeout(2_000),
		},
	)
	if (!cancelResponse.ok) process.exit(1)
} catch {
	process.exit(1)
}
