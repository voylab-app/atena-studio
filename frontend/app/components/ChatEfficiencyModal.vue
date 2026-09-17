<template>
  <Teleport to="body">
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md transition-all modal-backdrop-enter"
      @click.self="$emit('close')"
    >
      <div
        class="bg-[#0e111a] border border-[#23293f] rounded-3xl max-w-3xl w-full p-6 shadow-2xl space-y-5 text-slate-100 flex flex-col max-h-[90vh] overflow-hidden"
      >
        <!-- Modal Header -->
        <div class="flex items-center justify-between border-b border-[#1c2236] pb-4 flex-shrink-0">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shadow-inner">
              <Zap class="w-5 h-5" />
            </div>
            <div>
              <h2 class="text-base font-bold text-white flex items-center gap-2">
                <span>{{ $t('modals.efficiency.title') }}</span>
                <span class="px-2.5 py-0.5 rounded-full text-[10px] font-mono bg-indigo-500/15 text-indigo-300 border border-indigo-500/25 whitespace-nowrap">
                  {{ kvCacheQuantLabel }}
                </span>
              </h2>
              <p class="text-xs text-slate-400 mt-0.5">
                {{ $t('modals.efficiency.subtitle') }}
              </p>
            </div>
          </div>

          <button
            @click="$emit('close')"
            class="p-2 rounded-xl bg-[#141826] hover:bg-[#1f253a] text-slate-400 hover:text-white transition-all cursor-pointer border border-[#23293f]"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Scrollable Content -->
        <div class="space-y-5 overflow-y-auto pr-1 flex-1">
          <!-- Overall Efficiency Highlight Banner -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-3.5">
            <!-- 1. Cache Efficiency Gauge Card -->
            <div class="p-4 rounded-2xl bg-gradient-to-b from-[#141828] to-[#0f1320] border border-[#222a42] flex flex-col justify-between shadow-sm relative overflow-hidden">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider whitespace-nowrap">{{ $t('modals.efficiency.cache_efficiency') }}</span>
                <Sparkles class="w-4 h-4 text-indigo-400 flex-shrink-0" />
              </div>

              <div class="my-3 flex items-baseline gap-2 flex-wrap">
                <span class="text-3xl font-extrabold font-mono text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-300">
                  {{ overallEfficiencyPct.toFixed(1) }}%
                </span>
                <span class="text-xs text-emerald-400 font-medium font-mono whitespace-nowrap">{{ $t('modals.efficiency.reused') }}</span>
              </div>

              <!-- Progress Bar -->
              <div class="w-full bg-[#1b2236] h-2 rounded-full overflow-hidden">
                <div
                  class="bg-gradient-to-r from-emerald-500 to-teal-400 h-full rounded-full transition-all duration-500"
                  :style="{ width: `${Math.min(100, Math.max(0, overallEfficiencyPct))}%` }"
                ></div>
              </div>
            </div>

            <!-- 2. Prefill vs Cached Tokens Card -->
            <div class="p-4 rounded-2xl bg-gradient-to-b from-[#141828] to-[#0f1320] border border-[#222a42] flex flex-col justify-between shadow-sm">
              <div class="flex items-center justify-between mb-1">
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider whitespace-nowrap">{{ $t('modals.efficiency.context_tokens') }}</span>
                <Layers class="w-4 h-4 text-purple-400 flex-shrink-0" />
              </div>

              <div class="my-1.5 space-y-1.5">
                <div class="flex items-center justify-between text-xs gap-2">
                  <span class="text-slate-400 flex items-center gap-1.5 whitespace-nowrap">
                    <span class="w-2 h-2 rounded-full bg-emerald-400 flex-shrink-0"></span>
                    {{ $t('modals.efficiency.cached_reuse') }}
                  </span>
                  <span class="font-mono font-bold text-emerald-300 whitespace-nowrap">{{ totalCachedTokens }}</span>
                </div>
                <div class="flex items-center justify-between text-xs gap-2">
                  <span class="text-slate-400 flex items-center gap-1.5 whitespace-nowrap">
                    <span class="w-2 h-2 rounded-full bg-amber-400 flex-shrink-0"></span>
                    {{ $t('modals.efficiency.prefill_new') }}
                  </span>
                  <span class="font-mono font-bold text-amber-300 whitespace-nowrap">{{ totalPrefillTokens }}</span>
                </div>
                <div class="flex items-center justify-between text-xs pt-1.5 border-t border-[#1e253c] gap-2">
                  <span class="text-slate-400 flex items-center gap-1.5 whitespace-nowrap">
                    <span class="w-2 h-2 rounded-full bg-indigo-400 flex-shrink-0"></span>
                    {{ $t('modals.efficiency.generated_output') }}
                  </span>
                  <span class="font-mono font-bold text-indigo-300 whitespace-nowrap">{{ totalCompletionTokens }}</span>
                </div>
              </div>
            </div>

            <!-- 3. KV Cache & VRAM Savings Card -->
            <div class="p-4 rounded-2xl bg-gradient-to-b from-[#141828] to-[#0f1320] border border-[#222a42] flex flex-col justify-between shadow-sm">
              <div class="flex items-center justify-between mb-1">
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider whitespace-nowrap">{{ $t('modals.efficiency.vram_quant') }}</span>
                <Cpu class="w-4 h-4 text-sky-400 flex-shrink-0" />
              </div>

              <div class="my-1.5 space-y-1.5">
                <div class="flex items-center justify-between text-xs gap-2">
                  <span class="text-slate-400 whitespace-nowrap">{{ $t('modals.efficiency.quant_cache') }}</span>
                  <span class="font-mono font-bold text-sky-300 whitespace-nowrap text-right">{{ kvCacheQuantLabel }}</span>
                </div>
                <div class="flex items-center justify-between text-xs gap-2">
                  <span class="text-slate-400 whitespace-nowrap">{{ $t('modals.efficiency.vram_savings') }}</span>
                  <span class="font-mono font-bold text-emerald-400 whitespace-nowrap text-right">{{ $t('modals.efficiency.vram_savings_val', { pct: vramSavingsPct }) }}</span>
                </div>
                <div class="flex items-center justify-between text-xs pt-1.5 border-t border-[#1e253c] gap-2">
                  <span class="text-slate-400 whitespace-nowrap">{{ $t('modals.efficiency.flash_attention') }}</span>
                  <span class="font-mono font-bold text-indigo-300 whitespace-nowrap text-right">{{ flashAttnLabel }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Speed & Latency Metrics Bar -->
          <div class="p-3.5 rounded-2xl bg-[#111524] border border-[#1e253a] grid grid-cols-3 gap-3 text-center">
            <div>
              <span class="text-[10.5px] text-slate-400 block mb-0.5">{{ $t('modals.efficiency.avg_generation_speed') }}</span>
              <span class="text-lg font-bold font-mono text-indigo-300">
                {{ avgGenerationSpeed > 0 ? `${avgGenerationSpeed.toFixed(1)} t/s` : '—' }}
              </span>
            </div>
            <div class="border-x border-[#1e253a]">
              <span class="text-[10.5px] text-slate-400 block mb-0.5">{{ $t('modals.efficiency.prefill_speed') }}</span>
              <span class="text-lg font-bold font-mono text-emerald-300">
                {{ latestPrefillSpeed > 0 ? `${latestPrefillSpeed.toFixed(1)} t/s` : '—' }}
              </span>
            </div>
            <div>
              <span class="text-[10.5px] text-slate-400 block mb-0.5">{{ $t('modals.efficiency.latency_ttft') }}</span>
              <span class="text-lg font-bold font-mono text-purple-300">
                {{ latestTtft > 0 ? `${latestTtft} ms` : '—' }}
              </span>
            </div>
          </div>

          <!-- Turn-by-Turn History Breakdown -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-bold text-slate-200 uppercase tracking-wider flex items-center gap-1.5">
                <Clock class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('modals.efficiency.history_title') }}</span>
              </h3>
              <span class="text-[10.5px] text-slate-400 font-mono">
                {{ turnsWithMetrics.length === 1 ? $t('modals.efficiency.interactions_single') : $t('modals.efficiency.interactions_multiple', { count: turnsWithMetrics.length }) }}
              </span>
            </div>

            <div v-if="turnsWithMetrics.length === 0" class="p-6 rounded-2xl bg-[#111422] border border-[#1b2034] text-center text-xs text-slate-400">
              {{ $t('modals.efficiency.empty_history') }}
            </div>

            <div v-else class="rounded-2xl border border-[#1e2439] bg-[#0c0f18] overflow-hidden">
              <table class="w-full text-left text-xs">
                <thead class="bg-[#141826] text-slate-400 font-medium border-b border-[#1e2439] select-none text-[10.5px]">
                  <tr>
                    <th class="py-2.5 px-3">#</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_prompt') }}</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_cached') }}</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_prefill') }}</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_generated') }}</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_efficiency') }}</th>
                    <th class="py-2.5 px-3">{{ $t('modals.efficiency.col_speed') }}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-[#171d2e] font-mono text-[11px]">
                  <tr
                    v-for="(t, idx) in turnsWithMetrics"
                    :key="idx"
                    class="hover:bg-white/[0.02] transition-colors"
                  >
                    <td class="py-2 px-3 text-slate-500">#{{ idx + 1 }}</td>
                    <td class="py-2 px-3 text-slate-300 font-semibold">{{ t.metrics?.prompt_tokens ?? 0 }}</td>
                    <td class="py-2 px-3 text-emerald-400 font-semibold">{{ t.metrics?.cached_tokens ?? 0 }}</td>
                    <td class="py-2 px-3 text-amber-400 font-semibold">{{ t.metrics?.prefill_tokens ?? 0 }}</td>
                    <td class="py-2 px-3 text-indigo-300">{{ t.metrics?.completion_tokens ?? 0 }}</td>
                    <td class="py-2 px-3">
                      <span
                        :class="[
                          'px-1.5 py-0.5 rounded text-[10px] font-bold border',
                          (t.metrics?.cache_efficiency_pct ?? 0) >= 60
                            ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
                            : (t.metrics?.cache_efficiency_pct ?? 0) >= 30
                            ? 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30'
                            : 'bg-slate-500/15 text-slate-400 border-slate-500/30'
                        ]"
                      >
                        {{ (t.metrics?.cache_efficiency_pct || 0).toFixed(1) }}%
                      </span>
                    </td>
                    <td class="py-2 px-3 text-slate-300">
                      {{ t.metrics?.generation_speed_tps ? `${t.metrics.generation_speed_tps.toFixed(1)} t/s` : '—' }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Educational Info Tip -->
          <div class="p-3.5 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 text-xs text-indigo-200/90 leading-relaxed flex items-start gap-2.5">
            <Info class="w-4 h-4 text-indigo-400 flex-shrink-0 mt-0.5" />
            <div>
              <span class="font-bold text-white block mb-0.5">{{ $t('modals.efficiency.info_title') }}</span>
              {{ $t('modals.efficiency.info_desc') }}
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="border-t border-[#1c2236] pt-3 flex items-center justify-between flex-shrink-0">
          <div class="text-[11px] text-slate-400">
            {{ $t('modals.efficiency.model_label') }} <span class="font-semibold text-slate-200">{{ activeModel ? activeModel.name : $t('common.none') }}</span>
          </div>
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all cursor-pointer active:scale-95"
          >
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Zap,
  X,
  Sparkles,
  Layers,
  Cpu,
  Clock,
  Info
} from 'lucide-vue-next'
import { useAppLocale } from '~/composables/useLocale'
import type { Session, Model, GenerationParams } from '~/types'

