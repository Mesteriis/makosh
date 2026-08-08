import { createClient, type Client } from '@connectrpc/connect'

import { AttachmentPreviewEvidenceReplayCommandService } from '../../gen/makosh/attachment_preview_evidence_replay/v1/replay_pb'
import { createBrowserGatewayConnectTransport } from '../gateway/browserGatewayConnect'

let client: Client<typeof AttachmentPreviewEvidenceReplayCommandService> | null = null

export function getAttachmentPreviewEvidenceReplayCommandClient():
	Client<typeof AttachmentPreviewEvidenceReplayCommandService> {
	client ??= createClient(
		AttachmentPreviewEvidenceReplayCommandService,
		createBrowserGatewayConnectTransport(),
	)
	return client
}

export function resetAttachmentPreviewEvidenceReplayCommandClientForTests(): void {
	client = null
}
