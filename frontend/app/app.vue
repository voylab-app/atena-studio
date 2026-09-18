<template>
  <div class="h-screen w-screen flex bg-[#0f1117] text-[#f0f4ff] font-sans overflow-hidden select-none">
    <!-- Left Sidebar -->
    <Sidebar
      ref="sidebarRef"
      v-model:activeTab="activeTab"
      :sessions="sessions"
      :activeSessionId="activeSessionId"
      :hardware="hardware"
      :activeModel="activeModel"
      :enableMemory="Boolean(config.enable_cognitive_memory)"
      @selectSession="selectSession"
      @newChat="createNewSession"
      @newProjectChat="createNewProjectSession"
      @newPrivateChat="createNewSession(true)"
      @deleteSession="requestDeleteSession"
      @deleteSessions="requestDeleteSessions"
      @archiveSession="handleArchiveSession"
      @archiveSessions="handleArchiveSessions"
      @openArchivedModal="isArchivedModalOpen = true"
      @renameSession="handleRenameSession"
      @togglePinSession="handleTogglePinSession"
      @assignProject="handleAssignSessionProject"
      @deleteProject="handleDeleteProject"
      @openSetup="isSetupModalOpen = true"
      @openPlugins="isPluginsModalOpen = true"
    />

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full overflow-hidden">
      <!-- Top Header -->
      <Header :activeModel="activeModel" :hardware="hardware" :isDrawerOpen="isDrawerOpen"
        :isLoadingModel="isLoadingModel" :loadingModelName="loadingModelName"
        :loadingModelProgress="loadingModelProgress"
        :agyUsageSummary="agyUsageSummary" :currentSession="currentSession"
        :isDark="isDarkMode" :theme="theme"
        @toggleDrawer="isDrawerOpen = !isDrawerOpen" @unloadModel="unloadModel"
        @openAgyModal="isAgyLoginModalOpen = true" @toggleTheme="handleToggleTheme" />

      <!-- Active Screen View -->
      <div class="flex-1 overflow-hidden relative">
        <ChatScreen v-show="activeTab === 'chat'" :currentSession="currentSession" :activeModel="activeModel"
          :allModels="allModels" :isGenerating="isGenerating" :params="params" :config="config" :activeMcpToolsCount="activeMcpToolsCount"
          :showEfficiencyMetrics="config.show_efficiency_metrics !== false"
          :mcpTools="mcpTools"
          :projects="projects"
          @sendMessage="handleSendMessage" @resendMessage="handleResendMessage" @stopGeneration="handleStopGeneration" @clearChat="handleClearChat" @deleteMessage="handleDeleteMessage"
          @approveTool="handleApproveTool" @rejectTool="handleRejectTool"
          @reExecuteTool="handleReExecuteTool"
          @approveSelectedTools="handleApproveSelectedTools" @rejectSelectedTools="handleRejectSelectedTools"
          @approveAllTools="handleApproveAllTools" @rejectAllTools="handleRejectAllTools"
          @loadModel="loadModel"
          @assignProject="handleAssignSessionProject"
          @refreshTools="fetchMcpTools" @selectTab="activeTab = $event"
          @openParams="isDrawerOpen = true"
          @unarchiveSession="handleUnarchiveSession" />

        <ModelsScreen v-show="activeTab === 'models'" :models="allModels" :activeModelId="activeModel?.id"
          :loadingModelId="loadingModelId" :isLoadingModel="isLoadingModel"
          :loadingModelProgress="loadingModelProgress" :modelsDir="config.models_directory"
          :isScanning="isScanning" :activeDownloadsCount="runningDownloadsCount" :params="params"
          :agyUsageSummary="agyUsageSummary" :config="config"
          @loadModel="loadModel" @unloadModel="unloadModel" @rescanModels="scanModels"
          @openHfModal="isHfModalOpen = true" @selectFolder="handleSelectFolder"
          @openAgyModal="isAgyLoginModalOpen = true"
          @openSettings="handleOpenSettings"
          @hideModel="handleHideCloudModel" />

        <MemoryScreen
          v-if="Boolean(config.enable_cognitive_memory)"
          v-show="activeTab === 'memory'"
          :is-active="activeTab === 'memory'"
          :config="config"
          @saveConfig="handleSaveConfig"
        />

        <SettingsScreen v-show="activeTab === 'settings'" :config="config" :params="params"
          :serviceHealth="serviceHealth" :logs="logs" :hardware="hardware" :currentTheme="theme"
          :initialSection="settingsInitialSection" :sessions="sessions"
          @saveConfig="handleSaveConfig" @setTheme="handleSetTheme" @setLanguage="handleSetLanguage"
          @checkServices="checkServices" @startMlx="startMlx" @startOllama="startOllama"
          @stopAllServers="stopAllServers" @clearLogs="clearLogs" @refreshLogs="fetchLogs" @selectFolder="handleSelectFolder"
          @refreshMcp="fetchMcpTools" @openAgyLogin="isAgyLoginModalOpen = true" @openSetup="isSetupModalOpen = true"
          @refreshModels="scanModels" @openArchivedModal="isArchivedModalOpen = true" />
      </div>
    </main>

    <!-- Inference Parameters Drawer -->
    <ParamsDrawer v-model="isDrawerOpen" :params="params" />

    <!-- Hugging Face Model Hub & Downloader Modal -->
    <HfDownloadModal
      v-if="isHfModalOpen"
      :modelsDir="config.models_directory"
      :modelsDirs="config.models_directories"
      :hardware="hardware"
      :localModels="allModels"
      @close="isHfModalOpen = false"
      @selectFolder="handleSelectFolder"
      @setDestination="handleSetDownloadDestination"
      @modelDownloaded="scanModels"
      @loadModel="loadModel"
    />

    <!-- First-Run Onboarding Setup Modal -->
    <SetupModal
      v-if="isSetupModalOpen"
      :config="config"
      @close="isSetupModalOpen = false"
      @openAgyLogin="isAgyLoginModalOpen = true"
      @saveConfig="handleSaveConfig"
      @setupComplete="scanModels"
    />

    <!-- Google Antigravity (AGY) Login & Session Modal -->
    <AgyLoginModal
      v-if="isAgyLoginModalOpen"
      @close="() => { isAgyLoginModalOpen = false; fetchAgyUsage(); }"
      @sessionUpdated="() => { scanModels(); fetchAgyUsage(); }"
    />

    <!-- Delete Session & Memory Confirmation Modal -->
    <DeleteSessionModal
      :isOpen="isDeleteSessionModalOpen"
      :session="sessionToDelete"
      @close="closeDeleteSessionModal"
      @confirm="handleConfirmDeleteSession"
    />

    <!-- Plugins & Extensions Manager Modal -->
    <PluginsModal
      v-if="isPluginsModalOpen"
      @close="isPluginsModalOpen = false"
    />

    <!-- Archived Chats Modal -->
    <ArchivedChatsModal
      :isOpen="isArchivedModalOpen"
      :sessions="sessions"
      :projects="projects"
      @close="isArchivedModalOpen = false"
      @openSession="handleOpenArchivedSession"
      @unarchiveSession="handleUnarchiveSession"
      @unarchiveSessions="handleUnarchiveSessions"
      @deleteSession="requestDeleteSession"
    />

    <!-- Folder Path Selector Modal (Web Mode Fallback) -->
    <SelectFolderModal />

    <!-- Floating Toast Notifications Stack -->
    <div class="fixed bottom-5 right-5 z-50 flex flex-col gap-2 pointer-events-none select-none max-w-sm">
      <TransitionGroup
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="transform translate-y-4 opacity-0"
        enter-to-class="transform translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="transform translate-y-0 opacity-100"
        leave-to-class="transform translate-y-4 opacity-0"
      >
        <div
          v-for="toast in activeToasts"
          :key="toast.toastId"
          class="pointer-events-auto p-3 rounded-2xl bg-[#0f121e]/95 border border-[#252c48] shadow-2xl backdrop-blur-md flex items-start gap-3 text-slate-100 text-xs"
        >
          <div
            class="w-6 h-6 rounded-lg flex items-center justify-center flex-shrink-0 mt-0.5"
            :class="[
              toast.type === 'download_completed'
                ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30'
                : toast.type === 'download_cancelled'
                ? 'bg-slate-500/15 text-slate-400 border border-slate-500/30'
                : toast.type === 'download_error'
                ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30'
                : toast.type === 'model_deleted'
                ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30'
                : toast.type === 'model_favorited'
                ? 'bg-amber-500/15 text-amber-400 border border-amber-500/30'
                : 'bg-indigo-500/15 text-indigo-400 border border-indigo-500/30'
            ]"
          >
            <CheckCircle2 v-if="toast.type === 'download_completed'" class="w-3.5 h-3.5" />
            <Ban v-else-if="toast.type === 'download_cancelled'" class="w-3.5 h-3.5" />
            <AlertTriangle v-else-if="toast.type === 'download_error'" class="w-3.5 h-3.5" />
            <Trash2 v-else-if="toast.type === 'model_deleted'" class="w-3.5 h-3.5" />
            <Star v-else-if="toast.type === 'model_favorited'" class="w-3.5 h-3.5" />
            <Info v-else class="w-3.5 h-3.5" />
          </div>
          <div class="flex-1 truncate">
            <h5 class="font-bold text-xs text-slate-100 truncate">{{ toast.title }}</h5>
            <p class="text-[11px] text-slate-400 mt-0.5 line-clamp-2 leading-snug">{{ toast.message }}</p>
          </div>
          <button
            @click="dismissToast(toast.toastId)"
            class="text-slate-500 hover:text-white p-0.5 cursor-pointer"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { initIpcBridge } from '~/utils/ipcAdapter'
initIpcBridge()

import { ref, computed, watch, onMounted, onUnmounted, provide } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import {
  CheckCircle2,
  Ban,
  AlertTriangle,
  Trash2,
  Star,
  Info,
  X
} from 'lucide-vue-next'

import Sidebar from './components/Sidebar.vue'
import Header from './components/Header.vue'
import ParamsDrawer from './components/ParamsDrawer.vue'
import ChatScreen from './components/screens/ChatScreen.vue'
import ModelsScreen from './components/screens/ModelsScreen.vue'
import MemoryScreen from './components/screens/MemoryScreen.vue'
import SettingsScreen from './components/screens/SettingsScreen.vue'
import HfDownloadModal from './components/HfDownloadModal.vue'
import SetupModal from './components/SetupModal.vue'
import AgyLoginModal from './components/AgyLoginModal.vue'
import DeleteSessionModal from './components/DeleteSessionModal.vue'
import PluginsModal from './components/PluginsModal.vue'
import ArchivedChatsModal from './components/ArchivedChatsModal.vue'
import SelectFolderModal from './components/SelectFolderModal.vue'
import { getTemporalContextPrompt, getDefaultTimezone, formatUserMessageTime } from '~/utils/dateContext'
import { contractUserPath } from '~/utils/pathUtils'
import { activeToasts, addNotification, dismissToast } from '~/utils/notifications'
import { initPersonas } from '~/utils/personas'
import { useAppLocale } from './composables/useLocale'
import { usePlugins } from './composables/usePlugins'
import { useProjects } from './composables/useProjects'
import type { ChatMessage, ChatSession, ModelInfo, HardwareInfo, GenerationParams, AppConfig, McpToolWithServer, ServerRequestLog } from '~/types'

const {
  fetchPlugins,
  isPluginsModalOpen,
  isMemoryPluginActive,
  runBeforeChatHooks,
  runOnSystemPromptHooks,
  runOnChunkHooks,
  runAfterChatTurnHooks
} = usePlugins()

const {
  activeProjectId,
  selectProject,
  projects,
  activeProject
} = useProjects()

// Navigation
const activeTab = ref('chat')
const settingsInitialSection = ref('general')
const handleOpenSettings = (section = 'general') => {
  activeTab.value = 'settings'
  settingsInitialSection.value = section
}
const isDrawerOpen = ref(false)
const isHfModalOpen = ref(false)
const isSetupModalOpen = ref(false)
const isAgyLoginModalOpen = ref(false)
const isDeleteSessionModalOpen = ref(false)
const isArchivedModalOpen = ref(false)
const sessionToDelete = ref<ChatSession | ChatSession[] | null>(null)
const activeDownloads = ref<any[]>([])
const runningDownloadsCount = computed(() => {
  return activeDownloads.value.filter((d) => d.status === 'downloading').length
})

// State
const getInitialCachedModels = () => {
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      const cached = localStorage.getItem('atena_cached_models')
      if (cached) {
        const parsed = JSON.parse(cached)
        if (Array.isArray(parsed) && parsed.length > 0) {
          return parsed
        }
      }
    } catch (_) {}
  }
  return []
}

const isModelIdDisabled = (modelId: string | null | undefined, disabledList?: string[]) => {
  if (!modelId || !Array.isArray(disabledList) || disabledList.length === 0) return false
  const cleanId = String(modelId).trim()
  const rawId = cleanId.includes('/') ? cleanId.split('/').slice(1).join('/') : cleanId

  return disabledList.some((d) => {
    if (!d) return false
    const cleanD = String(d).trim()
    const rawD = cleanD.includes('/') ? cleanD.split('/').slice(1).join('/') : cleanD

    if (cleanId === cleanD || rawId === rawD) return true

    if (
      (cleanId.startsWith('agy/') || cleanId.startsWith('gemini/')) &&
      (cleanD.startsWith('agy/') || cleanD.startsWith('gemini/'))
    ) {
      const baseId = cleanId.replace(/-(high|medium|low)$/i, '')
      const baseD = cleanD.replace(/-(high|medium|low)$/i, '')
      if (baseId === baseD) return true
    }

    return false
  })
}

