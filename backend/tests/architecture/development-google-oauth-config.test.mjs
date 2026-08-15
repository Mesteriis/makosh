import assert from 'node:assert/strict'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'

import {
	readDevelopmentGoogleOAuthClientIdFromConfigFile,
	readDevelopmentGoogleOAuthClientIdFromEnvironmentFile,
} from '../../scripts/read-dev-google-oauth-client-id.mjs'

test('development Gmail config exposes only the installed public client id', () => {
	const root = mkdtempSync(join(tmpdir(), 'makosh-google-oauth-'))
	const configuration = join(root, 'client.json')
	const environment = join(root, '.env')
	writePrivate(configuration, JSON.stringify({
		installed: {
			client_id: 'desktop-client.apps.googleusercontent.com',
			client_secret: 'must-not-be-exported',
			redirect_uris: ['http://localhost'],
		},
	}))
	writePrivate(environment, [
		`HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_PATH=${configuration}`,
		'HERMES_TELEGRAM_API_HASH=must-not-be-exported',
	].join('\n'))

	assert.equal(
		readDevelopmentGoogleOAuthClientIdFromEnvironmentFile(environment),
		'desktop-client.apps.googleusercontent.com',
	)
	assert.equal(
		readDevelopmentGoogleOAuthClientIdFromConfigFile(configuration),
		'desktop-client.apps.googleusercontent.com',
	)
})

test('development Gmail config rejects web clients, non-loopback redirects and shell expansion', () => {
	const root = mkdtempSync(join(tmpdir(), 'makosh-google-oauth-invalid-'))
	const configuration = join(root, 'client.json')
	const environment = join(root, '.env')
	writePrivate(configuration, JSON.stringify({
		installed: {
			client_id: 'desktop-client.apps.googleusercontent.com',
			redirect_uris: ['https://example.test/callback'],
		},
	}))
	assert.throws(
		() => readDevelopmentGoogleOAuthClientIdFromConfigFile(configuration),
		/development Google OAuth installed client configuration is invalid/u,
	)
	writePrivate(environment, 'MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH=$(touch /tmp/not-allowed)')
	assert.throws(
		() => readDevelopmentGoogleOAuthClientIdFromEnvironmentFile(environment),
		/development Google OAuth configuration path is invalid/u,
	)
})

function writePrivate(path, content) {
	writeFileSync(path, content, { mode: 0o600 })
}
