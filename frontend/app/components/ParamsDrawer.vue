<template>
  <div>
    <!-- Backdrop Overlay -->
    <Transition
      enter-active-class="transition-opacity duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="modelValue"
        @click="$emit('update:modelValue', false)"
        class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40"
      ></div>
    </Transition>

    <!-- Slide-over Drawer Panel -->
    <Transition
      enter-active-class="transition-transform duration-300 cubic-bezier(0.16, 1, 0.3, 1)"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition-transform duration-200 ease-in"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <div
        v-if="modelValue"
        class="fixed top-0 right-0 h-full w-[360px] max-w-[90vw] bg-[#0f121d] border-l border-[#22283b] shadow-2xl z-50 flex flex-col select-none"
      >
        <!-- Drawer Header -->
        <div class="px-5 py-4 border-b border-[#22283b] flex items-center justify-between bg-[#131724]">
          <div class="flex items-center gap-2.5">
            <div class="p-1.5 rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
              <SlidersHorizontal class="w-4 h-4" />
            </div>
            <div>
              <h3 class="text-sm font-bold text-slate-100">{{ $t('params.drawer_title') }}</h3>
              <p class="text-[10px] text-slate-400">{{ $t('params.drawer_subtitle') }}</p>
            </div>
          </div>

          <button
            @click="$emit('update:modelValue', false)"
            class="p-1.5 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-[#1f2538] transition-all"
            :title="$t('params.close_tooltip')"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Drawer Body -->
        <div class="flex-1 overflow-y-auto p-5 space-y-5 text-xs text-slate-300">
          <!-- System Prompt -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="font-semibold text-slate-200">{{ $t('params.system_prompt') }}</label>
              <span class="text-[10px] text-slate-400 font-mono">{{ (params.system_prompt || '').length }} chars</span>
            </div>
            <textarea
              v-model="params.system_prompt"
              rows="3"
              :placeholder="$t('params.system_prompt_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] focus:border-indigo-500 focus:ring-2 focus:ring-indigo-500/20 text-slate-100 placeholder-slate-400 text-xs transition-all resize-none outline-none font-sans select-text"
            ></textarea>
            <p class="text-[10px] text-slate-400">
              {{ $t('params.system_prompt_desc') }}
            </p>
          </div>

          <!-- Modo de Raciocínio (Reasoning Mode) -->
          <div class="p-3.5 rounded-xl bg-gradient-to-b from-[#181426] to-[#120f1e] border border-[#3b2b5c]/80 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Brain class="w-4 h-4 text-purple-400" />
                <div>
                  <span class="font-bold text-slate-100 text-xs">{{ $t('params.reasoning_mode') }}</span>
                  <p class="text-[10px] text-purple-300/80">Gemma 4, DeepSeek R1, Qwen Thinking</p>
                </div>
              </div>
              <button
                type="button"
                role="switch"
                :aria-checked="Boolean(params.enable_thinking !== false)"
                @click="params.enable_thinking = !params.enable_thinking"
                :class="[
                  'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                  params.enable_thinking !== false ? 'bg-purple-600' : 'bg-slate-700'
                ]"
              >
                <span
                  :class="[
                    'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    params.enable_thinking !== false ? 'translate-x-4' : 'translate-x-0'
                  ]"
                />
              </button>
            </div>
          </div>

          <!-- Temperature -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">Temperature</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ Number(params.temperature).toFixed(2) }}
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.temperature"
              min="0.0"
              max="2.0"
              step="0.05"
            />
            <div class="flex justify-between text-[10px] text-slate-400 font-mono">
              <span>{{ $t('params.precise') }} (0.0)</span>
              <span>{{ $t('params.creative') }} (2.0)</span>
            </div>
          </div>

          <!-- Top P -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">Top P (Nucleus Sampling)</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ Number(params.top_p).toFixed(2) }}
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.top_p"
              min="0.0"
              max="1.0"
              step="0.05"
            />
            <p class="text-[10px] text-slate-400">
              {{ $t('params.top_p_desc') }}
            </p>
          </div>

          <!-- Janela de Contexto (Context Length / Window) -->
          <div class="space-y-2.5 p-3.5 rounded-xl bg-gradient-to-b from-[#151928] to-[#111422] border border-indigo-500/30 shadow-sm">
            <div class="flex justify-between items-center">
              <div>
                <label class="font-bold text-slate-100 text-xs">{{ $t('params.context_window') }}</label>
                <p class="text-[10px] text-slate-400">{{ $t('params.context_window_desc') }}</p>
              </div>
              <div class="flex items-center gap-1">
                <input
                  type="number"
                  v-model.number="params.context_length"
                  min="512"
                  max="131072"
                  step="512"
                  class="w-20 px-2 py-0.5 rounded-lg bg-[#0d101a] border border-[#22283b] focus:border-indigo-500 text-right font-mono text-xs font-bold text-indigo-300 outline-none"
                />
                <span class="text-[10.5px] text-slate-400 font-mono">tok</span>
              </div>
            </div>

            <!-- Slider -->
            <input
              type="range"
              v-model.number="params.context_length"
              min="1024"
              max="65536"
              step="1024"
              class="w-full cursor-pointer accent-indigo-500"
            />

            <!-- Quick Presets -->
            <div class="flex flex-wrap gap-1 pt-1">
              <button
                v-for="preset in [2048, 4096, 8000, 8192, 16384, 32768, 65536]"
                :key="preset"
                type="button"
                @click="params.context_length = preset"
                :class="[
                  'px-2 py-0.5 rounded-md text-[10px] font-mono font-semibold transition-all border cursor-pointer',
                  params.context_length === preset
                    ? 'bg-indigo-600 text-white border-indigo-500 shadow-sm shadow-indigo-600/30'
                    : 'bg-[#121522] text-slate-400 border-[#1f2538] hover:text-slate-200 hover:bg-[#181d2e]'
                ]"
              >
                {{ preset >= 1000 ? `${preset === 8000 ? '8k' : Math.round(preset / 1024) + 'k'}` : preset }}
              </button>
            </div>

            <p class="text-[10px] text-slate-400 leading-relaxed">
              {{ $t('params.context_window_tip') }}
            </p>
          </div>

          <!-- Max Tokens -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">{{ $t('params.max_tokens_label') }}</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ params.max_tokens }}
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.max_tokens"
              min="256"
              max="8192"
              step="128"
            />
            <div class="flex justify-between text-[10px] text-slate-400 font-mono">
              <span>256</span>
              <span>8192</span>
            </div>
          </div>

          <!-- GPU Layers Offload -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">{{ $t('params.gpu_layers') }}</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/20">
                {{ params.gpu_layers }}/64
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.gpu_layers"
              min="0"
              max="64"
              step="1"
            />
            <p class="text-[10px] text-slate-400">
              {{ $t('params.gpu_layers_desc') }}
            </p>
          </div>

          <!-- Top K -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">Top K</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ params.top_k || 40 }}
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.top_k"
              min="1"
              max="100"
              step="1"
            />
            <p class="text-[10px] text-slate-400">
              {{ $t('params.top_k_desc') }}
            </p>
          </div>

          <!-- Repeat Penalty -->
          <div class="space-y-1.5 p-3 rounded-xl bg-[#131724] border border-[#1e2436]">
            <div class="flex justify-between items-center">
              <label class="font-medium text-slate-200">Repeat Penalty</label>
              <span class="font-mono text-xs font-semibold px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ Number(params.repeat_penalty || 1.1).toFixed(2) }}
              </span>
            </div>
            <input
              type="range"
              v-model.number="params.repeat_penalty"
              min="1.0"
              max="2.0"
              step="0.05"
            />
            <p class="text-[10px] text-slate-400">
              {{ $t('params.repeat_penalty_desc') }}
            </p>
          </div>
        </div>

        <!-- Drawer Footer -->
        <div class="p-4 border-t border-[#22283b] bg-[#131724] flex items-center justify-between">
          <button
            @click="resetDefaults"
            class="text-[11px] text-slate-400 hover:text-slate-200 transition-colors"
          >
            {{ $t('modals.params.reset_defaults') }}
          </button>
          <button
            @click="$emit('update:modelValue', false)"
            class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs shadow-md shadow-indigo-600/20 transition-all"
          >
            {{ $t('modals.params.apply') }}
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { SlidersHorizontal, X, Brain } from 'lucide-vue-next'
import type { GenerationParams } from '~/types'

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    params: GenerationParams
  }>(),
  {
    modelValue: false,
    params: () => ({
      system_prompt: '',
      temperature: 0.7,
      top_p: 0.9,
      top_k: 40,
      max_tokens: 2048,
      repeat_penalty: 1.1,
      gpu_layers: 64,
      context_length: 8192,
      enable_thinking: true,
      thinking_budget: 2048
    })
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void
}>()

const resetDefaults = () => {
  props.params.temperature = 0.7
  props.params.top_p = 0.9
  props.params.top_k = 40
  props.params.max_tokens = 2048
  props.params.repeat_penalty = 1.1
  props.params.gpu_layers = 64
  props.params.context_length = 8192
}
</script>