const models = ref<ModelInfo[]>(getInitialCachedModels())
const allModels = computed(() => {
  const disabled = config.value?.cloud_providers?.disabled_models
  if (!Array.isArray(disabled) || disabled.length === 0) {
    return models.value
  }
  return models.value.filter((m) => !isModelIdDisabled(m.id, disabled))
})
const activeModel = ref<ModelInfo | null>(null)
const isLoadingModel = ref(false)
const loadingModelId = ref<string | null>(null)
const loadingModelName = ref('')
const loadingModelProgress = ref(0)
const isScanning = ref(false)
const isGenerating = ref(false)
const logs = ref<ServerRequestLog[]>([])
const mcpTools = ref<McpToolWithServer[]>([])


const platformInfo = ref({
  os: 'macos',
  supports_mlx: true,
  supports_whisper_local: true
})
const supportsMlx = computed(() => platformInfo.value.supports_mlx)

provide('platformInfo', platformInfo)
provide('supportsMlx', supportsMlx)

const serviceHealth = ref({
  mlx_online: false,
  ollama_online: false
})

const hardware = ref<HardwareInfo>({
  chip_name: 'Apple Silicon',
  gpu_cores: 16,
  metal_version: 'Metal 3',
  total_vram_gb: 16.0,
  used_vram_gb: 0.0,
  system_ram_gb: 16.0,
  used_system_ram_gb: 0.0
})

const params = ref<GenerationParams>({
  temperature: 0.7,
  top_p: 0.9,
  max_tokens: 4096,
  context_length: 8192,
  system_prompt: '',
  enable_thinking: true,
  thinking_budget: 2048,
  kv_cache_quant: 'q8_0',
  enable_prompt_cache: true,
  flash_attention: true
})

const { locale: appLocale, setLocale: setAppLocale, t } = useAppLocale()

const config = ref<AppConfig>({
  models_directory: '~/.atena/models',
  models_directories: ['~/.atena/models'],
  mlx_host: '127.0.0.1',
  mlx_port: 8080,
  ollama_host: '127.0.0.1',
  ollama_port: 11434,
  auto_load_last_model: true,
  timezone: getDefaultTimezone(),
  inject_current_date: false,
  inject_message_time: false,
  show_efficiency_metrics: true,
  enable_cognitive_memory: false,
  theme: 'dark',
  language: 'pt-BR',
  guardrail_mode: 'relaxed',
  guardrail_custom_limit_gb: 4,
  whisper_model: 'mlx-community/whisper-small-mlx',
  whisper_language: 'pt'
})

// Theme management (dark | light | system)
const theme = ref('dark')
const isDarkMode = ref(true)

let systemMediaWatcher: any = null

const onSystemThemeChange = (e: MediaQueryListEvent | MediaQueryList) => {
  if (theme.value === 'system') {
    applyThemeDom((e as any).matches ? 'dark' : 'light')
  }
}

const updateSystemThemeListener = () => {
  if (typeof window === 'undefined') return
  if (systemMediaWatcher) {
    try {
      systemMediaWatcher.removeEventListener('change', onSystemThemeChange)
    } catch (_) {}
    systemMediaWatcher = null
  }
  if (theme.value === 'system' && window.matchMedia) {
    systemMediaWatcher = window.matchMedia('(prefers-color-scheme: dark)')
    systemMediaWatcher.addEventListener('change', onSystemThemeChange)
  }
}

const applyThemeDom = (resolvedTheme: string) => {
  const isDark = resolvedTheme === 'dark'
  isDarkMode.value = isDark
  if (typeof document !== 'undefined') {
    const root = document.documentElement
    if (isDark) {
      root.classList.remove('light')
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
      root.classList.add('light')
    }
  }
}

const resolveTheme = (t: string) => {
  if (t === 'system') {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return 'dark'
  }
  return t === 'light' ? 'light' : 'dark'
}

const applyTheme = (newTheme?: string) => {
  const targetTheme = newTheme || 'dark'
  theme.value = targetTheme
  if (typeof window !== 'undefined') {
    localStorage.setItem('atena_theme', targetTheme)
  }
  config.value.theme = targetTheme
  updateSystemThemeListener()
  applyThemeDom(resolveTheme(targetTheme))
}

const handleToggleTheme = () => {
  const next = isDarkMode.value ? 'light' : 'dark'
  applyTheme(next)
}

const handleSetTheme = (newTheme: string) => {
  applyTheme(newTheme)
}

const handleSetLanguage = (newLang: string) => {
  setAppLocale(newLang)
  config.value.language = newLang
  handleSaveConfig(config.value)
}

provide('theme', theme)
provide('isDarkMode', isDarkMode)

// Chat Sessions
const sessions = ref<ChatSession[]>([
  {
    id: 'session-default-1',
    title: t('sidebar.new_chat'),
    created_at: new Date().toISOString(),
    messages: []
  }
])
const activeSessionId = ref('session-default-1')

// Sanitize any existing in-memory sessions on module load
sessions.value?.forEach?.((s) => {
  s.messages?.forEach?.((m) => {
    if (m.is_streaming) m.is_streaming = false
    m.tool_calls?.forEach?.((tc) => {
      if (tc.status === 'executing') tc.status = 'pending_approval'
    })
  })
})

const currentSession = computed<ChatSession>(() => {
  return sessions.value.find((s) => s.id === activeSessionId.value) || sessions.value[0]!
})

// Antigravity (AGY) Usage and Quota Summary
const loadCachedAgyUsage = () => {
  try {
    const raw = localStorage.getItem('atena_agy_usage_summary_cache')
    if (raw) return JSON.parse(raw)
  } catch (e) {}
  return null
}

interface AgyUsageSummary {
  authenticated: boolean
  weeklyPercent: number | null
  fiveHourPercent: number | null
  gemini5hPercent: number | null
  geminiWeeklyPercent: number | null
  claude5hPercent: number | null
  claudeWeeklyPercent: number | null
  lowest5hResetTime: string | null
  lowest5hCountdown: string
  lowestWeeklyResetTime: string | null
  lowestWeeklyCountdown: string
  activeAccount: string | null
  gemini5hResetTime?: string | null
  gemini5hCountdown?: string
  claude5hResetTime?: string | null
  claude5hCountdown?: string
  geminiWeeklyResetTime?: string | null
  geminiWeeklyCountdown?: string
  claudeWeeklyResetTime?: string | null
  claudeWeeklyCountdown?: string
  [key: string]: any
}

const defaultAgySummary: AgyUsageSummary = {
  authenticated: false,
  weeklyPercent: null,
  fiveHourPercent: null,
  gemini5hPercent: null,
  geminiWeeklyPercent: null,
  claude5hPercent: null,
  claudeWeeklyPercent: null,
  lowest5hResetTime: null,
  lowest5hCountdown: '',
  lowestWeeklyResetTime: null,
  lowestWeeklyCountdown: '',
  activeAccount: null
}

const agyUsageSummary = ref<AgyUsageSummary>(loadCachedAgyUsage() || { ...defaultAgySummary })

let agyUsageTimer: any = null

const calculateCountdown = (resetTimeStr?: string | null) => {
  if (!resetTimeStr) return ''
  try {
    const diffMs = new Date(resetTimeStr).getTime() - Date.now()
    if (diffMs <= 0) return '0m'
    const totalMinutes = Math.floor(diffMs / 60000)
    const hours = Math.floor(totalMinutes / 60)
    const days = Math.floor(hours / 24)
    const remHours = hours % 24
    const mins = totalMinutes % 60
    if (days > 0) return `${days}d ${remHours}h`
    if (hours > 0) return `${hours}h ${mins}m`
    return `${mins}m`
  } catch {
    return ''
  }
}

const fetchAgyUsage = async () => {
  try {
    const session = await invoke<any>('check_agy_session')
    if (session && session.authenticated) {
      const usage = await invoke<any>('get_agy_usage')
      if (usage && usage.groups) {
        // Reset all dynamic fields to avoid retaining stale values (e.g. non-existent 5h buckets)
        const newSummary: AgyUsageSummary = {
          ...defaultAgySummary,
          authenticated: true,
          activeAccount: session.active_account || null
        }

        let lowestWeeklyFraction: number | null = null
        let lowestWeeklyReset: string | null = null
        let lowest5hFraction: number | null = null
        let lowest5hReset: string | null = null

        for (const group of usage.groups) {
          const groupLower = (group.name || '').toLowerCase()
          const isGeminiGroup = groupLower.includes('gemini')
          const isClaudeGroup = groupLower.includes('claude') || groupLower.includes('gpt')

          for (const bucket of group.buckets || []) {
            const frac = typeof bucket.remaining_fraction === 'number' ? bucket.remaining_fraction : 1
            const win = (bucket.window || '').toLowerCase()

            if (win.includes('5h') || win.includes('hour')) {
              if (lowest5hFraction === null || frac < lowest5hFraction) {
                lowest5hFraction = frac
                lowest5hReset = bucket.reset_time
              }
              if (isGeminiGroup) {
                newSummary.gemini5hPercent = Math.round(frac * 100)
                newSummary.gemini5hResetTime = bucket.reset_time
                newSummary.gemini5hCountdown = calculateCountdown(bucket.reset_time)
              } else if (isClaudeGroup) {
                newSummary.claude5hPercent = Math.round(frac * 100)
                newSummary.claude5hResetTime = bucket.reset_time
                newSummary.claude5hCountdown = calculateCountdown(bucket.reset_time)
              }
            } else {
              // Weekly or general window
              if (lowestWeeklyFraction === null || frac < lowestWeeklyFraction) {
                lowestWeeklyFraction = frac
                lowestWeeklyReset = bucket.reset_time
              }
              if (isGeminiGroup) {
                newSummary.geminiWeeklyPercent = Math.round(frac * 100)
                newSummary.geminiWeeklyResetTime = bucket.reset_time
                newSummary.geminiWeeklyCountdown = calculateCountdown(bucket.reset_time)
              } else if (isClaudeGroup) {
                newSummary.claudeWeeklyPercent = Math.round(frac * 100)
                newSummary.claudeWeeklyResetTime = bucket.reset_time
                newSummary.claudeWeeklyCountdown = calculateCountdown(bucket.reset_time)
              }
            }
          }
        }

        if (lowestWeeklyFraction !== null) {
          newSummary.weeklyPercent = Math.round(lowestWeeklyFraction * 100)
          newSummary.lowestWeeklyResetTime = lowestWeeklyReset
          newSummary.lowestWeeklyCountdown = calculateCountdown(lowestWeeklyReset)
        }
        if (lowest5hFraction !== null) {
          newSummary.fiveHourPercent = Math.round(lowest5hFraction * 100)
          newSummary.lowest5hResetTime = lowest5hReset
          newSummary.lowest5hCountdown = calculateCountdown(lowest5hReset)
        }

        agyUsageSummary.value = newSummary
        try {
          localStorage.setItem('atena_agy_usage_summary_cache', JSON.stringify(newSummary))
        } catch (e) {}
      } else {
        agyUsageSummary.value.authenticated = true
        agyUsageSummary.value.activeAccount = session.active_account || null
      }
    } else {
      agyUsageSummary.value = { ...defaultAgySummary, authenticated: false }
      try {
        localStorage.removeItem('atena_agy_usage_summary_cache')
      } catch (e) {}
    }
  } catch (err) {
    // Non-critical background telemetry
  }
}

// Hardware polling
let hardwareTimer: any = null

const fetchHardware = async () => {
  try {
    const info = await invoke<any>('get_hardware_info')
    if (info) hardware.value = info
  } catch (err) {
    // Handled silently
  }
}

const checkServices = async () => {
  try {
    const health = await invoke<any>('check_services', {
      mlxHost: config.value.mlx_host,
      mlxPort: config.value.mlx_port,
      ollamaHost: config.value.ollama_host,
      ollamaPort: config.value.ollama_port
    })
    if (health) serviceHealth.value = health
    await fetchLogs()
  } catch (err) {
    console.error('Failed to check services:', err)
  }
}

let isScanInProgress = false

const scanModels = async () => {
  if (isScanInProgress) return
  isScanInProgress = true
  isScanning.value = true
  try {
    const list = await invoke('scan_models', {
      modelsDir: config.value.models_directory,
      modelsDirs: config.value.models_directories,
      ollamaHost: config.value.ollama_host,
      ollamaPort: config.value.ollama_port
    })
    if (Array.isArray(list)) {
      models.value = list
      try {
        localStorage.setItem('atena_cached_models', JSON.stringify(list))
      } catch (_) {}
    }

    // Restore saved model ONLY if user had explicitly selected it before and auto_load is enabled
    if (config.value.auto_load_last_model && !activeModel.value) {
      const savedModelId = localStorage.getItem('atena_active_model_id')
      if (savedModelId) {
        const match = allModels.value.find((m) => m.id === savedModelId)
        if (match) {
          await loadModel(match)
        }
      }
    }
  } catch (err) {
    console.error('Failed to scan models:', err)
  } finally {
    isScanning.value = false
    isScanInProgress = false
  }
}

