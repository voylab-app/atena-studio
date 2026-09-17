/**
 * Atena Studio IPC Web Adapter
 * Enables complete web browser access (Option A: Headless Server Mode).
 * Polyfills `window.__TAURI_INTERNALS__` when running outside the native Tauri desktop webview.
 */

interface PendingRequest {
  resolve: (value: any) => void
  reject: (reason?: any) => void
  timeoutId?: ReturnType<typeof setTimeout>
}

let nextCallbackId = 1
let nextRequestId = 1
const callbacks = new Map<number, (response: any) => void>()
const pendingRequests = new Map<number, PendingRequest>()
const channelMessageIndices = new Map<number, number>()

let socket: WebSocket | null = null
let isConnecting = false
const messageQueue: string[] = []

export function isTauriDesktop(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && !('__ATENA_WEB_ADAPTER__' in window)
}

export function isBrowserMode(): boolean {
  return !isTauriDesktop()
}

function getAuthToken(): string | null {
  if (typeof window === 'undefined') return null

  // 1. Check URL parameters
  const params = new URLSearchParams(window.location.search)
  const urlToken = params.get('token')
  if (urlToken) {
    try {
      localStorage.setItem('atena_server_token', urlToken)
    } catch {
      // Ignore storage errors
    }
    return urlToken
  }

  // 2. Check localStorage
  try {
    const stored = localStorage.getItem('atena_server_token')
    if (stored) return stored
  } catch {
    // Ignore storage errors
  }

  return null
}

function initWebSocket(): void {
  if (typeof window === 'undefined') return
  if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
    return
  }

  isConnecting = true
  const token = getAuthToken()
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  // When running Vite / Nuxt dev server on port 3000/5173, point WebSocket to default server port 7860
  const isFrontendDev = window.location.port === '3000' || window.location.port === '5173'
  const backendHost = isFrontendDev ? `${window.location.hostname}:7860` : window.location.host
  const wsUrl = `${protocol}//${backendHost}/api/ipc/ws${token ? `?token=${encodeURIComponent(token)}` : ''}`

  try {
    socket = new WebSocket(wsUrl)

    socket.onopen = () => {
      isConnecting = false
      console.info('🔌 [Atena IPC Adapter] Connected to backend WebSocket bridge')
      
      // Flush queued messages
      while (messageQueue.length > 0) {
        const msg = messageQueue.shift()
        if (msg && socket && socket.readyState === WebSocket.OPEN) {
          socket.send(msg)
        }
      }
    }

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)

        // 1. Handle Channel streaming messages
        if (data.type === 'channel') {
          const handler = callbacks.get(data.id)
          if (handler) {
            let idx = data.index
            if (typeof idx !== 'number') {
              const current = channelMessageIndices.get(data.id) || 0
              idx = current
              if (!data.end) {
                channelMessageIndices.set(data.id, current + 1)
              }
            } else {
              channelMessageIndices.set(data.id, idx + 1)
            }

            if (data.end) {
              channelMessageIndices.delete(data.id)
              handler({ end: true, index: idx })
            } else {
              handler({ message: data.message, index: idx })
            }
          }
          return
        }

        // 2. Handle standard command responses
        if (data.type === 'response') {
          const reqId = data.id
          const pending = pendingRequests.get(reqId)
          if (pending) {
            if (pending.timeoutId) clearTimeout(pending.timeoutId)
            pendingRequests.delete(reqId)

            if (data.error) {
              pending.reject(new Error(data.error))
            } else {
              pending.resolve(data.result)
            }
          }
          return
        }

        // 3. Heartbeat
        if (data.type === 'pong') {
          return
        }
      } catch (err) {
        console.warn('⚠️ [Atena IPC Adapter] Error handling incoming message:', err)
      }
    }

    socket.onerror = () => {
      isConnecting = false
    }

    socket.onclose = () => {
      isConnecting = false
      socket = null
      // Attempt reconnection after a short delay
      setTimeout(() => {
        initWebSocket()
      }, 2000)
    }
  } catch (err) {
    isConnecting = false
    socket = null
    setTimeout(() => {
      initWebSocket()
    }, 3000)
  }
}

