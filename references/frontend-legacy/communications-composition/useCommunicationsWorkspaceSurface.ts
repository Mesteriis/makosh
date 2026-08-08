// Historical pre-clean-room provider composition. It is not part of the active client graph.
import {
  communicationSurfaceChild,
  createCommunicationSurface
} from './communicationChannelSurface'
import { useCallsCommunicationsSurface } from './useCallsCommunicationsSurface'
import { useCommunicationTimelineSurface } from './useCommunicationTimelineSurface'
import { useDiscordCommunicationsSurface } from './useDiscordCommunicationsSurface'
import { useMailCommunicationsSurface } from './useMailCommunicationsSurface'
import { useMattermostCommunicationsSurface } from './useMattermostCommunicationsSurface'
import { useMeetingsCommunicationsSurface } from './useMeetingsCommunicationsSurface'
import { useSlackCommunicationsSurface } from './useSlackCommunicationsSurface'
import { useTelegramCommunicationsSurface } from './useTelegramCommunicationsSurface'
import { useWhatsappCommunicationsSurface } from './useWhatsappCommunicationsSurface'
import { useZulipCommunicationsSurface } from './useZulipCommunicationsSurface'

export function useCommunicationsWorkspaceSurface() {
  const mail = useMailCommunicationsSurface()
  const telegram = useTelegramCommunicationsSurface()
  const whatsapp = useWhatsappCommunicationsSurface()
  const calls = useCallsCommunicationsSurface()
  const meetings = useMeetingsCommunicationsSurface()
  const timeline = useCommunicationTimelineSurface()
  const zulip = useZulipCommunicationsSurface()
  const slack = useSlackCommunicationsSurface()
  const discord = useDiscordCommunicationsSurface()
  const mattermost = useMattermostCommunicationsSurface()

  const subSurfaces = [
    mail,
    telegram,
    whatsapp,
    calls,
    meetings,
    timeline,
    zulip,
    slack,
    discord,
    mattermost
  ] as const

  const childSurfaces = subSurfaces.map(communicationSurfaceChild)

  return createCommunicationSurface({
    surfaceId: 'communications',
    commonCapabilities: [
      {
        id: 'communications-provider-neutral-navigation',
        labelKey: 'Provider-neutral navigation',
        descriptionKey: 'Communications owns the shared account/provider path and exposes sub-surfaces through one frontend contract.',
        icon: 'tabler:route',
        status: 'available',
        kind: 'common',
        contract: 'CommunicationSurface.childSurfaces'
      },
      {
        id: 'communications-evidence-first',
        labelKey: 'Evidence-first projection',
        descriptionKey: 'Provider events become source evidence before Макошь promotes durable entities.',
        icon: 'tabler:shield-check',
        status: 'available',
        kind: 'projection',
        contract: 'raw -> accepted -> communications projection'
      },
      {
        id: 'communications-provider-command-boundary',
        labelKey: 'Provider command boundary',
        descriptionKey: 'Outbound writes are queued provider commands, not direct UI-owned provider mutations.',
        icon: 'tabler:list-check',
        status: 'available',
        kind: 'command',
        contract: 'communication.provider_command.requested'
      }
    ],
    mail,
    telegram,
    whatsapp,
    calls,
    meetings,
    timeline,
    zulip,
    slack,
    discord,
    mattermost,
    subSurfaces,
    childSurfaces
  })
}
