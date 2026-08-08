import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const sourceRoot = path.resolve(new URL('../', import.meta.url).pathname)

function collectVueFiles(rootPath: string): string[] {
  const entries = readdirSync(rootPath, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    const entryPath = path.join(rootPath, entry.name)
    if (entry.isDirectory()) {
      files.push(...collectVueFiles(entryPath))
      continue
    }
    if (entry.isFile() && entry.name.endsWith('.vue')) {
      files.push(entryPath)
    }
  }

  return files.sort()
}

function getScriptSection(source: string): string {
  const start = source.indexOf('<script')
  if (start === -1) return ''
  const openEnd = source.indexOf('>', start)
  if (openEnd === -1) return ''
  const end = source.indexOf('</script>', openEnd)
  if (end === -1) return ''
  return source.slice(openEnd + 1, end)
}

describe('App and shared Vue boundary contract', () => {
  it('keeps app and shared Vue files on UI-only responsibilities', () => {
    const appVueFiles = collectVueFiles(path.join(sourceRoot, 'app'))
    const sharedVueFiles = collectVueFiles(path.join(sourceRoot, 'shared'))
    const allVueFiles = [...appVueFiles, ...sharedVueFiles]

    for (const filePath of allVueFiles) {
      const source = readFileSync(filePath, 'utf8')
      const scriptSource = getScriptSection(source)
      const normalized = filePath.split(path.sep).join('/')

      expect(source).not.toContain('<style')
      expect(scriptSource).not.toMatch(/from ['"][^'"]*\/(api|connect|model|mappers|policies|services|helpers|lib)\/[^'"]*['"]/)
      expect(scriptSource).not.toMatch(/fetch\s*\(/)
      expect(scriptSource).not.toMatch(/client\.(get|post|put|delete|patch)\s*\(/)
      expect(scriptSource).not.toMatch(/localStorage\./)
      expect(scriptSource).not.toMatch(/sessionStorage\./)
      expect(scriptSource).not.toMatch(/queryClient\./)
      expect(scriptSource).not.toMatch(/invalidateQueries\s*\(/)

      if (normalized.includes('/app/')) {
        expect(scriptSource).not.toMatch(/\.(map|filter|reduce|sort)\s*\(/)
        expect(scriptSource).not.toMatch(/\bflatMap\s*\(/)
        expect(scriptSource).not.toMatch(/\bnew\s+(Set|Map)\s*\(/)
        expect(scriptSource).not.toMatch(/\bprovide\s*\(/)
        expect(scriptSource).not.toMatch(/\binject\s*\(/)
      }
    }

    const commandSource = readFileSync(path.join(sourceRoot, 'shared/ui/Command.vue'), 'utf8')
    const toastSource = readFileSync(path.join(sourceRoot, 'shared/ui/Toast.vue'), 'utf8')
    const toastHelperSource = readFileSync(path.join(sourceRoot, 'shared/ui/toast.ts'), 'utf8')
    const feedbackCss = readFileSync(path.join(sourceRoot, 'shared/ui/styles/feedback.css'), 'utf8')
    const progressSource = readFileSync(path.join(sourceRoot, 'shared/ui/Progress.vue'), 'utf8')

    expect(commandSource).toContain('const query = ref')
    expect(commandSource).toContain('flatMap')
    expect(commandSource).toContain('.filter((item)')
    expect(commandSource).toContain('nextTick(() => inputRef.value?.focus())')
    expect(commandSource).not.toContain('fetch(')

    expect(toastSource).toContain('provide(TOAST_INJECTION_KEY')
    expect(toastSource).toContain('const toasts = ref<ToastItem[]>')
    expect(toastSource).toContain('TOAST_EXIT_ANIMATION_MS = 820')
    expect(toastSource).toContain('scheduleToastRemoval')
    expect(toastSource).toContain('handleToastOpenChange(toast.id, $event)')
    expect(toastSource).toContain("from './toast'")
    expect(toastHelperSource).toContain("variant?: 'default' | 'info' | 'success' | 'warning' | 'error'")
    expect(toastHelperSource).toContain('export function useToast()')
    expect(toastSource).toContain(':data-toast-id="toast.id"')
    expect(toastSource).toContain('removeToast')
    expect(toastSource).not.toContain('queryClient')
    expect(feedbackCss).toContain('.makosh-toast-viewport')
    expect(feedbackCss).toContain("bottom: 16px")
    expect(feedbackCss).toContain("right: 16px")
    expect(feedbackCss).toContain(".makosh-toast-root[data-state='open']")
    expect(feedbackCss).toContain(".makosh-toast-root[data-state='closed']")
    expect(feedbackCss).toContain('makosh-toast-enter-bottom-right')
    expect(feedbackCss).toContain('makosh-toast-exit-up-right')
    expect(feedbackCss).toContain('translate(112%, -128px)')

    expect(progressSource).toContain('watchEffect(() => {')
    expect(progressSource).toContain('element.style.transform')
    expect(progressSource).not.toContain('fetch(')
  })
})
