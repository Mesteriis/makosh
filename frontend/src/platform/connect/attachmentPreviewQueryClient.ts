import { createClient, type Client } from '@connectrpc/connect'

import { AttachmentPreviewQueryService } from '../../gen/makosh/attachment_preview/v1/preview_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof AttachmentPreviewQueryService> | null = null

export function getAttachmentPreviewQueryClient():
	Client<typeof AttachmentPreviewQueryService> {
	client ??= createClient(
		AttachmentPreviewQueryService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetAttachmentPreviewQueryClientForTests(): void {
	client = null
}
