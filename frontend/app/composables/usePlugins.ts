import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PluginViewDefinition {
  id: string
  title: string
  icon?: string
  location?: 'sidebar' | 'chat_action' | 'settings'
  description?: string
}

export interface PluginSettingOption {
  label: string
  value: any
}

export interface PluginSettingField {
  id: string
  label: string
  description?: string
  type: 'boolean' | 'text' | 'number' | 'select' | 'password'
  default?: any
  options?: PluginSettingOption[]
  min?: number
  max?: number
  step?: number
  placeholder?: string
}

export interface PluginInfo {
  id: string
  name: string
  version: string
  description: string
  author: string
  icon?: string
  enabled: boolean
  is_builtin: boolean
  category: 'core_extension' | 'ui_extension' | 'chat_enhancer' | 'tool_provider' | 'cloud_provider' | 'custom'
  hooks: string[]
  views: PluginViewDefinition[]
  directory?: string
  entry?: string
  native_entry?: string
  readme?: string
  doc?: string
  locales?: string
  repository?: string
  license?: string
  has_settings: boolean
  settings_schema?: PluginSettingField[]
  settings_values?: Record<string, any>
}

export interface ChatHookContext {
  userMessage?: string
  messages?: any[]
  params?: any
  sessionId?: string | null
  sessionTitle?: string | null
}

export interface SystemPromptHookContext {
  basePrompt: string
  userMessage?: string
  messages?: any[]
  sessionId?: string | null
}

export interface AfterChatTurnContext {
  userMessage?: string
  assistantResponse: string
  sessionId?: string | null
  sessionTitle?: string | null
  elapsedMs?: number
  metrics?: any
}

// Global state across components
const plugins = ref<PluginInfo[]>([])
const isLoadingPlugins = ref(false)
const pluginsFolder = ref('')
const isPluginsModalOpen = ref(false)