const { t } = useAppLocale()

const props = defineProps<{
  show: boolean
  session?: Session | null
  activeModel?: Model | null
  params?: GenerationParams | null
}>()

defineEmits<{
  (e: 'close'): void
}>()

const turnsWithMetrics = computed(() => {
  if (!props.session?.messages) return []
  return props.session.messages
    .filter((m) => m.role === 'assistant' && !!m.metrics)
    .map((m) => ({
      metrics: m.metrics!
    }))
})

const totalCachedTokens = computed(() => {
  return turnsWithMetrics.value.reduce((acc, t) => acc + (t.metrics?.cached_tokens || 0), 0)
})

const totalPrefillTokens = computed(() => {
  return turnsWithMetrics.value.reduce((acc, t) => acc + (t.metrics?.prefill_tokens || 0), 0)
})

const totalCompletionTokens = computed(() => {
  return turnsWithMetrics.value.reduce((acc, t) => acc + (t.metrics?.completion_tokens || 0), 0)
})

const overallEfficiencyPct = computed(() => {
  const totPrompt = totalCachedTokens.value + totalPrefillTokens.value
  if (totPrompt <= 0) return 0
  return (totalCachedTokens.value / totPrompt) * 100
})

const avgGenerationSpeed = computed(() => {
  const speeds = turnsWithMetrics.value
    .map((t) => t.metrics?.generation_speed_tps)
    .filter((s): s is number => typeof s === 'number' && s > 0)
  if (speeds.length === 0) return 0
  return speeds.reduce((a, b) => a + b, 0) / speeds.length
})

