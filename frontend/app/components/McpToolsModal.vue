<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-fade-in select-none">
    <!-- Modal Card -->
    <div
      class="w-full max-w-3xl max-h-[85vh] bg-[#0d101a] border border-[#23293f] rounded-3xl shadow-2xl flex flex-col overflow-hidden text-slate-100 animate-scale-up"
      @click.stop
    >
      <!-- Modal Header -->
      <div class="p-5 bg-[#121626] border-b border-[#1f253d] flex items-center justify-between flex-shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-2xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shadow-sm">
            <Wrench class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-base font-bold text-slate-100">{{ $t('settings.mcp_modal_title') }}</h3>
              <span class="text-xs px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 font-mono font-semibold">
                {{ activeToolsCount }} {{ $t('settings.mcp_active_tools') }}
              </span>
            </div>
            <p class="text-xs text-slate-400 mt-0.5">
              {{ $t('settings.mcp_modal_subtitle') }}
            </p>
          </div>
        </div>

        <button
          @click="$emit('close')"
          class="p-2 rounded-xl bg-[#181d2e] hover:bg-[#22283e] text-slate-400 hover:text-white transition-all cursor-pointer"
          :title="$t('common.close')"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Action & Search Bar -->
      <div class="px-5 py-3 bg-[#0f121d] border-b border-[#1a1f33] flex items-center justify-between gap-3 flex-shrink-0">
        <div class="relative flex-1">
          <Search class="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            v-model="searchQuery"
            :placeholder="$t('settings.mcp_search_placeholder')"
            class="w-full bg-[#151928] border border-[#22293f] rounded-xl pl-9 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 outline-none focus:border-indigo-500/60 focus:ring-1 focus:ring-indigo-500/30 transition-all font-sans"
          />
        </div>

        <div class="flex items-center gap-2 flex-shrink-0">
          <input ref="mcpModalFileInput" type="file" accept=".json" class="hidden" @change="handleImportMcpFile" />
          <button
            @click="triggerImportMcp"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all active:scale-95 cursor-pointer shadow-xs"
            :title="$t('settings.mcp_import_tooltip')"
          >
            <Upload class="w-3.5 h-3.5 text-indigo-400" />
            <span class="hidden sm:inline">{{ $t('settings.mcp_import_btn') }}</span>
          </button>
          <button
            @click="exportMcpServers"
            :disabled="servers.length === 0"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer shadow-xs"
            :title="$t('settings.mcp_export_tooltip')"
          >
            <Download class="w-3.5 h-3.5 text-indigo-400" />
            <span class="hidden sm:inline">{{ $t('settings.mcp_export_btn') }}</span>
          </button>
          <button
            @click="refreshTools"
            :disabled="isRefreshing"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all active:scale-95 disabled:opacity-60 cursor-pointer"
            :title="$t('settings.mcp_reload_tooltip')"
          >
            <RefreshCw :class="['w-3.5 h-3.5 text-indigo-400', isRefreshing ? 'animate-spin' : '']" />
            <span>{{ isRefreshing ? $t('settings.mcp_rescanning') : $t('settings.mcp_rescan') }}</span>
          </button>
        </div>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        <!-- Loading State -->
        <div v-if="isLoading" class="p-12 text-center text-slate-400 text-xs flex flex-col items-center justify-center gap-2">
          <Loader2 class="w-6 h-6 text-indigo-400 animate-spin" />
          <span>{{ $t('settings.mcp_loading_servers_and_tools') }}</span>
        </div>

        <!-- Empty State -->
        <div
          v-else-if="filteredServers.length === 0"
          class="p-10 text-center rounded-2xl bg-[#111420] border border-[#1d2338] text-slate-400 text-xs space-y-3"
        >
          <div class="w-12 h-12 rounded-2xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto border border-indigo-500/20">
            <Wrench class="w-6 h-6" />
          </div>
          <p class="font-medium text-slate-300">{{ $t('settings.mcp_no_server_or_tool_found') }}</p>
          <p class="text-[11px] text-slate-400 max-w-md mx-auto">
            {{ $t('settings.mcp_no_server_or_tool_desc') }}
          </p>
        </div>

        <!-- Servers and Tools List -->
        <div v-else class="space-y-4">
          <div
            v-for="server in filteredServers"
            :key="server.id"
            class="rounded-2xl bg-[#111420] border border-[#1e243a] overflow-hidden transition-all shadow-sm"
          >
            <!-- Server Header -->
            <div class="p-3.5 bg-[#141828] border-b border-[#1d2338] flex items-center justify-between gap-3 select-none">
              <div class="flex items-center gap-2.5 min-w-0 flex-1">
                <span class="w-2 h-2 rounded-full flex-shrink-0" :class="server.enabled ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-slate-500'"></span>
                <span class="font-bold text-xs text-slate-100 flex-shrink-0" :title="server.name">{{ server.name }}</span>
                <span class="text-[9.5px] font-mono px-1.5 py-0.2 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 uppercase flex-shrink-0">
                  {{ server.transport }}
                </span>
                <span v-if="server.enabled && (serverTools[server.id] || []).length > 0" class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/5 text-slate-300 border border-white/10 flex-shrink-0">
                  {{ getActiveServerToolsCount(server) }}/{{ (serverTools[server.id] || []).length }} {{ $t('settings.mcp_active_tools') }}
                </span>
                <span class="text-[10.5px] font-mono text-slate-400 truncate hidden sm:inline flex-1 min-w-0" :title="server.transport === 'stdio' ? `${server.command || ''} ${(server.args || []).join(' ')}` : (server.url || '')">
                  {{ server.transport === 'stdio' ? `${server.command} ${(server.args || []).join(' ')}` : server.url }}
                </span>
              </div>

              <div class="flex items-center gap-2 flex-shrink-0">
                <!-- Permission Toggle -->
                <button
                  @click="toggleServerPermission(server)"
                  :class="[
                    'text-[10px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1 transition-all cursor-pointer shadow-sm',
                    server.permission_mode === 'auto'
                      ? 'bg-cyan-500/15 text-cyan-300 border-cyan-500/30 hover:bg-cyan-500/25'
                      : 'bg-amber-500/15 text-amber-300 border-amber-500/30 hover:bg-amber-500/25'
                  ]"
                  :title="server.permission_mode === 'auto' ? $t('settings.mcp_mode_auto_tooltip') : $t('settings.mcp_mode_ask_tooltip')"
                >
                  <component :is="server.permission_mode === 'auto' ? Zap : ShieldCheck" class="w-3 h-3" />
                  <span>{{ server.permission_mode === 'auto' ? $t('settings.mcp_mode_auto') : $t('settings.mcp_mode_ask') }}</span>
                </button>

                <!-- AI Translate Button -->
                <button
                  v-if="(serverTools[server.id] || []).length > 0"
                  @click="translateServerToolLabels(server)"
                  :disabled="isTranslatingTools[server.id] || isAnyTranslating"
                  class="px-2 py-0.5 rounded-lg bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/30 text-indigo-300 text-[10.5px] font-medium transition-all flex items-center gap-1 cursor-pointer disabled:opacity-50"
                  :title="$t('settings.mcp_translate_ai_tooltip')"
                >
                  <Sparkles :class="['w-3 h-3 text-indigo-400', isTranslatingTools[server.id] ? 'animate-spin' : '']" />
                  <span>{{ translatingProgress[server.id] ? $t('settings.mcp_translating_progress', { current: translatingProgress[server.id]?.current, total: translatingProgress[server.id]?.total }) : $t('settings.mcp_translate_ai') }}</span>
                </button>

                <!-- Enable/Disable Server Toggle -->
                <button
                  @click="toggleServer(server)"
                  :class="[
                    'px-2.5 py-1 rounded-lg text-xs font-semibold border transition-all cursor-pointer',
                    server.enabled
                      ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30 hover:bg-emerald-500/25'
                      : 'bg-[#181d2e] text-slate-400 border-[#262e45] hover:text-slate-200'
                  ]"
                >
                  {{ server.enabled ? $t('common.active') : $t('common.inactive') }}
                </button>
              </div>
            </div>

            <!-- Server Tools Grid -->
            <div class="p-3">
              <div v-if="!server.enabled" class="text-xs text-slate-400 italic py-1 text-center">
                {{ $t('settings.mcp_server_disabled_hint') }}
              </div>

              <!-- Error state for server -->
              <div v-else-if="serverErrors[server.id]" class="p-2.5 rounded-xl bg-rose-500/10 border border-rose-500/25 text-rose-300 text-xs space-y-1.5 mb-2">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1.5 font-semibold text-rose-300">
                    <AlertTriangle class="w-3.5 h-3.5 text-rose-400 flex-shrink-0" />
                    <span>{{ $t('settings.mcp_server_conn_failed') }}</span>
                  </div>
                  <button
                    @click="refreshTools"
                    :disabled="isRefreshing"
                    class="px-2 py-0.5 rounded-md bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/30 text-[10px] font-medium transition-all cursor-pointer flex items-center gap-1"
                  >
                    <RefreshCw :class="['w-2.5 h-2.5', isRefreshing ? 'animate-spin' : '']" />
                    <span>{{ $t('settings.mcp_reconnect') }}</span>
                  </button>
                </div>
                <pre class="p-1.5 rounded-lg bg-black/40 border border-rose-500/20 text-[10px] text-rose-200 font-mono overflow-x-auto whitespace-pre-wrap leading-relaxed select-text">{{ serverErrors[server.id] }}</pre>
              </div>

              <div v-else-if="!serverTools[server.id] || (serverTools[server.id] || []).length === 0" class="text-xs text-slate-400 italic py-1 text-center">
                {{ inspectingServerId === server.id ? $t('settings.mcp_discovering_tools') : $t('settings.mcp_no_tools_listed') }}
              </div>

              <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-2">
                <div
                  v-for="tool in serverTools[server.id]"
                  :key="tool.name"
                  :class="[
                    'p-3 rounded-xl border flex flex-col justify-between gap-2.5 transition-all shadow-xs',
                    isToolEnabled(server, tool.name)
                      ? 'bg-[#141826] border-[#22283b] hover:border-[#2f3750]'
                      : 'bg-[#0f121d]/80 border-[#1c2235] opacity-70 hover:opacity-95'
                  ]"
                >
                  <div class="space-y-1.5">
                    <!-- Top Row: Full Width Title, Status Toggle & Terminal identifier -->
                    <div class="min-w-0">
                      <div class="group/toollabel flex items-start justify-between gap-2">
                        <h4
                          @click="openEditToolModal(server, tool)"
                          class="text-xs font-bold text-slate-100 hover:text-indigo-300 transition-colors cursor-pointer truncate flex items-center gap-1.5 flex-1 min-w-0"
                          :title="server.tool_labels?.[tool.name] || tool.label || tool.name"
                        >
                          <span class="truncate">{{ server.tool_labels?.[tool.name] || tool.label || tool.name }}</span>
                          <Pencil class="w-2.5 h-2.5 opacity-0 group-hover/toollabel:opacity-100 text-indigo-400 transition-opacity flex-shrink-0" />
                        </h4>

                        <!-- Tool Active/Inactive Badge Toggle -->
                        <button
                          @click.stop="toggleTool(server, tool.name)"
                          :class="[
                            'text-[9.5px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1.5 transition-all cursor-pointer flex-shrink-0 shadow-2xs',
                            isToolEnabled(server, tool.name)
                              ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30 hover:bg-emerald-500/25'
                              : 'bg-[#181d2e] text-slate-400 border-[#262e45] hover:bg-[#20273c] hover:text-slate-200'
                          ]"
                          :title="isToolEnabled(server, tool.name) ? $t('modals.mcp_tool.disable_tool') : $t('modals.mcp_tool.enable_tool')"
                        >
                          <span class="w-1.5 h-1.5 rounded-full flex-shrink-0" :class="isToolEnabled(server, tool.name) ? 'bg-emerald-400' : 'bg-slate-500'"></span>
                          <span>{{ isToolEnabled(server, tool.name) ? $t('modals.mcp_tool.status_active') : $t('modals.mcp_tool.status_inactive') }}</span>
                        </button>
                      </div>
                      <div class="font-mono text-[10px] text-indigo-400/80 flex items-center gap-1 mt-0.5 truncate" :title="tool.name">
                        <Terminal class="w-2.5 h-2.5 flex-shrink-0" />
                        <span class="truncate">{{ tool.name }}</span>
                      </div>
                    </div>

                    <!-- Middle: Tool Description -->
                    <p class="text-[11px] text-slate-400 line-clamp-2 leading-relaxed">
                      {{ tool.description || $t('settings.mcp_no_description') }}
                    </p>
                  </div>

                  <!-- Bottom Row: Actions Bar -->
                  <div class="pt-2 border-t border-white/[0.04] flex items-center justify-between gap-2 flex-wrap mt-auto">
                    <!-- Per-Tool Permission Toggle Badge -->
                    <button
                      @click="toggleToolPermission(server, tool.name)"
                      :class="[
                        'text-[9.5px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1 transition-all cursor-pointer shadow-2xs',
                        getToolPermission(server, tool.name) === 'auto'
                          ? 'bg-cyan-500/15 text-cyan-300 border-cyan-500/30 hover:bg-cyan-500/25'
                          : 'bg-amber-500/15 text-amber-300 border-amber-500/30 hover:bg-amber-500/25'
                      ]"
                      :title="getToolPermission(server, tool.name) === 'auto' ? $t('settings.mcp_mode_auto_tooltip') : $t('settings.mcp_mode_ask_tooltip')"
                    >
                      <component :is="getToolPermission(server, tool.name) === 'auto' ? Zap : ShieldCheck" class="w-2.5 h-2.5" />
                      <span>{{ getToolPermission(server, tool.name) === 'auto' ? $t('settings.mcp_mode_direct') : $t('settings.mcp_mode_ask') }}</span>
                    </button>

                    <div class="flex items-center gap-1.5 flex-shrink-0">
                      <button
                        @click="openEditToolModal(server, tool)"
                        class="text-[10px] px-2.5 py-0.5 rounded-md bg-white/5 hover:bg-white/10 text-slate-300 hover:text-white border border-white/10 transition-all cursor-pointer flex items-center gap-1"
                        :title="$t('settings.mcp_edit_tool_tooltip')"
                      >
                        <Pencil class="w-2.5 h-2.5 text-indigo-400" />
                        <span>{{ $t('settings.mcp_edit_btn') }}</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-4 bg-[#121626] border-t border-[#1f253d] flex items-center justify-between flex-shrink-0">
        <span class="text-xs text-slate-400">
          {{ $t('settings.mcp_footer_hint') }}
        </span>

        <button
          @click="$emit('close')"
          class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white shadow-md shadow-indigo-600/30 transition-all active:scale-95 cursor-pointer"
        >
          {{ $t('settings.mcp_done') }}
        </button>
      </div>
    </div>

    <!-- Edit MCP Tool Modal -->
    <EditMcpToolModal
      :is-open="isEditToolModalOpen"
      :server="selectedServerToEdit"
      :tool="selectedToolToEdit"
      @close="isEditToolModalOpen = false"
      @saved="handleToolDetailsSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { saveTextFile } from '~/utils/exportMarkdown'
import { useAppLocale } from '~/composables/useLocale'
import EditMcpToolModal, { type EditMcpToolSavedPayload } from './EditMcpToolModal.vue'
import type { McpServerConfig, McpToolDefinition } from '~/types'

const { currentLocale, t } = useAppLocale()
import {
  Wrench,
  X,
  RefreshCw,
  Search,
  Zap,
  ShieldCheck,
  Terminal,
  Loader2,
  AlertTriangle,
  Sparkles,
  Download,
  Upload,
  Pencil,
  Check
} from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'updated'): void
}>()