// Helper: Checks memory guardrails and safely adapts context length to prevent system freeze on large models/prompts
const resolveSafeContextAndCheckGuardrails = (model: any) => {
  if (!model) return { allowed: true, contextLength: 8192 }
  const isAgy = model.backend === 'Antigravity' || model.format === 'Agy' || (model.id && model.id.startsWith('agy/'))
  const isCloud = model.backend === 'CloudOpenAi' || model.format === 'Cloud' || (model.id && (model.id.startsWith('openai/') || model.id.startsWith('openrouter/') || model.id.startsWith('gemini/') || model.id.startsWith('groq/') || model.id.startsWith('custom/')))
  if (isAgy || isCloud) {
    return {
      allowed: true,
      contextLength: params.value.context_length || model.context_length || 8192
    }
  }
  const mode = config.value.guardrail_mode || 'relaxed'
  if (mode === 'off') {
    return {
      allowed: true,
      contextLength: params.value.context_length || model.context_length || 8192
    }
  }

  const ram = Number(hardware.value?.total_ram_gb) || Number(hardware.value?.system_ram_gb) || 16.0
  let limitGb
  switch (mode) {
    case 'relaxed':
      limitGb = ram * 0.88
      break
    case 'balanced':
      limitGb = ram * 0.75
      break
    case 'strict':
      limitGb = ram * 0.60
      break
    case 'custom':
      limitGb = Number(config.value.guardrail_custom_limit_gb) || 4.0
      break
    default:
      limitGb = ram * 0.75
  }

  const modelSizeGb = Number(model.size_gb) || (model.size ? model.size / (1024 * 1024 * 1024) : 4.0)

  // 1. If model weights alone exceed the safety limit
  if (modelSizeGb > limitGb) {
    const proceed = window.confirm(
      t('chat.memory_protection_warning', {
        size: modelSizeGb.toFixed(1),
        limit: limitGb.toFixed(1),
        mode: mode.toUpperCase()
      })
    )
    if (!proceed) {
      return { allowed: false, contextLength: 4096 }
    }
  }

  // 2. Determine safe context length prioritizing user-configured params.context_length
  const requestedContext = params.value.context_length || 8192

  // Available room for KV cache & prompt prefill buffers (in GB)
  const remainingGbForKv = Math.max(0.5, limitGb - modelSizeGb)

  // Estimate KV cache size: ~0.12 GB per 1024 tokens for standard 7B-14B models in 8-bit/16-bit
  const maxSafeTokensByMemory = Math.floor((remainingGbForKv / 0.12) * 1024)
  
  // Safe clamped context
  let safeContext = Math.min(requestedContext, Math.max(2048, maxSafeTokensByMemory))
  
  // Round to multiple of 512
  safeContext = Math.floor(safeContext / 512) * 512

  if (safeContext < requestedContext) {
    console.warn(`[Guardrails] Context adjusted from ${requestedContext} to ${safeContext} tokens to keep memory within safe ceiling of ${limitGb.toFixed(1)} GB.`)
  }

  return {
    allowed: true,
    contextLength: safeContext
  }
}

const loadModel = async (model: any) => {
  if (isLoadingModel.value) {
    console.warn('[LoadModel] ⚠️ Model load attempt ignored: another load is already in progress.', { currentLoadingId: loadingModelId.value })
    return
  }

  const loadStart = performance.now()

  // Early detection for AGY / Cloud models — before any invoke or I/O
  const isAgyEarly = model.backend === 'Antigravity' || model.format === 'Agy' || (model.id && model.id.startsWith('agy/'))
  const isCloudEarly = model.backend === 'CloudOpenAi' || model.format === 'Cloud' || (model.id && (model.id.startsWith('openai/') || model.id.startsWith('openrouter/') || model.id.startsWith('gemini/') || model.id.startsWith('groq/') || model.id.startsWith('custom/')))
  const isRemoteModel = isAgyEarly || isCloudEarly

  console.log('[LoadModel] ▶ Starting model load:', {
    id: model.id,
    name: model.name,
    backend: model.backend,
    format: model.format,
    is_remote: isRemoteModel,
    is_agy: isAgyEarly,
    is_cloud: isCloudEarly,
  })

  // ═══════════════════════════════════════════════════════════════════
  // FAST-PATH: AGY Models (Antigravity) and Cloud Providers (OpenAI, OpenRouter, Gemini, Custom)
  // ═══════════════════════════════════════════════════════════════════
  if (isRemoteModel) {
    const remoteProviderName = isAgyEarly ? 'Antigravity' : (model.backend || 'Cloud')
    console.log(`[LoadModel] ⚡ Fast-path activated for ${remoteProviderName} model: ${model.id}`)
    isLoadingModel.value = true
    loadingModelId.value = model.id
    loadingModelName.value = model.name
    loadingModelProgress.value = 50

    try {
      if (activeModel.value && (activeModel.value.backend === 'Mlx' || activeModel.value.backend === 'MlxLm' || activeModel.value.format === 'Mlx' || activeModel.value.format === 'Gguf' || activeModel.value.backend === 'Ollama')) {
        console.log('[LoadModel] Stopping running local servers before activating AGY/Cloud model...')
        await invoke('stop_all_servers')
      }

      console.log('[LoadModel] Registering AGY/Cloud model via set_active_model...', { id: model.id })
      const finalModel = await invoke<any>('set_active_model', { model })

      const active = finalModel || model
      activeModel.value = active
      localStorage.setItem('atena_active_model_id', active.id)
      localStorage.setItem('atena_last_loaded_model_id', active.id)
      try {
        const raw = localStorage.getItem('atena_model_last_loaded_times')
        const times = raw ? JSON.parse(raw) : {}
        times[active.id] = Date.now()
        localStorage.setItem('atena_model_last_loaded_times', JSON.stringify(times))
      } catch (e) {
        console.error('[LoadModel] Failed to save loaded model timestamp:', e)
      }

      const totalMs = (performance.now() - loadStart).toFixed(0)
      console.log(`[LoadModel] ✅ AGY/Cloud model "${active.name}" (${active.id}) successfully activated in ${totalMs}ms`)
    } catch (err) {
      console.error('[LoadModel] ❌ Error activating AGY/Cloud model:', {
        error: err,
        message: typeof err === 'string' ? err : (err as any)?.message,
        model_id: model.id,
        model_name: model.name,
        backend: model.backend,
        format: model.format,
        elapsed_ms: (performance.now() - loadStart).toFixed(0),
      })
      alert(`Error activating model: ${err}`)
    } finally {
      isLoadingModel.value = false
      loadingModelId.value = null
      loadingModelName.value = ''
      loadingModelProgress.value = 0
      console.log('[LoadModel] 🔄 Loading state reset (fast-path).')
    }
    return
  }

  // ═══════════════════════════════════════════════════════════════════
  // STANDARD PATH: Local Models (GGUF, MLX, Ollama)
  // ═══════════════════════════════════════════════════════════════════

  // Clean / refresh cache of the selected model directly from disk
  let currentModel = model
  try {
    console.log('[LoadModel] [1/6] Refreshing model info via refresh_model_info...')
    const refreshStart = performance.now()
    const refreshed = await invoke<any>('refresh_model_info', { model })
    console.log(`[LoadModel] [1/6] ✓ refresh_model_info completed in ${(performance.now() - refreshStart).toFixed(0)}ms`, {
      refreshed_id: refreshed?.id,
      refreshed_backend: refreshed?.backend,
      refreshed_format: refreshed?.format,
    })
    if (refreshed && refreshed.id) {
      currentModel = refreshed
      const idx = models.value.findIndex((m) => m.id === refreshed.id || (m.local_path && m.local_path === refreshed.local_path))
      if (idx !== -1) {
        models.value[idx] = refreshed
      }
      try {
        localStorage.setItem('atena_cached_models', JSON.stringify(models.value))
      } catch (_) {}
    }
  } catch (err) {
    console.warn('[LoadModel] [1/6] ⚠️ Failed to refresh model info (continuing with cached data):', err)
  }

  console.log('[LoadModel] [2/6] Checking memory guardrails...', {
    model_id: currentModel.id,
    backend: currentModel.backend,
    format: currentModel.format,
    size_gb: currentModel.size_gb,
  })
  const guardrailCheck = resolveSafeContextAndCheckGuardrails(currentModel)
  if (!guardrailCheck.allowed) {
    console.warn('[LoadModel] [2/6] ✗ Guardrails blocked model loading:', currentModel.id)
    return
  }
  const safeContextLength = guardrailCheck.contextLength
  console.log(`[LoadModel] [2/6] ✓ Guardrails approved. Safe context length: ${safeContextLength}`)

  isLoadingModel.value = true
  loadingModelId.value = currentModel.id
  loadingModelName.value = currentModel.name
  loadingModelProgress.value = 5
  console.log('[LoadModel] [3/6] Loading state activated. Progress: 5%')

  const channel = new Channel<any>()
  channel.onmessage = (event: any) => {
    if (typeof event?.percent === 'number') {
      const newPercent = Math.min(100, Math.max(0, Math.round(event.percent)))
      console.log(`[LoadModel] 📊 Channel progress: ${newPercent}%`)
      loadingModelProgress.value = newPercent
    }
  }

  try {
    const isGguf = currentModel.format === 'Gguf' || (currentModel.local_path && currentModel.local_path.toLowerCase().endsWith('.gguf'))
    const isMlx = currentModel.backend === 'MlxLm' || currentModel.backend === 'Mlx' || currentModel.format === 'Mlx'
    const isOllama = currentModel.backend === 'Ollama' || currentModel.format === 'Ollama'

    console.log('[LoadModel] [4/6] Local model classification:', {
      isGguf,
      isMlx,
      isOllama,
      backend: currentModel.backend,
      format: currentModel.format,
      id: currentModel.id,
      local_path: currentModel.local_path || '(none)',
    })

    // Stop any previous local servers to free VRAM and ensure ports are released
    console.log('[LoadModel] [4/6] Stopping any previous servers before starting new local model...')
    await invoke('stop_all_servers')

    if (isGguf) {
      console.log('[LoadModel] [4/6] Starting LLaMA C++ backend (start_llama_server)...', {
        modelPath: currentModel.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
        contextLength: safeContextLength,
        kvCacheQuant: params.value.kv_cache_quant || 'f16',
        flashAttn: params.value.flash_attention !== false,
        promptCache: params.value.enable_prompt_cache !== false,
      })
      await invoke('start_llama_server', {
        modelPath: currentModel.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
        contextLength: safeContextLength,
        kvCacheQuant: params.value.kv_cache_quant || 'f16',
        flashAttn: params.value.flash_attention !== false,
        promptCache: params.value.enable_prompt_cache !== false,
        channel
      })
      console.log('[LoadModel] [4/6] ✓ llama-server started successfully')
      await checkServices()
    } else if (isMlx) {
      console.log('[LoadModel] [4/6] Starting MLX backend (start_mlx_server)...', {
        modelPath: currentModel.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
      })
      await invoke('start_mlx_server', {
        modelPath: currentModel.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
        channel
      })
      console.log('[LoadModel] [4/6] ✓ mlx-server started successfully')
      await checkServices()
    } else if (isOllama) {
      console.log('[LoadModel] [4/6] Starting Ollama backend (start_ollama_server)...')
      await invoke('start_ollama_server', { channel })
      console.log('[LoadModel] [4/6] ✓ ollama-server started successfully')
      await checkServices()
    } else {
      console.warn('[LoadModel] [4/6] ⚠️ Unrecognized backend for local model:', {
        backend: currentModel.backend,
        format: currentModel.format,
        local_path: currentModel.local_path,
        id: currentModel.id,
      })
      throw new Error(`Unsupported backend for local model: ${currentModel.name}`)
    }

    loadingModelProgress.value = 100
    console.log('[LoadModel] [5/6] Registering active model via set_active_model...', { id: currentModel.id })
    const setActiveStart = performance.now()
    const finalModel = await invoke<any>('set_active_model', { model: currentModel })
    console.log(`[LoadModel] [5/6] ✓ set_active_model completed in ${(performance.now() - setActiveStart).toFixed(0)}ms`)

    const active = finalModel || currentModel
    activeModel.value = active
    localStorage.setItem('atena_active_model_id', active.id)
    localStorage.setItem('atena_last_loaded_model_id', active.id)
    try {
      const raw = localStorage.getItem('atena_model_last_loaded_times')
      const times = raw ? JSON.parse(raw) : {}
      times[active.id] = Date.now()
      localStorage.setItem('atena_model_last_loaded_times', JSON.stringify(times))
    } catch (e) {
      console.error('[LoadModel] Failed to save loaded model timestamp:', e)
    }

    console.log('[LoadModel] [6/6] Verifying services post-activation...')
    await checkServices()

    const totalMs = (performance.now() - loadStart).toFixed(0)
    console.log(`[LoadModel] ✅ Local model "${active.name}" (${active.id}) successfully activated in ${totalMs}ms`)
  } catch (err) {
    console.error('[LoadModel] ❌ Error loading local model:', {
      error: err,
      message: typeof err === 'string' ? err : (err as any)?.message,
      model_id: currentModel.id,
      model_name: currentModel.name,
      backend: currentModel.backend,
      format: currentModel.format,
      elapsed_ms: (performance.now() - loadStart).toFixed(0),
    })
    alert(`Error loading model: ${err}`)
  } finally {
    isLoadingModel.value = false
    loadingModelId.value = null
    loadingModelName.value = ''
    loadingModelProgress.value = 0
    console.log('[LoadModel] 🔄 Loading state reset.')
  }
}

const unloadModel = async (model?: any) => {
  activeModel.value = null
  localStorage.removeItem('atena_active_model_id')
  try {
    await invoke('set_active_model', { model: null })
    await invoke('stop_all_servers')
    await checkServices()
  } catch (err) {
    console.error('Failed to unload model:', err)
  }
}