const latestPrefillSpeed = computed(() => {
  const last = turnsWithMetrics.value[turnsWithMetrics.value.length - 1]
  return last?.metrics?.prefill_speed_tps || 0
})

const latestTtft = computed(() => {
  const last = turnsWithMetrics.value[turnsWithMetrics.value.length - 1]
  return last?.metrics?.time_to_first_token_ms || 0
})

const kvCacheQuantLabel = computed(() => {
  const q = props.params?.kv_cache_quant || 'f16'
  switch (q.toLowerCase()) {
    case 'q8_0':
      return 'Q8_0 (8-bit)'
    case 'q4_0':
      return 'Q4_0 (4-bit)'
    case 'q4_1':
      return 'Q4_1 (4-bit)'
    case 'q5_0':
      return 'Q5_0 (5-bit)'
    case 'iq4_nl':
      return 'IQ4_NL (4-bit)'
    default:
      return t('modals.efficiency.f16_uncompressed')
  }
})

const vramSavingsPct = computed(() => {
  const q = props.params?.kv_cache_quant || 'f16'
  switch (q.toLowerCase()) {
    case 'q4_0':
    case 'q4_1':
    case 'iq4_nl':
      return 75
    case 'q5_0':
      return 68
    case 'q8_0':
      return 50
    default:
      return 0
  }
})

const flashAttnLabel = computed(() => {
  return props.params?.flash_attention !== false ? t('modals.efficiency.flash_attn_active') : t('common.disabled')
})
</script>

<style scoped>
.modal-backdrop-enter {
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