const servers: Ref<McpServerConfig[]> = ref([])
const serverTools: Ref<Record<string, McpToolDefinition[]>> = ref({})
const serverErrors: Ref<Record<string, string>> = ref({})
const isLoading = ref(true)
const isRefreshing = ref(false)
const inspectingServerId = ref<string | null>(null)
const searchQuery = ref('')
const mcpModalFileInput = ref<HTMLInputElement | null>(null)
const isTranslatingTools: Ref<Record<string, boolean>> = ref({})
const isAnyTranslating = computed(() => Object.values(isTranslatingTools.value).some(Boolean))
const translatingProgress: Ref<Record<string, { current: number; total: number }>> = ref({})
const editingTool = ref<any>(null)

const isEditToolModalOpen = ref(false)
const selectedToolToEdit = ref<McpToolDefinition | null>(null)
const selectedServerToEdit = ref<McpServerConfig | null>(null)

const openEditToolModal = (server: McpServerConfig, tool: McpToolDefinition) => {
  selectedServerToEdit.value = server
  selectedToolToEdit.value = tool
  isEditToolModalOpen.value = true
}

const handleToolDetailsSaved = async ({ serverId, toolName, label, fieldLabels, enabled }: EditMcpToolSavedPayload) => {
  const server = servers.value.find((s) => s.id === serverId)
  if (server) {
    if (!server.tool_labels) server.tool_labels = {}
    if (!server.tool_field_labels) server.tool_field_labels = {}
    if (!server.disabled_tools) server.disabled_tools = []
    if (label) {
      server.tool_labels[toolName] = label
    } else {
      delete server.tool_labels[toolName]
    }
    if (fieldLabels && Object.keys(fieldLabels).length > 0) {
      server.tool_field_labels[toolName] = fieldLabels
    } else {
      delete server.tool_field_labels[toolName]
    }
    if (typeof enabled === 'boolean') {
      if (enabled) {
        server.disabled_tools = server.disabled_tools.filter((x: string) => x !== toolName)
      } else if (!server.disabled_tools.includes(toolName)) {
        server.disabled_tools.push(toolName)
      }
    }
    await saveServers()
    const tools = serverTools.value[serverId] || []
    for (const t of tools) {
      if (t.name === toolName) {
        t.label = label || null
        t.field_labels = fieldLabels || {}
      }
    }
  }
  emit('updated')
}

