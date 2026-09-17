<template>
  <header
    class="h-14 min-h-[56px] bg-[#0c0e16] border-b border-[#1c2030] px-3 sm:px-5 flex items-center justify-between gap-2 sm:gap-4 select-none relative z-30">
    <!-- Left Section: Minimal Status -->
    <div class="flex items-center gap-2 flex-shrink-0">
      <div
        v-if="activeModel && !isLoadingModel"
        class="flex items-center gap-1.5 px-2 py-1 rounded-xl text-[10px] font-bold tracking-wider uppercase border transition-all bg-emerald-500/10 text-emerald-400 border-emerald-500/30">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="hidden sm:inline">{{ $t('header.active') }}</span>
      </div>

      <!-- Private Chat Badge -->
      <div v-if="currentSession?.is_private"
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[10.5px] font-semibold tracking-wide border bg-violet-500/10 text-violet-300 border-violet-500/25 shadow-sm select-none animate-in fade-in"
        :title="$t('header.private_mode_tooltip')">
        <EyeOff class="w-3.5 h-3.5 text-violet-400 flex-shrink-0" />
        <span class="hidden sm:inline">{{ $t('header.private_mode') }}</span>
      </div>

      <!-- Archived Chat Badge -->
      <div v-if="currentSession?.archived"
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[10.5px] font-semibold tracking-wide border bg-amber-500/10 text-amber-300 border-amber-500/25 shadow-sm select-none animate-in fade-in"
        :title="$t('header.archived_mode_tooltip')">
        <Archive class="w-3.5 h-3.5 text-amber-400 flex-shrink-0" />
        <span class="hidden sm:inline">{{ $t('header.archived_mode') }}</span>
      </div>

      <!-- Web Server Mode Badge -->
      <div v-if="isWebMode"
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[10.5px] font-semibold tracking-wide border bg-sky-500/10 text-sky-300 border-sky-500/25 shadow-sm select-none animate-in fade-in"
        :title="$t('header.web_mode_tooltip')">
        <Globe class="w-3.5 h-3.5 text-sky-400 flex-shrink-0" />
        <span class="hidden sm:inline">{{ $t('header.web_mode') }}</span>
      </div>
    </div>

    <!-- Center Section: AI Model Capsule (Responsive Flex Center) -->
    <div class="flex-1 flex items-center justify-center min-w-0 max-w-xl mx-auto gap-1.5">
      <div :class="[
        'flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-3.5 py-1.5 rounded-2xl border transition-all shadow-md backdrop-blur-md min-w-0 max-w-full overflow-hidden relative',
        isLoadingModel
          ? 'bg-[#181a28] border-amber-500/40 text-amber-200'
          : activeModel
            ? 'bg-[#121524]/95 border-[#20273d] hover:border-indigo-500/40 text-slate-100'
            : 'bg-[#101320]/80 border-[#1a1f30] text-slate-400'
      ]">
        <!-- Loading State -->
        <template v-if="isLoadingModel">
          <Loader2 class="w-3.5 h-3.5 animate-spin text-amber-400 flex-shrink-0" />
          <span class="text-xs font-semibold text-amber-300 truncate max-w-[120px] sm:max-w-[180px]">
            {{ loadingModelName || $t('models.loading_model') }}
          </span>
          <span class="text-[10.5px] font-mono font-bold text-amber-300 bg-amber-500/25 px-1.5 py-0.5 rounded-md border border-amber-500/30 flex-shrink-0">
            {{ loadingModelProgress }}%
          </span>
          <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-amber-500/20 overflow-hidden">
            <div class="h-full bg-gradient-to-r from-amber-500 to-amber-300 transition-all duration-300 ease-out"
              :style="{ width: `${loadingModelProgress}%` }"></div>
          </div>
        </template>

        <!-- Loaded Model State -->
        <template v-else-if="activeModel">
          <span class="text-xs font-semibold text-slate-100 truncate max-w-[120px] sm:max-w-[200px] md:max-w-xs"
            :title="activeModel.name">
            {{ activeModel.name }}
          </span>

          <span class="text-[11px] text-slate-500">•</span>
          <span class="text-[11px] font-mono text-indigo-300 font-medium whitespace-nowrap">
            {{ displayedQuantization }}
          </span>

          <span v-if="activeModel.size_gb && activeModel.size_gb > 0"
            class="hidden md:inline text-[11px] text-slate-500">•</span>
          <span v-if="activeModel.size_gb && activeModel.size_gb > 0"
            class="hidden md:inline text-[11px] font-mono text-slate-400 whitespace-nowrap">
            {{ `${formatGb(activeModel.size_gb || 0)} GB` }}
          </span>

          <!-- Antigravity Limit Indicator (Only for loaded AGY models with available quota) -->
          <template v-if="isAgyActive && activeAgyLimitPercent !== null && activeAgyLimitPercent !== undefined">
            <span class="text-[11px] text-slate-500">•</span>
            <div
              @click="$emit('openAgyModal')"
              class="flex items-center gap-1.5 px-2 py-0.5 rounded-lg text-[10.5px] font-medium border cursor-pointer hover:brightness-110 active:scale-95 transition-all"
              :class="[
                activeAgyLimitPercent >= 50
                  ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/25'
                  : activeAgyLimitPercent >= 20
                    ? 'bg-amber-500/10 text-amber-300 border-amber-500/25'
                    : 'bg-rose-500/10 text-rose-300 border-rose-500/25'
              ]" :title="activeAgyLimitTooltip">
              <Gauge class="w-3 h-3 text-amber-400 flex-shrink-0" />
              <span>{{ activeAgyLimitLabel }}:</span>
              <span class="font-mono font-bold">
                {{ `${activeAgyLimitPercent}%` }}
              </span>
              <span v-if="activeAgyLimitCountdown"
                class="text-[9.5px] text-slate-400 font-mono hidden lg:inline">
                ({{ activeAgyLimitCountdown }})
              </span>
            </div>
          </template>

          <!-- Capability Badges (Compact Icons) -->
          <div class="hidden sm:flex items-center gap-1 ml-0.5 sm:ml-1 flex-shrink-0">
            <span v-if="activeModel.supports_thinking"
              class="w-4.5 h-4.5 rounded-md bg-purple-500/15 text-purple-300 border border-purple-500/30 flex items-center justify-center p-0.5 cursor-help transition-transform hover:scale-110"
              :title="$t('header.thinking_tooltip')">
              <Brain class="w-3 h-3 text-purple-400" />
            </span>

            <span v-if="activeModel.supports_vision"
              class="w-4.5 h-4.5 rounded-md bg-sky-500/15 text-sky-300 border border-sky-500/30 flex items-center justify-center p-0.5 cursor-help transition-transform hover:scale-110"
              :title="$t('header.multimodal_tooltip')">
              <Eye class="w-3 h-3 text-sky-400" />
            </span>

            <span v-if="activeModel.supports_tools"
              class="w-4.5 h-4.5 rounded-md bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 flex items-center justify-center p-0.5 cursor-help transition-transform hover:scale-110"
              :title="$t('header.tools_tooltip')">
              <Wrench class="w-3 h-3 text-emerald-400" />
            </span>
          </div>
        </template>

        <!-- No Model Loaded State -->
        <template v-else>
          <span class="text-xs text-slate-400 italic truncate">
            {{ $t('header.no_model_selected') }}
          </span>
        </template>
      </div>

      <!-- Quick Unload / Eject Button next to capsule -->
      <button v-if="activeModel && !isLoadingModel" @click="$emit('unloadModel')"
        class="p-1.5 rounded-xl bg-[#121522] hover:bg-rose-500/20 border border-[#1e2338] hover:border-rose-500/40 text-slate-400 hover:text-rose-300 transition-all shadow-sm active:scale-90 cursor-pointer flex-shrink-0"
        :title="$t('models.unload_model')">
        <Power class="w-3.5 h-3.5 text-rose-400" />
      </button>
    </div>

    <!-- Right Section: Telemetry & Parameters Drawer -->
    <div class="flex items-center gap-1.5 sm:gap-2 relative flex-shrink-0">
      <!-- AI VRAM Status Pill -->
      <div
        class="hidden md:flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-mono font-medium border bg-[#121522] border-teal-500/20 text-teal-300 transition-all cursor-help"
        :title="$t('header.ai_vram_tooltip', { vram: formatGb(hardware.ai_ram_gb || hardware.used_vram_gb), used: formatGb(hardware.used_ram_gb), total: formatGb(hardware.total_ram_gb), pct: ramPct, free: formatGb(hardware.free_ram_gb) })">
        <Zap class="w-3 h-3 text-teal-400 flex-shrink-0" />
        <span class="text-[10px] uppercase font-sans font-semibold text-slate-400 hidden lg:inline">IA VRAM</span>
        <span class="font-bold text-teal-300">{{ formatGb(hardware.ai_ram_gb || hardware.used_vram_gb) }} GB</span>
      </div>

      <!-- Theme Quick Toggle (Light / Dark) -->
      <button
        @click="$emit('toggleTheme')"
        class="p-2 rounded-xl border bg-[#121522] hover:bg-[#181d2e] border-[#1e2338] hover:border-indigo-500/30 text-slate-200 transition-all shadow-sm active:scale-90 cursor-pointer flex items-center justify-center"
        :title="isDark ? $t('header.toggle_theme_light') : $t('header.toggle_theme_dark')"
      >
        <Sun v-if="isDark" class="w-3.5 h-3.5 text-amber-400 hover:rotate-45 transition-transform" />
        <Moon v-else class="w-3.5 h-3.5 text-indigo-400 hover:-rotate-12 transition-transform" />
      </button>

      <!-- Parameters Drawer Button -->
      <button @click="$emit('toggleDrawer')" :class="[
        'flex items-center gap-1.5 px-2.5 sm:px-3.5 py-1.5 rounded-xl text-xs font-medium transition-all border shadow-sm active:scale-[0.98] cursor-pointer',
        isDrawerOpen
          ? 'bg-indigo-600 border-indigo-500 text-white shadow-indigo-600/25'
          : 'bg-[#121522] hover:bg-[#181d2e] border-[#1e2338] hover:border-indigo-500/30 text-slate-200'
      ]">
        <SlidersHorizontal class="w-3.5 h-3.5 text-indigo-400" />
        <span class="hidden md:inline">{{ isDrawerOpen ? $t('common.close') : $t('header.parameters') }}</span>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, inject, type ComputedRef } from 'vue'
