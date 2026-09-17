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
            <Blocks class="w-5 h-5" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-base font-bold text-slate-100">{{ $t('plugins.title') }}</h3>
              <span class="text-xs px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 font-mono font-semibold">
                {{ activePluginsCount }} {{ $t('plugins.active_badge') }}
              </span>
            </div>
            <p class="text-xs text-slate-400 mt-0.5">
              {{ $t('plugins.subtitle') }}
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
            :placeholder="$t('plugins.search_placeholder')"
            class="w-full bg-[#151928] border border-[#22293f] rounded-xl pl-9 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 outline-none focus:border-indigo-500/60 focus:ring-1 focus:ring-indigo-500/30 transition-all font-sans"
          />
        </div>

        <div class="flex items-center gap-2 flex-shrink-0">
          <button
            @click="openPluginsFolder"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all active:scale-95 cursor-pointer"
            :title="$t('plugins.open_folder_tooltip')"
          >
            <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
            <span>{{ $t('plugins.open_folder') }}</span>
          </button>

          <button
            @click="reloadPlugins"
            :disabled="isLoadingPlugins"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all active:scale-95 disabled:opacity-60 cursor-pointer"
            :title="$t('plugins.reload')"
          >
            <RefreshCw :class="['w-3.5 h-3.5 text-indigo-400', isLoadingPlugins ? 'animate-spin' : '']" />
            <span>{{ isLoadingPlugins ? $t('plugins.reloading') : $t('plugins.reload') }}</span>
          </button>
        </div>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        <!-- Loading State -->
        <div v-if="isLoadingPlugins" class="p-12 text-center text-slate-400 text-xs flex flex-col items-center justify-center gap-2">
          <Loader2 class="w-6 h-6 text-indigo-400 animate-spin" />
          <span>{{ $t('plugins.loading') }}</span>
        </div>

        <!-- Empty State -->
        <div
          v-else-if="filteredPlugins.length === 0"
          class="p-8 text-center rounded-2xl bg-[#111422] border border-[#1d2338] text-slate-400 text-xs space-y-3"
        >
          <div class="w-12 h-12 rounded-2xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto border border-indigo-500/20">
            <Blocks class="w-6 h-6" />
          </div>
          <p class="font-medium text-slate-200">{{ $t('plugins.no_plugins_title') }}</p>
          <p class="text-[11px] text-slate-400 max-w-md mx-auto leading-relaxed">
            {{ $t('plugins.no_plugins_desc') }}
          </p>
          <div class="pt-1">
            <button
              @click="openPluginsFolder"
              class="px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer inline-flex items-center gap-1.5"
            >
              <FolderOpen class="w-3.5 h-3.5" />
              <span>{{ $t('plugins.open_plugins_folder') }}</span>
            </button>
          </div>
        </div>

        <!-- Plugins List -->
        <div v-else class="space-y-3">
          <div
            v-for="plugin in filteredPlugins"
            :key="plugin.id"
            :class="[
              'p-4 rounded-2xl border transition-all flex flex-col gap-3',
              plugin.enabled
                ? 'bg-[#121627] border-[#222b48] shadow-sm'
                : 'bg-[#101320]/60 border-[#1a1f33] opacity-75'
            ]"
          >
            <!-- Plugin Header & Toggle -->
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-start gap-3">
                <div
                  :class="[
                    'w-9 h-9 rounded-xl flex items-center justify-center flex-shrink-0 mt-0.5 border',
                    plugin.id === 'atena-plugin-memory'
                      ? 'bg-purple-500/15 border-purple-500/30 text-purple-400'
                      : plugin.is_builtin
                      ? 'bg-indigo-500/15 border-indigo-500/30 text-indigo-400'
                      : 'bg-emerald-500/15 border-emerald-500/30 text-emerald-400'
                  ]"
                >
                  <Brain v-if="plugin.id === 'atena-plugin-memory'" class="w-4 h-4" />
                  <Sparkles v-else-if="plugin.category === 'chat_enhancer'" class="w-4 h-4" />
                  <LayoutGrid v-else-if="plugin.category === 'ui_extension'" class="w-4 h-4" />
                  <Blocks v-else class="w-4 h-4" />
                </div>

                <div>
                  <div class="flex items-center gap-2 flex-wrap">
                    <h4 class="text-sm font-bold text-slate-100">{{ plugin.name }}</h4>
                    <span class="text-[10px] px-2 py-0.5 rounded font-mono font-semibold bg-[#1a2035] text-slate-400 border border-[#242b44]">
                      v{{ plugin.version }}
                    </span>
                    <span
                      v-if="plugin.is_builtin"
                      class="text-[10px] px-2 py-0.5 rounded font-semibold bg-indigo-500/10 text-indigo-300 border border-indigo-500/20"
                    >
                      {{ $t('plugins.official') }}
                    </span>
                    <span
                      v-else
                      class="text-[10px] px-2 py-0.5 rounded font-semibold bg-emerald-500/10 text-emerald-300 border border-emerald-500/20"
                    >
                      {{ $t('plugins.community') }}
                    </span>
                  </div>

                  <p class="text-xs text-slate-300 mt-1 leading-relaxed">
                    {{ plugin.description }}
                  </p>
                </div>
              </div>

              <!-- Switch Toggle -->
              <div class="flex items-center gap-3 flex-shrink-0">
                <button
                  type="button"
                  role="switch"
                  :aria-checked="plugin.enabled"
                  @click="handleToggle(plugin)"
                  :class="[
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    plugin.enabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      plugin.enabled ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>
            </div>

            <!-- Capabilities Badges & Footer -->
            <div class="pt-2 border-t border-[#1a2034] flex items-center justify-between flex-wrap gap-2 text-[11px] text-slate-400">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-slate-500 text-[10.5px]">{{ $t('plugins.features') }}</span>
                <span
                  v-for="hook in plugin.hooks"
                  :key="hook"
                  class="px-1.5 py-0.5 rounded bg-[#161a2c] text-indigo-300/80 border border-[#212740] font-mono text-[9.5px]"
                >
                  hook:{{ hook }}
                </span>
                <span
                  v-for="view in plugin.views"
                  :key="view.id"
                  class="px-1.5 py-0.5 rounded bg-[#161a2c] text-amber-300/80 border border-[#212740] text-[9.5px] flex items-center gap-1"
                >
                  <LayoutGrid class="w-2.5 h-2.5" />
                  aba:{{ view.title }}
                </span>
              </div>

              <div class="flex items-center gap-2 text-[10.5px] text-slate-500">
                <span>{{ $t('plugins.by_author') }} <strong class="text-slate-400 font-medium">{{ plugin.author }}</strong></span>
                <span v-if="plugin.id === 'atena-plugin-memory'" class="text-purple-400 font-medium">
                  {{ $t('plugins.memory_ram_save') }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Dev Guide Box -->
        <div class="p-4 rounded-2xl bg-[#0f121e] border border-[#1e253d] flex items-start gap-3 mt-4">
          <div class="w-8 h-8 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center flex-shrink-0 mt-0.5">
            <Code2 class="w-4 h-4" />
          </div>
          <div class="flex-1">
            <h5 class="text-xs font-bold text-slate-200">{{ $t('plugins.dev_guide_title') }}</h5>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('plugins.dev_guide_desc') }}
            </p>
          </div>
          <button
            @click="openPluginsFolder"
            class="px-3 py-1.5 rounded-xl bg-[#161a2c] hover:bg-[#20273f] border border-[#252c48] text-xs font-medium text-slate-200 hover:text-white transition-all cursor-pointer flex-shrink-0"
          >
            {{ $t('plugins.open_folder') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Blocks,
  X,
  Search,
  FolderOpen,
  RefreshCw,
  Loader2,
  Brain,
  Sparkles,
  LayoutGrid,
  Code2
} from 'lucide-vue-next'
import { usePlugins, type PluginInfo } from '~/composables/usePlugins'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  plugins,
  isLoadingPlugins,
  pluginsFolder,
  fetchPlugins,
  togglePlugin,
  openPluginsFolder
} = usePlugins()

const searchQuery = ref('')

const activePluginsCount = computed(() => {
  return plugins.value.filter((p) => p.enabled).length
})

const filteredPlugins = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return plugins.value
  return plugins.value.filter((p) => {
    return (
      p.name.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.author.toLowerCase().includes(q) ||
      p.id.toLowerCase().includes(q)
    )
  })
})

const handleToggle = async (plugin: PluginInfo) => {
  const newStatus = !plugin.enabled
  await togglePlugin(plugin.id, newStatus)
}

const reloadPlugins = async () => {
  await fetchPlugins()
}
</script>