const isToolEnabled = (server: any, toolName: string): boolean => {
  if (!server) return true
  if (!Array.isArray(server.disabled_tools)) return true
  return !server.disabled_tools.includes(toolName)
}

const toggleTool = async (server: any, toolName: string) => {
  if (!server) return
  if (!server.disabled_tools) {
    server.disabled_tools = []
  }
  if (server.disabled_tools.includes(toolName)) {
    server.disabled_tools = server.disabled_tools.filter((x: string) => x !== toolName)
  } else {
    server.disabled_tools.push(toolName)
  }
  await saveServers()
}

const getActiveServerToolsCount = (server: any): number => {
  const tools = serverTools.value[server.id] || []
  if (!server.enabled || tools.length === 0) return 0
  const disabled = new Set(server.disabled_tools || [])
  return tools.filter((t) => !disabled.has(t.name)).length
}

const activeToolsCount = computed(() => {
  let count = 0
  for (const s of servers.value) {
    if (s.enabled && serverTools.value[s.id]) {
      const disabled = new Set(s.disabled_tools || [])
      count += (serverTools.value[s.id] || []).filter((t) => !disabled.has(t.name)).length
    }
  }
  return count
})

const filteredServers = computed(() => {
  if (!searchQuery.value.trim()) return servers.value
  const q = searchQuery.value.toLowerCase()
  return servers.value.filter((s) => {
    if (s.name.toLowerCase().includes(q)) return true
    const tools = serverTools.value[s.id] || []
    return tools.some(
      (t) => t.name.toLowerCase().includes(q) || (t.description && t.description.toLowerCase().includes(q))
    )
  })
})

