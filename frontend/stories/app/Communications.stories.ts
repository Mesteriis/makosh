import type { Meta, StoryObj } from '@storybook/vue3-vite'
import CanonicalCommunicationsPage from '../../src/domains/communications/presentation/CanonicalCommunicationsPage.vue'
import type { CanonicalCommunicationsPageModel } from '../../src/domains/communications/presentation/canonicalCommunicationsPageModel'

const meta = {
  title: 'Макошь App/Communications/Canonical',
  component: CanonicalCommunicationsPage,
  parameters: { layout: 'fullscreen' }
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: CanonicalCommunicationsPageModel = {
  status: 'ready',
  statusMessage: '',
  accounts: [
    { key: 'source-a', sourceLabel: 'Source 1', identityLabel: 'Account #84a90cf3281e', observedRangeLabel: 'Jul 24, 2026, 09:18 — Jul 26, 2026, 10:42', selected: true },
    { key: 'source-b', sourceLabel: 'Source 3', identityLabel: 'Account #10d7ce928bc1', observedRangeLabel: 'Jul 25, 2026, 18:06', selected: false }
  ],
  conversations: [
    { key: 'conversation-a', identityLabel: 'Conversation #dc15a041f218', sourceLabel: 'Source 1', observedRangeLabel: 'Jul 26, 2026, 10:42', selected: true },
    { key: 'conversation-b', identityLabel: 'Conversation #7d88270cc915', sourceLabel: 'Source 1', observedRangeLabel: 'Jul 25, 2026, 21:10', selected: false }
  ],
  messages: [
    { key: 'message-a', identityLabel: 'Message #ed115728ca2f', stateLabel: 'Body 2 · lifecycle 1', directionLabel: 'Direction 1', observedRangeLabel: 'Jul 26, 2026, 10:42' },
    { key: 'message-b', identityLabel: 'Message #49fc816a9b0b', stateLabel: 'Body 2 · lifecycle 1', directionLabel: 'Direction 2', observedRangeLabel: 'Jul 26, 2026, 09:57' }
  ],
  searchText: 'contract decision',
  searchStatus: 'ready',
  searchMessage: '',
  searchResults: [
    { key: 'evidence-a', evidenceLabel: 'Evidence #178ab2100117', messageLabel: 'Message #ed115728ca2f', conversationLabel: 'Conversation #dc15a041f218', observedAtLabel: 'Jul 26, 2026, 10:42', matchLabel: '2 exact tokens' }
  ]
}

export const Default: Story = {
  args: { model }
}
