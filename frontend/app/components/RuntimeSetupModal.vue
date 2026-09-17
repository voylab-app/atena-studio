<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-fade-in">
    <div class="w-full max-w-lg rounded-2xl bg-[#0e111a] border border-[#20273c] shadow-2xl p-6 space-y-4 text-slate-100 font-sans">
      
      <!-- Header -->
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-indigo-500 via-purple-500 to-pink-500 flex items-center justify-center text-white font-bold text-base shadow-lg shadow-indigo-500/20">
            <Cpu class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-100">Gerenciador de Motores de IA</h3>
            <p class="text-xs text-slate-400">Instalação, reparação e atualização 100% isolada</p>
          </div>
        </div>

        <button 
          v-if="!isBootstrapping" 
          @click="closeModal" 
          class="p-1 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-[#1a2032] transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Tab Selector -->
      <div class="flex items-center p-1 rounded-xl bg-[#131726] border border-[#1e2538] gap-1 text-xs">
        <button
          type="button"
          @click="activeTab = 'all'"
          :disabled="isBootstrapping"
          class="flex-1 py-1.5 px-2 rounded-lg font-medium transition-all text-center cursor-pointer"
          :class="[
            activeTab === 'all'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          Todos os Motores
        </button>

        <button
          type="button"
          @click="activeTab = 'llama'"
          :disabled="isBootstrapping"
          class="flex-1 py-1.5 px-2 rounded-lg font-medium transition-all text-center cursor-pointer"
          :class="[
            activeTab === 'llama'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          GGUF (llama.cpp)
        </button>

        <button
          type="button"
          @click="activeTab = 'ffmpeg'"
          :disabled="isBootstrapping"
          class="flex-1 py-1.5 px-2 rounded-lg font-medium transition-all text-center cursor-pointer"
          :class="[
            activeTab === 'ffmpeg'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          FFmpeg (Áudio)
        </button>

        <button
          v-if="supportsMlx"
          type="button"
          @click="activeTab = 'mlx'"
          :disabled="isBootstrapping"
          class="flex-1 py-1.5 px-2 rounded-lg font-medium transition-all text-center cursor-pointer"
          :class="[
            activeTab === 'mlx'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          Apple MLX
        </button>
      </div>

      <!-- Tab Contents -->
      <div class="space-y-3 text-xs text-slate-300 leading-relaxed">
        <!-- ALL TAB -->
        <div v-if="activeTab === 'all'" class="space-y-2.5">
          <p>
            Configura simultaneamente os motores <strong>GGUF (llama.cpp com Metal)</strong>, <strong>FFmpeg (Áudio & Whisper)</strong> e a <strong>aceleração Apple Silicon MLX</strong> em ambiente isolado.
          </p>

          <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
            <div class="p-2.5 rounded-xl bg-[#141826] border border-[#20273d] space-y-1">
              <div class="flex items-center justify-between text-[11px]">
                <span class="font-bold text-slate-200">GGUF llama-server</span>
                <span :class="['w-2 h-2 rounded-full', currentLlamaReady ? 'bg-emerald-400' : 'bg-amber-400']"></span>
              </div>
              <p class="text-[10px] text-slate-400 font-mono truncate">{{ currentLlamaReady ? 'Pronto (Metal GPU)' : 'Pendente' }}</p>
            </div>

            <div class="p-2.5 rounded-xl bg-[#141826] border border-[#20273d] space-y-1">
              <div class="flex items-center justify-between text-[11px]">
                <span class="font-bold text-slate-200">FFmpeg Áudio</span>
                <span :class="['w-2 h-2 rounded-full', currentFfmpegReady ? 'bg-emerald-400' : 'bg-amber-400']"></span>
              </div>
              <p class="text-[10px] text-slate-400 font-mono truncate">{{ currentFfmpegReady ? 'Pronto (Áudio PCM)' : 'Pendente' }}</p>
            </div>

            <div v-if="supportsMlx" class="p-2.5 rounded-xl bg-[#141826] border border-[#20273d] space-y-1">
              <div class="flex items-center justify-between text-[11px]">
                <span class="font-bold text-slate-200">Apple MLX</span>
                <span :class="['w-2 h-2 rounded-full', currentMlxVersion ? 'bg-emerald-400' : 'bg-amber-400']"></span>
              </div>
              <p class="text-[10px] text-slate-400 font-mono truncate">{{ currentMlxVersion ? `mlx-lm v${currentMlxVersion}` : 'Pendente' }}</p>
            </div>
          </div>
        </div>

        <!-- LLAMA TAB -->
        <div v-else-if="activeTab === 'llama'" class="space-y-2.5">
          <p>
            O motor <strong>llama-server</strong> é responsável pela execução de modelos <strong>.gguf</strong> com aceleração completa de GPU Metal e quantização de KV cache.
          </p>

          <div class="p-3 rounded-xl bg-[#141826] border border-[#20273d] space-y-2 text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Status atual:</span>
              <span class="font-bold font-mono" :class="currentLlamaReady ? 'text-emerald-400' : 'text-amber-400'">
                {{ currentLlamaReady ? 'Pronto e Funcional' : 'Não Instalado ou Incompleto' }}
              </span>
            </div>
            <div v-if="currentLlamaVersion" class="flex items-center justify-between">
              <span class="text-slate-400">Versão:</span>
              <span class="font-mono text-indigo-300 font-medium truncate max-w-[200px]">{{ currentLlamaVersion }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Local de instalação:</span>
              <span class="font-mono text-slate-300 truncate max-w-[200px]">~/Library/.../runtime/llama</span>
            </div>
          </div>
        </div>

        <!-- FFMPEG TAB -->
        <div v-else-if="activeTab === 'ffmpeg'" class="space-y-2.5">
          <p>
            O motor <strong>FFmpeg</strong> é responsável pela conversão ultrarrápida e decodificação de áudio PCM para transcrição de voz com modelos <strong>Whisper</strong>.
          </p>

          <div class="p-3 rounded-xl bg-[#141826] border border-[#20273d] space-y-2 text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Status atual:</span>
              <span class="font-bold font-mono" :class="currentFfmpegReady ? 'text-emerald-400' : 'text-amber-400'">
                {{ currentFfmpegReady ? 'Pronto e Funcional' : 'Não Instalado ou Incompleto' }}
              </span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Recursos:</span>
              <span class="font-mono text-indigo-300 font-medium truncate max-w-[200px]">Conversão PCM f32le 16kHz</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Local de instalação:</span>
              <span class="font-mono text-slate-300 truncate max-w-[200px]">~/Library/.../runtime/bin/ffmpeg</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Impacto no sistema:</span>
              <span class="text-emerald-400 font-semibold">Nenhum (100% isolado)</span>
            </div>
          </div>
        </div>

        <!-- MLX TAB -->
        <div v-else-if="activeTab === 'mlx'" class="space-y-2.5">
          <p>
            Ambiente Python 3.11 100% isolado em <code>runtime/venv</code> para inferência nativa em Apple Silicon.
          </p>

          <div class="p-3 rounded-xl bg-[#141826] border border-[#20273d] space-y-2 text-[11px]">
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Pacotes:</span>
              <span class="font-mono text-indigo-300 font-medium">mlx-lm, mlx-vlm, mlx-whisper</span>
            </div>
            <div v-if="currentMlxVersion" class="flex items-center justify-between">
              <span class="text-slate-400">Versão instalada:</span>
              <span class="font-mono text-emerald-400 font-bold">mlx-lm v{{ currentMlxVersion }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-slate-400">Impacto no seu sistema:</span>
              <span class="text-emerald-400 font-semibold">Nenhum (100% isolado)</span>
            </div>
          </div>

          <div class="flex items-center justify-between p-2.5 rounded-xl bg-[#121624] border border-[#1e2538] text-[11px]">
            <div class="space-y-0.5">
              <span class="font-medium text-slate-200">Baixar direto do GitHub oficial</span>
              <p class="text-[10px] text-slate-400">Instala a versão de ponta do repositório da Apple MLX</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" v-model="useGithub" :disabled="isBootstrapping" class="sr-only peer" />
              <div class="w-8 h-4.5 bg-[#1e2333] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-indigo-600"></div>
            </label>
          </div>
        </div>

        <!-- Progress Bar & Status -->
        <div v-if="isBootstrapping" class="space-y-2 pt-2">
          <div class="flex items-center justify-between text-xs font-medium">
            <span class="text-slate-200 truncate flex items-center gap-2">
              <Loader2 class="w-3.5 h-3.5 text-indigo-400 animate-spin flex-shrink-0" />
              <span class="truncate">{{ currentMessage || 'Configurando...' }}</span>
            </span>
            <span class="font-mono text-indigo-400 font-bold ml-2">{{ progressPercent }}%</span>
          </div>

          <div class="w-full h-2 rounded-full bg-[#181d2c] overflow-hidden">
            <div 
              class="h-full bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-400 rounded-full transition-all duration-300 ease-out" 
              :style="{ width: `${progressPercent}%` }"
            ></div>
          </div>
        </div>

        <div v-if="errorMessage" class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs">
          <strong>Erro:</strong> {{ errorMessage }}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-end gap-3 pt-2">
        <button 
          v-if="!isBootstrapping" 
          @click="closeModal" 
          class="px-3.5 py-1.5 rounded-xl border border-[#22283b] hover:bg-[#151926] text-xs font-medium text-slate-300 transition-all cursor-pointer"
        >
          Fechar
        </button>

        <button 
          v-if="!isBootstrapping && progressPercent < 100" 
          @click="startAction" 
          class="flex items-center gap-2 px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-lg shadow-indigo-600/30 transition-all active:scale-95 cursor-pointer"
        >
          <DownloadCloud class="w-3.5 h-3.5" />
          <span>{{ actionButtonText }}</span>
        </button>

        <button 
          v-if="progressPercent === 100" 
          @click="closeModal" 
          class="flex items-center gap-2 px-4 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-lg shadow-emerald-600/30 transition-all active:scale-95 cursor-pointer"
        >
          <Check class="w-3.5 h-3.5" />
          <span>Concluído!</span>
        </button>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch } from 'vue'
import { Cpu, X, Loader2, DownloadCloud, Check } from 'lucide-vue-next'
import { invoke, Channel } from '@tauri-apps/api/core'

const supportsMlx = inject('supportsMlx', computed(() => true))

const props = defineProps<{
  modelValue?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'ready'): void
}>()