const loadServers = async () => {
  isLoading.value = true
  try {
    const list = await invoke('get_mcp_servers')
    if (list) {
      servers.value = (list as McpServerConfig[]) || []
      await refreshTools()
    }
  } catch (err) {
    console.error('Failed to load MCP servers:', err)
  } finally {
    isLoading.value = false
  }
}

const refreshTools = async () => {
  if (isRefreshing.value) return
  isRefreshing.value = true
  try {
    for (const server of servers.value) {
      if (server.enabled) {
        inspectingServerId.value = server.id
        try {
          const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server })
          serverTools.value[server.id] = tools || []
          delete serverErrors.value[server.id]
        } catch (e) {
          console.warn(`Failed to inspect ${server.name}:`, e)
          serverErrors.value[server.id] = String(e)
          serverTools.value[server.id] = []
        }
      } else {
        serverTools.value[server.id] = []
        delete serverErrors.value[server.id]
      }
    }
    emit('updated')
  } finally {
    isRefreshing.value = false
    inspectingServerId.value = null
  }
}

const saveServers = async () => {
  try {
    await invoke('save_mcp_servers', { servers: servers.value })
    emit('updated')
  } catch (err) {
    console.error('Failed to save servers:', err)
  }
}

const startEditLabel = (serverId: string, toolName: string, currentLabel?: string | null) => {
  editingTool.value = { serverId, toolName, label: currentLabel || '' }
}