import {
  Power,
  SlidersHorizontal,
  Loader2,
  Brain,
  Eye,
  EyeOff,
  Wrench,
  Zap,
  Gauge,
  Archive,
  Sun,
  Moon,
  Globe
} from 'lucide-vue-next'
import { isBrowserMode } from '~/utils/ipcAdapter'
import type { Model, HardwareInfo, Session } from '~/types'

const isWebMode = computed(() => isBrowserMode())

const supportsMlx = inject<ComputedRef<boolean>>('supportsMlx', computed(() => true))

const props = defineProps<{
  activeModel?: Model | null
  isLoadingModel?: boolean
  loadingModelName?: string
  loadingModelProgress?: number
  hardware?: HardwareInfo | any
  agyUsageSummary?: any
  isDrawerOpen?: boolean
  currentSession?: Session | null
  isDark?: boolean
  theme?: string
}>()

defineEmits<{
  (e: 'unloadModel'): void
  (e: 'toggleDrawer'): void
  (e: 'openAgyModal'): void
  (e: 'toggleTheme'): void
}>()

const formatGb = (val: any) => (val ? Number(val).toFixed(1) : '0.0')

const isAgyActive = computed(() => {
  if (!props.activeModel) return false
  return (
    props.activeModel.backend === 'Antigravity' ||
    props.activeModel.format === 'Agy' ||
    (props.activeModel.id && props.activeModel.id.startsWith('agy/')) ||
    props.activeModel.quantization === 'Antigravity'
  )
})