const startMlx = async () => {
  if (!supportsMlx.value) return
  if (!activeModel.value?.local_path) return

  const guardrailCheck = resolveSafeContextAndCheckGuardrails(activeModel.value)
  if (!guardrailCheck.allowed) return
  const safeContextLength = guardrailCheck.contextLength

  try {
    const channel = new Channel<any>()
    channel.onmessage = (event: any) => {
      if (typeof event?.percent === 'number') {
        loadingModelProgress.value = Math.min(100, Math.max(0, Math.round(event.percent)))
      }
    }
    const isGguf = activeModel.value.format === 'Gguf' || activeModel.value.local_path.toLowerCase().endsWith('.gguf')
    if (isGguf) {
      await invoke('start_llama_server', {
        modelPath: activeModel.value.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
        contextLength: safeContextLength,
        kvCacheQuant: params.value.kv_cache_quant || 'f16',
        flashAttn: params.value.flash_attention !== false,
        promptCache: params.value.enable_prompt_cache !== false,
        channel
      })
    } else {
      await invoke('start_mlx_server', {
        modelPath: activeModel.value.local_path,
        host: config.value.mlx_host,
        port: config.value.mlx_port,
        channel
      })
    }
    await checkServices()
  } catch (err) {
    console.error('Failed to start model server:', err)
  }
}

const startOllama = async () => {
  try {
    const channel = new Channel<any>()
    channel.onmessage = (event: any) => {
      if (typeof event?.percent === 'number') {
        loadingModelProgress.value = Math.min(100, Math.max(0, Math.round(event.percent)))
      }
    }
    await invoke('start_ollama_server', { channel })
    await checkServices()
  } catch (err) {
    console.error('Failed to start Ollama:', err)
  }
}

const stopAllServers = async () => {
  try {
    await invoke('stop_all_servers')
    await checkServices()
  } catch (err) {
    console.error('Failed to stop servers:', err)
  }
}

const fetchLogs = async () => {
  try {
    const res = await invoke<ServerRequestLog[]>('get_server_logs')
    if (res && Array.isArray(res)) {
      logs.value = res
    }
  } catch (err) {
    console.error('Failed to fetch server logs:', err)
  }
}

const clearLogs = async () => {
  try {
    await invoke('clear_server_logs')
    logs.value = []
  } catch (err) {
    console.error('Failed to clear logs:', err)
  }
}

// Chat Actions
const createNewSession = (isPrivate = false, projectId: string | null = null) => {
  const targetProjectId = projectId || null
  const newId = `session-${Date.now()}`
  const nowIso = new Date().toISOString()
  const newSession: ChatSession = {
    id: newId,
    title: isPrivate ? t('sidebar.private_chat') : t('sidebar.new_chat'),
    is_private: !!isPrivate,
    model_id: activeModel.value?.id || null,
    model_name: activeModel.value?.name || activeModel.value?.id || null,
    project_id: targetProjectId,
    created_at: nowIso,
    updated_at: nowIso,
    messages: []
  }
  if (targetProjectId) {
    selectProject(targetProjectId)
  }
  sessions.value.unshift(newSession)
  activeSessionId.value = newId
  activeTab.value = 'chat'
  saveSessions(true)
}

const createNewProjectSession = (projectId: string) => {
  createNewSession(false, projectId)
}

const sidebarRef = ref<any>(null)

const handleRenameSession = ({ id, title }: { id: string; title: string }) => {
  const s = sessions.value.find((sess) => sess.id === id)
  if (s) {
    s.title = title
    s.updated_at = new Date().toISOString()
    saveSessions()
  }
}

const handleTogglePinSession = (id: string) => {
  const s = sessions.value.find((sess) => sess.id === id)
  if (s) {
    s.pinned = !s.pinned
    saveSessions()
  }
}

const handleArchiveSession = (id: string) => {
  const s = sessions.value.find((sess) => sess.id === id)
  if (s) {
    s.archived = true
    s.archived_at = new Date().toISOString()
    saveSessions()
    if (activeSessionId.value === id) {
      const nextActive = sessions.value.find((sess) => !sess.archived)
      if (nextActive) {
        activeSessionId.value = nextActive.id
      } else {
        createNewSession()
      }
    }
  }
}

const handleArchiveSessions = (ids: string[]) => {
  if (!ids || ids.length === 0) return
  const idSet = new Set(ids)
  const now = new Date().toISOString()
  sessions.value.forEach((s) => {
    if (idSet.has(s.id)) {
      s.archived = true
      s.archived_at = now
    }
  })
  saveSessions()
  if (idSet.has(activeSessionId.value)) {
    const nextActive = sessions.value.find((sess) => !sess.archived)
    if (nextActive) {
      activeSessionId.value = nextActive.id
    } else {
      createNewSession()
    }
  }
}

const handleUnarchiveSession = (id: string) => {
  const s = sessions.value.find((sess) => sess.id === id)
  if (s) {
    s.archived = false
    saveSessions()
  }
}

const handleUnarchiveSessions = (ids: string[]) => {
  if (!ids || ids.length === 0) return
  const idSet = new Set(ids)
  sessions.value.forEach((s) => {
    if (idSet.has(s.id)) {
      s.archived = false
    }
  })
  saveSessions()
}

const handleOpenArchivedSession = (id: string) => {
  activeSessionId.value = id
  activeTab.value = 'chat'
  const session = sessions.value.find((s) => s.id === id)
  selectProject(session?.project_id || null)
  isArchivedModalOpen.value = false
}

const selectSession = (id: string) => {
  activeSessionId.value = id
  activeTab.value = 'chat'
  const session = sessions.value.find((s) => s.id === id)
  selectProject(session?.project_id || null)
}

const handleAssignSessionProject = ({ sessionId, projectId }: { sessionId: string; projectId: string | null }) => {
  const s = sessions.value.find((sess) => sess.id === sessionId)
  if (s) {
    s.project_id = projectId || null
    saveSessions()
    if (activeSessionId.value === sessionId) {
      selectProject(projectId || null)
    }
  }
}

const handleDeleteProject = (projectId: string) => {
  let changed = false
  sessions.value.forEach((s) => {
    if (s.project_id === projectId) {
      s.project_id = null
      changed = true
    }
  })
  if (changed) {
    saveSessions()
  }
}

const requestDeleteSession = (sessionOrId: string | ChatSession) => {
  const targetId = typeof sessionOrId === 'object' ? sessionOrId.id : sessionOrId
  const s = sessions.value.find((sess) => sess.id === targetId)
  if (s) {
    sessionToDelete.value = s
    isDeleteSessionModalOpen.value = true
  }
}

const requestDeleteSessions = (sessionListOrIds: string[] | ChatSession[]) => {
  if (!sessionListOrIds || sessionListOrIds.length === 0) return
  const ids = sessionListOrIds.map((item) => (typeof item === 'object' ? item.id : item))
  if (ids.length === 1) {
    requestDeleteSession(ids[0]!)
    return
  }
  const idSet = new Set(ids)
  const list = sessions.value.filter((s) => idSet.has(s.id))
  if (list.length > 0) {
    sessionToDelete.value = list
    isDeleteSessionModalOpen.value = true
  }
}

const closeDeleteSessionModal = () => {
  isDeleteSessionModalOpen.value = false
  sessionToDelete.value = null
}

const handleConfirmDeleteSession = async ({ sessionId, sessionIds, deleteEpisodes, deleteMemories }: { sessionId?: string; sessionIds?: string[]; deleteEpisodes?: boolean; deleteMemories?: boolean }) => {
  const idsToDelete = sessionIds && sessionIds.length > 0 ? sessionIds : (sessionId ? [sessionId] : [])
  for (const id of idsToDelete) {
    try {
      if (deleteEpisodes || deleteMemories) {
        await invoke('session_delete_memories', {
          sessionId: id,
          deleteEpisodes: !!deleteEpisodes,
          deleteMemories: !!deleteMemories
        })
      }
    } catch (err) {
      console.error('Failed to clear session memories/episodes:', err)
    }
  }

  const idSet = new Set(idsToDelete)
  sessions.value = sessions.value.filter((s) => !idSet.has(s.id))
  if (idsToDelete.length > 0) {
    invoke('db_delete_sessions_batch', { sessionIds: idsToDelete }).catch((err) => {
      console.error('Failed to delete sessions from database:', err)
    })
  }
  if (sessions.value.length === 0) {
    createNewSession()
  } else if (idSet.has(activeSessionId.value)) {
    activeSessionId.value = sessions.value[0]!.id
  }
  saveSessions()
  sidebarRef.value?.clearSelection?.()
  closeDeleteSessionModal()
}

const deleteSession = (id: string) => {
  requestDeleteSession(id)
}

const handleClearChat = () => {
  if (currentSession.value) {
    currentSession.value.messages = []
    saveSessions()
  }
}

const handleDeleteMessage = (payload: any) => {
  if (!currentSession.value || !currentSession.value.messages) return
  const targetId = typeof payload === 'object' ? payload?.id : payload
  const targetIndex = typeof payload === 'object' ? payload?.index : undefined

  let idx = -1
  if (targetId) {
    idx = currentSession.value.messages.findIndex((m) => m.id === targetId)
  }
  if (idx === -1 && typeof targetIndex === 'number' && targetIndex >= 0 && targetIndex < currentSession.value.messages.length) {
    idx = targetIndex
  }

  if (idx !== -1) {
    currentSession.value.messages.splice(idx, 1)
    saveSessions()
  }
}

const handleStopGeneration = () => {
  isGenerating.value = false
  if (currentSession.value && currentSession.value.messages) {
    const lastAssistantMsg = [...currentSession.value.messages].reverse().find((m) => m.role === 'assistant')
    if (lastAssistantMsg && lastAssistantMsg.is_streaming) {
      lastAssistantMsg.is_streaming = false
      if (!lastAssistantMsg.content && !lastAssistantMsg.thinking && (!lastAssistantMsg.tool_calls || lastAssistantMsg.tool_calls.length === 0)) {
        lastAssistantMsg.content = t('chat.generation_interrupted')
      }
    }
  }
  saveSessions()
}

const activeMcpToolsCount = computed(() => {
  return mcpTools.value.filter((t) => t.enabled && t.server_id !== 'atena_native' && t.server_id !== 'atena' && t.server_id !== 'skills').length
})

const fetchMcpTools = async () => {
  try {
    const list = await invoke<any[]>('list_all_mcp_tools')
    if (list) {
      mcpTools.value = list
    }
  } catch (err) {
    console.warn('Failed to load MCP tools:', err)
  }
}

const buildPromptWithAttachments = (text: string, attachments: any[] = []) => {
  let prompt = text || ''
  if (!attachments || attachments.length === 0) return prompt

  for (const att of attachments) {
    if (att.text && !prompt.includes(`--- [Attached XML File Content: ${att.name}]`) && !prompt.includes(`--- [Attached PDF Document: ${att.name}`)) {
      if (att.kind === 'pdf') {
        prompt += `\n\n--- [Attached PDF Document: ${att.name}${att.pages ? ` (${att.pages} pages)` : ''}] ---\n${att.text}\n--- [End of PDF Document: ${att.name}] ---`
      } else if (att.kind === 'audio') {
        const durStr = att.duration ? ` (${Math.floor(att.duration / 60)}m ${Math.floor(att.duration % 60)}s)` : ''
        prompt += `\n\n--- [Attached Audio Transcription: ${att.name}${durStr}] ---\n${att.text}\n--- [End of Audio Transcription: ${att.name}] ---`
      } else if (att.kind === 'xml' || (att.name && att.name.toLowerCase().endsWith('.xml'))) {
        prompt += `\n\n--- [Attached XML File Content: ${att.name}] ---\n${att.text}\n--- [End of XML File: ${att.name}] ---`
      } else if (att.kind === 'json' || (att.name && att.name.toLowerCase().endsWith('.json'))) {
        prompt += `\n\n--- [Attached JSON File Content: ${att.name}] ---\n${att.text}\n--- [End of JSON File: ${att.name}] ---`
      } else {
        prompt += `\n\n--- [Attached File Content: ${att.name}] ---\n${att.text}\n--- [End of File: ${att.name}] ---`
      }
    }
  }
  return prompt
}

const handleResendMessage = async (msg: any) => {
  if (isGenerating.value || !msg) return

  // Se a mensagem subsequente foi erro ou aviso de modelo não selecionado, remover da conversa
  const msgIndex = currentSession.value.messages.findIndex((m) => m.id === msg.id)
  if (msgIndex !== -1) {
    const nextMsg = currentSession.value.messages[msgIndex + 1]
    if (nextMsg && nextMsg.role === 'assistant') {
      const content = nextMsg.content || ''
      const isErrorMsg =
        !content ||
        content.includes('Nenhum modelo selecionado') ||
        content.includes('No model selected') ||
        content.includes('Ningún modelo seleccionado') ||
        content.includes('⚠️ Erro') ||
        content.includes('⚠️ Error')
      if (isErrorMsg) {
        // Remove a mensagem de erro do assistente e a mensagem do usuário que falhou para reinserir com status limpo
        currentSession.value.messages.splice(msgIndex, 2)
      }
    }
  }

  // Extrair texto limpo (removendo tags temporais [Horário local: ...] se presentes)
  let cleanText = msg.display_text || msg.content || ''
  if (!msg.display_text && cleanText) {
    cleanText = cleanText.replace(/^\[(Horário local|Local time): [^\]]+\]\n?/i, '')
  }

  const fullPrompt = buildPromptWithAttachments(cleanText, msg.attachments)

  await handleSendMessage({
    text: fullPrompt,
    display_text: msg.display_text || cleanText,
    images: msg.images,
    attachments: msg.attachments
  })
}