const saveToolLabelEdit = async (server: any) => {
  if (!editingTool.value) return
  if (!server.tool_labels) server.tool_labels = {}
  const cleanLabel = editingTool.value.label.trim()
  if (cleanLabel) {
    server.tool_labels[editingTool.value.toolName] = cleanLabel
  } else {
    delete server.tool_labels[editingTool.value.toolName]
  }
  const toolName = editingTool.value.toolName
  editingTool.value = null
  await saveServers()
  const tools = serverTools.value[server.id] || []
  for (const t of tools) {
    if (t.name === toolName) {
      t.label = cleanLabel || null
    }
  }
}

const exportMcpServers = async () => {
  const content = JSON.stringify(servers.value, null, 2)
  await saveTextFile({
    content,
    filename: `atena_mcp_tools_${new Date().toISOString().slice(0, 10)}.json`,
    title: t('settings.mcp_export_dialog_title'),
    filters: [
      { name: 'JSON (*.json)', extensions: ['json'] },
      { name: t('settings.mcp_all_files'), extensions: ['*'] }
    ]
  })
}

const triggerImportMcp = () => {
  if (mcpModalFileInput.value) mcpModalFileInput.value.click()
}

const handleImportMcpFile = async (e: any) => {
  const file = e.target.files?.[0]
  if (!file) return
  try {
    const text = await file.text()
    const imported = JSON.parse(text)
    if (Array.isArray(imported)) {
      for (const s of imported) {
        if (!s.id || !s.name) continue
        const idx = servers.value.findIndex((x) => x.id === s.id)
        if (idx !== -1) {
          servers.value[idx] = { ...servers.value[idx], ...s }
        } else {
          servers.value.push(s)
        }
      }
      await saveServers()
      await refreshTools()
    }
  } catch (err) {
    alert(t('settings.mcp_import_json_error'))
  } finally {
    e.target.value = ''
  }
}