const displayedQuantization = computed(() => {
  if (!props.activeModel) return ''
  if (isAgyActive.value) return 'Antigravity'
  return props.activeModel.quantization || ''
})

const vramPct = computed(() => {
  if (!props.hardware?.total_vram_gb) return 0
  return Math.min(100, Math.round(((props.hardware.ai_ram_gb || props.hardware.used_vram_gb || 0) / props.hardware.total_vram_gb) * 100))
})

const ramPct = computed(() => {
  if (!props.hardware?.total_ram_gb) return 0
  return Math.min(100, Math.round(((props.hardware.used_ram_gb || 0) / props.hardware.total_ram_gb) * 100))
})

const resolveQuotaPriority = (
  percent5h?: number | null,
  countdown5h?: string | null,
  percentWeekly?: number | null,
  countdownWeekly?: string | null
) => {
  const has5h = percent5h !== null && percent5h !== undefined
  const hasWeekly = percentWeekly !== null && percentWeekly !== undefined

  if (has5h && hasWeekly) {
    if (percent5h <= percentWeekly) {
      return { percent: percent5h, label: '5h', countdown: countdown5h || '' }
    } else {
      return { percent: percentWeekly, label: 'Semana', countdown: countdownWeekly || '' }
    }
  }
  if (hasWeekly) {
    return { percent: percentWeekly, label: 'Semana', countdown: countdownWeekly || '' }
  }
  if (has5h) {
    return { percent: percent5h, label: '5h', countdown: countdown5h || '' }
  }
  return null
}

