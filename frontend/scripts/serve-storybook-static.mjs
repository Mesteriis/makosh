import { createReadStream } from 'node:fs'
import { stat } from 'node:fs/promises'
import { createServer } from 'node:http'
import { extname, join, normalize, resolve, sep } from 'node:path'

const root = resolve(process.cwd(), process.env.MAKOSH_STORYBOOK_STATIC_DIR ?? 'storybook-static')
const host = process.env.MAKOSH_STORYBOOK_HOST ?? 'localhost'
const port = Number(process.env.MAKOSH_STORYBOOK_PORT ?? '6006')

const mimeTypes = new Map([
	['.css', 'text/css; charset=utf-8'],
	['.html', 'text/html; charset=utf-8'],
	['.js', 'text/javascript; charset=utf-8'],
	['.json', 'application/json; charset=utf-8'],
	['.map', 'application/json; charset=utf-8'],
	['.png', 'image/png'],
	['.svg', 'image/svg+xml'],
	['.wasm', 'application/wasm'],
	['.woff', 'font/woff'],
	['.woff2', 'font/woff2']
])

const server = createServer(async (request, response) => {
	const filePath = await resolveFilePath(request.url)
	if (!filePath) {
		response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' })
		response.end('Not found')
		return
	}

	response.writeHead(200, {
		'cache-control': 'no-store',
		'content-type': mimeTypes.get(extname(filePath)) ?? 'application/octet-stream'
	})
	createReadStream(filePath)
		.on('error', () => {
			response.destroy()
		})
		.pipe(response)
})

server.listen(port, host, () => {
	console.log(`Макошь Storybook static server listening at http://${host}:${port}`)
})

process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)

async function resolveFilePath(requestUrl) {
	const url = new URL(requestUrl ?? '/', `http://${host}:${port}`)
	const pathname = decodeURIComponent(url.pathname)
	const normalizedPath = normalize(pathname).replace(/^(\.\.(\/|\\|$))+/, '')
	const requestedPath = resolve(root, `.${normalizedPath}`)
	if (requestedPath !== root && !requestedPath.startsWith(`${root}${sep}`)) {
		return undefined
	}

	const requestedStats = await stat(requestedPath).catch(() => undefined)
	if (requestedStats?.isDirectory()) {
		return existingFile(join(requestedPath, 'index.html'))
	}
	if (requestedStats?.isFile()) {
		return requestedPath
	}
	return undefined
}

async function existingFile(filePath) {
	const fileStats = await stat(filePath).catch(() => undefined)
	return fileStats?.isFile() ? filePath : undefined
}

function shutdown() {
	server.close(() => {
		process.exit(0)
	})
	setTimeout(() => {
		process.exit(1)
	}, 5_000).unref()
}
