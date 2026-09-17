<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fade-in select-none"
  >
    <div
      class="w-full max-w-md bg-[#0f121f] border border-[#232a42] rounded-3xl shadow-2xl shadow-black/80 overflow-hidden flex flex-col animate-in zoom-in-95 duration-150"
    >
      <!-- Modal Header -->
      <div class="p-5 border-b border-[#1b2135] bg-[#121627] flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-2xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400 shadow-inner">
            <Trash2 class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-100 tracking-tight">
              {{ isBulk ? $t('modals.delete_session.title_bulk', { count: sessionsList.length }) : $t('modals.delete_session.title') }}
            </h3>
            <p class="text-[11px] text-slate-400 mt-0.5">{{ $t('modals.delete_session.subtitle') }}</p>
          </div>
        </div>

        <button
          @click="$emit('close')"
          class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-[#1a2035] transition-colors cursor-pointer"
          :title="$t('common.close')"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4 text-xs text-slate-300">
        <!-- Session(s) Info Box -->
        <div class="p-3 rounded-2xl bg-[#14192b] border border-[#202844] flex items-start gap-2.5">
          <MessageSquare class="w-4 h-4 text-indigo-400 shrink-0 mt-0.5" />
          <div class="min-w-0 flex-1">
            <template v-if="!isBulk">
              <div class="font-semibold text-slate-200 truncate text-[12px]">
                {{ currentSession?.title || $t('chat.new_chat') }}
              </div>
              <div class="text-[10.5px] text-slate-400 mt-0.5">
                {{ $t('modals.delete_session.single_history_info', { count: currentSession?.messages?.length || 0 }) }}
              </div>
            </template>
            <template v-else>
              <div class="font-semibold text-slate-200 text-[12px]">
                {{ $t('modals.delete_session.selected_sessions', { count: sessionsList.length }) }}
              </div>
              <div class="text-[10.5px] text-slate-400 mt-0.5">
                {{ $t('modals.delete_session.bulk_history_info', { count: totalMessagesCount }) }}
              </div>
              <div class="flex flex-wrap gap-1 mt-2 max-h-24 overflow-y-auto pr-1">
                <span
                  v-for="s in sessionsList"
                  :key="s.id"
                  class="text-[10px] px-2 py-0.5 rounded-md bg-[#0d101e] border border-[#1e253e] text-slate-300 truncate max-w-[200px]"
                >
                  {{ s.title || $t('chat.new_chat') }}
                </span>
              </div>
            </template>
          </div>
        </div>

        <!-- Episodic Memory Stats Card -->
        <div
          v-if="isLoadingStats"
          class="p-3 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-center gap-2.5 text-indigo-300"
        >
          <Loader2 class="w-4 h-4 animate-spin text-indigo-400 shrink-0" />
          <span class="text-[11.5px]">{{ $t('modals.delete_session.querying_records') }}</span>
        </div>

        <div
          v-else-if="stats.episodes_count > 0 || stats.memories_count > 0"
          class="p-3.5 rounded-2xl bg-gradient-to-br from-[#161c33] to-[#121628] border border-indigo-500/30 space-y-2.5 shadow-sm"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Brain class="w-4 h-4 text-amber-400" />
              <span class="font-bold text-slate-200 text-xs">{{ $t('modals.delete_session.linked_memory_title') }}</span>
            </div>
            <div class="flex items-center gap-1.5 font-mono text-[10px]">
              <span class="px-2 py-0.5 rounded-md bg-amber-500/15 text-amber-300 border border-amber-500/30 font-bold">
                {{ stats.episodes_count === 1 ? $t('modals.delete_session.diary_page', { count: stats.episodes_count }) : $t('modals.delete_session.diary_pages', { count: stats.episodes_count }) }}
              </span>
            </div>
          </div>

          <p class="text-[11px] text-slate-400 leading-relaxed">
            {{ isBulk ? $t('modals.delete_session.linked_memory_desc_bulk') : $t('modals.delete_session.linked_memory_desc_single') }}
          </p>

          <!-- Checkbox Toggle -->
          <label class="flex items-start gap-2.5 pt-1 cursor-pointer select-none group">
            <input
              type="checkbox"
              v-model="deleteMemoriesOption"
              class="w-4 h-4 mt-0.5 rounded border-slate-600 text-rose-600 focus:ring-rose-500 focus:ring-offset-0 bg-[#0d101d] cursor-pointer"
            />
            <div class="text-[11.5px] leading-tight flex-1">
              <span :class="deleteMemoriesOption ? 'text-rose-300 font-semibold' : 'text-slate-300 group-hover:text-slate-200'">
                {{ $t('modals.delete_session.delete_memories_toggle', { count: stats.episodes_count }) }}
              </span>
              <p class="text-[10px] text-slate-500 mt-0.5">
                {{ deleteMemoriesOption
                  ? $t('modals.delete_session.delete_memories_hint_active')
                  : $t('modals.delete_session.delete_memories_hint_inactive') }}
              </p>
            </div>
          </label>
        </div>

        <div v-else class="text-[11.5px] text-slate-400 leading-relaxed">
          {{ isBulk ? $t('modals.delete_session.confirm_desc_bulk', { count: sessionsList.length }) : $t('modals.delete_session.confirm_desc') }}
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-4 border-t border-[#1b2135] bg-[#121627] flex items-center justify-end gap-2">
        <button
          @click="$emit('close')"
          class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-slate-200 hover:bg-[#1a2035] transition-all cursor-pointer"
        >
          {{ $t('common.cancel') }}
        </button>

        <button
          @click="handleConfirm"
          :disabled="isDeleting"
          class="px-4 py-2 rounded-xl bg-gradient-to-r from-rose-600 to-rose-500 hover:from-rose-500 hover:to-rose-400 text-white text-xs font-semibold shadow-lg shadow-rose-600/25 transition-all active:scale-[0.98] flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
        >
          <Loader2 v-if="isDeleting" class="w-3.5 h-3.5 animate-spin" />
          <Trash2 v-else class="w-3.5 h-3.5" />
          <span>{{ isBulk ? $t('modals.delete_session.delete_button_bulk', { count: sessionsList.length }) : $t('modals.delete_session.delete_button') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Trash2,
  X,
  MessageSquare,
  Brain,
  Loader2
} from 'lucide-vue-next'
import type { Session } from '~/types'

export interface DeleteSessionConfirmPayload {
  sessionId?: string
  sessionIds: string[]
  deleteEpisodes: boolean
  deleteMemories: boolean
}

const props = defineProps<{
  isOpen: boolean
  session?: Session | Session[] | null
  sessions?: Session[] | null
}>()

const emit = defineEmits<{
  close: []
  confirm: [payload: DeleteSessionConfirmPayload]
}>()

const sessionsList = computed<Session[]>(() => {
  if (Array.isArray(props.session)) return props.session
  if (Array.isArray(props.sessions) && props.sessions.length > 0) return props.sessions
  if (props.session && typeof props.session === 'object') return [props.session as Session]
  return []
})

const currentSession = computed<Session | null>(() => {
  if (Array.isArray(props.session)) return props.session[0] || null
  if (props.session) return props.session as Session
  return sessionsList.value[0] || null
})

const isBulk = computed(() => sessionsList.value.length > 1)

const totalMessagesCount = computed(() => {
  return sessionsList.value.reduce((acc, s) => acc + (s?.messages?.length || 0), 0)
})

const isLoadingStats = ref(false)
const isDeleting = ref(false)
const deleteMemoriesOption = ref(true)
const stats = ref({
  episodes_count: 0,
  memories_count: 0
})

const fetchSessionStats = async () => {
  if (sessionsList.value.length === 0) return
  isLoadingStats.value = true
  let totalEpisodes = 0
  let totalMemories = 0
  try {
    for (const sess of sessionsList.value) {
      if (!sess?.id) continue
      try {
        const res: any = await invoke('session_memory_stats', {
          sessionId: sess.id
        })
        if (res) {
          totalEpisodes += res.episodes_count || 0
          totalMemories += res.memories_count || 0
        }
      } catch (_) {}
    }
    stats.value = {
      episodes_count: totalEpisodes,
      memories_count: totalMemories
    }
    deleteMemoriesOption.value = (totalEpisodes > 0 || totalMemories > 0)
  } catch (err) {
    console.error('Erro ao buscar estatísticas de memória da sessão:', err)
    stats.value = { episodes_count: 0, memories_count: 0 }
  } finally {
    isLoadingStats.value = false
  }
}

watch(
  () => [props.isOpen, sessionsList.value.length],
  ([newOpen]) => {
    if (newOpen && sessionsList.value.length > 0) {
      fetchSessionStats()
    }
  },
  { immediate: true }
)

const handleConfirm = async () => {
  isDeleting.value = true
  try {
    emit('confirm', {
      sessionId: sessionsList.value[0]?.id,
      sessionIds: sessionsList.value.map((s) => s.id),
      deleteEpisodes: deleteMemoriesOption.value,
      deleteMemories: deleteMemoriesOption.value
    })
  } finally {
    isDeleting.value = false
  }
}
</script>
