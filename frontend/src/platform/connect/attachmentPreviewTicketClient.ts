import { createClient, type Client } from '@connectrpc/connect'

import { AttachmentPreviewTicketService } from '../../gen/makosh/attachment_preview/v1/preview_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof AttachmentPreviewTicketService> | null = null

export function getAttachmentPreviewTicketClient():
	Client<typeof AttachmentPreviewTicketService> {
	client ??= createClient(
		AttachmentPreviewTicketService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetAttachmentPreviewTicketClientForTests(): void {
	client = null
}
