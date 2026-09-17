import { ref, computed, type Ref, type ComputedRef } from 'vue'
import type { Persona } from '../types'

export interface PersonaItem extends Persona {
  is_custom?: boolean
  is_modified?: boolean
}

export const DEFAULT_PERSONAS: Persona[] = [
  {
    id: 'default',
    name: 'Atena Studio',
    tag: 'Default (0 tokens)',
    iconName: 'Sparkles',
    color: 'indigo',
    description: 'Pure, original model responses without additional system instructions. Saves 100% of system tokens.',
    system_prompt: ''
  },
  {
    id: 'atena_warm',
    name: 'Conversational Atena',
    tag: 'Personal & Welcoming',
    iconName: 'HeartHandshake',
    color: 'purple',
    description: 'Friendly, welcoming, and helpful assistant for daily questions, advice, and ideas.',
    system_prompt: 'You are Atena, an intelligent, friendly, helpful, and warm personal assistant. Your explanations are clear, natural, and practical. You always aim to make the user\'s daily life easier, answering with kindness, empathy, and without unnecessary jargon.'
  },
  {
    id: 'organizer_pro',
    name: 'Organizer & Routine',
    tag: 'Personal Productivity',
    iconName: 'CheckCircle2',
    color: 'teal',
    description: 'Helps organize tasks, checklists, weekly planning, and quick summaries.',
    system_prompt: 'You are a personal organization and productivity assistant. You help structure daily routines, plan goals, and organize shopping or to-do lists into clear, actionable, and easy-to-follow items.'
  },
  {
    id: 'writer_pro',
    name: 'Communication & Writing',
    tag: 'Text & Messages',
    iconName: 'PenTool',
    color: 'amber',
    description: 'Helps craft cordial emails, clear messages, and proofread texts.',
    system_prompt: 'You are an expert in empathetic communication and writing. You help draft emails, announcements, messages, formal requests, and clear texts, always adapting to the desired tone of voice.'
  },
  {
    id: 'socratic_tutor',
    name: 'Teacher & Knowledge',
    tag: 'Didactic & Learning',
    iconName: 'GraduationCap',
    color: 'sky',
    description: 'Explains any topic simply, patiently, and with relatable analogies.',
    system_prompt: 'You are a patient and didactic teacher. Your goal is to explain complex concepts in such a simple and intuitive way that anyone, regardless of age, can understand them completely.'
  },
  {
    id: 'dev_expert',
    name: 'Software Engineer',
    tag: 'Code & Systems',
    iconName: 'Code2',
    color: 'emerald',
    description: 'Specialist in programming, software development, and technical solutions.',
    system_prompt: 'You are a Senior Software Engineer specializing in Rust, TypeScript, Vue 3/Nuxt, and MLX. Your answers prioritize clean architecture, idiomatic code, high performance, and strict typing.'
  }
]

import { invoke } from '@tauri-apps/api/core'

const STORAGE_KEY = 'atena_custom_personas_v1'
const ACTIVE_PERSONA_KEY = 'atena_active_persona_id'
const OVERRIDES_STORAGE_KEY = 'atena_persona_overrides_v1'

export const customPersonas: Ref<Persona[]> = ref([])
export const personaOverrides: Ref<Record<string, Partial<Persona>>> = ref({})
export const activePersonaId: Ref<string> = ref('default')

export async function initPersonas(): Promise<void> {
  if (typeof window === 'undefined') return

  // 1. Initial immediate fast load from localStorage cache if available
  try {
    const savedCustom = localStorage.getItem(STORAGE_KEY)
    if (savedCustom) customPersonas.value = JSON.parse(savedCustom)
    const savedOverrides = localStorage.getItem(OVERRIDES_STORAGE_KEY)
    if (savedOverrides) personaOverrides.value = JSON.parse(savedOverrides)
    const savedActive = localStorage.getItem(ACTIVE_PERSONA_KEY)
    if (savedActive) activePersonaId.value = savedActive
  } catch (e) {
    console.warn('Error reading personas cache:', e)
  }

  // 2. Fetch authoritative data from SQLite database
  try {
    const dbData = await invoke<{
      custom_personas: Persona[]
      overrides: Record<string, string>
      active_persona_id?: string | null
    }>('db_get_personas_data')

    if (dbData) {
      if (Array.isArray(dbData.custom_personas) && dbData.custom_personas.length > 0) {
        customPersonas.value = dbData.custom_personas
      } else if (customPersonas.value.length > 0) {
        // Auto-migrate legacy localStorage custom personas to SQLite
        for (const p of customPersonas.value) {
          invoke('db_save_persona', { persona: p }).catch(() => {})
        }
      }

      if (dbData.overrides && Object.keys(dbData.overrides).length > 0) {
        const parsedOverrides: Record<string, Partial<Persona>> = {}
        for (const [id, prompt] of Object.entries(dbData.overrides)) {
          parsedOverrides[id] = { system_prompt: prompt }
        }
        personaOverrides.value = parsedOverrides
      } else if (Object.keys(personaOverrides.value).length > 0) {
        // Auto-migrate overrides
        for (const [id, over] of Object.entries(personaOverrides.value)) {
          if (over.system_prompt) {
            invoke('db_save_persona_override', { personaId: id, systemPrompt: over.system_prompt }).catch(() => {})
          }
        }
      }

      if (dbData.active_persona_id) {
        activePersonaId.value = dbData.active_persona_id
      }
    }
  } catch (err) {
    console.warn('Could not sync personas with SQLite backend:', err)
  }
}

