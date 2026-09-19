import { invoke } from '@tauri-apps/api/core'
import { isBrowserMode } from './ipcAdapter'
import { addNotification } from './notifications'
import type { Session, Model } from '../types'
import ptBR from '../locales/pt-BR.json'
import en from '../locales/en.json'
import es from '../locales/es.json'
import zhCN from '../locales/zh-CN.json'
import ru from '../locales/ru.json'

const catalogs: Record<string, any> = {
  'pt-BR': ptBR,
  en,
  es,
  'zh-CN': zhCN,
  ru,
}

function getStoredLocale(): string {
  if (typeof window !== 'undefined' && window.localStorage) {
    const saved = localStorage.getItem('atena_locale')
    if (saved && catalogs[saved]) return saved
  }
  return 'en'
}

export function translate(key: string, params?: Record<string, any>, customLocale?: string): string {
  const locale = customLocale || getStoredLocale()
  const dict = catalogs[locale] || catalogs['en'] || {}
  const keys = key.split('.')
  let current: any = dict
  for (const k of keys) {
    if (current && typeof current === 'object' && k in current) {
      current = current[k]
    } else {
      current = undefined
      break
    }
  }

  // Fallback to English if not found
  if (typeof current !== 'string' && locale !== 'en') {
    let enCurr: any = catalogs['en']
    for (const k of keys) {
      if (enCurr && typeof enCurr === 'object' && k in enCurr) {
        enCurr = enCurr[k]
      } else {
        enCurr = undefined
        break
      }
    }
    if (typeof enCurr === 'string') current = enCurr
  }

  if (typeof current !== 'string') return key

  let result = current
  if (params) {
    for (const [pKey, pVal] of Object.entries(params)) {
      result = result.replaceAll(`{${pKey}}`, String(pVal))
    }
  }
  return result
}

/**
 * Utility to format and export an Atena chat session to Markdown (.md)
 */

export function getFriendlyModelName(model?: Model | string | null, session?: Session | null, customLocale?: string): string {
  let rawName = ''
  if (typeof model === 'string' && model.trim()) {
    rawName = model.trim()
  } else if (model && typeof model === 'object') {
    if (model.name && String(model.name).trim()) {
      rawName = String(model.name).trim()
    } else if (model.id && String(model.id).trim()) {
      rawName = String(model.id).trim()
    }
  } else if (session?.model_name && String(session.model_name).trim()) {
    rawName = String(session.model_name).trim()
  } else if (session?.model_id && String(session.model_id).trim()) {
    rawName = String(session.model_id).trim()
  } else if (session?.messages && session.messages.length > 0) {
    const lastWithModel = [...session.messages].reverse().find((m: any) => m.model || m.model_name)
    if (lastWithModel) {
      rawName = String((lastWithModel as any).model || (lastWithModel as any).model_name).trim()
    }
  }

  if (!rawName) return translate('chat.local_model', undefined, customLocale)

  // Clean filesystem absolute paths
  if (rawName.includes('/') || rawName.includes('\\')) {
    const parts = rawName.split(/[/\\]/).filter(Boolean)
    return parts[parts.length - 1] || rawName
  }

  if (rawName.startsWith('ollama/')) {
    return rawName.replace('ollama/', '')
  }

  return rawName
}

