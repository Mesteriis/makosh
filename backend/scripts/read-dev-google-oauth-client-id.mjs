#!/usr/bin/env node

import { lstatSync, readFileSync } from 'node:fs'
import { isAbsolute } from 'node:path'
import { pathToFileURL } from 'node:url'

const MAX_CONFIGURATION_BYTES = 128 * 1024
const GOOGLE_CONFIGURATION_KEYS = [
	'HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH',
	'MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH',
]

export function readDevelopmentGoogleOAuthClientIdFromEnvironmentFile(environmentFile) {
	const source = privateRegularFile(environmentFile, 'development credentials environment file')
	const assignments = new Map()
	for (const rawLine of source.toString('utf8').split(/\r?\n/u)) {
		const line = rawLine.trim()
		if (!line || line.startsWith('#')) continue
		const assignment = line.startsWith('export ') ? line.slice('export '.length) : line
		const separator = assignment.indexOf('=')
		if (separator < 1) continue
		const name = assignment.slice(0, separator).trim()
		if (!GOOGLE_CONFIGURATION_KEYS.includes(name)) continue
		if (assignments.has(name)) throw new Error('development Google OAuth configuration is invalid')
		assignments.set(name, literalValue(assignment.slice(separator + 1).trim()))
	}
	const configured = GOOGLE_CONFIGURATION_KEYS
		.filter((name) => assignments.has(name))
		.map((name) => assignments.get(name))
	if (configured.length === 0) return ''
	if (configured.length !== 1) throw new Error('development Google OAuth configuration is ambiguous')
	return readDevelopmentGoogleOAuthClientIdFromConfigFile(configured[0])
}

export function readDevelopmentGoogleOAuthClientIdFromConfigFile(configurationFile) {
	if (!isAbsolute(configurationFile)) {
		throw new Error('development Google OAuth configuration path must be absolute')
	}
	const source = privateRegularFile(configurationFile, 'development Google OAuth configuration')
	let parsed
	try {
		parsed = JSON.parse(source.toString('utf8'))
	} catch {
		throw new Error('development Google OAuth configuration is invalid')
	}
	const installed = record(parsed)?.installed
	const clientId = record(installed)?.client_id
	const redirectUris = record(installed)?.redirect_uris
	if (
		typeof clientId !== 'string'
		|| !/^[A-Za-z0-9._-]{1,4096}$/u.test(clientId)
		|| !Array.isArray(redirectUris)
		|| redirectUris.length === 0
		|| !redirectUris.every(validConfiguredLoopbackRedirect)
	) {
		throw new Error('development Google OAuth installed client configuration is invalid')
	}
	return clientId
}

function privateRegularFile(path, label) {
	let metadata
	try {
		metadata = lstatSync(path)
	} catch {
		throw new Error(`${label} is unavailable`)
	}
	if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size > MAX_CONFIGURATION_BYTES) {
		throw new Error(`${label} is invalid`)
	}
	if ((metadata.mode & 0o077) !== 0) throw new Error(`${label} permissions must be owner-only`)
	return readFileSync(path)
}

function literalValue(value) {
	const quote = value.at(0)
	if ((quote === '"' || quote === "'") && value.at(-1) === quote) {
		value = value.slice(1, -1)
	}
	if (!value || /[\0\r\n]/u.test(value) || /[`$]/u.test(value)) {
		throw new Error('development Google OAuth configuration path is invalid')
	}
	return value
}

function validConfiguredLoopbackRedirect(value) {
	if (typeof value !== 'string' || value.length > 4096) return false
	let parsed
	try {
		parsed = new URL(value)
	} catch {
		return false
	}
	return parsed.protocol === 'http:'
		&& ['127.0.0.1', 'localhost'].includes(parsed.hostname)
		&& !parsed.username
		&& !parsed.password
		&& !parsed.hash
}

function record(value) {
	return typeof value === 'object' && value !== null && !Array.isArray(value) ? value : undefined
}

function run(arguments_) {
	if (arguments_.length !== 2 || !['--env-file', '--config-file'].includes(arguments_[0])) {
		throw new Error('usage: read-dev-google-oauth-client-id.mjs (--env-file|--config-file) PATH')
	}
	const clientId = arguments_[0] === '--env-file'
		? readDevelopmentGoogleOAuthClientIdFromEnvironmentFile(arguments_[1])
		: readDevelopmentGoogleOAuthClientIdFromConfigFile(arguments_[1])
	process.stdout.write(clientId)
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
	try {
		run(process.argv.slice(2))
	} catch (error) {
		process.stderr.write(`${error instanceof Error ? error.message : 'development Google OAuth configuration failed'}\n`)
		process.exitCode = 1
	}
}