const activeAgyModelLimitInfo = computed(() => {
  if (!props.activeModel || !props.agyUsageSummary || !isAgyActive.value) {
    return { percent: null, label: 'Cota', countdown: '' }
  }
  const s = props.agyUsageSummary
  const modelName = (props.activeModel.name || '').toLowerCase()
  const modelId = (props.activeModel.id || '').toLowerCase()
  const isClaudeOrGpt = modelName.includes('claude') || modelId.includes('claude') || modelName.includes('gpt') || modelId.includes('gpt')

  if (isClaudeOrGpt) {
    const q = resolveQuotaPriority(s.claude5hPercent, s.claude5hCountdown || s.lowest5hCountdown, s.claudeWeeklyPercent, s.claudeWeeklyCountdown || s.lowestWeeklyCountdown)
    if (q) return q
  } else {
    // Gemini
    const q = resolveQuotaPriority(s.gemini5hPercent, s.gemini5hCountdown || s.lowest5hCountdown, s.geminiWeeklyPercent, s.geminiWeeklyCountdown || s.lowestWeeklyCountdown)
    if (q) return q
  }

  // Fallbacks gerais
  const fallback = resolveQuotaPriority(s.fiveHourPercent, s.lowest5hCountdown, s.weeklyPercent, s.lowestWeeklyCountdown)
  if (fallback) return fallback

  return { percent: null, label: 'Cota', countdown: '' }
})

const activeAgyLimitPercent = computed(() => activeAgyModelLimitInfo.value.percent)
const activeAgyLimitLabel = computed(() => activeAgyModelLimitInfo.value.label)
const activeAgyLimitCountdown = computed(() => activeAgyModelLimitInfo.value.countdown)

const activeAgyLimitTooltip = computed(() => {
  const p = activeAgyLimitPercent.value
  const lbl = activeAgyLimitLabel.value
  const pStr = p !== null && p !== undefined ? `${p}% disponível` : 'Disponível'
  const cd = activeAgyLimitCountdown.value
  const countStr = cd ? ` • Renova em ${cd}` : ''
  const modelTitle = props.activeModel?.name || 'Modelo'
  return `Limite ${lbl === 'Semana' ? 'Semanal' : 'da janela de ' + lbl} (${modelTitle}): ${pStr}${countStr}. Clique para ver cotas completas.`
})
</script>