export function usePlugins() {
  const fetchPlugins = async () => {
    isLoadingPlugins.value = true
    try {
      const list = await invoke<PluginInfo[]>('list_plugins')
      plugins.value = list
      const folder = await invoke<string>('get_plugins_folder')
      pluginsFolder.value = folder
    } catch (err) {
      console.error('Falha ao carregar plugins do Atena:', err)
    } finally {
      isLoadingPlugins.value = false
    }
  }

  const togglePlugin = async (id: string, enabled: boolean) => {
    try {
      await invoke('toggle_plugin', { id, enabled })
      const target = plugins.value.find((p) => p.id === id)
      if (target) {
        target.enabled = enabled
      }
      return true
    } catch (err) {
      console.error(`Erro ao alternar status do plugin ${id}:`, err)
      return false
    }
  }

  const openPluginsFolder = async () => {
    try {
      await invoke('open_plugins_folder')
    } catch (err) {
      console.error('Erro ao abrir pasta de plugins:', err)
    }
  }

  // Verifica se o plugin de memória cognitiva está instalado e habilitado
  const isMemoryPluginActive = computed(() => {
    const memPlugin = plugins.value.find((p) => p.id === 'atena-plugin-memory')
    return memPlugin ? memPlugin.enabled : false
  })

  // Verifica se o conector do Antigravity (AGY) está habilitado
  const isAgyPluginActive = computed(() => {
    const agyPlugin = plugins.value.find((p) => p.id === 'atena-plugin-agy')
    return agyPlugin ? agyPlugin.enabled : false
  })

  // Executa comando em binário nativo de plugin em Rust (via stdin/stdout IPC seguro)
  const invokePluginNative = async <T = any>(pluginId: string, command: string, payload?: any): Promise<T> => {
    return await invoke<T>('run_plugin_native', {
      pluginId,
      command,
      payload: payload ?? null
    })
  }

  // Carrega o código fonte JS/TS do plugin para extensão de UI dinâmica
  const loadPluginScript = async (pluginId: string): Promise<string> => {
    return await invoke<string>('read_plugin_script', { pluginId })
  }

  // Retorna as telas de navegação registradas por plugins ativos
  const activePluginSidebarViews = computed(() => {
    const views: (PluginViewDefinition & { pluginId: string })[] = []
    for (const plugin of plugins.value) {
      if (plugin.enabled && plugin.views) {
        for (const view of plugin.views) {
          if (view.location === 'sidebar' || !view.location) {
            views.push({
              ...view,
              pluginId: plugin.id
            })
          }
        }
      }
    }
    return views
  })

  // Hook 1: Before Chat — chamado antes de enviar mensagem para a inferência
  const runBeforeChatHooks = async (context: ChatHookContext) => {
    let ctx = { ...context }
    for (const plugin of plugins.value) {
      if (plugin.enabled && plugin.hooks?.includes('before_chat')) {
        // Ponto de extensão para middlewares de chat
        console.debug(`[Plugin:${plugin.id}] executando before_chat hook`)
      }
    }
    return ctx
  }

  // Hook 2: On System Prompt — chamado para enriquecer instruções do modelo
  const runOnSystemPromptHooks = async (context: SystemPromptHookContext): Promise<string> => {
    let enrichedPrompt = context.basePrompt || ''
    for (const plugin of plugins.value) {
      if (plugin.enabled && plugin.hooks?.includes('on_system_prompt')) {
        console.debug(`[Plugin:${plugin.id}] executando on_system_prompt hook`)
      }
    }
    return enrichedPrompt
  }

  // Hook 3: On Chunk — chamado para cada token streaming recebido
  const runOnChunkHooks = (chunk: any, context: ChatHookContext) => {
    for (const plugin of plugins.value) {
      if (plugin.enabled && plugin.hooks?.includes('on_chunk')) {
        // Interceptador de streaming
      }
    }
  }

  // Hook 4: After Chat Turn — chamado ao concluir a resposta
  const runAfterChatTurnHooks = async (context: AfterChatTurnContext) => {
    for (const plugin of plugins.value) {
      if (plugin.enabled && plugin.hooks?.includes('after_chat_turn')) {
        console.debug(`[Plugin:${plugin.id}] executando after_chat_turn hook`)
      }
    }
  }

  const savePluginSettings = async (pluginId: string, settings: Record<string, any>) => {
    try {
      await invoke('save_plugin_settings', { pluginId, settings })
      const target = plugins.value.find((p) => p.id === pluginId)
      if (target) {
        target.settings_values = { ...(target.settings_values || {}), ...settings }
      }
      return true
    } catch (err) {
      console.error(`Erro ao salvar configurações do plugin ${pluginId}:`, err)
      return false
    }
  }

  const updatePluginSetting = async (pluginId: string, key: string, value: any) => {
    const target = plugins.value.find((p) => p.id === pluginId)
    const current = { ...(target?.settings_values || {}) }
    current[key] = value
    return await savePluginSettings(pluginId, current)
  }

  // Plugins ativos que possuem configurações ou abas em Ajustes (excluindo provedores de nuvem que ficam na tela unificada)
  const activePluginSettingsViews = computed(() => {
    return plugins.value.filter((p) => {
      if (!p.enabled) return false
      if (p.category === 'cloud_provider' || p.id === 'atena-plugin-agy' || p.id === 'atena-plugin-cloud') return false
      const hasSchema = p.settings_schema && p.settings_schema.length > 0
      const hasSettingsView = p.views && p.views.some((v) => v.location === 'settings')
      return hasSchema || hasSettingsView
    })
  })

  return {
    plugins,
    isLoadingPlugins,
    pluginsFolder,
    isPluginsModalOpen,
    fetchPlugins,
    togglePlugin,
    openPluginsFolder,
    savePluginSettings,
    updatePluginSetting,
    activePluginSettingsViews,
    isMemoryPluginActive,
    isAgyPluginActive,
    invokePluginNative,
    loadPluginScript,
    activePluginSidebarViews,
    runBeforeChatHooks,
    runOnSystemPromptHooks,
    runOnChunkHooks,
    runAfterChatTurnHooks
  }
}
