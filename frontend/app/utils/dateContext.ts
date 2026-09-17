/**
 * Utilitário para formatação de contexto temporal e injeção de data/hora atual para a IA.
 */

/**
 * Retorna o nome do fuso horário padrão do sistema
 */
export function getDefaultTimezone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || 'America/Sao_Paulo'
  } catch (_) {
    return 'America/Sao_Paulo'
  }
}

/**
 * Formata uma data e hora para exibição em tempo real na interface de configurações.
 */
export function formatLiveDateTime(date: Date = new Date(), timezone: string = 'America/Sao_Paulo'): string {
  try {
    const tz = timezone || getDefaultTimezone()
    const formatter = new Intl.DateTimeFormat('pt-BR', {
      timeZone: tz,
      weekday: 'short',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    })
    return formatter.format(date)
  } catch (e) {
    return date.toLocaleString('pt-BR')
  }
}

/**
 * Constrói o bloco de instruções temporais injetado sob os panos no System Prompt da IA.
 */
export function getTemporalContextPrompt(timezone: string = 'America/Sao_Paulo'): string {
  try {
    const now = new Date()
    const tz = timezone || getDefaultTimezone()

    // Formatação em inglês para interoperabilidade universal com LLMs
    const dateFormatter = new Intl.DateTimeFormat('en-US', {
      timeZone: tz,
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })

    const shortDateFormatter = new Intl.DateTimeFormat('en-CA', {
      timeZone: tz,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit'
    })

    let offsetStr = ''
    try {
      const offsetFormatter = new Intl.DateTimeFormat('en-US', {
        timeZone: tz,
        timeZoneName: 'shortOffset'
      })
      const parts = offsetFormatter.formatToParts(now)
      const tzPart = parts.find((p) => p.type === 'timeZoneName')
      if (tzPart) offsetStr = ` (${tzPart.value})`
    } catch (_) {}

    const formattedFullDate = dateFormatter.format(now)
    const formattedShortDate = shortDateFormatter.format(now)

    // Capitalizar o dia da semana
    const fullDateCap = formattedFullDate.charAt(0).toUpperCase() + formattedFullDate.slice(1)

    return `\n\n# System Temporal Context:
- Current Date: ${fullDateCap} (${formattedShortDate})
- Timezone: ${tz}${offsetStr}
- Temporal instruction: Use the date above as the current reference date for any questions regarding dates, weekdays, and time intervals.`
  } catch (err) {
    return `\n\n# System Temporal Context:
- Timezone: ${timezone || 'America/Sao_Paulo'}`
  }
}

/**
 * Retorna o horário de envio formatado no fuso horário configurado para contextualizar a mensagem do usuário sem invalidar o KV Cache.
 */
export function formatUserMessageTime(date: Date = new Date(), timezone?: string): string {
  try {
    const tz = timezone || getDefaultTimezone()
    const formatter = new Intl.DateTimeFormat('pt-BR', {
      timeZone: tz,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    })
    return formatter.format(date)
  } catch (_) {
    return date.toLocaleTimeString('pt-BR')
  }
}