export function formatSessionToMarkdown(session?: Session | null, activeModel?: Model | string | null, customLocale?: string): string {
  if (!session) return ''

  const dateStr = session.created_at
    ? new Date(session.created_at).toLocaleString()
    : new Date().toLocaleString()

  const modelName = getFriendlyModelName(activeModel, session, customLocale)
  const messagesCount = session.messages?.length || 0

  const untitled = translate('chat.untitled_conversation', undefined, customLocale)
  const dateLabel = translate('chat.export_date', undefined, customLocale)
  const modelLabel = translate('chat.export_model', undefined, customLocale)
  const messagesLabel = translate('chat.export_messages', undefined, customLocale)
  const emptyLabel = translate('chat.export_empty', undefined, customLocale)
  const userLabel = translate('chat.user_label', undefined, customLocale)
  const attachmentsLabel = translate('chat.attachments_label', undefined, customLocale)
  const attachmentSingular = translate('chat.attachment_singular', undefined, customLocale)
  const thinkingLabel = translate('chat.thinking_label', undefined, customLocale)
  const mcpToolLabel = translate('chat.mcp_tool_label', undefined, customLocale)
  const serverLabel = translate('chat.server_label', undefined, customLocale)
  const statusCompleted = translate('chat.status_completed', undefined, customLocale)
  const statusRejected = translate('chat.status_rejected', undefined, customLocale)
  const statusPending = translate('chat.status_pending', undefined, customLocale)
  const parametersLabel = translate('chat.parameters_label', undefined, customLocale)
  const refusalReasonTitle = translate('chat.refusal_reason_title', undefined, customLocale)
  const toolResultTitle = translate('chat.tool_result_title', undefined, customLocale)

  let md = `# 💬 ${session.title || untitled}\n\n`
  md += `- **${dateLabel}:** ${dateStr}\n`
  md += `- **${modelLabel}:** ${modelName}\n`
  md += `- **${messagesLabel}:** ${messagesCount}\n\n`
  md += `---\n\n`

  if (!session.messages || session.messages.length === 0) {
    md += `*${emptyLabel}*\n`
    return md
  }

  for (const msg of session.messages) {
    const isUser = msg.role === 'user' || msg.role === 'User'
    const timeStr = msg.timestamp
      ? new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      : ''

    if (isUser) {
      md += `### 👤 **${userLabel}** ${timeStr ? `\`(${timeStr})\`` : ''}\n\n`

      // Attachments or images
      if (msg.attachments && msg.attachments.length > 0) {
        md += `> 📎 **${attachmentsLabel}:** ` + msg.attachments.map((a: any) => a?.name || attachmentSingular).join(', ') + '\n\n'
      }

      const splitPart = (msg.content || '').replace(/^\[Horário local: [^\]]+\]\n?/, '').split(/\n\n--- (?:\[Arquivo Anexado:|\[Conteúdo do Arquivo|\[Documento PDF:|\[Transcrição de Áudio:)/)[0] || ''
      const userText = msg.display_text || splitPart.trim() || msg.content || ''
      md += `${userText}\n\n`
    } else {
      let metaStr = ''
      if (msg.generation_speed_tps) {
        metaStr += ` • ${Number(msg.generation_speed_tps).toFixed(1)} t/s`
      }
      const ttft = msg.time_to_first_token_ms || msg.metrics?.time_to_first_token_ms
      if (ttft && ttft > 0) {
        const ttftStr = ttft < 1000 ? `${Math.round(ttft)}ms` : `${(ttft / 1000).toFixed(2)}s`
        metaStr += ` • ${ttftStr} TTFT`
      }
      if (msg.tokens_count) {
        metaStr += ` • ${msg.tokens_count} tok`
      }

      md += `### 🤖 **Atena Studio** ${timeStr ? `\`(${timeStr}${metaStr})\`` : ''}\n\n`

      // Thinking / Reasoning Block
      if (msg.thinking && msg.thinking.trim()) {
        md += `<details>\n<summary>🧠 <strong>${thinkingLabel}</strong></summary>\n\n`
        md += `${msg.thinking.trim()}\n\n`
        md += `</details>\n\n`
      }

      // Tool Calls
      if (msg.tool_calls && msg.tool_calls.length > 0) {
        for (const tc of msg.tool_calls) {
          const serverInfo = tc.server_name ? ` (${serverLabel}: ${tc.server_name})` : ''
          const statusInfo = tc.status === 'completed' ? statusCompleted : tc.status === 'rejected' ? statusRejected : statusPending

          md += `<details>\n<summary>🔧 <strong>${mcpToolLabel}:</strong> <code>${tc.name}</code>${serverInfo} — <em>[${statusInfo}]</em></summary>\n\n`
          
          if (tc.arguments && Object.keys(tc.arguments).length > 0) {
            md += `**${parametersLabel}:**\n\`\`\`json\n${JSON.stringify(tc.arguments, null, 2)}\n\`\`\`\n\n`
          }

          if (tc.status === 'rejected' && tc.rejection_reason) {
            md += `**${refusalReasonTitle}:** ${tc.rejection_reason}\n\n`
          }

          if (tc.result) {
            md += `**${toolResultTitle}:**\n\`\`\`json\n${typeof tc.result === 'string' ? tc.result : JSON.stringify(tc.result, null, 2)}\n\`\`\`\n\n`
          }

          md += `</details>\n\n`
        }
      }

      // Cleaned assistant response
      const cleanContent = (msg.content || '')
        .replace(/<memoria\b[^>]*\/?>/gi, '')
        .replace(/<memory\b[^>]*\/?>/gi, '')
        .replace(/<memorizar\b[^>]*\/?>/gi, '')
        .replace(/<habilidade\b[^>]*\/?>/gi, '')
        .replace(/<skill\b[^>]*\/?>/gi, '')
        .replace(/<forget\b[^>]*\/?>/gi, '')
        .replace(/<esquecer\b[^>]*\/?>/gi, '')
        .replace(/<remover\b[^>]*\/?>/gi, '')
        .replace(/<\|?(?:audio|video|image|vision|channel|thought|start_of_turn|end_of_turn|im_start|im_end|endoftext|eot_id|start_header_id|end_header_id)[^>]*\|?>/gi, '')
        .replace(/<\|[a-zA-Z0-9_\-:]+\|?>/gi, '')
        .replace(/<[a-zA-Z0-9_\-]+(?:\|>|\|)/gi, '')
        .trim()

      if (cleanContent) {
        md += `${cleanContent}\n\n`
      }
    }

    md += `---\n\n`
  }

  return md
}