const handleSendMessage = async (payload: any) => {
  const isObject = typeof payload === 'object' && payload !== null
  let userText = isObject ? (payload.text || '') : (payload || '')
  const displayText = isObject ? (payload.display_text || '') : ''
  const userImages = isObject ? payload.images : undefined
  const userAttachments = isObject ? payload.attachments : undefined

  // Guarantee attached files content is embedded if not yet attached
  if (userAttachments && userAttachments.length > 0 && !userText.includes('--- [')) {
    userText = buildPromptWithAttachments(userText, userAttachments)
  }

  if ((!userText.trim() && (!userImages || userImages.length === 0) && (!userAttachments || userAttachments.length === 0)) || isGenerating.value) return

  // Automatically unarchive session if a new message is sent to it
  if (currentSession.value?.archived) {
    currentSession.value.archived = false
    saveSessions()
  }

  // Prevent sending new message if previous assistant message has tools awaiting authorization
  const pendingAssistantMsg = (currentSession.value?.messages || []).slice().reverse().find(
    (m) => m.role === 'assistant' && m.tool_calls && m.tool_calls.some((tc: any) => tc.status === 'pending_approval' || tc.status === 'executing')
  )
  if (pendingAssistantMsg && !pendingAssistantMsg.is_streaming && !isGenerating.value) {
    console.warn('[Chat] Blocked sending: pending tool calls require user authorization first.')
    return
  }

  const now = new Date()
  const timeStr = formatUserMessageTime(now, config.value.timezone)
  const fullContent = config.value.inject_message_time === true
    ? `[Local time: ${timeStr}]\n${userText}`
    : userText

  const userMsg: ChatMessage = {
    id: `msg-${Date.now()}-user`,
    role: 'user',
    content: fullContent,
    display_text: displayText || (userText.startsWith('---') ? '' : userText.split('\n\n--- [')[0].trim()),
    images: userImages,
    attachments: userAttachments,
    timestamp: now.toISOString(),
    tokens_count: Math.ceil(fullContent.length / 4)
  }

  currentSession.value.messages.push(userMsg)
  currentSession.value.updated_at = now.toISOString()

  // Bring active session to top of list if not already at index 0
  const sessionIndex = sessions.value.findIndex((s) => s.id === currentSession.value.id)
  if (sessionIndex > 0) {
    const active = sessions.value.splice(sessionIndex, 1)[0]
    if (active) {
      sessions.value.unshift(active)
    }
  }

  // Update session title if first message
  if (currentSession.value.messages.length === 1 || currentSession.value.title === 'Nova Conversa' || currentSession.value.title === t('sidebar.new_chat')) {
    const titleCandidate = userMsg.display_text || (userAttachments?.[0]?.name ? `Doc: ${userAttachments[0].name}` : '') || (userImages?.length ? t('chat.visual_analysis_title') : t('chat.conversation_title'))
    currentSession.value.title = titleCandidate.slice(0, 24) || t('sidebar.new_chat')
  }

  // Create placeholder Assistant message
  const assistantMsgId = `msg-${Date.now()}-assistant`
  const assistantMsg: ChatMessage = {
    id: assistantMsgId,
    role: 'assistant',
    content: '',
    thinking: null,
    tool_calls: null,
    timestamp: new Date().toISOString(),
    is_streaming: true,
    tokens_count: 0,
    generation_speed_tps: 0
  }

  currentSession.value.messages.push(assistantMsg)
  isGenerating.value = true

  // Build full conversation history (excluding the pending assistant response)
  const chatHistory = currentSession.value.messages
    .filter((m) => (m.role === 'user' || m.role === 'assistant') && m.id !== assistantMsgId)
    .map((m) => ({
      role: m.role,
      content: m.content || '',
      images: m.images || null,
      tool_calls: m.tool_calls || null,
      tool_call_id: m.tool_call_id || null
    }))

  const startTime = Date.now()

  try {
    const channel = new Channel<any>()

    channel.onmessage = (chunk: any) => {
      if (!isGenerating.value) return

      const targetMsg = currentSession.value.messages.find((m) => m.id === assistantMsgId)
      if (targetMsg) {
        targetMsg.content = chunk.content || ''
        targetMsg.thinking = chunk.thinking || null
        targetMsg.is_streaming = !chunk.is_done
        if (chunk.metrics) {
          targetMsg.metrics = chunk.metrics
        }

        if (chunk.tool_calls && chunk.tool_calls.length > 0) {
          mergeToolCallsInPlace(targetMsg, chunk.tool_calls)
        }

        // Include both thinking and content in total token count and generation speed calculation
        const totalChars = (chunk.content?.length || 0) + (chunk.thinking?.length || 0)
        const tokCount = chunk.metrics?.completion_tokens || Math.ceil(totalChars / 4)
        targetMsg.tokens_count = tokCount

        const elapsed = chunk.elapsed_ms || (Date.now() - startTime)
        if (chunk.metrics?.generation_speed_tps) {
          targetMsg.generation_speed_tps = chunk.metrics.generation_speed_tps
        } else if (elapsed > 0) {
          targetMsg.generation_speed_tps = parseFloat((tokCount / (elapsed / 1000)).toFixed(1))
        }

        runOnChunkHooks(chunk, {
          userMessage: userText,
          sessionId: currentSession.value?.id
        })

        if (chunk.is_done) {
          isGenerating.value = false
          // Filter out any unfinished streaming placeholders upon completion
          if (targetMsg.tool_calls && targetMsg.tool_calls.length > 0) {
            targetMsg.tool_calls = targetMsg.tool_calls.filter((tc: any) => tc.status !== 'streaming')
          }
          saveSessions()

          runAfterChatTurnHooks({
            userMessage: userText,
            assistantResponse: targetMsg.content,
            sessionId: currentSession.value?.id,
            sessionTitle: currentSession.value?.title,
            elapsedMs: chunk.elapsed_ms || (Date.now() - startTime),
            metrics: chunk.metrics
          })

          // Check if any tool calls are set to automatic execution
          if (targetMsg.tool_calls && targetMsg.tool_calls.length > 0) {
            processAutoTools(targetMsg)
          }
        }
      }
    }

    await runBeforeChatHooks({
      userMessage: userText,
      messages: chatHistory,
      params: params.value,
      sessionId: currentSession.value?.id,
      sessionTitle: currentSession.value?.title
    })

    let effectivePrompt = getEffectiveSystemPrompt()
    effectivePrompt = await runOnSystemPromptHooks({
      basePrompt: effectivePrompt,
      userMessage: userText,
      messages: chatHistory,
      sessionId: currentSession.value?.id
    })

    const isPrivate = !!currentSession.value?.is_private
    const isMemoryDisabled = !config.value.enable_cognitive_memory || isPrivate
    const activeTools = (isMemoryDisabled
      ? mcpTools.value.filter((t) => t.server_id !== 'atena_native' && t.server_id !== 'atena' && t.server_id !== 'skills')
      : mcpTools.value.filter((t) => {
          if (config.value.enable_skills_memory === false) {
            if (t.server_id === 'skills' || t.tool?.name === 'run_skill_script' || t.tool?.name === 'create_procedural_skill' || t.tool?.name === 'update_procedural_skill' || t.tool?.name === 'edit_procedural_skill') return false
          }
          if (config.value.enable_facts_memory === false) {
            if (t.tool?.name === 'atena_search_memory') return false
          }
          if (config.value.enable_episodic_memory === false) {
            if (t.tool?.name === 'atena_search_episodes' || t.tool?.name === 'atena_read_episode') return false
          }
          return true
        })
    ).filter((t) => t.enabled !== false)

    const inferenceParams = {
      ...params.value,
      mcp_tools: activeTools.length > 0 ? activeTools : undefined
    }

    await invoke('stream_chat', {
      model: activeModel.value,
      systemPrompt: effectivePrompt,
      messages: chatHistory,
      userMessage: userText,
      sessionId: currentSession.value?.id || null,
      sessionTitle: currentSession.value?.title || null,
      params: inferenceParams,
      mlxHost: config.value.mlx_host,
      mlxPort: config.value.mlx_port,
      ollamaHost: config.value.ollama_host,
      ollamaPort: config.value.ollama_port,
      enableMemory: Boolean(config.value.enable_cognitive_memory) && !currentSession.value?.is_private,
      enableFactsMemory: config.value.enable_facts_memory !== false,
      enableSkillsMemory: config.value.enable_skills_memory !== false,
      enableEpisodicMemory: config.value.enable_episodic_memory !== false,
      onEvent: channel
    })
  } catch (err) {
    console.error('Inference error:', err)
    const targetMsg = currentSession.value.messages.find((m) => m.id === assistantMsgId)
    if (targetMsg) {
      targetMsg.content = t('chat.inference_error', { error: err })
      targetMsg.is_streaming = false
    }
    isGenerating.value = false
  } finally {
    saveSessions()
  }
}

// Helper to inject temporal context under the hood into system prompt
const getEffectiveSystemPrompt = () => {
  let basePrompt = params.value.system_prompt || ''
  if (config.value.inject_current_date === true) {
    const temporalContext = getTemporalContextPrompt(config.value.timezone)
    basePrompt = basePrompt ? `${basePrompt}${temporalContext}` : temporalContext.trim()
  }

  return basePrompt
}

// Set of tool keys currently executing an MCP call in background
const activeToolRuns = ref(new Set())

const getToolRunKey = (toolCall: any) => {
  if (!toolCall) return ''
  return toolCall.id || `${toolCall.server_id || ''}_${toolCall.name}_${JSON.stringify(toolCall.arguments || {})}`
}

const isToolRunning = (toolCall: any) => {
  if (!toolCall) return false
  return activeToolRuns.value.has(getToolRunKey(toolCall))
}

// Merges streamed tool calls in-place without breaking object references or resetting executing states
const mergeToolCallsInPlace = (targetMsg: any, incomingToolCalls: any[]) => {
  if (!incomingToolCalls || incomingToolCalls.length === 0) return
  if (!targetMsg.tool_calls || targetMsg.tool_calls.length === 0) {
    targetMsg.tool_calls = incomingToolCalls.map((tc: any) => ({
      ...tc,
      status: tc.status || 'pending_approval',
      result: tc.result || null
    }))
    return
  }

  incomingToolCalls.forEach((tc: any, tcIdx: number) => {
    const existing = targetMsg.tool_calls.find((e: any, eIdx: number) => {
      if (e.id && tc.id && e.id === tc.id) return true
      return eIdx === tcIdx && e.name === tc.name
    })

    if (existing) {
      existing.name = tc.name
      existing.arguments = tc.arguments
      if (tc.server_id) existing.server_id = tc.server_id
      if (tc.server_name) existing.server_name = tc.server_name
      if (tc.permission_mode) existing.permission_mode = tc.permission_mode
      if (tc.rejection_reason) existing.rejection_reason = tc.rejection_reason
      if (tc.label) existing.label = tc.label
      if (tc.field_labels) existing.field_labels = tc.field_labels
      if (!existing.status || existing.status === 'streaming') {
        existing.status = tc.status || 'pending_approval'
      }
    } else {
      targetMsg.tool_calls.push({
        ...tc,
        status: tc.status || 'pending_approval',
        rejection_reason: tc.rejection_reason || null,
        result: tc.result || null
      })
    }
  })
}

// Function to check if a message with tool calls is fully resolved and ready to trigger follow-up
const checkAndTriggerFollowUp = async (assistantMsg: any) => {
  if (!assistantMsg || !assistantMsg.tool_calls || assistantMsg.tool_calls.length === 0) return
  if (assistantMsg.is_streaming || isGenerating.value) return

  // Check if any tool call in this batch is still pending approval, executing, actively running or streaming
  const hasUnfinishedTools = assistantMsg.tool_calls.some(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'streaming' || isToolRunning(tc)
  )

  if (hasUnfinishedTools) {
    // Still waiting for other tools to be approved/rejected/executed
    return
  }

  // Prevent multiple triggers for the same message turn
  if (assistantMsg._followUpTriggered) return
  assistantMsg._followUpTriggered = true

  try {
    await triggerFollowUpWithToolResults(assistantMsg)
  } catch (err) {
    console.error('Failed to trigger follow-up:', err)
    assistantMsg._followUpTriggered = false
  }
}