function sendWsMessage(msg: string): void {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(msg)
  } else {
    messageQueue.push(msg)
    if (!socket || socket.readyState === WebSocket.CLOSED) {
      initWebSocket()
    }
  }
}

export function initIpcBridge(): void {
  if (typeof window === 'undefined') return

  // If running inside native Tauri desktop webview, keep native bindings untouched
  if ('__TAURI_INTERNALS__' in window) {
    console.info('🖥️ [Atena] Running in Native Tauri Desktop Mode')
    return
  }

  console.info('🌐 [Atena] Initializing Browser IPC Adapter Bridge')
  initWebSocket()

  // Polyfill `window.__TAURI_INTERNALS__` expected by `@tauri-apps/api/core`
  const internals = {
    __ATENA_WEB_ADAPTER__: true,
    plugins: {},

    transformCallback(callback: (response: any) => void, once = false): number {
      const id = nextCallbackId++
      if (once) {
        callbacks.set(id, (resp) => {
          callbacks.delete(id)
          callback(resp)
        })
      } else {
        callbacks.set(id, callback)
      }
      return id
    },

    unregisterCallback(id: number): void {
      callbacks.delete(id)
    },

    convertFileSrc(filePath: string): string {
      return filePath
    },

    async invoke(cmd: string, args: Record<string, any> = {}): Promise<any> {
      // 1. Intercept folder selector in browser mode -> dispatch custom event to show in-app modal
      if (cmd === 'select_folder') {
        return new Promise((resolve) => {
          if (typeof window !== 'undefined') {
            window.dispatchEvent(
              new CustomEvent('atena:open-folder-picker', {
                detail: {
                  defaultPath: args.defaultPath,
                  resolve
                }
              })
            )
          } else {
            resolve(null)
          }
        })
      }

      // 2. Intercept file export / saving in browser mode -> trigger direct client-side download
      if (cmd === 'save_file_content') {
        const content = args.content || ''
        const filename = args.defaultName || 'export.txt'
        try {
          const blob = new Blob([content], { type: 'text/plain;charset=utf-8;' })
          const url = URL.createObjectURL(blob)
          const link = document.createElement('a')
          link.setAttribute('href', url)
          link.setAttribute('download', filename)
          document.body.appendChild(link)
          link.click()
          document.body.removeChild(link)
          URL.revokeObjectURL(url)
          return Promise.resolve(filename)
        } catch (e) {
          console.error('Failed to download file in browser:', e)
          return Promise.resolve(null)
        }
      }

      const reqId = nextRequestId++

      return new Promise((resolve, reject) => {
        // Long-running streaming or heavy operational commands should not have an arbitrary client timeout
        const noTimeoutCommands = new Set([
          'stream_chat',
          'start_mlx_server',
          'start_llama_server',
          'start_ollama_server',
          'bootstrap_all_runtimes',
          'bootstrap_mlx_runtime',
          'update_mlx_packages',
          'bootstrap_llama_runtime',
          'bootstrap_ffmpeg_runtime',
          'scan_models'
        ])

        let timeoutId: ReturnType<typeof setTimeout> | undefined
        if (!noTimeoutCommands.has(cmd)) {
          timeoutId = setTimeout(() => {
            if (pendingRequests.has(reqId)) {
              pendingRequests.delete(reqId)
              reject(new Error(`Timeout waiting for command response: ${cmd}`))
            }
          }, 120000) // 2 minutes timeout only for short unary commands
        }

        pendingRequests.set(reqId, { resolve, reject, timeoutId })

        const payload = JSON.stringify({
          id: reqId,
          type: 'invoke',
          cmd,
          args
        })

        sendWsMessage(payload)
      })
    }
  }

  // Inject into window
  ;(window as any).__TAURI_INTERNALS__ = internals
}
