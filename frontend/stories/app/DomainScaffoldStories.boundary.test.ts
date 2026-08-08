import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

type DomainScaffoldStoryExpectation = {
  fileName: string
  storyTitle: string
  modelKey: string
}

const domainScaffoldStories: readonly DomainScaffoldStoryExpectation[] = [
  {
    fileName: 'Agents.stories.ts',
    storyTitle: 'Макошь App/AI Agents/Scaffold',
    modelKey: 'agents'
  },
  {
    fileName: 'Calendar.stories.ts',
    storyTitle: 'Макошь App/Calendar/Scaffold',
    modelKey: 'calendar'
  },
  {
    fileName: 'Documents.stories.ts',
    storyTitle: 'Макошь App/Documents/Scaffold',
    modelKey: 'documents'
  },
  {
    fileName: 'EventTraces.stories.ts',
    storyTitle: 'Макошь App/Event Traces/Scaffold',
    modelKey: 'eventTraces'
  },
  {
    fileName: 'Home.stories.ts',
    storyTitle: 'Макошь App/Home/Scaffold',
    modelKey: 'home'
  },
  {
    fileName: 'Knowledge.stories.ts',
    storyTitle: 'Макошь App/Knowledge Graph/Scaffold',
    modelKey: 'knowledge'
  },
  {
    fileName: 'Notes.stories.ts',
    storyTitle: 'Макошь App/Notes/Scaffold',
    modelKey: 'notes'
  },
  {
    fileName: 'Organizations.stories.ts',
    storyTitle: 'Макошь App/Organizations/Scaffold',
    modelKey: 'organizations'
  },
  {
    fileName: 'Projects.stories.ts',
    storyTitle: 'Макошь App/Projects/Scaffold',
    modelKey: 'projects'
  },
  {
    fileName: 'Review.stories.ts',
    storyTitle: 'Макошь App/Review/Scaffold',
    modelKey: 'review'
  },
  {
    fileName: 'Tasks.stories.ts',
    storyTitle: 'Макошь App/Tasks/Scaffold',
    modelKey: 'tasks'
  },
  {
    fileName: 'Timeline.stories.ts',
    storyTitle: 'Макошь App/Timeline/Scaffold',
    modelKey: 'timeline'
  }
]

describe('domain scaffold Storybook coverage', () => {
  it('keeps one app Storybook scaffold per planned domain', () => {
    for (const story of domainScaffoldStories) {
      const storyUrl = new URL(`./${story.fileName}`, import.meta.url)
      expect(existsSync(storyUrl)).toBe(true)

      const source = readFileSync(storyUrl, 'utf8')
      expect(source).toContain(`title: '${story.storyTitle}'`)
      expect(source).toContain(`domainScaffoldModels.${story.modelKey}`)
      expect(source).toContain('createDomainScaffoldStory')
      expect(source).not.toContain('createDomainSurfaceStory')
    }
  })

  it('keeps Storybook scaffolds separate from TS surface facades', () => {
    const storySources = domainScaffoldStories
      .map((story) => readFileSync(new URL(`./${story.fileName}`, import.meta.url), 'utf8'))
      .join('\n')
    const helperSource = readFileSync(new URL('./domainScaffoldStory.ts', import.meta.url), 'utf8')

    expect(storySources).not.toMatch(/use[A-Z][A-Za-z]+Surface/)
    expect(storySources).not.toContain('/queries/')
    expect(storySources).not.toContain('/Surface')
    expect(helperSource).not.toContain('surfacePath')
    expect(helperSource).not.toContain('contract')
  })

  it('keeps Personas on the rebuilt workspace story instead of the scaffold placeholder', () => {
    const source = readFileSync(new URL('./Personas.stories.ts', import.meta.url), 'utf8')
    const componentSource = readFileSync(new URL('./PersonasComponents.stories.ts', import.meta.url), 'utf8')

    expect(source).toContain("title: 'Макошь App/Personas/Workspace'")
    expect(source).toContain('PersonasWorkspaceComponent')
    expect(source).not.toContain('createDomainScaffoldStory')
    expect(source).not.toContain('domainScaffoldModels.persons')
    expect(componentSource).toContain("title: 'Макошь App/Personas/Components'")
    expect(componentSource).toContain('PersonaDirectoryPanel')
    expect(componentSource).toContain('PersonaOverviewPanel')
    expect(componentSource).not.toContain('PersonDirectoryPanel')
    expect(componentSource).not.toContain('PersonOverviewPanel')
    expect(componentSource).toContain('UnavailableSkeletonPanel')
    expect(componentSource).toContain('directoryFilter')
    expect(componentSource).toContain('toggleAddressBook')
    expect(componentSource).toContain('is_address_book')
  })

  it('keeps Communications on the canonical owner story instead of the scaffold placeholder', () => {
    const source = readFileSync(new URL('./Communications.stories.ts', import.meta.url), 'utf8')

    expect(source).toContain("title: 'Макошь App/Communications/Canonical'")
    expect(source).toContain('CanonicalCommunicationsPage')
    expect(source).not.toContain('createDomainScaffoldStory')
    expect(source).not.toContain('domainScaffoldModels.communications')
  })

  it('keeps Settings on the app owner workbench story instead of the scaffold placeholder', () => {
    const source = readFileSync(new URL('./Settings.stories.ts', import.meta.url), 'utf8')

    expect(source).toContain("title: 'Макошь App/Settings/Clean Room'")
    expect(source).toContain('AppSettingsPage')
    expect(source).not.toContain('createDomainScaffoldStory')
    expect(source).not.toContain('domainScaffoldModels.settings')
  })
})
