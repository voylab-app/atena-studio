<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[9999] flex items-center justify-center p-4 sm:p-6 bg-black/80 backdrop-blur-md select-none animate-in fade-in duration-150"
      @click.self="cancel"
    >
      <div
        class="bg-[#0e111d] border border-[#242b45] rounded-3xl w-full max-w-lg shadow-2xl shadow-black/95 flex flex-col overflow-hidden text-slate-100 animate-in zoom-in-95 duration-150"
      >
        <!-- Modal Header -->
        <div class="px-6 py-5 bg-[#121627] border-b border-[#1e253e] flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-2xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shadow-sm">
              <FolderPlus class="w-5 h-5" />
            </div>
            <div>
              <h3 class="text-base font-bold text-slate-100 leading-tight">
                {{ $t('modals.select_folder.title') }}
              </h3>
              <p class="text-xs text-slate-400 mt-0.5">
                {{ $t('modals.select_folder.subtitle') }}
              </p>
            </div>
          </div>

          <button
            type="button"
            @click="cancel"
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-4">
          <!-- Folder Path Input -->
          <div class="space-y-2">
            <label class="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <span>{{ $t('modals.select_folder.path_label') }}</span>
              <span class="text-rose-400">*</span>
            </label>
            <input
              ref="inputRef"
              v-model="pathInput"
              type="text"
              :placeholder="$t('modals.select_folder.path_placeholder')"
              class="w-full px-4 py-2.5 rounded-2xl bg-[#141829] border border-[#222a44] focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/30 text-slate-100 placeholder-slate-500 outline-none transition-all shadow-inner font-mono text-xs"
              @keydown.enter="confirm"
              @keydown.esc="cancel"
            />
          </div>

          <!-- Detected Directories Chips -->
          <div v-if="detectedDirs.length > 0" class="space-y-2">
            <label class="text-xs font-semibold text-slate-400">
              {{ $t('modals.select_folder.detected_title') }}
            </label>
            <div class="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto pr-1">
              <button
                v-for="dir in detectedDirs"
                :key="dir"
                type="button"
                @click="pathInput = dir"
                class="px-2.5 py-1.5 rounded-xl bg-[#141828] hover:bg-[#1f253d] border border-[#232b44] hover:border-indigo-500/40 text-[11px] font-mono text-slate-300 hover:text-indigo-300 transition-colors flex items-center gap-1.5 cursor-pointer text-left truncate max-w-full"
                :class="{ '!border-indigo-500/60 !bg-indigo-500/15 !text-indigo-300': pathInput === dir }"
              >
                <Folder class="w-3.5 h-3.5 text-indigo-400 flex-shrink-0" />
                <span class="truncate">{{ dir }}</span>
              </button>
            </div>
          </div>

          <!-- Error Alert -->
          <div v-if="errorMessage" class="p-3 rounded-2xl bg-rose-500/10 border border-rose-500/25 text-rose-300 text-xs flex items-center gap-2.5">
            <AlertTriangle class="w-4 h-4 flex-shrink-0 text-rose-400" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 bg-[#121627] border-t border-[#1e253e] flex items-center justify-end gap-2.5">
          <button
            type="button"
            @click="cancel"
            class="px-4 py-2 rounded-2xl bg-[#181c2d] hover:bg-[#20253c] text-xs font-medium text-slate-300 hover:text-white transition-colors cursor-pointer"
          >
            {{ $t('modals.select_folder.cancel_button') }}
          </button>
          <button
            type="button"
            @click="confirm"
            class="px-4 py-2 rounded-2xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all cursor-pointer active:scale-95 flex items-center gap-1.5"
          >
            <Check class="w-3.5 h-3.5" />
            <span>{{ $t('modals.select_folder.confirm_button') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderPlus, Folder, X, AlertTriangle, Check } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

const isOpen = ref(false)
const pathInput = ref('')
const detectedDirs = ref<string[]>([])
const errorMessage = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
let activeResolver: ((path: string | null) => void) | null = null

const handleOpenEvent = async (e: Event) => {
  const custom = e as CustomEvent<{ defaultPath?: string; resolve: (path: string | null) => void }>
  pathInput.value = custom.detail?.defaultPath || ''
  activeResolver = custom.detail?.resolve || null
  errorMessage.value = ''
  isOpen.value = true

  try {
    const detected = await invoke<string[]>('detect_model_directories')
    if (Array.isArray(detected)) {
      detectedDirs.value = detected
    }
  } catch {
    // Ignore error loading detected dirs
  }

  nextTick(() => {
    inputRef.value?.focus()
  })
}

const confirm = () => {
  const trimmed = pathInput.value.trim()
  if (!trimmed) {
    errorMessage.value = t('modals.select_folder.empty_path_error')
    return
  }
  const resolver = activeResolver
  activeResolver = null
  isOpen.value = false
  if (resolver) {
    resolver(trimmed)
  }
}

const cancel = () => {
  const resolver = activeResolver
  activeResolver = null
  isOpen.value = false
  if (resolver) {
    resolver(null)
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('atena:open-folder-picker', handleOpenEvent)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('atena:open-folder-picker', handleOpenEvent)
  }
})
</script>
