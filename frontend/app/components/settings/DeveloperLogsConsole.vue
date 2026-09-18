<template>
  <div class="rounded-2xl bg-[#0e111a] border border-[#1e2336] overflow-hidden shadow-lg flex flex-col h-[380px]">
    <!-- Console Top Bar -->
    <div class="px-4 py-2.5 border-b border-[#1e2336] flex items-center justify-between bg-[#131622] gap-3 flex-wrap">
      <!-- Left: Title & Tabs -->
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5 text-xs font-bold text-slate-200 uppercase tracking-wider">
          <Terminal class="w-4 h-4 text-indigo-400" />
          <span>{{ $t('settings.developer_logs_title') }}</span>
        </div>

        <!-- Tab Switcher -->
        <div class="flex items-center bg-[#090b12] p-0.5 rounded-lg border border-[#1e2336]">
          <button
            type="button"
            @click="activeTab = 'developer'"
            :class="[
              'px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer flex items-center gap-1.5',
              activeTab === 'developer'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-slate-200'
            ]"
          >
            <span>{{ $t('settings.developer_logs_tab') }}</span>
            <span
              v-if="developerLogs.length > 0"
              class="text-[9.5px] px-1.5 py-0.2 rounded-full"
              :class="activeTab === 'developer' ? 'bg-indigo-700 text-indigo-100' : 'bg-slate-800 text-slate-400'"
            >
              {{ developerLogs.length }}
            </span>
          </button>

          <button
            type="button"
            @click="activeTab = 'requests'"
            :class="[
              'px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer flex items-center gap-1.5',
              activeTab === 'requests'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-slate-400 hover:text-slate-200'
            ]"
          >
            <span>{{ $t('settings.http_requests_tab') }}</span>
            <span
              v-if="logs.length > 0"
              class="text-[9.5px] px-1.5 py-0.2 rounded-full"
              :class="activeTab === 'requests' ? 'bg-indigo-700 text-indigo-100' : 'bg-slate-800 text-slate-400'"
            >
              {{ logs.length }}
            </span>
          </button>
        </div>

        <!-- Live LM Studio Inference Status Badge -->
        <div
          v-if="isGenerating"
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 text-[10px] font-mono animate-pulse"
        >
          <Loader2 class="w-3 h-3 animate-spin text-emerald-400" />
          <span>{{ $t('settings.generating_tokens') }}</span>
        </div>
      </div>

      <!-- Right: Level Filter & Actions -->
      <div class="flex items-center gap-2">
        <!-- Level Filters (Developer tab only) -->
        <div v-if="activeTab === 'developer'" class="hidden sm:flex items-center gap-1 bg-[#090b12] p-0.5 rounded-lg border border-[#1e2336] text-[10px] font-mono">
          <button
            v-for="lvl in levels"
            :key="lvl"
            type="button"
            @click="selectedLevel = lvl"
            :class="[
              'px-2 py-0.5 rounded transition-all cursor-pointer',
              selectedLevel === lvl
                ? getFilterActiveClass(lvl)
                : 'text-slate-400 hover:text-slate-200'
            ]"
          >
            {{ lvl }}
          </button>
        </div>

        <!-- Auto-scroll toggle -->
        <button
          v-if="activeTab === 'developer'"
          type="button"
          @click="autoScroll = !autoScroll"
          :title="$t('settings.autoscroll')"
          :class="[
            'text-[11px] flex items-center gap-1 px-2 py-1 rounded border transition-colors cursor-pointer',
            autoScroll
              ? 'bg-indigo-500/10 border-indigo-500/30 text-indigo-300'
              : 'bg-[#181d2c] border-[#22283a] text-slate-500 hover:text-slate-300'
          ]"
        >
          <ArrowDownToLine class="w-3 h-3" />
          <span class="hidden md:inline">{{ $t('settings.autoscroll') }}</span>
        </button>

        <!-- Copy Logs Button -->
        <button
          type="button"
          @click="copyLogsToClipboard"
          :disabled="activeTab === 'developer' ? filteredDevLogs.length === 0 : logs.length === 0"
          :title="$t('settings.copy_logs')"
          class="text-[11px] text-slate-400 hover:text-slate-200 flex items-center gap-1 px-2 py-1 rounded bg-[#181d2c] border border-[#22283a] hover:bg-[#20273b] transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <component :is="hasCopied ? Check : Copy" class="w-3 h-3 text-indigo-400" />
          <span class="hidden sm:inline">{{ hasCopied ? $t('settings.logs_copied') : $t('settings.copy_logs') }}</span>
        </button>

        <!-- Refresh Button -->
        <button
          type="button"
          @click="handleRefresh"
          :title="$t('settings.http_logs_refresh')"
          class="text-[11px] text-slate-400 hover:text-indigo-300 flex items-center gap-1 px-2 py-1 rounded bg-[#181d2c] border border-[#22283a] hover:bg-[#20273b] transition-colors cursor-pointer"
        >
          <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isRefreshing }" />
          <span class="hidden sm:inline">{{ $t('settings.http_logs_refresh') }}</span>
        </button>

        <!-- Clear Button -->
        <button
          type="button"
          @click="handleClear"
          :disabled="activeTab === 'developer' ? developerLogs.length === 0 : logs.length === 0"
          :title="$t('settings.http_logs_clear')"
          class="text-[11px] text-slate-400 hover:text-rose-300 hover:bg-rose-500/10 flex items-center gap-1 px-2 py-1 rounded bg-[#181d2c] border border-[#22283a] transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <Trash2 class="w-3 h-3" />
          <span class="hidden sm:inline">{{ $t('settings.http_logs_clear') }}</span>
        </button>
      </div>
    </div>

    <!-- Console Main Output -->
    <div
      ref="consoleContainer"
      class="flex-1 overflow-y-auto p-3.5 font-mono text-[11.5px] leading-relaxed select-text bg-[#080a10]"
    >
      <!-- VIEW 1: DEVELOPER LOGS (LM Studio Terminal Style) -->
      <div v-if="activeTab === 'developer'" class="space-y-3">
        <div v-if="filteredDevLogs.length === 0" class="flex flex-col items-center justify-center py-16 text-center text-slate-500 space-y-2">
          <Terminal class="w-8 h-8 text-slate-600 opacity-60 mb-1" />
          <p class="text-xs font-medium text-slate-400">{{ $t('settings.developer_logs_empty') }}</p>
          <p class="text-[11px] text-slate-500 max-w-md">{{ $t('settings.developer_logs_empty_desc') }}</p>
        </div>

        <div
          v-for="entry in filteredDevLogs"
          :key="entry.id"
          class="group font-mono transition-colors"
        >
          <!-- Header line: YYYY-MM-DD HH:MM:SS [LEVEL] -->
          <div class="flex items-center gap-2 flex-wrap text-[11px]">
            <span class="text-slate-500">{{ formatTimestamp(entry.timestamp) }}</span>
            <span :class="['font-bold px-1 py-0.2 rounded text-[10px]', getLevelBadgeClass(entry.level)]">
              [{{ entry.level }}]
            </span>
            <span v-if="entry.tag" class="text-indigo-400/90 font-semibold">
              [{{ entry.tag }}]
            </span>
          </div>

          <!-- Message Body -->
          <div class="pl-2 mt-0.5 text-slate-300 whitespace-pre-wrap break-words border-l-2 border-transparent hover:border-slate-700">
            <!-- Normal Message Text -->
            <span>{{ entry.message }}</span>

            <!-- If there are expandable details or JSON payload -->
            <div v-if="entry.details" class="mt-1.5">
              <button
                type="button"
                @click="toggleLogExpansion(entry.id)"
                class="text-[10px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 cursor-pointer underline decoration-dotted"
              >
                <component :is="expandedLogIds.has(entry.id) ? ChevronDown : ChevronRight" class="w-3 h-3" />
                <span>{{ expandedLogIds.has(entry.id) ? $t('settings.collapse_details') : $t('settings.expand_details') }}</span>
              </button>

              <div
                v-if="expandedLogIds.has(entry.id)"
                class="mt-1.5 p-2.5 rounded-lg bg-[#0e111a] border border-[#1e2338] text-[10.5px] text-slate-300 relative group/box"
              >
                <button
                  type="button"
                  @click="copySnippet(entry.details)"
                  class="absolute top-2 right-2 px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-[10px] text-slate-300 border border-slate-700 flex items-center gap-1 opacity-0 group-hover/box:opacity-100 transition-opacity cursor-pointer"
                >
                  <Copy class="w-2.5 h-2.5" />
                  <span>{{ $t('settings.copy_payload') }}</span>
                </button>
                <pre class="overflow-x-auto select-text">{{ entry.details }}</pre>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- VIEW 2: HTTP REQUESTS (Table / Request List) -->
      <div v-else class="space-y-1.5">
        <div v-if="logs.length === 0" class="flex flex-col items-center justify-center py-16 text-center text-slate-500 space-y-1.5">
          <Terminal class="w-8 h-8 text-slate-600 opacity-60 mb-1" />
          <p class="text-xs font-medium text-slate-400">{{ $t('settings.http_logs_empty') }}</p>
          <p class="text-[11px] text-slate-500 max-w-md">{{ $t('settings.http_logs_empty_desc') }}</p>
        </div>

        <div
          v-for="log in logs"
          :key="log.id"
          class="rounded-lg bg-[#0e111a] border border-[#1c2030] hover:border-[#2b334a] transition-all overflow-hidden"
        >
          <!-- Summary Row -->
          <div
            @click="toggleRequestExpansion(log.id)"
            class="flex items-center justify-between px-3 py-2 text-[11px] cursor-pointer hover:bg-[#121624] transition-colors"
          >
            <div class="flex items-center gap-2.5 min-w-0 pr-2">
              <span :class="['px-1.5 py-0.5 rounded text-[9.5px] font-bold border uppercase tracking-wider flex-shrink-0', getMethodClass(log.method)]">
                {{ log.method }}
              </span>
              <span class="text-slate-200 truncate font-mono" :title="log.path">{{ log.path }}</span>
              <span v-if="log.model" class="text-[10px] px-1.5 py-0.2 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20 truncate hidden lg:inline">
                {{ log.model }}
              </span>
              <span v-if="log.timestamp" class="text-[10px] text-slate-500 hidden sm:inline flex-shrink-0">
                {{ formatLogTime(log.timestamp) }}
              </span>
            </div>

            <div class="flex items-center gap-3 flex-shrink-0">
              <span v-if="log.tokens_prompt || log.tokens_completion" class="text-[10px] text-slate-400 hidden md:inline font-mono">
                {{ log.tokens_prompt }}/{{ log.tokens_completion }} tok
              </span>
              <span :class="['font-semibold font-mono text-xs', getStatusClass(log.status_code)]">
                {{ log.status_code }}
              </span>
              <span class="text-slate-400 font-mono text-[10.5px] min-w-[45px] text-right">
                {{ log.latency_ms }}ms
              </span>
              <component :is="expandedRequestIds.has(log.id) ? ChevronDown : ChevronRight" class="w-3.5 h-3.5 text-slate-500" />
            </div>
          </div>

          <!-- Expanded Payload Details -->
          <div
            v-if="expandedRequestIds.has(log.id)"
            class="px-3.5 py-3 border-t border-[#1a1f30] bg-[#090b12] space-y-2 text-[10.5px]"
          >
            <div class="flex items-center justify-between text-slate-400">
              <div class="flex items-center gap-3 flex-wrap">
                <span><strong>ID:</strong> {{ log.id }}</span>
                <span v-if="log.model"><strong>Model:</strong> {{ log.model }}</span>
                <span><strong>Prompt Tokens:</strong> {{ log.tokens_prompt }}</span>
                <span><strong>Completion Tokens:</strong> {{ log.tokens_completion }}</span>
                <span><strong>Total Latency:</strong> {{ log.latency_ms }}ms</span>
              </div>
              <button
                v-if="log.body_preview"
                type="button"
                @click.stop="copySnippet(log.body_preview)"
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 flex items-center gap-1 cursor-pointer transition-colors"
              >
                <Copy class="w-2.5 h-2.5" />
                <span>{{ $t('settings.copy_payload') }}</span>
              </button>
            </div>

            <div v-if="log.body_preview" class="p-2.5 rounded-lg bg-[#0e111a] border border-[#1e2338] text-slate-300">
              <pre class="overflow-x-auto select-text">{{ log.body_preview }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  Terminal,
  RefreshCw,
  Trash2,
  Copy,
  Check,
  ChevronDown,
  ChevronRight,
  ArrowDownToLine,
  Loader2
} from 'lucide-vue-next'
import type { ServerRequestLog, DeveloperLogEntry } from '~/types'

