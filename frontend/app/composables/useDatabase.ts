import { invoke } from '@tauri-apps/api/core'
import type { ChatSession, Persona } from '~/types'

export interface PersonasData {
  custom_personas: Persona[]
  overrides: Record<string, string>
  active_persona_id?: string | null
}

export function useDatabase() {
  const getSessions = async (): Promise<ChatSession[]> => {
    try {
      const res = await invoke<ChatSession[]>('db_get_sessions')
      return res || []
    } catch (err) {
      console.error('Failed to load sessions from database:', err)
      return []
    }
  }

  const getSession = async (sessionId: string): Promise<ChatSession | null> => {
    try {
      const res = await invoke<ChatSession | null>('db_get_session', { sessionId })
      return res || null
    } catch (err) {
      console.error(`Failed to load session ${sessionId} from database:`, err)
      return null
    }
  }

  const saveSession = async (session: ChatSession): Promise<void> => {
    try {
      await invoke('db_save_session', { session })
    } catch (err) {
      console.error(`Failed to save session ${session.id} to database:`, err)
    }
  }

  const saveSessionsBatch = async (sessions: ChatSession[]): Promise<void> => {
    try {
      await invoke('db_save_sessions_batch', { sessions })
    } catch (err) {
      console.error('Failed to batch save sessions to database:', err)
    }
  }

  const deleteSession = async (sessionId: string): Promise<void> => {
    try {
      await invoke('db_delete_session', { sessionId })
    } catch (err) {
      console.error(`Failed to delete session ${sessionId} from database:`, err)
    }
  }

  const deleteSessionsBatch = async (sessionIds: string[]): Promise<void> => {
    try {
      await invoke('db_delete_sessions_batch', { sessionIds })
    } catch (err) {
      console.error('Failed to batch delete sessions from database:', err)
    }
  }

  const getPersonasData = async (): Promise<PersonasData> => {
    try {
      const res = await invoke<PersonasData>('db_get_personas_data')
      return res || { custom_personas: [], overrides: {}, active_persona_id: null }
    } catch (err) {
      console.error('Failed to load personas from database:', err)
      return { custom_personas: [], overrides: {}, active_persona_id: null }
    }
  }

  const savePersona = async (persona: Persona): Promise<void> => {
    try {
      await invoke('db_save_persona', { persona })
    } catch (err) {
      console.error(`Failed to save persona ${persona.id} to database:`, err)
    }
  }

  const deletePersona = async (personaId: string): Promise<void> => {
    try {
      await invoke('db_delete_persona', { personaId })
    } catch (err) {
      console.error(`Failed to delete persona ${personaId} from database:`, err)
    }
  }

  const savePersonaOverride = async (personaId: string, systemPrompt: string): Promise<void> => {
    try {
      await invoke('db_save_persona_override', { personaId, systemPrompt })
    } catch (err) {
      console.error(`Failed to save persona override for ${personaId}:`, err)
    }
  }

  const resetPersonaOverride = async (personaId: string): Promise<void> => {
    try {
      await invoke('db_reset_persona_override', { personaId })
    } catch (err) {
      console.error(`Failed to reset persona override for ${personaId}:`, err)
    }
  }

  const getSetting = async (key: string): Promise<string | null> => {
    try {
      const res = await invoke<string | null>('db_get_setting', { key })
      return res ?? null
    } catch (err) {
      console.error(`Failed to get setting ${key}:`, err)
      return null
    }
  }

  const setSetting = async (key: string, value: string): Promise<void> => {
    try {
      await invoke('db_set_setting', { key, value })
    } catch (err) {
      console.error(`Failed to set setting ${key}:`, err)
    }
  }

  const getAllSettings = async (): Promise<Record<string, string>> => {
    try {
      const res = await invoke<Record<string, string>>('db_get_all_settings')
      return res || {}
    } catch (err) {
      console.error('Failed to get all settings:', err)
      return {}
    }
  }

  return {
    getSessions,
    getSession,
    saveSession,
    saveSessionsBatch,
    deleteSession,
    deleteSessionsBatch,
    getPersonasData,
    savePersona,
    deletePersona,
    savePersonaOverride,
    resetPersonaOverride,
    getSetting,
    setSetting,
    getAllSettings,
  }
}
