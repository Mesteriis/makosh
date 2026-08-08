const DATABASE = 'makosh-browser-identity-v1'
const STORE = 'local-keys'

type BrowserLocalKeyPair = {
	publicKey: Uint8Array
	privateKey: CryptoKey
}

type BrowserLocalKeyRecord = {
	credentialId: string
	privateKey: CryptoKey
}

/**
 * A non-extractable P-256 key that is local to this browser profile. Its
 * public point is bound to the paired device; its private key is never sent
 * to Gateway and is not serialised into application storage.
 */
export async function createBrowserLocalDeviceKey(): Promise<BrowserLocalKeyPair> {
	const pair = await crypto.subtle.generateKey(
		{ name: 'ECDSA', namedCurve: 'P-256' },
		false,
		['sign', 'verify'],
	) as CryptoKeyPair
	const publicJwk = await crypto.subtle.exportKey('jwk', pair.publicKey)
	if (typeof publicJwk.x !== 'string' || typeof publicJwk.y !== 'string') {
		throw new Error('browser local key is unavailable')
	}
	const x = base64UrlDecode(publicJwk.x)
	const y = base64UrlDecode(publicJwk.y)
	if (x.byteLength !== 32 || y.byteLength !== 32) throw new Error('browser local key is unavailable')
	const publicKey = new Uint8Array(65)
	publicKey[0] = 4
	publicKey.set(x, 1)
	publicKey.set(y, 33)
	return { publicKey, privateKey: pair.privateKey }
}

export async function saveBrowserLocalDeviceKey(credentialId: string, privateKey: CryptoKey): Promise<void> {
	const database = await openDatabase()
	await transaction(database, 'readwrite', (store) => store.put({ credentialId, privateKey } satisfies BrowserLocalKeyRecord))
}

export async function deleteBrowserLocalDeviceKey(credentialId: string): Promise<void> {
	const database = await openDatabase()
	await transaction(database, 'readwrite', (store) => store.delete(credentialId))
}

export async function signBrowserLocalDeviceChallenge(credentialId: string, challenge: ArrayBuffer): Promise<Uint8Array> {
	const database = await openDatabase()
	const record = await transaction<BrowserLocalKeyRecord | undefined>(database, 'readonly', (store) => store.get(credentialId))
	if (!record?.privateKey) throw new Error('browser local device key is unavailable')
	const signature = await crypto.subtle.sign(
		{ name: 'ECDSA', hash: 'SHA-256' },
		record.privateKey,
		challenge,
	)
	const bytes = new Uint8Array(signature)
	if (bytes.byteLength !== 64) throw new Error('browser local device key is unavailable')
	return bytes
}

function openDatabase(): Promise<IDBDatabase> {
	if (!globalThis.indexedDB) return Promise.reject(new Error('browser local device key is unavailable'))
	return new Promise((resolve, reject) => {
		const request = globalThis.indexedDB.open(DATABASE, 1)
		request.onupgradeneeded = () => request.result.createObjectStore(STORE, { keyPath: 'credentialId' })
		request.onsuccess = () => resolve(request.result)
		request.onerror = () => reject(new Error('browser local device key is unavailable'))
	})
}

function transaction<T = void>(database: IDBDatabase, mode: IDBTransactionMode, operation: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
	return new Promise((resolve, reject) => {
		const request = operation(database.transaction(STORE, mode).objectStore(STORE))
		request.onsuccess = () => resolve(request.result)
		request.onerror = () => reject(new Error('browser local device key is unavailable'))
	})
}

function base64UrlDecode(value: string): Uint8Array {
	if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('browser local key is unavailable')
	const normalized = value.replaceAll('-', '+').replaceAll('_', '/')
	const binary = atob(normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '='))
	return Uint8Array.from(binary, (character) => character.charCodeAt(0))
}
