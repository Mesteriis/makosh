import { createClient, type Client } from '@connectrpc/connect'

import { AttachmentPreviewCommandService } from '../../gen/makosh/attachment_preview/v1/preview_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof AttachmentPreviewCommandService> | null = null

export function getAttachmentPreviewCommandClient():
	Client<typeof AttachmentPreviewCommandService> {
	client ??= createClient(
		AttachmentPreviewCommandService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetAttachmentPreviewCommandClientForTests(): void {
	client = null
}