interface Props {
  logs?: ServerRequestLog[]
  developerLogs?: DeveloperLogEntry[]
  isGenerating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  logs: () => [],
  developerLogs: () => [],
  isGenerating: false
})

const emit = defineEmits<{
  refreshLogs: []
  refreshDeveloperLogs: []
  clearLogs: []
  clearDeveloperLogs: []
}>()

const activeTab = ref<'developer' | 'requests'>('developer')
const levels = ['ALL', 'INFO', 'DEBUG', 'WARN', 'ERROR']
const selectedLevel = ref('ALL')
const autoScroll = ref(true)
const hasCopied = ref(false)
const isRefreshing = ref(false)

const expandedRequestIds = ref<Set<string>>(new Set())
const expandedLogIds = ref<Set<string>>(new Set())
const consoleContainer = ref<HTMLElement | null>(null)

const toggleRequestExpansion = (id: string) => {
  if (expandedRequestIds.value.has(id)) {
    expandedRequestIds.value.delete(id)
  } else {
    expandedRequestIds.value.add(id)
  }
}

const toggleLogExpansion = (id: string) => {
  if (expandedLogIds.value.has(id)) {
    expandedLogIds.value.delete(id)
  } else {
    expandedLogIds.value.add(id)
  }
}

const filteredDevLogs = computed(() => {
  if (selectedLevel.value === 'ALL') {
    return props.developerLogs
  }
  return props.developerLogs.filter(
    (l) => (l.level || '').toUpperCase() === selectedLevel.value
  )
})

