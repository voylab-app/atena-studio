<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fade-in select-none"
    @click.self="$emit('close')"
  >
    <div
      class="w-full max-w-2xl bg-[#0f121f] border border-[#232a42] rounded-3xl shadow-2xl shadow-black/80 overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150"
    >
      <!-- Modal Header -->
      <div class="p-5 border-b border-[#1b2135] bg-[#121627] flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-2xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-400 shadow-inner">
            <Archive class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-slate-100 tracking-tight">
                {{ $t('modals.archived_chats.title') }}
              </h3>
              <span
                v-if="archivedSessions.length > 0"
                class="px-2 py-0.5 rounded-md bg-amber-500/10 border border-amber-500/20 text-amber-300 font-mono text-[10px] font-bold"
              >
                {{ archivedSessions.length }}
              </span>
            </div>
            <p class="text-[11px] text-slate-400 mt-0.5">
              {{ $t('modals.archived_chats.subtitle') }}
            </p>
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

      <!-- Search and Bulk Action Toolbar -->
      <div class="p-4 border-b border-[#1b2135] bg-[#0d101d] space-y-3">
        <div class="flex items-center gap-2">
          <!-- Search Input -->
          <div class="relative flex-1">
            <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none" />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="$t('modals.archived_chats.search_placeholder')"
              class="w-full pl-9 pr-8 py-2 rounded-xl bg-[#131726] border border-[#20273d] focus:border-amber-500/50 focus:ring-1 focus:ring-amber-500/20 text-slate-200 placeholder-slate-500 text-xs outline-none transition-all"
            />
            <button
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 p-0.5 cursor-pointer"
              :title="$t('common.cancel')"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Bulk Unarchive Button (When Items Selected) -->
          <button
            v-if="selectedSessionIds.size > 0"
            @click="handleUnarchiveSelected"
            class="px-3 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm active:scale-95 cursor-pointer flex-shrink-0"
            :title="$t('modals.archived_chats.unarchive_selected', { count: selectedSessionIds.size })"
          >
            <ArchiveRestore class="w-3.5 h-3.5" />
            <span>{{ $t('modals.archived_chats.unarchive_selected', { count: selectedSessionIds.size }) }}</span>
          </button>

          <!-- Unarchive All Button (When Nothing Selected and has Items) -->
          <button
            v-else-if="archivedSessions.length > 0"
            @click="handleUnarchiveAll"
            class="px-3 py-2 rounded-xl bg-[#161c30] hover:bg-[#1f2642] border border-[#263152] hover:border-amber-500/40 text-slate-300 hover:text-amber-200 text-xs font-medium flex items-center gap-1.5 transition-all cursor-pointer flex-shrink-0"
            :title="$t('modals.archived_chats.unarchive_all')"
          >
            <ArchiveRestore class="w-3.5 h-3.5 text-amber-400" />
            <span>{{ $t('modals.archived_chats.unarchive_all') }}</span>
          </button>
        </div>
      </div>

      <!-- Scrollable List of Archived Chats -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2 max-h-[50vh]">
        <!-- Empty State: No Archived Chats -->
        <div
          v-if="archivedSessions.length === 0"
          class="py-14 flex flex-col items-center justify-center text-center space-y-2.5 select-none"
        >
          <div class="w-14 h-14 rounded-3xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shadow-inner">
            <Archive class="w-7 h-7" />
          </div>
          <h4 class="text-sm font-semibold text-slate-300">
            {{ $t('modals.archived_chats.no_archived') }}
          </h4>
          <p class="text-xs text-slate-500 max-w-sm leading-relaxed">
            {{ $t('modals.archived_chats.no_archived_desc') }}
          </p>
        </div>

        <!-- Empty State: No Search Matches -->
        <div
          v-else-if="filteredSessions.length === 0"
          class="py-12 flex flex-col items-center justify-center text-center space-y-2 select-none"
        >
          <Search class="w-8 h-8 text-slate-600" />
          <p class="text-xs text-slate-400">
            {{ $t('modals.archived_chats.no_results', { query: searchQuery }) }}
          </p>
        </div>

        <!-- List Items -->
        <div
          v-for="session in filteredSessions"
          :key="session.id"
          :class="[
            'p-3.5 rounded-2xl border transition-all duration-200 flex items-center justify-between gap-3 group',
            selectedSessionIds.has(session.id)
              ? 'bg-[#181d32] border-amber-500/50 shadow-md ring-1 ring-amber-500/20'
              : 'bg-[#131726] hover:bg-[#171c2f] border-[#20273d] hover:border-amber-500/30'
          ]"
        >
          <!-- Left: Selection Checkbox & Info -->
          <div class="flex items-start gap-3 min-w-0 flex-1">
            <button
              type="button"
              @click="toggleSelect(session.id)"
              class="p-1 rounded-lg text-slate-400 hover:text-amber-300 transition-colors mt-0.5 cursor-pointer flex-shrink-0"
            >
              <component
                :is="selectedSessionIds.has(session.id) ? CheckSquare : Square"
                class="w-4 h-4"
                :class="selectedSessionIds.has(session.id) ? 'text-amber-400' : 'text-slate-500'"
              />
            </button>

            <div class="min-w-0 flex-1">
              <!-- Title and Badges -->
              <div class="flex items-center gap-2 flex-wrap">
                <span class="font-semibold text-xs text-slate-200 truncate max-w-md">
                  {{ session.title || $t('chat.new_chat') }}
                </span>

                <!-- Project Tag if Assigned -->
                <span
                  v-if="getProjectName(session.project_id)"
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-indigo-500/10 border border-indigo-500/25 text-indigo-300 text-[10px] font-medium"
                >
                  <Folder class="w-3 h-3 text-indigo-400" />
                  <span>{{ getProjectName(session.project_id) }}</span>
                </span>

                <!-- Model Tag -->
                <span
                  v-if="session.model_name || session.model_id"
                  class="inline-flex items-center px-1.5 py-0.2 rounded bg-[#0d101a] border border-[#1e253e] text-slate-400 text-[9.5px] font-mono"
                >
                  {{ session.model_name || session.model_id }}
                </span>

                <!-- Private Badge -->
                <span
                  v-if="session.is_private"
                  class="inline-flex items-center gap-1 px-1.5 py-0.2 rounded bg-violet-500/15 border border-violet-500/30 text-violet-300 text-[9.5px]"
                >
                  <EyeOff class="w-3 h-3" />
                  <span>{{ $t('sidebar.private') }}</span>
                </span>
              </div>

              <!-- Metadata & Last Message Snippet -->
              <div class="flex items-center gap-3 mt-1 text-[10.5px] text-slate-400 flex-wrap">
                <span>
                  {{ $t('modals.archived_chats.messages_count', { count: session.messages?.length || 0 }) }}
                </span>
                <span>•</span>
                <span>
                  {{ formatRelativeDate(session.archived_at || session.created_at) }}
                </span>
              </div>

              <!-- Last message snippet preview -->
              <p
                v-if="getLastMessageSnippet(session)"
                class="text-[11px] text-slate-500 mt-1 line-clamp-1 italic font-sans"
              >
                "{{ getLastMessageSnippet(session) }}"
              </p>
            </div>
          </div>

          <!-- Right: Actions Group -->
          <div class="flex items-center gap-1 flex-shrink-0">
            <!-- Open Conversation Button -->
            <button
              @click="$emit('openSession', session.id)"
              class="px-2.5 py-1.5 rounded-xl bg-[#1b2238] hover:bg-indigo-600/30 border border-[#252f4c] hover:border-indigo-500/40 text-slate-300 hover:text-indigo-200 text-xs font-medium flex items-center gap-1 transition-all cursor-pointer shadow-sm active:scale-95"
              :title="$t('modals.archived_chats.open_tooltip')"
            >
              <ArrowUpRight class="w-3.5 h-3.5 text-indigo-400" />
              <span>{{ $t('modals.archived_chats.open_chat') }}</span>
            </button>

            <!-- Unarchive / Restore Button -->
            <button
              @click="$emit('unarchiveSession', session.id)"
              class="px-2.5 py-1.5 rounded-xl bg-[#1b2238] hover:bg-amber-500/20 border border-[#252f4c] hover:border-amber-500/40 text-slate-300 hover:text-amber-200 text-xs font-medium flex items-center gap-1 transition-all cursor-pointer shadow-sm active:scale-95"
              :title="$t('modals.archived_chats.restore_tooltip')"
            >
              <ArchiveRestore class="w-3.5 h-3.5 text-amber-400" />
              <span>{{ $t('modals.archived_chats.restore') }}</span>
            </button>

            <!-- Delete Permanently Button -->
            <button
              @click="$emit('deleteSession', session)"
              class="p-1.5 rounded-xl text-slate-400 hover:text-rose-400 hover:bg-rose-500/15 transition-colors cursor-pointer"
              :title="$t('modals.archived_chats.delete_tooltip')"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-4 border-t border-[#1b2135] bg-[#121627] flex items-center justify-between">
        <span class="text-xs text-slate-400 font-medium">
          {{ $t('modals.archived_chats.messages_count', { count: totalArchivedMessagesCount }) }}
        </span>

        <button
          @click="$emit('close')"
          class="px-4 py-1.5 rounded-xl bg-[#171c2f] hover:bg-[#1e243c] text-slate-300 text-xs font-medium transition-colors border border-[#232b45] cursor-pointer"
        >
          {{ $t('common.close') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Archive,
  ArchiveRestore,
  Trash2,
  X,
  Search,
  Folder,
  EyeOff,
  ArrowUpRight,
  CheckSquare,
  Square
} from 'lucide-vue-next'
import type { Session, Project } from '~/types'

const props = defineProps<{
  isOpen: boolean
  sessions: Session[]
  projects?: Project[]
}>()

const emit = defineEmits<{
  'close': []
  'openSession': [id: string]
  'unarchiveSession': [id: string]
  'unarchiveSessions': [ids: string[]]
  'deleteSession': [session: Session]
}>()

const searchQuery = ref('')
const selectedSessionIds = ref<Set<string>>(new Set())

const archivedSessions = computed<Session[]>(() => {
  if (!props.sessions) return []
  return props.sessions.filter((s) => Boolean(s.archived))
})

const filteredSessions = computed<Session[]>(() => {
  let list = archivedSessions.value
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter((s) => {
      const titleMatch = s.title?.toLowerCase().includes(q)
      const messageMatch = s.messages?.some((m) => m.content?.toLowerCase().includes(q))
      return titleMatch || messageMatch
    })
  }
  return [...list].sort((a, b) => {
    const timeA = new Date(a.archived_at || a.created_at).getTime()
    const timeB = new Date(b.archived_at || b.created_at).getTime()
    return timeB - timeA
  })
})

const totalArchivedMessagesCount = computed(() => {
  return archivedSessions.value.reduce((sum, s) => sum + (s.messages?.length || 0), 0)
})

const toggleSelect = (id: string) => {
  const next = new Set(selectedSessionIds.value)
  if (next.has(id)) {
    next.delete(id)
  } else {
    next.add(id)
  }
  selectedSessionIds.value = next
}

const handleUnarchiveSelected = () => {
  if (selectedSessionIds.value.size === 0) return
  emit('unarchiveSessions', Array.from(selectedSessionIds.value))
  selectedSessionIds.value = new Set()
}

const handleUnarchiveAll = () => {
  const allIds = archivedSessions.value.map((s) => s.id)
  if (allIds.length === 0) return
  emit('unarchiveSessions', allIds)
  selectedSessionIds.value = new Set()
}

const getProjectName = (projectId?: string | null): string | null => {
  if (!projectId || !props.projects) return null
  const p = props.projects.find((item) => item.id === projectId)
  return p ? p.name : null
}

const getLastMessageSnippet = (session: Session): string | null => {
  if (!session.messages || session.messages.length === 0) return null
  const last = session.messages[session.messages.length - 1]
  if (!last || !last.content) return null
  const clean = last.content.replace(/\n+/g, ' ').trim()
  return clean.length > 100 ? `${clean.slice(0, 100)}...` : clean
}

const formatRelativeDate = (dateStr?: string): string => {
  if (!dateStr) return ''
  try {
    const d = new Date(dateStr)
    return d.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateStr
  }
}
</script>