const isOpen = computed({
  get: () => props.modelValue ?? false,
  set: (v) => emit('update:modelValue', v)
})

const activeTab = ref<'all' | 'llama' | 'ffmpeg' | 'mlx'>('all')
const isBootstrapping = ref(false)
const progressPercent = ref(0)
const currentMessage = ref('')
const errorMessage = ref('')
const currentMlxVersion = ref<string | null>(null)
const currentLlamaVersion = ref<string | null>(null)
const currentLlamaReady = ref(false)
const currentFfmpegReady = ref(false)
const useGithub = ref(false)

const actionButtonText = computed(() => {
  if (activeTab.value === 'all') {
    return 'Instalar / Reparar Todos'
  }
  if (activeTab.value === 'llama') {
    return currentLlamaReady.value ? 'Reinstalar / Reparar GGUF' : 'Instalar llama-server'
  }
  if (activeTab.value === 'ffmpeg') {
    return currentFfmpegReady.value ? 'Reinstalar / Reparar FFmpeg' : 'Instalar FFmpeg'
  }
  return currentMlxVersion.value ? 'Atualizar / Reparar MLX' : 'Inicializar Ambiente MLX'
})

const loadStatus = async () => {
  try {
    const st: any = await invoke('get_runtime_status')
    if (st) {
      currentMlxVersion.value = st.mlx_version || null
      currentLlamaVersion.value = st.llama_version || null
      currentLlamaReady.value = !!st.llama_server_available
      currentFfmpegReady.value = !!st.ffmpeg_available
    }
  } catch {}
}

