import { ref, computed, type Ref } from 'vue'
import type { NotificationItem, Toast } from '../types'

export type { NotificationItem, Toast }

const STORAGE_KEY = 'atena_notifications'

// Load saved notifications from localStorage
const loadSavedNotifications = (): NotificationItem[] => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (raw) {
        const parsed = JSON.parse(raw)
        if (Array.isArray(parsed)) return parsed
      }
    }
  } catch (e) {
    console.error('Falha ao carregar notificações salvas:', e)
  }
  return []
}

export const notifications: Ref<NotificationItem[]> = ref(loadSavedNotifications())
export const activeToasts: Ref<Toast[]> = ref([])

const saveToLocalStorage = () => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      // Keep max 50 recent notifications
      const trimmed = notifications.value.slice(0, 50)
      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmed))
    }
  } catch (e) {
    console.error('Falha ao salvar notificações:', e)
  }
}

export interface AddNotificationOptions {
  type?: 'info' | 'success' | 'warning' | 'error' | string
  title?: string
  message?: string
  data?: any
}

export const addNotification = ({
  type = 'info',
  title,
  message,
  data = null
}: AddNotificationOptions): string => {
  const id = `notif-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
  const item: NotificationItem = {
    id,
    type,
    title: title || 'Notificação',
    message: message || '',
    timestamp: new Date().toISOString(),
    read: false,
    data
  }

  // Prepend to notifications list
  notifications.value.unshift(item)
  saveToLocalStorage()

  // Also trigger temporary visual toast
  const toastItem: Toast = { ...item, toastId: id }
  activeToasts.value.push(toastItem)
  setTimeout(() => {
    activeToasts.value = activeToasts.value.filter((t) => t.toastId !== id)
  }, 4500)

  return id
}

export const dismissToast = (toastId?: string): void => {
  if (!toastId) return
  activeToasts.value = activeToasts.value.filter((t) => t.toastId !== toastId)
}

export const removeNotification = (id: string): void => {
  notifications.value = notifications.value.filter((n) => n.id !== id)
  saveToLocalStorage()
}

export const clearAllNotifications = (): void => {
  notifications.value = []
  saveToLocalStorage()
}

export const markAsRead = (id: string): void => {
  const target = notifications.value.find((n) => n.id === id)
  if (target) {
    target.read = true
    saveToLocalStorage()
  }
}

export const markAllAsRead = (): void => {
  notifications.value.forEach((n) => {
    n.read = true
  })
  saveToLocalStorage()
}

export const unreadCount = computed<number>(() => {
  return notifications.value.filter((n) => !n.read).length
})

export const formatRelativeTime = (isoString?: string | null): string => {
  if (!isoString) return ''
  try {
    const diff = Date.now() - new Date(isoString).getTime()
    const sec = Math.floor(diff / 1000)
    if (sec < 5) return 'Agora'
    if (sec < 60) return `${sec}s atrás`
    const min = Math.floor(sec / 60)
    if (min < 60) return `${min}m atrás`
    const hrs = Math.floor(min / 60)
    if (hrs < 24) return `${hrs}h atrás`
    const days = Math.floor(hrs / 24)
    return `${days}d atrás`
  } catch {
    return ''
  }
}