const translateServerToolLabels = async (server: any) => {
  if (isTranslatingTools.value[server.id] || isAnyTranslating.value) return
  const tools = serverTools.value[server.id] || []
  if (tools.length === 0) return

  isTranslatingTools.value[server.id] = true
  translatingProgress.value[server.id] = { current: 1, total: tools.length }

  try {
    if (!server.tool_labels) server.tool_labels = {}
    if (!server.tool_field_labels) server.tool_field_labels = {}

    for (let i = 0; i < tools.length; i++) {
      const tool = tools[i]
      if (!tool) continue
      translatingProgress.value[server.id] = { current: i + 1, total: tools.length }

      try {
        const result: any = await invoke('translate_mcp_tool_labels', {
          tools: [tool],
          mlxHost: '127.0.0.1',
          mlxPort: 8080,
          ollamaHost: '127.0.0.1',
          ollamaPort: 11434,
          targetLocale: currentLocale.value
        })

        if (result && typeof result === 'object') {
          if (result.tool_labels && result.tool_labels[tool.name]) {
            server.tool_labels[tool.name] = result.tool_labels[tool.name]
            tool.label = result.tool_labels[tool.name]
          }
          if (result.tool_field_labels && result.tool_field_labels[tool.name]) {
            server.tool_field_labels[tool.name] = result.tool_field_labels[tool.name]
            tool.field_labels = result.tool_field_labels[tool.name]
          }
        }
      } catch (toolErr) {
        console.warn(`Failed to translate tool ${tool.name}:`, toolErr)
      }

      await saveServers()
      await new Promise((r) => setTimeout(r, 60))
    }
  } catch (err) {
    console.error('Failed to translate labels:', err)
  } finally {
    isTranslatingTools.value[server.id] = false
    delete translatingProgress.value[server.id]
  }
}

const toggleServer = async (server: any) => {
  server.enabled = !server.enabled
  await saveServers()
  if (server.enabled) {
    try {
      const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server })
      serverTools.value[server.id] = tools || []
      delete serverErrors.value[server.id]
    } catch (e) {
      serverErrors.value[server.id] = String(e)
      serverTools.value[server.id] = []
    }
  } else {
    serverTools.value[server.id] = []
    delete serverErrors.value[server.id]
  }
  emit('updated')
}

const toggleServerPermission = async (server: any) => {
  server.permission_mode = server.permission_mode === 'auto' ? 'ask' : 'auto'
  await saveServers()
}

const getToolPermission = (server: any, toolName: string): string => {
  if (server.tool_permissions && server.tool_permissions[toolName]) {
    return server.tool_permissions[toolName]
  }
  return server.permission_mode || 'ask'
}

const toggleToolPermission = async (server: any, toolName: string) => {
  if (!server.tool_permissions) {
    server.tool_permissions = {}
  }
  const current = getToolPermission(server, toolName)
  server.tool_permissions[toolName] = current === 'auto' ? 'ask' : 'auto'
  await saveServers()
}

onMounted(() => {
  loadServers()
})
</script>