watch(isOpen, (open) => {
  if (open) {
    loadStatus()
    errorMessage.value = ''
    if (progressPercent.value === 100) {
      progressPercent.value = 0
    }
  }
})

const closeModal = () => {
  if (!isBootstrapping.value) {
    isOpen.value = false
  }
}

const startAction = async () => {
  isBootstrapping.value = true
  errorMessage.value = ''
  progressPercent.value = 5
  currentMessage.value = 'Iniciando operação...'

  try {
    const channel = new Channel<any>()
    channel.onmessage = (event) => {
      if (event.message) currentMessage.value = event.message
      if (typeof event.progress_percent === 'number') progressPercent.value = event.progress_percent
    }

    if (activeTab.value === 'all') {
      await invoke('bootstrap_all_runtimes', { channel, fromGithub: useGithub.value })
    } else if (activeTab.value === 'llama') {
      await invoke('bootstrap_llama_runtime', { channel })
    } else if (activeTab.value === 'ffmpeg') {
      await invoke('bootstrap_ffmpeg_runtime', { channel })
    } else if (activeTab.value === 'mlx') {
      if (!supportsMlx.value) {
        throw new Error('MLX requer macOS com Apple Silicon.')
      }
      await invoke('bootstrap_mlx_runtime', { channel, fromGithub: useGithub.value })
    }

    await loadStatus()
    progressPercent.value = 100
    currentMessage.value = 'Operação concluída com sucesso!'
    emit('ready')
    setTimeout(() => {
      isBootstrapping.value = false
      isOpen.value = false
    }, 1500)
  } catch (err: any) {
    errorMessage.value = String(err)
    isBootstrapping.value = false
  }
}

defineExpose({
  open: (tab?: 'all' | 'llama' | 'ffmpeg' | 'mlx') => {
    if (tab) activeTab.value = tab
    isOpen.value = true
    errorMessage.value = ''
    loadStatus()
  }
})
</script>
