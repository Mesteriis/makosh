import { getCurrentInstance, inject, type InjectionKey } from 'vue'

export interface ToastItem {
  id: string
  title?: string
  description?: string
  variant?: 'default' | 'info' | 'success' | 'warning' | 'error'
  duration?: number
}

export type DefaultToastItem = Omit<ToastItem, 'id'> & {
  id?: string
}

export type ToastContext = {
  addToast(item: Omit<ToastItem, 'id'>): string
  removeToast(id: string): void
  success(title: string, description?: string): string
  warning(title: string, description?: string): string
  error(title: string, description?: string): string
}

export const TOAST_INJECTION_KEY: InjectionKey<ToastContext> =
  Symbol('makosh-toast-context')

const noopToastContext: ToastContext = {
  addToast: () => '',
  removeToast: () => {},
  success: () => '',
  warning: () => '',
  error: () => '',
}

export function useToast(): ToastContext {
  if (!getCurrentInstance()) return noopToastContext

  return inject(TOAST_INJECTION_KEY, noopToastContext)
}