export function saveCustomPersona(persona: Persona): void {
  const existingIdx = customPersonas.value.findIndex((p) => p.id === persona.id)
  if (existingIdx >= 0) {
    customPersonas.value[existingIdx] = { ...persona }
  } else {
    customPersonas.value.push({ ...persona })
  }
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(customPersonas.value))
  }
  invoke('db_save_persona', { persona }).catch((err) => {
    console.error('Failed to save persona to database:', err)
  })
}

export function deleteCustomPersona(id: string): void {
  customPersonas.value = customPersonas.value.filter((p) => p.id !== id)
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(customPersonas.value))
  }
  invoke('db_delete_persona', { personaId: id }).catch((err) => {
    console.error('Failed to delete persona from database:', err)
  })
  if (activePersonaId.value === id) {
    setActivePersona('default')
  }
}

export function updatePersona(personaData: Persona): PersonaItem | null {
  if (!personaData || !personaData.id) return null

  if (personaData.id.startsWith('custom_')) {
    saveCustomPersona(personaData)
    return personaData
  }

  // It's a default persona override
  personaOverrides.value = {
    ...personaOverrides.value,
    [personaData.id]: {
      name: personaData.name,
      tag: personaData.tag,
      iconName: personaData.iconName,
      color: personaData.color,
      description: personaData.description,
      system_prompt: personaData.system_prompt
    }
  }

  if (typeof window !== 'undefined') {
    localStorage.setItem(OVERRIDES_STORAGE_KEY, JSON.stringify(personaOverrides.value))
  }

  invoke('db_save_persona_override', {
    personaId: personaData.id,
    systemPrompt: personaData.system_prompt || ''
  }).catch((err) => {
    console.error('Failed to save persona override to database:', err)
  })

  return allPersonas.value.find((p) => p.id === personaData.id) || personaData
}

export function resetDefaultPersona(id: string): Persona | null {
  if (personaOverrides.value[id]) {
    const nextOverrides = { ...personaOverrides.value }
    delete nextOverrides[id]
    personaOverrides.value = nextOverrides
    if (typeof window !== 'undefined') {
      localStorage.setItem(OVERRIDES_STORAGE_KEY, JSON.stringify(personaOverrides.value))
    }
    invoke('db_reset_persona_override', { personaId: id }).catch((err) => {
      console.error('Failed to reset persona override in database:', err)
    })
  }
  return DEFAULT_PERSONAS.find((p) => p.id === id) || null
}

export function duplicatePersona(sourcePersona: Persona): Persona {
  const newId = `custom_${Date.now()}`
  const newPersona: Persona = {
    id: newId,
    name: `${sourcePersona.name || 'Nova Persona'} (Cópia)`,
    tag: sourcePersona.tag || 'Custom',
    iconName: sourcePersona.iconName || 'Sparkles',
    color: sourcePersona.color || 'indigo',
    description: sourcePersona.description || '',
    system_prompt: sourcePersona.system_prompt || ''
  }
  saveCustomPersona(newPersona)
  return newPersona
}

export function setActivePersona(id: string): void {
  activePersonaId.value = id
  if (typeof window !== 'undefined') {
    localStorage.setItem(ACTIVE_PERSONA_KEY, id)
  }
  invoke('db_set_setting', { key: 'active_persona_id', value: id }).catch(() => {})
}

export const allPersonas: ComputedRef<PersonaItem[]> = computed(() => {
  const defaults: PersonaItem[] = DEFAULT_PERSONAS.map((def) => {
    const override = personaOverrides.value[def.id]
    if (override) {
      return {
        ...def,
        ...override,
        is_custom: false,
        is_modified: true
      }
    }
    return {
      ...def,
      is_custom: false,
      is_modified: false
    }
  })

  const customs: PersonaItem[] = customPersonas.value.map((c) => ({
    ...c,
    is_custom: true,
    is_modified: false
  }))

  return [...defaults, ...customs]
})

export const activePersona: ComputedRef<PersonaItem> = computed(() => {
  return (
    allPersonas.value.find((p) => p.id === activePersonaId.value) ||
    allPersonas.value[0] ||
    (DEFAULT_PERSONAS[0] as PersonaItem)
  )
})