// Executes an MCP tool call and marks result
const executeTool = async (toolCall: any, assistantMsg: any, force = false) => {
  if (!toolCall || !assistantMsg) return

  // Always find the exact tool call object within assistantMsg.tool_calls to guarantee Vue reactivity
  const targetTool = assistantMsg.tool_calls?.find(
    (t: any) => (t.id && toolCall.id && t.id === toolCall.id) || t === toolCall
  ) || toolCall

  const runKey = getToolRunKey(targetTool)
  if (activeToolRuns.value.has(runKey)) return
  if (!force && targetTool.status === 'completed') return

  const isRetrying = force || targetTool.status === 'error'
  if (isRetrying && assistantMsg) {
    assistantMsg._followUpTriggered = false
  }

  activeToolRuns.value.add(runKey)
  targetTool.status = 'executing'
  if (toolCall !== targetTool) toolCall.status = 'executing'
  saveSessions()

  try {
    const rawArgs = typeof targetTool.arguments === 'string'
      ? JSON.parse(targetTool.arguments || '{}')
      : (targetTool.arguments || {})

    let result: any
    const isUpdateSkill =
      targetTool.name === 'update_procedural_skill' ||
      targetTool.name === 'edit_procedural_skill'

    const isProceduralSkill =
      targetTool.name === 'create_procedural_skill' ||
      isUpdateSkill ||
      (rawArgs && (Array.isArray(rawArgs.steps) || Array.isArray(rawArgs.triggers)))

    if (
      targetTool.server_id === 'skills' ||
      targetTool.name === 'run_skill_command' ||
      targetTool.name === 'run_skill_script' ||
      targetTool.name === 'run_command' ||
      isProceduralSkill
    ) {
      if (isUpdateSkill) {
        const id = rawArgs.id || targetTool.arguments?.id || null
        const name = rawArgs.name || rawArgs.title || targetTool.name || null
        const description = rawArgs.description || rawArgs.desc || null
        const triggers = Array.isArray(rawArgs.triggers)
          ? rawArgs.triggers.map((t: any) => String(t))
          : (typeof rawArgs.triggers === 'string' ? rawArgs.triggers.split(',').map((s: string) => s.trim()) : null)
        const steps = Array.isArray(rawArgs.steps) ? rawArgs.steps : null
        const scripts = Array.isArray(rawArgs.scripts) ? rawArgs.scripts : null
        const refinementNote = rawArgs.refinement_note || rawArgs.refinementNote || null
        const envVars = rawArgs.env_vars || rawArgs.envVars || null
        const permissionMode = rawArgs.permission_mode || rawArgs.permissionMode || null

        result = await invoke('skills_update_with_scripts', {
          id,
          name,
          description,
          triggers,
          steps,
          scripts,
          refinementNote,
          envVars,
          permissionMode
        })
      } else if (isProceduralSkill) {
        const name = rawArgs.name || rawArgs.title || targetTool.name || ''
        const description = rawArgs.description || rawArgs.desc || ''
        const triggers = Array.isArray(rawArgs.triggers)
          ? rawArgs.triggers.map((t: any) => String(t))
          : (typeof rawArgs.triggers === 'string' ? rawArgs.triggers.split(',').map((s: string) => s.trim()) : [])
        const steps = Array.isArray(rawArgs.steps) ? rawArgs.steps : []
        const scripts = Array.isArray(rawArgs.scripts) ? rawArgs.scripts : []
        const envVars = rawArgs.env_vars || rawArgs.envVars || null

        result = await invoke('skills_create_with_scripts', {
          name,
          description,
          triggers,
          steps,
          scripts,
          envVars
        })
      } else if (targetTool.name === 'run_skill_script' || rawArgs.script_file || rawArgs.script || rawArgs.file_name) {
        const scriptName =
          rawArgs.scriptName ||
          rawArgs.script_name ||
          rawArgs.script_file ||
          rawArgs.script ||
          rawArgs.file_name ||
          rawArgs.fileName ||
          rawArgs.filename ||
          targetTool.arguments?.scriptName ||
          targetTool.arguments?.script_name ||
          targetTool.arguments?.script_file ||
          targetTool.arguments?.script ||
          targetTool.arguments?.file_name ||
          targetTool.arguments?.fileName ||
          targetTool.arguments?.filename ||
          ''
        const skillId =
          rawArgs.skillId ||
          rawArgs.skill_id ||
          rawArgs.slug ||
          rawArgs.id ||
          targetTool.arguments?.skillId ||
          targetTool.arguments?.skill_id ||
          targetTool.arguments?.slug ||
          targetTool.arguments?.id ||
          ''
        const args = Array.isArray(rawArgs.args)
          ? rawArgs.args.map((a: any) => String(a))
          : (rawArgs.args ? [String(rawArgs.args)] : [])
        const timeoutMs = rawArgs.timeout_ms || rawArgs.timeoutMs || undefined

        result = await invoke('skills_run_script', {
          skillId,
          slug: skillId,
          scriptName,
          scriptFile: scriptName,
          args,
          timeoutMs
        })
      } else {
        const command = rawArgs.command || rawArgs.cmd || (typeof rawArgs === 'string' ? rawArgs : '')
        const timeoutMs = rawArgs.timeout_ms || rawArgs.timeoutMs || rawArgs.timeout || undefined
        result = await invoke('skills_run_command', {
          command: String(command),
          timeoutMs: timeoutMs ? Number(timeoutMs) : undefined
        })
      }
    } else {
      if (!targetTool.server_id) {
        // Defensive check: verify if the tool name corresponds to an existing procedural skill
        try {
          const allSkills = await invoke<any[]>('skills_get_all')
          const matchedSkill = allSkills?.find((s: any) => {
            if (s.enabled === false) return false
            const sName = (s.name || '').toLowerCase().replace(/[_-]/g, ' ')
            const tName = (targetTool.name || '').toLowerCase().replace(/[_-]/g, ' ')
            return s.name === targetTool.name || s.id === targetTool.name || sName === tName
          })
          if (matchedSkill) {
            targetTool.server_id = 'skills'
            targetTool.server_name = 'Procedural Skills'
            const extractedCmd =
              matchedSkill.steps?.find((st: any) => st.command)?.command ||
              matchedSkill.steps?.map((st: any) => st.instruction?.match(/`([^`]+)`/)?.[1]).find(Boolean)
            const scriptStep = matchedSkill.steps?.find((st: any) => st.script_file)
            if (extractedCmd) {
              result = await invoke('skills_run_command', {
                command: String(extractedCmd)
              })
            } else if (scriptStep && scriptStep.script_file) {
              result = await invoke('skills_run_script', {
                skillId: matchedSkill.id,
                slug: matchedSkill.id,
                scriptName: scriptStep.script_file,
                scriptFile: scriptStep.script_file,
                args: []
              })
            } else {
              result = { success: true, message: `Skill "${matchedSkill.name}" executed successfully.` }
            }
          } else {
            throw new Error(`Tool "${targetTool.name}" is not associated with any active MCP server.`)
          }
        } catch (mcpErr: any) {
          throw new Error(mcpErr.message || `Tool "${targetTool.name}" is not associated with any active MCP server.`)
        }
      } else {
        result = await invoke('call_mcp_tool', {
          serverId: targetTool.server_id,
          toolName: targetTool.name,
          arguments: rawArgs
        })
      }
    }

    targetTool.status = 'completed'
    targetTool.result = result
    if (toolCall !== targetTool) {
      toolCall.status = 'completed'
      toolCall.result = result
    }
  } catch (err) {
    console.error(`Failed to execute tool ${targetTool.name}:`, err)
    targetTool.status = 'error'
    targetTool.result = { error: String(err) }
    if (toolCall !== targetTool) {
      toolCall.status = 'error'
      toolCall.result = { error: String(err) }
    }
  } finally {
    activeToolRuns.value.delete(runKey)
    saveSessions()
    // Check if all tools in this assistant message batch are finished
    await checkAndTriggerFollowUp(assistantMsg)
  }
}

// Execute all automatic tools in batch
const processAutoTools = async (assistantMsg: any) => {
  if (!assistantMsg.tool_calls || assistantMsg.tool_calls.length === 0) return

  // Check against always allowed commands whitelist
  let alwaysAllowedCmds: string[] = []
  try {
    const raw = localStorage.getItem('atena_always_allowed_commands')
    if (raw) alwaysAllowedCmds = JSON.parse(raw)
  } catch {}

  // Check procedural skills permissions
  let allSkills: any[] = []
  try {
    const fetched = await invoke<any[]>('skills_get_all')
    if (Array.isArray(fetched)) allSkills = fetched
  } catch {}

  for (const tc of assistantMsg.tool_calls) {
    if (tc.status === 'pending_approval') {
      const rawArgs = typeof tc.arguments === 'string' ? JSON.parse(tc.arguments || '{}') : (tc.arguments || {})
      const cmd = (rawArgs.command || rawArgs.cmd || '').trim()
      if (cmd && alwaysAllowedCmds.includes(cmd)) {
        tc.permission_mode = 'auto'
        continue
      }

      if (tc.server_id === 'skills' || tc.name === 'run_command' || tc.name === 'run_skill_command' || tc.name === 'run_skill_script') {
        const slug = (rawArgs.slug || rawArgs.skillId || rawArgs.skill_id || rawArgs.id || '').trim().toLowerCase()
        const script = (rawArgs.script_file || rawArgs.script || rawArgs.script_name || rawArgs.file_name || '').trim().toLowerCase()

        const matchedSkill = allSkills.find((s: any) => {
          if (s.enabled === false) return false
          const sId = (s.id || '').toLowerCase()
          const sName = (s.name || '').toLowerCase()
          const cleanSId = sId.replace(/^skill-/, '')
          const cleanSlug = slug.replace(/^skill-/, '')
          if (slug && (sId === slug || sName === slug || cleanSId === cleanSlug || sId === `skill-${cleanSlug}`)) return true
          if (cmd && s.steps?.some((st: any) => st.command === cmd)) return true
          if (script && (s.scripts?.some((sc: string) => sc.toLowerCase() === script) || s.steps?.some((st: any) => st.script_file?.toLowerCase() === script))) return true
          return false
        })

        if (matchedSkill?.permission_mode === 'auto') {
          tc.permission_mode = 'auto'
        }
      }
    }
  }

  const autoTools = assistantMsg.tool_calls.filter(
    (tc: any) => tc.permission_mode === 'auto' && tc.status === 'pending_approval'
  )
  if (autoTools.length > 0) {
    for (const tc of autoTools) {
      await executeTool(tc, assistantMsg)
    }
  }
}

// Trigger follow-up AI response with ALL tool results from the batch
const triggerFollowUpWithToolResults = async (previousAssistantMsg: any) => {
  const followUpAssistantId = `msg-${Date.now()}-assistant-reply`
  const followUpMsg: ChatMessage = {
    id: followUpAssistantId,
    role: 'assistant',
    content: '',
    thinking: null,
    tool_calls: null,
    timestamp: new Date().toISOString(),
    is_streaming: true,
    tokens_count: 0,
    generation_speed_tps: 0
  }
  currentSession.value.messages.push(followUpMsg)
  isGenerating.value = true

  const history = []
  for (const m of currentSession.value.messages) {
    if (m.id === followUpAssistantId) continue
    if (m.role === 'user' || m.role === 'assistant') {
      history.push({
        role: m.role,
        content: m.content || '',
        images: m.images || null,
        tool_calls: m.tool_calls || null,
        tool_call_id: m.tool_call_id || null
      })
    }
  }

  // Format all tool results in previousAssistantMsg.tool_calls
  const toolResultsBlocks = (previousAssistantMsg.tool_calls || []).map((tc: any) => {
    if (tc.status === 'completed') {
      const resStr = typeof tc.result === 'string' ? tc.result : JSON.stringify(tc.result, null, 2)
      return `### Tool '${tc.name}':\n\`\`\`json\n${resStr}\n\`\`\``
    } else if (tc.status === 'rejected') {
      const reasonPart = tc.rejection_reason && tc.rejection_reason.trim()
        ? ` Reason / Justification for user rejection: "${tc.rejection_reason.trim()}"`
        : ''
      return `### Tool '${tc.name}':\n[Execution rejected by user.${reasonPart}]`
    } else {
      const errStr = typeof tc.result === 'object' ? JSON.stringify(tc.result) : (tc.result || 'Unknown error')
      return `### Tool '${tc.name}':\n[Execution error: ${errStr}]`
    }
  }).join('\n\n')

  history.push({
    role: 'user',
    content: `[MCP Tools Results]:\n\n${toolResultsBlocks}\n\nPlease answer the user's request by clearly integrating and correlating the data returned above with all previous context from this conversation.`
  })

  const startTime = Date.now()
  try {
    const channel = new Channel<any>()
    channel.onmessage = (chunk: any) => {
      if (!isGenerating.value) return
      const target = currentSession.value.messages.find((m) => m.id === followUpAssistantId)
      if (target) {
        target.content = chunk.content || ''
        target.thinking = chunk.thinking || null
        target.is_streaming = !chunk.is_done
        if (chunk.metrics) {
          target.metrics = chunk.metrics
        }
        if (chunk.tool_calls && chunk.tool_calls.length > 0) {
          mergeToolCallsInPlace(target, chunk.tool_calls)
        }
        const totalChars = (chunk.content?.length || 0) + (chunk.thinking?.length || 0)
        const tokCount = chunk.metrics?.completion_tokens || Math.ceil(totalChars / 4)
        target.tokens_count = tokCount
        const elapsed = chunk.elapsed_ms || (Date.now() - startTime)
        if (chunk.metrics?.generation_speed_tps) {
          target.generation_speed_tps = chunk.metrics.generation_speed_tps
        } else if (elapsed > 0) {
          target.generation_speed_tps = parseFloat((tokCount / (elapsed / 1000)).toFixed(1))
        }
        runOnChunkHooks(chunk, {
          sessionId: currentSession.value?.id
        })

        if (chunk.is_done) {
          isGenerating.value = false
          // Filter out any unfinished streaming placeholders upon completion
          if (target.tool_calls && target.tool_calls.length > 0) {
            target.tool_calls = target.tool_calls.filter((tc: any) => tc.status !== 'streaming')
          }
          saveSessions()

          runAfterChatTurnHooks({
            assistantResponse: target.content,
            sessionId: currentSession.value?.id,
            sessionTitle: currentSession.value?.title,
            elapsedMs: chunk.elapsed_ms || (Date.now() - startTime),
            metrics: chunk.metrics
          })

          // Check if this follow-up also generated new tool calls!
          if (target.tool_calls && target.tool_calls.length > 0) {
            processAutoTools(target)
          }
        }
      }
    }

    const isPrivate = !!currentSession.value?.is_private
    const isMemoryDisabled = !config.value.enable_cognitive_memory || isPrivate
    const activeTools = (isMemoryDisabled
      ? mcpTools.value.filter((t) => t.server_id !== 'atena_native' && t.server_id !== 'atena' && t.server_id !== 'skills')
      : mcpTools.value.filter((t) => {
          if (config.value.enable_skills_memory === false) {
            if (t.server_id === 'skills' || t.tool?.name === 'run_skill_script' || t.tool?.name === 'create_procedural_skill' || t.tool?.name === 'update_procedural_skill' || t.tool?.name === 'edit_procedural_skill') return false
          }
          if (config.value.enable_facts_memory === false) {
            if (t.tool?.name === 'atena_search_memory') return false
          }
          if (config.value.enable_episodic_memory === false) {
            if (t.tool?.name === 'atena_search_episodes' || t.tool?.name === 'atena_read_episode') return false
          }
          return true
        })
    ).filter((t) => t.enabled !== false)

    const inferenceParams = {
      ...params.value,
      mcp_tools: activeTools.length > 0 ? activeTools : undefined
    }

    await invoke('stream_chat', {
      model: activeModel.value,
      systemPrompt: getEffectiveSystemPrompt(),
      messages: history,
      userMessage: null,
      sessionId: currentSession.value?.id || null,
      sessionTitle: currentSession.value?.title || null,
      params: inferenceParams,
      mlxHost: config.value.mlx_host,
      mlxPort: config.value.mlx_port,
      ollamaHost: config.value.ollama_host,
      ollamaPort: config.value.ollama_port,
      enableMemory: Boolean(config.value.enable_cognitive_memory) && !currentSession.value?.is_private,
      enableFactsMemory: config.value.enable_facts_memory !== false,
      enableSkillsMemory: config.value.enable_skills_memory !== false,
      enableEpisodicMemory: config.value.enable_episodic_memory !== false,
      onEvent: channel
    })
  } catch (err) {
    console.error('Failed to generate post-tool response:', err)
    const target = currentSession.value.messages.find((m) => m.id === followUpAssistantId)
    if (target) {
      target.content = t('chat.response_generation_error', { error: err })
      target.is_streaming = false
    }
    isGenerating.value = false
  } finally {
    saveSessions()
  }
}

const handleApproveTool = async ({
  toolCall,
  message,
  alwaysAllow
}: {
  toolCall: any
  message: any
  alwaysAllow?: boolean
}) => {
  if (message?.is_streaming || isGenerating.value) return

  if (alwaysAllow) {
    const rawArgs = typeof toolCall.arguments === 'string'
      ? JSON.parse(toolCall.arguments || '{}')
      : (toolCall.arguments || {})

    // If it has skill_id or is linked to an existing skill, persist permission_mode: 'auto'
    try {
      const skills = await invoke<any[]>('skills_get_all')
      const targetCmd = (rawArgs.command || rawArgs.cmd || '').trim()
      const matchedSkill = skills.find((s: any) =>
        s.enabled !== false && (
          (rawArgs.skill_id && s.id === rawArgs.skill_id) ||
          (rawArgs.skill_name && s.name.toLowerCase() === rawArgs.skill_name.toLowerCase()) ||
          (targetCmd && s.steps?.some((st: any) => st.command && st.command.trim() === targetCmd))
        )
      )
      if (matchedSkill) {
        await invoke('skills_set_permission_mode', {
          skillId: matchedSkill.id,
          permissionMode: 'auto'
        })
      }
    } catch (e) {
      console.warn('Could not auto-set skill permission mode:', e)
    }

    // Save in localStorage whitelist for command strings
    try {
      const STORAGE_KEY = 'atena_always_allowed_commands'
      const raw = localStorage.getItem(STORAGE_KEY)
      const allowed: string[] = raw ? JSON.parse(raw) : []
      const cmd = (rawArgs.command || rawArgs.cmd || '').trim()
      if (cmd && !allowed.includes(cmd)) {
        allowed.push(cmd)
        localStorage.setItem(STORAGE_KEY, JSON.stringify(allowed))
      }
    } catch (e) {
      console.warn('Could not save command to whitelist:', e)
    }

    toolCall.permission_mode = 'auto'
  }

  await executeTool(toolCall, message)
}

const handleReExecuteTool = async ({ toolCall, message }: { toolCall: any; message: any }) => {
  if (message?.is_streaming || isGenerating.value) return
  await executeTool(toolCall, message, true)
}

const handleRejectTool = async ({ toolCall, message, reason }: { toolCall: any; message: any; reason?: string }) => {
  if (message?.is_streaming || isGenerating.value) return
  const targetTool = message.tool_calls?.find(
    (t: any) => (t.id && toolCall.id && t.id === toolCall.id) || t === toolCall
  ) || toolCall
  targetTool.status = 'rejected'
  if (reason && reason.trim()) {
    targetTool.rejection_reason = reason.trim()
  }
  if (toolCall !== targetTool) {
    toolCall.status = 'rejected'
    if (reason && reason.trim()) toolCall.rejection_reason = reason.trim()
  }
  saveSessions()
  await checkAndTriggerFollowUp(message)
}

const handleApproveSelectedTools = async ({ toolCalls, message }: { toolCalls: any[]; message: any }) => {
  if (!message || !toolCalls || toolCalls.length === 0) return
  if (message.is_streaming || isGenerating.value) return

  for (const tc of toolCalls) {
    await executeTool(tc, message)
  }
}

const handleRejectSelectedTools = async ({ toolCalls, message, reason }: { toolCalls: any[]; message: any; reason?: string }) => {
  if (!message || !toolCalls || toolCalls.length === 0) return
  if (message.is_streaming || isGenerating.value) return

  for (const tc of toolCalls) {
    const targetTool = message.tool_calls.find(
      (t: any) => (t.id && tc.id && t.id === tc.id) || t === tc
    ) || tc
    targetTool.status = 'rejected'
    if (reason && reason.trim()) {
      targetTool.rejection_reason = reason.trim()
    }
    if (tc !== targetTool) {
      tc.status = 'rejected'
      if (reason && reason.trim()) tc.rejection_reason = reason.trim()
    }
  }
  saveSessions()
  await checkAndTriggerFollowUp(message)
}

const handleApproveAllTools = async ({ message }: { message: any }) => {
  if (!message || !message.tool_calls || message.tool_calls.length === 0) return
  if (message.is_streaming || isGenerating.value) return

  // Select all tools that require execution (pending, previously stuck in executing, or failed with error)
  const pendingTools = message.tool_calls.filter(
    (tc: any) => (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error') && !isToolRunning(tc)
  )

  for (const tc of pendingTools) {
    await executeTool(tc, message)
  }
}

const handleRejectAllTools = async ({ message, reason }: { message: any; reason?: string }) => {
  if (!message || !message.tool_calls || message.tool_calls.length === 0) return
  if (message.is_streaming || isGenerating.value) return

  const pendingTools = message.tool_calls.filter(
    (tc: any) => (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error') && !isToolRunning(tc)
  )

  for (const tc of pendingTools) {
    const targetTool = message.tool_calls.find(
      (t: any) => (t.id && tc.id && t.id === tc.id) || t === tc
    ) || tc
    targetTool.status = 'rejected'
    if (reason && reason.trim()) {
      targetTool.rejection_reason = reason.trim()
    }
    if (tc !== targetTool) {
      tc.status = 'rejected'
      if (reason && reason.trim()) tc.rejection_reason = reason.trim()
    }
  }
  saveSessions()
  await checkAndTriggerFollowUp(message)
}

// SQLite & Cache Persistence
let saveDbSessionsTimeout: ReturnType<typeof setTimeout> | null = null

const sanitizeSessions = (sessionList: any[]) => {
  sessionList.forEach((session: any) => {
    session.messages?.forEach((msg: any, idx: number) => {
      if (!msg.id) {
        msg.id = `msg-${session.id || 'sess'}-${idx}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
      }
      if (msg.is_streaming) msg.is_streaming = false
      msg.tool_calls?.forEach((tc: any) => {
        if (tc.status === 'executing') {
          tc.status = 'pending_approval'
        }
      })
    })
  })
}

const saveSessions = (immediate = false) => {
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem('atena_sessions', JSON.stringify(sessions.value))
      localStorage.setItem('atena_active_session', activeSessionId.value)
    } catch (_) {}
  }

  const persistToDb = () => {
    invoke('db_save_sessions_batch', { sessions: sessions.value }).catch((err) => {
      console.error('Failed to sync sessions to database:', err)
    })
    if (activeSessionId.value) {
      invoke('db_set_setting', { key: 'active_session_id', value: activeSessionId.value }).catch(() => {})
    }
  }

  if (immediate) {
    if (saveDbSessionsTimeout) clearTimeout(saveDbSessionsTimeout)
    persistToDb()
  } else {
    if (saveDbSessionsTimeout) clearTimeout(saveDbSessionsTimeout)
    saveDbSessionsTimeout = setTimeout(persistToDb, 300)
  }
}

const loadSessions = async () => {
  // 1. Initial fast load from localStorage cache if available
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('atena_sessions')
    const savedActive = localStorage.getItem('atena_active_session')
    if (saved) {
      try {
        const parsed = JSON.parse(saved)
        sanitizeSessions(parsed)
        sessions.value = parsed
      } catch (__) { }
    }
    if (savedActive && sessions.value.some((s) => s.id === savedActive)) {
      activeSessionId.value = savedActive
    }
  }

  // 2. Authoritative load from SQLite database
  try {
    const dbSessions = await invoke<ChatSession[]>('db_get_sessions')
    if (Array.isArray(dbSessions) && dbSessions.length > 0) {
      sanitizeSessions(dbSessions)
      sessions.value = dbSessions
      const dbActive = await invoke<string | null>('db_get_setting', { key: 'active_session_id' })
      if (dbActive && sessions.value.some((s) => s.id === dbActive)) {
        activeSessionId.value = dbActive
      } else if (!activeSessionId.value && sessions.value.length > 0) {
        activeSessionId.value = sessions.value[0]?.id || ''
      }
    } else if (sessions.value.length > 0) {
      // Auto-migrate legacy localStorage sessions to SQLite
      await invoke('db_save_sessions_batch', { sessions: sessions.value })
      if (activeSessionId.value) {
        await invoke('db_set_setting', { key: 'active_session_id', value: activeSessionId.value })
      }
    }
  } catch (err) {
    console.warn('Could not sync sessions with SQLite database:', err)
  }
}

const handleSaveConfig = async (newConfig: any) => {
  config.value = { ...newConfig }
  if (config.value.models_directory) {
    config.value.models_directory = contractUserPath(config.value.models_directory)
  }
  if (Array.isArray(config.value.models_directories)) {
    config.value.models_directories = config.value.models_directories.map(contractUserPath)
  }
  if (config.value.enable_cognitive_memory === false && activeTab.value === 'memory') {
    activeTab.value = 'chat'
  }
  if (typeof window !== 'undefined') {
    localStorage.setItem('atena_config', JSON.stringify(config.value))
  }
  try {
    await invoke('save_app_config', { config: config.value })
  } catch (err) {
    console.error('Failed to save native configuration:', err)
  }
}

const handleHideCloudModel = async (model: ModelInfo) => {
  if (!model?.id) return
  if (!config.value.cloud_providers) {
    config.value.cloud_providers = { disabled_models: [] }
  }
  if (!Array.isArray(config.value.cloud_providers.disabled_models)) {
    config.value.cloud_providers.disabled_models = []
  }

  if (!isModelIdDisabled(model.id, config.value.cloud_providers.disabled_models)) {
    config.value.cloud_providers.disabled_models.push(model.id)
  }

  if (activeModel.value && isModelIdDisabled(activeModel.value.id, config.value.cloud_providers.disabled_models)) {
    await unloadModel()
  }

  const cleanPayload = JSON.parse(JSON.stringify(config.value.cloud_providers))

  try {
    await invoke('save_cloud_providers_config', {
      cloudProviders: cleanPayload
    })
  } catch (err) {
    console.error('Failed to save cloud providers:', err)
  }

  if (typeof window !== 'undefined') {
    localStorage.setItem('atena_cloud_providers', JSON.stringify(cleanPayload))
    localStorage.setItem('atena_config', JSON.stringify(config.value))
  }

  addNotification({
    type: 'info',
    title: t('models.model_hidden_title') || 'Modelo Ocultado',
    message: t('models.model_hidden_msg', { name: model.name || model.id }) || `O modelo "${model.name || model.id}" foi ocultado.`
  })

  scanModels()
}

watch(
  () => config.value?.cloud_providers?.disabled_models,
  (disabledList) => {
    if (activeModel.value && isModelIdDisabled(activeModel.value.id, disabledList)) {
      unloadModel()
    }
  },
  { deep: true }
)

watch(
  () => config.value.enable_cognitive_memory,
  (enabled) => {
    if (enabled === false && activeTab.value === 'memory') {
      activeTab.value = 'chat'
    }
  }
)

const handleSelectFolder = async () => {
  try {
    const selected = await invoke<string | null>('select_folder', {
      defaultPath: config.value.models_directory
    })
    if (selected) {
      const contracted = contractUserPath(selected)
      config.value.models_directory = contracted
      if (!Array.isArray(config.value.models_directories)) {
        config.value.models_directories = []
      }
      config.value.models_directories = config.value.models_directories.map(contractUserPath)
      if (!config.value.models_directories.includes(contracted)) {
        config.value.models_directories.unshift(contracted)
      } else {
        const idx = config.value.models_directories.indexOf(contracted)
        if (idx > 0) {
          config.value.models_directories.splice(idx, 1)
          config.value.models_directories.unshift(contracted)
        }
      }
      handleSaveConfig(config.value)
      await scanModels()
    }
  } catch (err) {
    console.error('Failed to select directory:', err)
  }
}

const handleSetDownloadDestination = (dir: string) => {
  if (!dir) return
  const contracted = contractUserPath(dir)
  config.value.models_directory = contracted
  if (!Array.isArray(config.value.models_directories)) {
    config.value.models_directories = []
  }
  config.value.models_directories = config.value.models_directories.map(contractUserPath)
  if (!config.value.models_directories.includes(contracted)) {
    config.value.models_directories.unshift(contracted)
  } else {
    const idx = config.value.models_directories.indexOf(contracted)
    if (idx > 0) {
      config.value.models_directories.splice(idx, 1)
      config.value.models_directories.unshift(contracted)
    }
  }
  handleSaveConfig(config.value)
}

let downloadsTimer: any = null
let prevCompletedCount = 0
const fetchGlobalDownloads = async () => {
  try {
    const list = await invoke<any[]>('get_active_downloads')
    if (list) {
      activeDownloads.value = list
      const completedCount = list.filter((d: any) => d.status === 'completed').length
      if (completedCount > prevCompletedCount) {
        prevCompletedCount = completedCount
        scanModels()
      }
    }
  } catch (err) {
    // Handled silently
  }
}

let unlistenSessionsUpdated: (() => void) | null = null

onMounted(async () => {
  await loadSessions()
  await initPersonas()

  if (typeof window !== 'undefined') {
    window.addEventListener('focus', loadSessions)
  }
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenSessionsUpdated = await listen('db_sessions_updated', () => {
      loadSessions()
    })
  } catch (_) {}

  // Load native app configuration if available
  try {
    const nativeConfig = await invoke<any>('get_app_config')
    if (nativeConfig) {
      config.value = { ...config.value, ...nativeConfig }
    }
  } catch (_) {}

  const savedConf = localStorage.getItem('atena_config')
  if (savedConf) {
    try {
      const parsed = JSON.parse(savedConf)
      config.value = {
        ...config.value,
        ...parsed
      }
      if (config.value.models_directory) {
        config.value.models_directory = contractUserPath(config.value.models_directory)
      }
      if (Array.isArray(config.value.models_directories)) {
        config.value.models_directories = config.value.models_directories.map(contractUserPath)
      }
      if (!Array.isArray(config.value.models_directories) || config.value.models_directories.length === 0) {
        config.value.models_directories = config.value.models_directory ? [config.value.models_directory] : ['~/.atena/models']
      }
      // If models_directory is not among the configured models_directories, sync it to the primary directory
      if (!config.value.models_directory || !config.value.models_directories.includes(config.value.models_directory)) {
        config.value.models_directory = config.value.models_directories[0]
      }
      if (!config.value.whisper_model) {
        config.value.whisper_model = 'mlx-community/whisper-small-mlx'
      }
      if (!config.value.whisper_language) {
        config.value.whisper_language = 'pt'
      }
      if (config.value.enable_cognitive_memory === false && activeTab.value === 'memory') {
        activeTab.value = 'chat'
      }
    } catch (__) { }
  } else {
    if (!config.value.whisper_model) {
      config.value.whisper_model = 'mlx-community/whisper-small-mlx'
    }
    if (!config.value.whisper_language) {
      config.value.whisper_language = 'pt'
    }
  }

  const savedParams = localStorage.getItem('atena_params')
  if (savedParams) {
    try {
      params.value = { ...params.value, ...JSON.parse(savedParams) }
    } catch (__) { }
  }
  try {
    const dbParams = await invoke<string | null>('db_get_setting', { key: 'generation_params' })
    if (dbParams) {
      params.value = { ...params.value, ...JSON.parse(dbParams) }
    } else if (savedParams) {
      invoke('db_set_setting', { key: 'generation_params', value: savedParams }).catch(() => {})
    }
  } catch (_) {}

  fetchPlatformInfo()
  fetchHardware()
  checkServices()
  fetchLogs()
  // Fallback to disk cache if localStorage had no models
  if (models.value.length === 0) {
    invoke('get_cached_models').then((diskCache) => {
      if (Array.isArray(diskCache) && diskCache.length > 0 && models.value.length === 0) {
        models.value = diskCache
        try {
          localStorage.setItem('atena_cached_models', JSON.stringify(diskCache))
        } catch (_) {}
      }
    }).catch(() => {})
  }

  // Sincroniza provedores de nuvem e modelos desabilitados no arranque
  try {
    const cp = await invoke('get_cloud_providers_config')
    if (cp) {
      config.value.cloud_providers = cp
    }
  } catch (_) {
    if (typeof window !== 'undefined') {
      const localCp = localStorage.getItem('atena_cloud_providers')
      if (localCp) {
        try {
          config.value.cloud_providers = JSON.parse(localCp)
        } catch (__) {}
      }
    }
  }

  scanModels()
  fetchMcpTools()
  fetchPlugins()
  fetchGlobalDownloads()
  fetchAgyUsage()

  // Initialize Theme
  const savedTheme = localStorage.getItem('atena_theme') || config.value.theme || 'dark'
  applyTheme(savedTheme)

  // Initialize Language
  const savedLocale = localStorage.getItem('atena_locale') || config.value.language || 'pt-BR'
  if (savedLocale) {
    setAppLocale(savedLocale)
    config.value.language = savedLocale
  }

  // First-run onboarding check
  if (typeof window !== 'undefined') {
    const setupCompleted = localStorage.getItem('atena_setup_completed')
    if (setupCompleted !== 'true') {
      isSetupModalOpen.value = true
    }
  }

  hardwareTimer = setInterval(() => {
    fetchHardware()
  }, 2500)

  downloadsTimer = setInterval(() => {
    fetchGlobalDownloads()
  }, 1200)

  // Periodic refresh of AGY usage quota (every 60s)
  agyUsageTimer = setInterval(() => {
    fetchAgyUsage()
  }, 60000)

  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', handleGlobalKeydown)
    window.addEventListener('beforeunload', handleBeforeUnload)
    window.addEventListener('click', handleGlobalClick, true)
  }
})

const handleGlobalClick = (e: MouseEvent) => {
  const target = (e.target as HTMLElement)?.closest('a')
  if (target && target.href) {
    const href = target.getAttribute('href') || target.href
    if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
      e.preventDefault()
      e.stopPropagation()
      invoke('open_url', { url: href }).catch(() => {
        if (typeof window !== 'undefined') {
          window.open(href, '_blank')
        }
      })
    }
  }
}

const handleBeforeUnload = () => {
  try {
    saveSessions(true)
    invoke('stop_all_servers')
  } catch (_) {}
}

const handleGlobalKeydown = (e: KeyboardEvent) => {
  const isMetaOrCtrl = e.metaKey || e.ctrlKey

  // Don't intercept when user is typing in a textarea or input unless it's a specific navigation shortcut
  const isInputTarget = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement

  // Detect Return / Enter, Backspace / Delete, and Escape across all platforms and keyboards
  const isEnterOrReturn = e.key === 'Enter' || e.code === 'Enter' || e.code === 'NumpadEnter'
  const isBackspaceOrDelete = e.key === 'Backspace' || e.key === 'Delete' || e.code === 'Backspace' || e.code === 'Delete'
  const isRejectKey = e.key === 'Escape' || isBackspaceOrDelete

  // Check if there is an active session with pending tools
  const pendingMsg = (currentSession.value?.messages || []).slice().reverse().find(
    (m) => m.tool_calls && m.tool_calls.some((tc) => tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error')
  )
  const pendingTools = (pendingMsg && !pendingMsg.is_streaming && !isGenerating.value && pendingMsg.tool_calls)
    ? pendingMsg.tool_calls.filter(
        (tc: any) => (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error') && !isToolRunning(tc)
      )
    : []
  const hasPendingTools = pendingTools.length > 0

  // Tool approval/rejection shortcuts
  // 1. APPROVE ALL:
  // - With modifier: ⌘⇧Enter, ⌥⇧Enter, Ctrl+Shift+Enter, Alt+Shift+Enter (works anywhere, including inside input)
  // - Outside input: Shift+Enter
  const isApproveAll =
    hasPendingTools &&
    isEnterOrReturn &&
    (
      ((isMetaOrCtrl || e.altKey) && e.shiftKey) ||
      (!isInputTarget && e.shiftKey)
    )

  // 2. APPROVE SINGLE / NEXT:
  // - With modifier: ⌘Enter, ⌥Enter, Ctrl+Enter, Alt+Enter (works anywhere, including inside input)
  // - Outside input: Enter alone
  const isApproveSingle =
    hasPendingTools &&
    isEnterOrReturn &&
    !e.shiftKey &&
    (
      isMetaOrCtrl ||
      e.altKey ||
      !isInputTarget
    )

  if (isApproveAll || isApproveSingle) {
    e.preventDefault()
    if (isApproveAll || pendingTools.length === 1) {
      handleApproveAllTools({ message: pendingMsg })
    } else {
      // When tools are grouped, single approve executes the next pending tool sequentially
      handleApproveTool({ toolCall: pendingTools[0], message: pendingMsg })
    }
    return
  }

  // 3. REJECT ALL:
  // - With modifier: ⌘⇧Delete, ⌥⇧Delete, Ctrl+Shift+Backspace, Alt+Shift+Backspace
  // - Outside input: Shift+Esc, Shift+Delete, Shift+Backspace
  const isRejectAll =
    hasPendingTools &&
    (
      (isBackspaceOrDelete && (isMetaOrCtrl || e.altKey) && e.shiftKey) ||
      (!isInputTarget && e.shiftKey && isRejectKey)
    )

  // 4. REJECT SINGLE / NEXT:
  // - With modifier: ⌘Delete, ⌥Delete, Ctrl+Backspace, Alt+Backspace
  // - Outside input: Esc, Delete, Backspace (when not typing in textarea)
  const isRejectSingle =
    hasPendingTools &&
    !e.shiftKey &&
    (
      (isBackspaceOrDelete && (isMetaOrCtrl || e.altKey)) ||
      (!isInputTarget && isRejectKey)
    )

  if (isRejectAll || isRejectSingle) {
    e.preventDefault()
    if (isRejectAll || pendingTools.length === 1) {
      handleRejectAllTools({ message: pendingMsg, reason: 'Recusado via atalho de teclado' })
    } else {
      // When tools are grouped, reject the next pending tool sequentially
      handleRejectTool({ toolCall: pendingTools[0], message: pendingMsg, reason: 'Recusado via atalho de teclado' })
    }
    return
  }

  // Delete / Backspace: Delete selected folder, chat, or multi-selected chats
  // Guard: NEVER trigger when there are pending tools waiting, or when inside an input
  if ((e.key === 'Delete' || e.key === 'Backspace') && !isInputTarget && !hasPendingTools) {
    if (!isDeleteSessionModalOpen.value && !isHfModalOpen.value && !isSetupModalOpen.value) {
      const handled = sidebarRef.value?.triggerDeleteShortcut?.()
      if (handled) {
        e.preventDefault()
        return
      }
    }
  }

  // Escape: clear sidebar multi-selection if active
  if (e.key === 'Escape' && !isInputTarget && !hasPendingTools) {
    if (sidebarRef.value?.selectedSessionIds?.size > 0) {
      e.preventDefault()
      sidebarRef.value?.clearSelection?.()
      return
    }
  }

  // Cmd+N: New Conversation
  if (isMetaOrCtrl && e.key.toLowerCase() === 'n' && !e.shiftKey) {
    e.preventDefault()
    createNewSession()
    return
  }

  // Cmd+K: Focus Search in Sidebar
  if (isMetaOrCtrl && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    sidebarRef.value?.focusSearch?.()
    return
  }

  // Cmd+B: Toggle Inference Drawer
  if (isMetaOrCtrl && e.key.toLowerCase() === 'b' && !isInputTarget) {
    e.preventDefault()
    isDrawerOpen.value = !isDrawerOpen.value
    return
  }

  // Cmd+, : Settings Screen
  if (isMetaOrCtrl && e.key === ',') {
    e.preventDefault()
    activeTab.value = 'settings'
    return
  }
}

const fetchPlatformInfo = async () => {
  try {
    const info = await invoke<any>('get_platform_info')
    if (info) {
      platformInfo.value = info
    }
  } catch (err) {
    console.error('Failed to get platform info:', err)
  }
}

watch(
  params,
  (newParams) => {
    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem('atena_params', JSON.stringify(newParams))
      } catch (_) {}
    }
    invoke('db_set_setting', {
      key: 'generation_params',
      value: JSON.stringify(newParams)
    }).catch(() => {})
  },
  { deep: true }
)

onUnmounted(() => {
  if (hardwareTimer) clearInterval(hardwareTimer)
  if (downloadsTimer) clearInterval(downloadsTimer)
  if (agyUsageTimer) clearInterval(agyUsageTimer)
  if (systemMediaWatcher) {
    try {
      systemMediaWatcher.removeEventListener('change', onSystemThemeChange)
    } catch (_) {}
  }
  if (typeof window !== 'undefined') {
    window.removeEventListener('focus', loadSessions)
    window.removeEventListener('keydown', handleGlobalKeydown)
    window.removeEventListener('beforeunload', handleBeforeUnload)
    window.removeEventListener('click', handleGlobalClick, true)
  }
  if (unlistenSessionsUpdated) {
    unlistenSessionsUpdated()
  }
})

</script>

