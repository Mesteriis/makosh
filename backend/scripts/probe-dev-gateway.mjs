#!/usr/bin/env node

import { lstatSync, readFileSync } from 'node:fs'
import { isAbsolute } from 'node:path'
import { request } from 'node:http'

const proofFile = process.argv[2]
if (proofFile === undefined || !isAbsolute(proofFile)) {
	process.exit(2)
}

let proof
try {
	const metadata = lstatSync(proofFile)
	if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o077) !== 0) {
		process.exit(2)
	}
	proof = readFileSync(proofFile, 'utf8')
} catch {
	process.exit(2)
}
if (!/^[0-9a-fA-F]{64}$/.test(proof)) {
	process.exit(2)
}

const probe = request({
	host: '127.0.0.1',
	port: 9444,
	path: '/readyz',
	method: 'GET',
	headers: {
		host: '127.0.0.1:5173',
		origin: 'http://127.0.0.1:5173',
		'x-makosh-development-proxy-proof': proof,
	},
	timeout: 2_000,
}, (response) => {
	response.resume()
	response.on('end', () => process.exit(response.statusCode === 200 ? 0 : 1))
})

probe.on('timeout', () => probe.destroy())
probe.on('error', () => process.exit(1))
probe.end()