// Auto scroll on new logs
watch(
  () => props.developerLogs.length,
  async () => {
    if (autoScroll.value && activeTab.value === 'developer') {
      await nextTick()
      scrollToBottom()
    }
  }
)

function scrollToBottom() {
  if (consoleContainer.value) {
    consoleContainer.value.scrollTop = consoleContainer.value.scrollHeight
  }
}

function formatTimestamp(iso?: string) {
  if (!iso) return ''
  try {
    const d = new Date(iso)
    const year = d.getFullYear()
    const month = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const hours = String(d.getHours()).padStart(2, '0')
    const minutes = String(d.getMinutes()).padStart(2, '0')
    const seconds = String(d.getSeconds()).padStart(2, '0')
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
  } catch (_) {
    return iso
  }
}

function formatLogTime(iso?: string) {
  if (!iso) return ''
  try {
    const d = new Date(iso)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch (_) {
    return ''
  }
}

function getLevelBadgeClass(level: string) {
  switch ((level || '').toUpperCase()) {
    case 'INFO':
      return 'text-sky-400 bg-sky-500/10 border border-sky-500/20'
    case 'DEBUG':
      return 'text-purple-400 bg-purple-500/10 border border-purple-500/20'
    case 'WARN':
      return 'text-amber-400 bg-amber-500/10 border border-amber-500/20'
    case 'ERROR':
      return 'text-rose-400 bg-rose-500/10 border border-rose-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10 border border-slate-500/20'
  }
}

function getFilterActiveClass(lvl: string) {
  switch (lvl) {
    case 'INFO':
      return 'bg-sky-500/20 text-sky-300 font-bold border border-sky-500/40'
    case 'DEBUG':
      return 'bg-purple-500/20 text-purple-300 font-bold border border-purple-500/40'
    case 'WARN':
      return 'bg-amber-500/20 text-amber-300 font-bold border border-amber-500/40'
    case 'ERROR':
      return 'bg-rose-500/20 text-rose-300 font-bold border border-rose-500/40'
    default:
      return 'bg-indigo-600 text-white font-bold'
  }
}

function getMethodClass(method: string) {
  switch ((method || '').toUpperCase()) {
    case 'GET':
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
    case 'POST':
      return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20'
    case 'DELETE':
      return 'bg-rose-500/10 text-rose-400 border-rose-500/20'
    case 'PUT':
    case 'PATCH':
      return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
    default:
      return 'bg-slate-500/10 text-slate-400 border-slate-500/20'
  }
}

function getStatusClass(status: number) {
  if (status >= 200 && status < 300) return 'text-emerald-400'
  if (status >= 300 && status < 400) return 'text-sky-400'
  if (status >= 400 && status < 500) return 'text-amber-400'
  return 'text-rose-400'
}

async function copySnippet(text: string) {
  try {
    await navigator.clipboard.writeText(text)
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
  }
}

async function copyLogsToClipboard() {
  let content = ''
  if (activeTab.value === 'developer') {
    content = filteredDevLogs.value
      .map((entry) => {
        const tag = entry.tag ? `[${entry.tag}] ` : ''
        const details = entry.details ? `\n${entry.details}` : ''
        return `${formatTimestamp(entry.timestamp)} [${entry.level}] ${tag}${entry.message}${details}`
      })
      .join('\n\n')
  } else {
    content = props.logs
      .map((log) => {
        const payload = log.body_preview ? `\nPayload: ${log.body_preview}` : ''
        return `${log.timestamp} ${log.method} ${log.path} - ${log.status_code} (${log.latency_ms}ms, ${log.tokens_prompt}/${log.tokens_completion} tok)${payload}`
      })
      .join('\n')
  }

  try {
    await navigator.clipboard.writeText(content)
    hasCopied.value = true
    setTimeout(() => {
      hasCopied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy logs:', err)
  }
}

function handleRefresh() {
  isRefreshing.value = true
  if (activeTab.value === 'developer') {
    emit('refreshDeveloperLogs')
  } else {
    emit('refreshLogs')
  }
  setTimeout(() => {
    isRefreshing.value = false
  }, 600)
}

function handleClear() {
  if (activeTab.value === 'developer') {
    emit('clearDeveloperLogs')
  } else {
    emit('clearLogs')
  }
}
</script>
