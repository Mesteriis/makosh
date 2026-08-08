import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { domainScaffoldModels } from './domainScaffoldFixtures'
import { createDomainScaffoldStory } from './domainScaffoldStory'

const meta = {
  title: 'Макошь App/Projects/Scaffold'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => createDomainScaffoldStory(domainScaffoldModels.projects)
}