export interface SaveTextFileOptions {
  content: string
  filename?: string
  title?: string
  filters?: Array<{ name: string; extensions: string[] }>
  customLocale?: string
}

export async function saveTextFile({ content, filename = 'export.txt', title, filters = [], customLocale }: SaveTextFileOptions): Promise<string | null> {
  if (typeof window === 'undefined') return null

  const resolvedTitle = title || translate('chat.save_file_title', undefined, customLocale)

  // 1. Direct browser download if running in web browser mode
  if (isBrowserMode()) {
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

      try {
        addNotification({
          type: 'success',
          title: translate('chat.export_completed', undefined, customLocale),
          message: translate('chat.export_saved_message', { path: filename }, customLocale),
        })
      } catch {
        // Ignore if notification fails
      }
      return filename
    } catch (e) {
      console.error('Failed to download file in browser:', e)
      return null
    }
  }

  // 2. Save natively via Tauri dialog (Desktop mode)
  try {
    const savedPath = await invoke<string | null>('save_file_content', {
      title: resolvedTitle,
      defaultName: filename,
      defaultPath: null,
      filters,
      content,
    })

    if (savedPath) {
      try {
        addNotification({
          type: 'success',
          title: translate('chat.export_completed', undefined, customLocale),
          message: translate('chat.export_saved_message', { path: savedPath }, customLocale),
        })
      } catch {
        // Ignore if notification fails
      }
      return savedPath
    }
    // User cancelled dialog
    return null
  } catch (err) {
    console.warn('Tauri save_file_content unavailable or failed, falling back to browser:', err)
  }

  // 2. Fallback for traditional web browser
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
    return filename
  } catch (e) {
    console.error('Failed to download file in browser:', e)
    return null
  }
}

export async function downloadMarkdownFile(markdownContent: string, filename = 'conversation.md', customLocale?: string): Promise<string | null> {
  return await saveTextFile({
    content: markdownContent,
    filename,
    title: translate('chat.export_markdown_title', undefined, customLocale),
    filters: [
      { name: translate('chat.filter_markdown', undefined, customLocale), extensions: ['md'] },
      { name: translate('chat.filter_all_files', undefined, customLocale), extensions: ['*'] }
    ],
    customLocale
  })
}
