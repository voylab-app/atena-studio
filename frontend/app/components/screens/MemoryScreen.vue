<template>
  <div class="h-full flex flex-col bg-[#090a0f] overflow-hidden relative select-none">
    <!-- Top Stats & Controls Header Bar (Fully Responsive) -->
    <div class="px-3 sm:px-5 py-2.5 bg-[#0e111d]/95 backdrop-blur-md border-b border-[#1b2135] flex items-center justify-between gap-2 sm:gap-4 z-20 shrink-0">
      <!-- Left: Title & Tab Switcher -->
      <div class="flex items-center gap-2 sm:gap-3 shrink-0">
        <div class="w-8 h-8 rounded-xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-teal-500/20 border border-indigo-500/30 flex items-center justify-center shadow-md shrink-0">
          <Brain class="w-4 h-4 text-indigo-400" />
        </div>
        <div class="hidden 2xl:block">
          <div class="flex items-center gap-2">
            <h2 class="text-xs font-bold text-slate-100 whitespace-nowrap">{{ $t('memory.neural_network_title') }}</h2>
            <span class="px-1.5 py-0.2 rounded-full text-[9px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 whitespace-nowrap">
              {{ $t('memory.realtime_badge') }}
            </span>
          </div>
        </div>

        <!-- Tab Switcher (Facts Network vs. Skills vs. Diary) -->
        <div class="flex items-center bg-[#101424] p-0.5 sm:p-1 rounded-xl border border-[#202740] shadow-sm shrink-0">
          <button
            @click="activeMemoryTab = 'graph'"
            class="flex items-center gap-1.5 px-2 sm:px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer whitespace-nowrap"
            :class="activeMemoryTab === 'graph' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'"
          >
            <span
              class="w-1.5 h-1.5 rounded-full shrink-0 transition-colors"
              :class="isFactsActive ? 'bg-emerald-400' : 'bg-amber-400/80'"
            />
            <Network class="w-3.5 h-3.5 shrink-0" />
            <span>{{ $t('memory.tab_facts_network') }}</span>
          </button>
          <button
            @click="activeMemoryTab = 'skills'"
            class="flex items-center gap-1.5 px-2 sm:px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer whitespace-nowrap"
            :class="activeMemoryTab === 'skills' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'"
          >
            <span
              class="w-1.5 h-1.5 rounded-full shrink-0 transition-colors"
              :class="isSkillsActive ? 'bg-emerald-400' : 'bg-amber-400/80'"
            />
            <Wrench class="w-3.5 h-3.5 shrink-0" />
            <span>{{ $t('memory.tab_skills') }}</span>
            <span
              v-if="skillsList.length > 0"
              class="px-1.5 py-0.2 text-[9px] font-mono font-bold rounded-full"
              :class="activeMemoryTab === 'skills' ? 'bg-white/20 text-white' : 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30'"
            >
              {{ skillsList.length }}
            </span>
          </button>
          <button
            @click="activeMemoryTab = 'episodes'"
            class="flex items-center gap-1.5 px-2 sm:px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer whitespace-nowrap"
            :class="activeMemoryTab === 'episodes' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'"
          >
            <span
              class="w-1.5 h-1.5 rounded-full shrink-0 transition-colors"
              :class="isEpisodesActive ? 'bg-emerald-400' : 'bg-amber-400/80'"
            />
            <BookOpen class="w-3.5 h-3.5 shrink-0" />
            <span>{{ $t('memory.tab_diary') }}</span>
            <span
              v-if="episodesList.length > 0"
              class="px-1.5 py-0.2 text-[9px] font-mono font-bold rounded-full"
              :class="activeMemoryTab === 'episodes' ? 'bg-white/20 text-white' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'"
            >
              {{ episodesList.length }}
            </span>
          </button>
        </div>

        <!-- Active Layer Inference Toggle -->
        <div class="flex items-center gap-1.5 sm:gap-2 px-2 sm:px-2.5 py-1 bg-[#101424] border border-[#202740] rounded-xl shadow-sm shrink-0">
          <span
            class="text-[10.5px] font-semibold hidden md:inline select-none"
            :class="isCurrentLayerActive ? 'text-emerald-400' : 'text-amber-400'"
          >
            {{ isCurrentLayerActive ? $t('memory.layer_active_badge') : $t('memory.layer_paused_badge') }}
          </span>
          <button
            type="button"
            role="switch"
            :aria-checked="isCurrentLayerActive"
            @click="toggleCurrentLayer"
            :title="isCurrentLayerActive ? $t('memory.tooltip_pause_layer') : $t('memory.tooltip_activate_layer')"
            :class="[
              'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              isCurrentLayerActive ? 'bg-emerald-600 shadow-sm shadow-emerald-600/30' : 'bg-[#22283b]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                isCurrentLayerActive ? 'translate-x-4' : 'translate-x-0'
              ]"
            />
          </button>
        </div>
      </div>

      <!-- Center: Key Metrics Capsule (Only on very wide screens) -->
      <div class="hidden 2xl:flex items-center gap-2 bg-[#121626] px-2.5 py-1 rounded-xl border border-[#202740] shadow-sm shrink-0">
        <div class="flex items-center gap-1.5 px-2 border-r border-[#202740]">
          <CircleDot class="w-3.5 h-3.5 text-indigo-400" />
          <span class="text-[11px] text-slate-400 font-medium">{{ $t('memory.stat_nodes') }}</span>
          <span class="text-xs font-mono font-bold text-slate-100">{{ stats.total_nodes }}</span>
        </div>
        <div class="flex items-center gap-1.5 px-2 border-r border-[#202740]">
          <Share2 class="w-3.5 h-3.5 text-purple-400" />
          <span class="text-[11px] text-slate-400 font-medium">{{ $t('memory.stat_synapses') }}</span>
          <span class="text-xs font-mono font-bold text-slate-100">{{ stats.total_edges }}</span>
        </div>
        <div class="flex items-center gap-1.5 px-2">
          <Zap class="w-3.5 h-3.5 text-teal-400" />
          <span class="text-[11px] text-slate-400 font-medium">{{ $t('memory.stat_hit') }}</span>
          <span class="text-xs font-mono font-bold text-teal-300">{{ stats.cache_hit_rate.toFixed(0) }}%</span>
        </div>
      </div>

      <!-- Right: Action Buttons -->
      <div class="flex items-center gap-1.5 sm:gap-2 shrink-0">
        <!-- Optimize & Prune -->
        <button
          @click="handleOptimizeAndPruneGraph"
          :disabled="isOptimizingGraph"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#14192b] hover:bg-[#1c233d] border border-purple-500/30 hover:border-purple-500/60 text-purple-300 hover:text-purple-200 text-xs font-semibold shadow-sm transition-all cursor-pointer active:scale-95 whitespace-nowrap shrink-0"
          :title="$t('memory.tooltip_optimize_prune')"
        >
          <Scissors class="w-3.5 h-3.5 text-purple-400" :class="{ 'animate-spin': isOptimizingGraph }" />
          <span class="hidden md:inline">{{ $t('memory.btn_optimize_prune') }}</span>
        </button>

        <!-- Action Button based on active tab -->
        <button
          v-if="activeMemoryTab === 'graph'"
          @click="openLearnModal()"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 border border-indigo-400/30 transition-all cursor-pointer active:scale-95 whitespace-nowrap shrink-0"
        >
          <Plus class="w-3.5 h-3.5" />
          <span class="hidden sm:inline">{{ $t('memory.btn_teach_fact') }}</span>
        </button>
        <button
          v-else
          @click="openNewSkillModal()"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 border border-indigo-400/30 transition-all cursor-pointer active:scale-95 whitespace-nowrap shrink-0"
        >
          <Plus class="w-3.5 h-3.5" />
          <span class="hidden sm:inline">{{ $t('memory.btn_new_skill') }}</span>
        </button>

        <!-- Export Graph Dropdown -->
        <div v-if="activeMemoryTab === 'graph'" class="relative shrink-0">
          <button
            @click="showExportMenu = !showExportMenu"
            :disabled="isExporting || graphData.nodes.length === 0"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#14192b] hover:bg-[#1c233d] border border-teal-500/30 hover:border-teal-500/60 text-teal-300 hover:text-teal-200 text-xs font-semibold shadow-sm transition-all cursor-pointer active:scale-95 whitespace-nowrap shrink-0 disabled:opacity-40"
            :title="$t('memory.tooltip_export')"
          >
            <Download class="w-3.5 h-3.5 text-teal-400" :class="{ 'animate-bounce': isExporting }" />
            <span class="hidden md:inline">{{ isExporting ? $t('memory.exporting') : $t('memory.export') }}</span>
          </button>

          <!-- Dropdown Menu -->
          <div
            v-if="showExportMenu"
            class="absolute right-0 top-full mt-1.5 w-44 rounded-xl bg-[#0e1220]/95 backdrop-blur-xl border border-[#222b48] shadow-2xl py-1.5 z-50 animate-in fade-in zoom-in-95 duration-100"
          >
            <button
              @click="handleExportGraph('json')"
              class="w-full flex items-center gap-2 px-3 py-2 text-xs text-slate-200 hover:bg-white/10 hover:text-white transition-colors cursor-pointer text-left"
            >
              <FileJson class="w-4 h-4 text-amber-400 shrink-0" />
              <span>{{ $t('memory.export_json') }}</span>
            </button>
            <button
              @click="handleExportGraph('markdown')"
              class="w-full flex items-center gap-2 px-3 py-2 text-xs text-slate-200 hover:bg-white/10 hover:text-white transition-colors cursor-pointer text-left"
            >
              <FileText class="w-4 h-4 text-indigo-400 shrink-0" />
              <span>{{ $t('memory.export_markdown') }}</span>
            </button>
          </div>
        </div>

        <!-- Reload Graph -->
        <button
          @click="fetchGraph"
          class="p-1.5 sm:p-2 rounded-xl bg-[#14192b] hover:bg-[#1c233d] border border-[#222b48] hover:border-indigo-500/40 text-slate-300 hover:text-white transition-all cursor-pointer shadow-sm active:scale-95 shrink-0"
          :title="$t('memory.reload')"
        >
          <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isRefreshing }" />
        </button>

        <!-- Clear Memory (Open Custom Modal) -->
        <button
          v-if="activeMemoryTab === 'graph'"
          @click="isConfirmResetModalOpen = true"
          class="p-1.5 sm:p-2 rounded-xl bg-[#14192b] hover:bg-rose-950/40 border border-[#222b48] hover:border-rose-500/40 text-slate-400 hover:text-rose-400 transition-all cursor-pointer shadow-sm active:scale-95 shrink-0"
          :title="$t('memory.reset_memory')"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Paused Layer Warning Banner -->
    <div
      v-if="!isCurrentLayerActive"
      class="px-4 py-2 bg-amber-500/10 border-b border-amber-500/20 flex items-center justify-between text-xs text-amber-300 z-20 shrink-0 animate-in fade-in duration-150"
    >
      <div class="flex items-center gap-2 min-w-0">
        <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0" />
        <span class="truncate">{{ currentLayerPausedMessage }}</span>
      </div>
      <button
        @click="toggleCurrentLayer"
        class="px-2.5 py-1 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-200 text-xs font-semibold transition-all cursor-pointer whitespace-nowrap ml-3 active:scale-95 shrink-0"
      >
        {{ $t('memory.btn_enable_layer') }}
      </button>
    </div>

    <!-- Main Workspace (Canvas + Side Inspection Panel + Hippocampus Drawer) -->
    <div v-show="activeMemoryTab === 'graph'" class="flex-1 flex overflow-hidden relative">
      <!-- Search and Filter Floating Overlay -->
      <div class="absolute top-4 left-4 z-10 flex flex-col gap-2 max-w-md w-full">
        <div class="flex items-center gap-2 w-full">
          <div class="flex-1 relative">
            <Search class="w-4 h-4 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="$t('memory.search_graph_placeholder')"
              class="w-full pl-9 pr-8 py-2 rounded-xl bg-[#0e1220]/90 backdrop-blur-md border border-[#222b48] focus:border-indigo-500/60 text-xs text-slate-200 placeholder-slate-500 focus:outline-none transition-all shadow-xl"
            />
            <button
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-white p-0.5 cursor-pointer"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Button to Expand/Hide Filters -->
          <button
            @click="showFiltersBar = !showFiltersBar"
            class="p-2 rounded-xl backdrop-blur-md border transition-all shadow-xl cursor-pointer"
            :class="showFiltersBar || (isFilterActive && !searchQuery)
              ? 'bg-indigo-600/30 border-indigo-500 text-indigo-300'
              : 'bg-[#0e1220]/90 border-[#222b48] text-slate-400 hover:text-white'"
            :title="showFiltersBar ? $t('memory.hide_filters') : $t('memory.quick_filters')"
          >
            <Filter class="w-4 h-4" />
          </button>

          <button
            @click="centerGraph"
            class="p-2 rounded-xl bg-[#0e1220]/90 backdrop-blur-md border border-[#222b48] text-slate-300 hover:text-white transition-all shadow-xl cursor-pointer"
            :title="$t('memory.recenter_view')"
          >
            <Maximize2 class="w-4 h-4" />
          </button>
        </div>

        <!-- Quick Filters (Chips / Pills) -->
        <div
          v-if="showFiltersBar || isFilterActive"
          class="bg-[#0e1220]/95 backdrop-blur-xl border border-[#222b48] rounded-2xl p-2.5 shadow-2xl space-y-2 animate-in fade-in duration-150 text-[11px]"
        >
          <!-- Scope: All, Global, Private -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <span class="text-[10px] text-slate-400 font-semibold shrink-0">{{ $t('memory.filter_scope_label') }}</span>
            <button
              @click="scopeFilter = 'all'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="scopeFilter === 'all' ? 'bg-indigo-600 text-white border-indigo-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-400 border-[#222b48] hover:text-white'"
            >
              {{ $t('memory.filter_all') }}
            </button>
            <button
              @click="scopeFilter = 'global'"
              class="flex items-center gap-1 px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="scopeFilter === 'global' ? 'bg-emerald-600 text-white border-emerald-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-400 border-[#222b48] hover:text-white'"
            >
              <Globe class="w-3 h-3 text-emerald-400" />
              <span>{{ $t('memory.filter_scope_global') }}</span>
            </button>
            <button
              @click="scopeFilter = 'private'"
              class="flex items-center gap-1 px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="scopeFilter === 'private' ? 'bg-amber-600 text-white border-amber-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-400 border-[#222b48] hover:text-white'"
            >
              <Lock class="w-3 h-3 text-amber-400" />
              <span>{{ $t('memory.filter_scope_private') }}</span>
            </button>
          </div>

          <!-- Valence: All, Positive, Neutral, Inhibition -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <span class="text-[10px] text-slate-400 font-semibold shrink-0">{{ $t('memory.filter_valence_label') }}</span>
            <button
              @click="valenceFilter = 'all'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="valenceFilter === 'all' ? 'bg-indigo-600 text-white border-indigo-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-400 border-[#222b48] hover:text-white'"
            >
              {{ $t('memory.filter_all_feminine') }}
            </button>
            <button
              @click="valenceFilter = 'positive'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="valenceFilter === 'positive' ? 'bg-emerald-600 text-white border-emerald-500 font-semibold shadow-sm' : 'bg-[#14192b] text-emerald-400 border-[#222b48] hover:border-emerald-500/40'"
            >
              {{ $t('memory.filter_valence_positive') }}
            </button>
            <button
              @click="valenceFilter = 'neutral'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="valenceFilter === 'neutral' ? 'bg-slate-600 text-white border-slate-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-300 border-[#222b48] hover:border-slate-500/40'"
            >
              {{ $t('memory.filter_valence_neutral') }}
            </button>
            <button
              @click="valenceFilter = 'negative'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="valenceFilter === 'negative' ? 'bg-rose-600 text-white border-rose-500 font-semibold shadow-sm' : 'bg-[#14192b] text-rose-400 border-[#222b48] hover:border-rose-500/40'"
            >
              {{ $t('memory.filter_valence_negative') }}
            </button>
          </div>

          <!-- Type: All, Object, Attribute, Location, Rule -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <span class="text-[10px] text-slate-400 font-semibold shrink-0">{{ $t('memory.filter_type_label') }}</span>
            <button
              @click="typeFilter = 'all'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="typeFilter === 'all' ? 'bg-indigo-600 text-white border-indigo-500 font-semibold shadow-sm' : 'bg-[#14192b] text-slate-400 border-[#222b48] hover:text-white'"
            >
              Todos
            </button>
            <button
              @click="typeFilter = 'object'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="typeFilter === 'object' ? 'bg-indigo-600 text-white border-indigo-500 font-semibold shadow-sm' : 'bg-[#14192b] text-indigo-300 border-[#222b48]'"
            >
              {{ $t('memory.filter_type_object') }}
            </button>
            <button
              @click="typeFilter = 'attribute'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="typeFilter === 'attribute' ? 'bg-teal-600 text-white border-teal-500 font-semibold shadow-sm' : 'bg-[#14192b] text-teal-300 border-[#222b48]'"
            >
              {{ $t('memory.filter_type_attribute') }}
            </button>
            <button
              @click="typeFilter = 'container'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="typeFilter === 'container' ? 'bg-amber-600 text-white border-amber-500 font-semibold shadow-sm' : 'bg-[#14192b] text-amber-300 border-[#222b48]'"
            >
              {{ $t('memory.filter_type_container') }}
            </button>
            <button
              @click="typeFilter = 'rule'"
              class="px-2 py-0.5 rounded-lg border transition-all cursor-pointer"
              :class="typeFilter === 'rule' ? 'bg-rose-600 text-white border-rose-500 font-semibold shadow-sm' : 'bg-[#14192b] text-rose-300 border-[#222b48]'"
            >
              {{ $t('memory.filter_type_rule') }}
            </button>
          </div>

          <!-- Matches Indicator and Clear Filters -->
          <div v-if="isFilterActive" class="flex items-center justify-between pt-1.5 border-t border-[#1b2135] text-[10.5px]">
            <span class="text-indigo-300 font-medium">
              {{ $t('memory.matching_nodes_count', { matches: matchingNodeIds.size, total: graphData.nodes.length }) }}
            </span>
            <button
              @click="clearAllFilters"
              class="text-rose-400 hover:text-rose-300 underline cursor-pointer"
            >
              {{ $t('memory.clear_filters') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Spreading Activation Test Tool Floating Bar (Dismissible) -->
      <div v-if="showActivationPanel" class="absolute bottom-4 left-4 z-10 bg-[#0e1220]/95 backdrop-blur-xl border border-[#222b48] rounded-2xl p-3.5 shadow-2xl max-w-md w-full animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-center justify-between mb-2">
          <span class="text-[11px] font-bold text-slate-200 flex items-center gap-1.5">
            <Radio class="w-3.5 h-3.5 text-indigo-400 animate-pulse" />
            {{ $t('memory.test_association_title') }}
          </span>
          <div class="flex items-center gap-2">
            <span class="text-[10px] text-slate-500 font-mono">{{ $t('memory.depth_label', { depth: 3 }) }}</span>
            <button
              @click="showActivationPanel = false"
              class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
              :title="$t('memory.close_activation_tooltip')"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <!-- Quick Real Example Chips from Loaded Graph -->
        <div v-if="suggestedNodes.length > 0" class="flex items-center gap-1.5 mb-2.5 overflow-x-auto pb-0.5">
          <span class="text-[10px] text-slate-400 shrink-0 font-medium">{{ $t('memory.real_examples_label') }}</span>
          <button
            v-for="sNode in suggestedNodes"
            :key="sNode.id"
            @click="testWithNode(sNode.label)"
            class="px-2 py-0.5 rounded-lg bg-indigo-500/15 hover:bg-indigo-500/30 border border-indigo-500/30 text-[10px] font-semibold text-indigo-300 transition-all cursor-pointer whitespace-nowrap active:scale-95 hover:border-indigo-400/60"
            :title="$t('memory.test_node_tooltip', { label: sNode.label })"
          >
            {{ sNode.label }}
          </button>
        </div>

        <div class="flex gap-2">
          <input
            v-model="activationAnchor"
            type="text"
            :placeholder="suggestedPlaceholder"
            class="flex-1 px-3 py-1.5 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-200 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-medium"
            @keyup.enter="handleTestActivation"
          />
          <button
            @click="handleTestActivation"
            :disabled="!activationAnchor.trim() || isActivating"
            class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold transition-all cursor-pointer flex items-center gap-1 shadow-md shadow-indigo-600/20"
          >
            <Play class="w-3 h-3" />
            <span>{{ $t('memory.test_btn') }}</span>
          </button>
        </div>

        <!-- Synthesized LLM Result Preview -->
        <div v-if="synthesizedContext" class="mt-2.5 p-2.5 rounded-xl bg-[#121626] border border-indigo-500/30 text-[11px] text-indigo-200 font-mono leading-relaxed">
          <div class="text-[9.5px] text-slate-400 font-bold uppercase tracking-wider mb-1 flex items-center justify-between">
            <span>{{ $t('memory.injected_context_title') }}</span>
            <span class="text-[9px] text-emerald-400 font-bold">{{ $t('memory.active_in_ai') }}</span>
          </div>
          {{ synthesizedContext }}
        </div>
      </div>

      <!-- Button to reopen Association Test when closed -->
      <button
        v-else
        @click="showActivationPanel = true"
        class="absolute bottom-4 left-4 z-10 flex items-center gap-1.5 px-3 py-2 rounded-xl bg-[#0e1220]/90 hover:bg-[#14192b] border border-[#222b48] hover:border-indigo-500/50 text-indigo-300 hover:text-white text-xs font-semibold shadow-xl backdrop-blur-md transition-all cursor-pointer active:scale-95"
        :title="$t('memory.open_activation_tooltip')"
      >
        <Radio class="w-3.5 h-3.5 text-indigo-400" />
        <span>{{ $t('memory.test_association') }}</span>
      </button>

      <!-- Canvas Area for Physics Graph -->
      <div class="flex-1 h-full relative overflow-hidden bg-slate-100 dark:bg-[#07080d]" ref="canvasContainerRef">
        <canvas
          ref="canvasRef"
          class="w-full h-full cursor-grab active:cursor-grabbing block"
          @mousedown="handleMouseDown"
          @mousemove="handleMouseMove"
          @mouseup="handleMouseUp"
          @mouseleave="handleMouseUp"
          @wheel="handleWheel"
        ></canvas>

        <!-- Empty State Guide -->
        <div
          v-if="graphData.nodes.length === 0 && !isLoading"
          class="absolute inset-0 flex flex-col items-center justify-center text-center p-6 space-y-3 pointer-events-none"
        >
          <div class="w-16 h-16 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shadow-2xl">
            <Brain class="w-8 h-8 opacity-60" />
          </div>
          <h3 class="text-sm font-bold text-slate-200">{{ $t('memory.empty_memory_title') }}</h3>
          <p class="text-xs text-slate-400 max-w-sm leading-relaxed">
            {{ $t('memory.empty_memory_desc') }}
          </p>
        </div>
      </div>

      <!-- Right Inspection Sidebar (Details of Selected Node) -->
      <transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="transform translate-x-full opacity-0"
        enter-to-class="transform translate-x-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="transform translate-x-0 opacity-100"
        leave-to-class="transform translate-x-full opacity-0"
      >
        <div
          v-if="selectedNode"
          class="w-80 h-full bg-[#0d101c]/98 backdrop-blur-2xl border-l border-[#1b2135] p-4 flex flex-col gap-4 overflow-y-auto z-20 shadow-2xl shrink-0"
        >
          <!-- Node Header -->
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2.5 min-w-0 flex-1">
              <div
                class="w-8 h-8 rounded-xl flex items-center justify-center text-white shadow-md shrink-0"
                :style="{ backgroundColor: getNodeColor(selectedNode.type_flag, selectedNode.valence) }"
              >
                <Component :is="getNodeIcon(selectedNode.type_flag, selectedNode.valence)" class="w-4 h-4" />
              </div>
              <div class="min-w-0 flex-1">
                <h4 class="text-xs font-bold text-slate-100 leading-tight truncate">{{ selectedNode.label }}</h4>
                <span class="text-[10px] font-mono font-medium text-slate-400">ID: {{ selectedNode.id }}</span>
              </div>
            </div>
            <button
              @click="selectedNode = null"
              class="text-slate-400 hover:text-white p-1.5 rounded-lg hover:bg-white/10 shrink-0 ml-1"
              :title="$t('memory.close_panel')"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Type & Valence Tag -->
          <div class="flex items-center gap-2 flex-wrap">
            <span
              class="px-2.5 py-1 rounded-lg text-[10.5px] font-semibold border"
              :style="{
                backgroundColor: `${getNodeColor(selectedNode.type_flag, selectedNode.valence)}20`,
                borderColor: `${getNodeColor(selectedNode.type_flag, selectedNode.valence)}50`,
                color: getNodeColor(selectedNode.type_flag, selectedNode.valence)
              }"
            >
              {{ getNodeTypeName(selectedNode.type_flag) }}
            </span>
            <span
              v-if="selectedNode.valence === 1"
              class="px-2 py-0.5 rounded-lg text-[10px] font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
            >
              {{ $t('memory.reward_valence') }}
            </span>
            <span
              v-else-if="selectedNode.valence === -1"
              class="px-2 py-0.5 rounded-lg text-[10px] font-bold bg-rose-500/20 text-rose-300 border border-rose-500/30"
            >
              {{ $t('memory.inhibition_valence') }}
            </span>
          </div>

          <!-- Node Creation Timestamp -->
          <div v-if="selectedNode.created_at" class="flex items-center gap-1.5 text-[10.5px] text-slate-400 font-mono bg-[#101424] px-2.5 py-1.5 rounded-xl border border-[#1b2135]">
            <Clock class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
            <span>{{ $t('memory.created_label', { time: formatTimestamp(selectedNode.created_at) }) }}</span>
          </div>

          <!-- Memory Scope (Global vs Private) -->
          <div class="flex items-center justify-between gap-2 p-2.5 rounded-xl bg-[#101424] border border-[#1b2135]">
            <div class="flex items-center gap-2 min-w-0">
              <Globe v-if="!selectedNode.session_id" class="w-4 h-4 text-emerald-400 shrink-0" />
              <Lock v-else class="w-4 h-4 text-amber-400 shrink-0" />
              <div class="min-w-0">
                <div class="text-[11px] font-bold" :class="selectedNode.session_id ? 'text-amber-300' : 'text-emerald-300'">
                  {{ selectedNode.session_id ? $t('memory.private_memory') : $t('memory.global_memory') }}
                </div>
                <div v-if="selectedNode.session_id" class="text-[9.5px] font-mono text-slate-400 truncate" :title="selectedNode.session_id">
                  {{ $t('memory.session_label', { id: selectedNode.session_id }) }}
                </div>
                <div v-else class="text-[9.5px] text-slate-500">
                  {{ $t('memory.shared_all_sessions') }}
                </div>
              </div>
            </div>
            <button
              @click="handleToggleNodeScope(selectedNode)"
              class="px-2 py-1 rounded-lg text-[10px] font-semibold transition-all border shrink-0 cursor-pointer"
              :class="selectedNode.session_id
                ? 'bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border-emerald-500/30'
                : 'bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border-amber-500/30'"
              :title="selectedNode.session_id ? $t('memory.make_global_tooltip') : $t('memory.make_private_tooltip')"
            >
              {{ selectedNode.session_id ? $t('memory.make_global') : $t('memory.make_private') }}
            </button>
          </div>

          <!-- Connected Synapses List -->
          <div class="flex-1 space-y-2">
            <h5 class="text-[11px] font-bold text-slate-300 uppercase tracking-wider">{{ $t('memory.connected_synapses') }}</h5>
            <div v-if="selectedNodeEdges.length === 0" class="text-xs text-slate-500 py-3 text-center">
              {{ $t('memory.no_direct_connection') }}
            </div>
            <div
              v-for="(edge, idx) in selectedNodeEdges"
              :key="idx"
              class="p-2.5 rounded-xl bg-[#131728] border border-[#202740] space-y-1.5"
            >
              <div class="flex items-center justify-between text-[11px]">
                <span class="font-bold" :class="edge.relation_type === 0xA4 || edge.relation_type === 'AvoidAction' ? 'text-rose-400' : 'text-indigo-300'">
                  {{ getRelationName(edge.relation_type) }}
                </span>
                <span class="text-[10px] font-mono text-teal-400 font-bold">{{ $t('memory.weight_label', { weight: (edge.weight || 1).toFixed(2) }) }}</span>
              </div>
              <div class="text-xs text-slate-200 font-medium truncate">
                ➔ {{ getNodeLabel(edge.target_id) }}
              </div>
              <div class="flex items-center justify-between text-[9.5px] text-slate-400 font-mono pt-1.5 border-t border-[#1b2135]">
                <span>{{ $t('memory.access_count', { count: edge.access_count || 0 }) }}</span>
                <span class="flex items-center gap-1" :title="$t('memory.created_at_synapse', { created: formatTimestamp(edge.created_at || edge.last_accessed), last: formatTimestamp(edge.last_accessed) })">
                  <Clock class="w-3 h-3 text-indigo-400" />
                  {{ formatTimestamp(edge.created_at || edge.last_accessed) }}
                </span>
              </div>
            </div>
          </div>

            <!-- Node Actions -->
            <div class="pt-2 border-t border-[#1b2135] space-y-2">
              <button
                @click="testWithNode(selectedNode.label)"
                class="w-full py-2 px-3 rounded-xl bg-indigo-500/15 hover:bg-indigo-500/25 border border-indigo-500/40 text-indigo-300 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1.5 cursor-pointer active:scale-95"
                :title="$t('memory.test_node_association_tooltip')"
              >
                <Radio class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('memory.test_node_association_btn') }}</span>
              </button>
              <div class="flex gap-2">
                <button
                  @click="handleReinforceSelected"
                  class="flex-1 py-2 px-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-all flex items-center justify-center gap-1 cursor-pointer shadow-md shadow-indigo-600/20"
                >
                  <Sparkles class="w-3.5 h-3.5" />
                  <span>{{ $t('memory.reinforce_synapses') }}</span>
                </button>
                <button
                  @click="isConfirmDeleteNodeModalOpen = true"
                  class="p-2 rounded-xl bg-rose-950/30 hover:bg-rose-900/50 border border-rose-500/30 text-rose-300 transition-all cursor-pointer shadow-sm active:scale-95"
                  :title="$t('memory.delete_node_tooltip')"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </transition>
      </div>

    <!-- Main Workspace: Procedural Skills & Tasks -->
    <div v-show="activeMemoryTab === 'skills'" class="flex-1 overflow-y-auto p-6 bg-[#07080d] space-y-6">
      <!-- Skills Header & Search -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 max-w-5xl mx-auto">
        <div>
          <h3 class="text-base font-bold text-slate-100 flex items-center gap-2">
            <Wrench class="w-4 h-4 text-indigo-400" />
            {{ $t('memory.procedural_skills_title') }}
          </h3>
          <p class="text-xs text-slate-400 mt-0.5">
            {{ $t('memory.procedural_skills_desc') }}
          </p>
        </div>

        <div class="flex items-center gap-3">
          <div class="relative w-56">
            <Search class="w-4 h-4 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              v-model="skillSearchQuery"
              type="text"
              :placeholder="$t('memory.filter_skills_placeholder')"
              class="w-full pl-9 pr-4 py-2 rounded-xl bg-[#0f1322] border border-[#222b48] focus:border-indigo-500/60 text-xs text-slate-200 placeholder-slate-500 focus:outline-none transition-all shadow-md"
            />
          </div>
          <button
            @click="handleOpenSkillsFolder()"
            class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-[#0f1322] hover:bg-[#151b2e] border border-[#222b48] hover:border-indigo-500/40 text-slate-300 hover:text-white text-xs font-semibold shadow-md transition-all cursor-pointer whitespace-nowrap active:scale-95"
            :title="$t('memory.open_skills_folder_tooltip')"
          >
            <Folder class="w-3.5 h-3.5 text-indigo-400" />
            <span>{{ $t('memory.open_skills_folder') }}</span>
          </button>
          <button
            @click="openNewSkillModal()"
            class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer whitespace-nowrap active:scale-95"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>{{ $t('memory.btn_new_skill') }}</span>
          </button>
        </div>
      </div>

      <!-- Skills Cards Grid -->
      <div class="max-w-5xl mx-auto">
        <div v-if="filteredSkills.length === 0" class="flex flex-col items-center justify-center p-12 rounded-2xl bg-[#0d101d] border border-[#1b2135] text-center space-y-3">
          <div class="w-12 h-12 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <Wrench class="w-6 h-6" />
          </div>
          <div class="text-sm font-bold text-slate-200">{{ $t('memory.no_skills_title') }}</div>
          <p class="text-xs text-slate-400 max-w-md">
            {{ $t('memory.no_skills_desc') }}
          </p>
          <button
            @click="openNewSkillModal()"
            class="mt-2 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold cursor-pointer shadow-lg shadow-indigo-600/20"
          >
            {{ $t('memory.teach_first_skill') }}
          </button>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            v-for="skill in filteredSkills"
            :key="skill.id"
            class="p-5 rounded-2xl bg-[#0e1222] border border-[#1e2744] hover:border-indigo-500/40 transition-all shadow-lg flex flex-col justify-between space-y-4 group"
          >
            <div class="space-y-3">
              <!-- Card Header -->
              <div class="space-y-2.5">
                <div class="flex items-start justify-between gap-2.5">
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <div class="w-8 h-8 rounded-xl bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-300 shrink-0">
                      <Wrench class="w-4 h-4" />
                    </div>
                    <div class="min-w-0 flex-1">
                      <h4 class="text-sm font-bold text-slate-100 group-hover:text-indigo-300 transition-colors truncate" :title="skill.name">
                        {{ skill.name }}
                      </h4>
                      <span class="text-[10px] text-slate-500 font-mono block">{{ formatTimestamp(skill.last_refined_at) }}</span>
                    </div>
                  </div>

                  <span class="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30 shrink-0 self-start">
                    v{{ skill.version }}.0 {{ skill.version > 1 ? $t('memory.skill_refined') : $t('memory.skill_initial') }}
                  </span>
                </div>

                <!-- Status & Permission Badges Bar -->
                <div class="flex items-center gap-1.5 flex-wrap">
                  <!-- Permission Mode Toggle Badge (Ask vs Auto) -->
                  <button
                    @click.stop="toggleSkillPermission(skill)"
                    :class="[
                      'px-2 py-0.5 rounded-full text-[10px] font-mono font-bold border transition-all cursor-pointer flex items-center gap-1 active:scale-95 shadow-2xs',
                      skill.permission_mode === 'auto'
                        ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 hover:bg-amber-500/30'
                        : 'bg-indigo-500/10 text-indigo-300 border-indigo-500/30 hover:bg-indigo-500/20'
                    ]"
                    :title="skill.permission_mode === 'auto' ? $t('memory.skill_perm_auto_tooltip') : $t('memory.skill_perm_ask_tooltip')"
                  >
                    <component :is="skill.permission_mode === 'auto' ? Zap : ShieldCheck" class="w-3 h-3" />
                    <span>{{ skill.permission_mode === 'auto' ? $t('memory.skill_perm_auto') : $t('memory.skill_perm_ask') }}</span>
                  </button>
                  <span
                    v-if="skill.scripts && skill.scripts.length > 0"
                    class="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-sky-500/20 text-sky-300 border border-sky-500/30"
                  >
                    📁 {{ $t('memory.skill_has_scripts', { count: skill.scripts.length }) }}
                  </span>
                </div>
              </div>

              <!-- Description -->
              <p v-if="skill.description" class="text-xs text-slate-300 leading-relaxed font-medium">
                {{ skill.description }}
              </p>

              <!-- Triggers -->
              <div v-if="skill.triggers && skill.triggers.length > 0" class="space-y-1">
                <div class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">{{ $t('memory.triggers_label') }}</div>
                <div class="flex flex-wrap gap-1.5">
                  <span
                    v-for="(trig, ti) in skill.triggers"
                    :key="ti"
                    class="px-2 py-0.5 rounded-lg text-[10.5px] font-mono bg-indigo-950/40 text-indigo-300 border border-indigo-500/20"
                  >
                    ⚡ {{ trig }}
                  </span>
                </div>
              </div>

              <!-- Ordered Steps with Command / Script Execution -->
              <div class="space-y-1.5 pt-1">
                <div class="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
                  <ListOrdered class="w-3 h-3 text-indigo-400" />
                  {{ $t('memory.steps_flow_label') }}
                </div>
                <div class="space-y-2 bg-[#14192b] p-2.5 rounded-xl border border-[#202740]">
                  <div
                    v-for="step in skill.steps"
                    :key="step.order"
                    class="flex flex-col gap-1 text-xs text-slate-200"
                  >
                    <div class="flex items-start gap-2">
                      <span class="w-4 h-4 rounded-full bg-indigo-600/30 border border-indigo-500/30 text-indigo-300 flex items-center justify-center text-[10px] font-mono font-bold shrink-0 mt-0.5">
                        {{ step.order }}
                      </span>
                      <span class="leading-relaxed flex-1">{{ step.instruction }}</span>
                    </div>

                    <!-- Step Command or Script Pill -->
                    <div v-if="step.command || step.script_file" class="ml-6 flex items-center gap-2 flex-wrap pt-0.5">
                      <div v-if="step.command" class="flex items-center gap-1.5 bg-[#0b0e1a] px-2 py-0.5 rounded-lg border border-[#1e2744] text-[11px] font-mono text-emerald-400">
                        <Terminal class="w-3 h-3 text-emerald-400/80" />
                        <span class="truncate max-w-[180px]">{{ step.command }}</span>
                        <button
                          @click.stop="handleRunStepCommand(skill.id, step.command)"
                          class="ml-1 px-1.5 py-0.5 rounded bg-emerald-600/20 hover:bg-emerald-600/40 text-emerald-300 text-[10px] font-sans font-semibold transition-all flex items-center gap-1 cursor-pointer active:scale-95"
                          :title="$t('memory.run_command_btn')"
                        >
                          <Play class="w-2.5 h-2.5" />
                          <span>{{ $t('memory.run_command_btn') }}</span>
                        </button>
                      </div>

                      <div v-if="step.script_file" class="flex items-center gap-1.5 bg-[#0b0e1a] px-2 py-0.5 rounded-lg border border-[#1e2744] text-[11px] font-mono text-sky-400">
                        <Code2 class="w-3 h-3 text-sky-400/80" />
                        <span class="truncate max-w-[180px]">{{ step.script_file }}</span>
                        <button
                          @click.stop="handleRunSkillScript(skill.id, step.script_file.replace(/^scripts\//, ''))"
                          class="ml-1 px-1.5 py-0.5 rounded bg-sky-600/20 hover:bg-sky-600/40 text-sky-300 text-[10px] font-sans font-semibold transition-all flex items-center gap-1 cursor-pointer active:scale-95"
                          :title="$t('memory.run_script_btn')"
                        >
                          <Play class="w-2.5 h-2.5" />
                          <span>{{ $t('memory.run_script_btn') }}</span>
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Folder Scripts (if any) -->
              <div v-if="skill.scripts && skill.scripts.length > 0" class="space-y-1">
                <div class="text-[10px] font-bold text-sky-400 uppercase tracking-wider flex items-center gap-1">
                  <Folder class="w-3 h-3 text-sky-400" />
                  {{ $t('memory.scripts_folder_title') }}
                </div>
                <div class="flex flex-wrap gap-1.5">
                  <div
                    v-for="scr in skill.scripts"
                    :key="scr"
                    class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10.5px] font-mono bg-[#141b30] text-sky-300 border border-sky-500/30"
                  >
                    <span>{{ scr }}</span>
                    <button
                      @click.stop="handleRunSkillScript(skill.id, scr)"
                      class="p-0.5 rounded hover:bg-sky-500/30 text-sky-200 transition-colors cursor-pointer"
                      :title="$t('memory.run_script_btn')"
                    >
                      <Play class="w-2.5 h-2.5" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Refinement History -->
              <div v-if="skill.refinement_notes && skill.refinement_notes.length > 0" class="space-y-1">
                <div class="text-[10px] font-bold text-purple-400 uppercase tracking-wider flex items-center gap-1">
                  <History class="w-3 h-3 text-purple-400" />
                  {{ $t('memory.refinement_history_label') }}
                </div>
                <div class="text-[11px] text-purple-200/90 font-mono bg-purple-950/20 border border-purple-500/20 p-2 rounded-lg space-y-0.5 max-h-20 overflow-y-auto">
                  <div v-for="(note, ni) in skill.refinement_notes" :key="ni">
                    • {{ note }}
                  </div>
                </div>
              </div>
            </div>

            <!-- Card Actions -->
            <div class="pt-3 border-t border-[#1a2139] flex items-center justify-between gap-2">
              <span class="text-[10.5px] text-slate-500 font-mono">
                {{ $t('memory.steps_count', { count: skill.steps.length }) }}
              </span>
              <div class="flex items-center gap-1.5">
                <button
                  @click="handleOpenSkillFolder(skill.id)"
                  class="p-1.5 rounded-lg bg-[#151b2e] hover:bg-[#1f2742] border border-[#242e4d] text-slate-400 hover:text-indigo-300 transition-all cursor-pointer active:scale-95"
                  :title="$t('memory.open_skill_folder_tooltip')"
                >
                  <Folder class="w-3.5 h-3.5 text-indigo-400" />
                </button>
                <button
                  @click="openNewSkillModal(skill)"
                  class="px-2.5 py-1.5 rounded-lg bg-[#151b2e] hover:bg-[#1f2742] border border-[#242e4d] text-slate-300 hover:text-white text-xs font-semibold flex items-center gap-1 cursor-pointer transition-all active:scale-95"
                  :title="$t('memory.edit_skill_tooltip')"
                >
                  <Edit3 class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('memory.refine_btn') }}</span>
                </button>
                <button
                  @click="handleDeleteSkill(skill.id)"
                  class="p-1.5 rounded-lg bg-[#151b2e] hover:bg-rose-950/50 border border-[#242e4d] hover:border-rose-500/40 text-slate-400 hover:text-rose-300 transition-all cursor-pointer active:scale-95"
                  :title="$t('memory.delete_skill_tooltip')"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Workspace: Captain's Log in Markdown -->
    <div v-show="activeMemoryTab === 'episodes'" class="flex-1 flex overflow-hidden bg-[#07080d]">
      <!-- Left Column: Log Pages List -->
      <div class="w-80 sm:w-96 shrink-0 border-r border-[#1a2139] flex flex-col bg-[#0b0e1b] overflow-hidden">
        <!-- List Header -->
        <div class="p-3.5 border-b border-[#1a2139] bg-[#0d1020]/60 space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <BookOpen class="w-4 h-4 text-amber-400" />
              <h3 class="text-xs font-bold text-slate-100 uppercase tracking-wide">{{ $t('memory.diary_title') }}</h3>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-amber-500/15 text-amber-300 border border-amber-500/30">
                {{ $t('memory.pages_count', { count: episodesList.length }) }}
              </span>
            </div>
            <div class="flex items-center gap-1">
              <button
                @click="openEpisodesFolder"
                class="p-1.5 rounded-lg bg-[#14192b] hover:bg-indigo-600/30 border border-[#222b48] hover:border-indigo-500/50 text-slate-300 hover:text-indigo-200 transition-all cursor-pointer active:scale-95"
                :title="$t('memory.open_diary_folder_tooltip')"
              >
                <FolderOpen class="w-3.5 h-3.5" />
              </button>
              <button
                @click="fetchEpisodes"
                :disabled="isEpisodesLoading"
                class="p-1.5 rounded-lg bg-[#14192b] hover:bg-slate-700/50 border border-[#222b48] text-slate-300 hover:text-white transition-all cursor-pointer active:scale-95"
                :title="$t('memory.refresh_diary_tooltip')"
              >
                <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isEpisodesLoading }" />
              </button>
            </div>
          </div>

          <!-- Log Search Bar -->
          <div class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2" />
            <input
              v-model="episodeSearchQuery"
              type="text"
              :placeholder="$t('memory.search_diary_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#121626] border border-[#202740] focus:border-amber-500/60 text-xs text-slate-200 placeholder-slate-500 focus:outline-none transition-all"
            />
          </div>
        </div>

        <!-- Scrollable Log Pages List -->
        <div class="flex-1 overflow-y-auto p-2.5 space-y-2">
          <div v-if="filteredEpisodes.length === 0" class="flex flex-col items-center justify-center p-8 text-center text-slate-500 space-y-2">
            <BookOpen class="w-8 h-8 text-slate-600" />
            <span class="text-xs">{{ $t('memory.no_diary_records') }}</span>
          </div>

          <div
            v-for="ep in filteredEpisodes"
            :key="ep.file_name"
            @click="selectedEpisode = ep"
            class="p-3 rounded-xl border transition-all cursor-pointer flex flex-col gap-1.5 group relative"
            :class="selectedEpisode?.file_name === ep.file_name
              ? 'bg-[#151c33] border-indigo-500/70 shadow-lg shadow-indigo-950/40'
              : 'bg-[#0e1222] border-[#1b2238] hover:border-slate-600/50 hover:bg-[#12172b]'"
          >
            <!-- Card Header Row -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5">
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold"
                  :class="selectedEpisode?.file_name === ep.file_name ? 'bg-indigo-600 text-white' : 'bg-[#1e253e] text-indigo-300'">
                  #{{ ep.index }}
                </span>
                <span class="text-[10px] text-slate-400 font-mono">{{ ep.date_time || $t('memory.no_date') }}</span>
              </div>
              <span
                v-if="ep.learned_memories.length > 0"
                class="px-1.5 py-0.2 rounded text-[9.5px] font-semibold bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 flex items-center gap-1"
              >
                <span>{{ $t('memory.facts_count', { count: ep.learned_memories.length }) }}</span>
              </span>
            </div>

            <!-- Chat/Session Tag if available -->
            <div v-if="ep.session_title" class="flex items-center gap-1.5 text-[10px] text-indigo-300 bg-indigo-500/10 px-2 py-0.5 rounded-md border border-indigo-500/20 truncate">
              <MessageSquare class="w-3 h-3 shrink-0 text-indigo-400" />
              <span class="truncate font-medium">{{ ep.session_title }}</span>
            </div>

            <!-- User Message Preview -->
            <div class="text-xs text-slate-200 line-clamp-2 leading-snug font-medium">
              <span class="text-slate-400 font-normal">👤 </span>
              {{ ep.user_message || $t('memory.user_msg_empty') }}
            </div>

            <!-- Link Status Footer -->
            <div class="flex items-center justify-between pt-1 border-t border-white/5 text-[9.5px] text-slate-500 font-mono">
              <span class="flex items-center gap-1">
                <Link2 class="w-3 h-3 text-slate-500" />
                <span>{{ ep.prev_file ? $t('memory.prev_page') : $t('memory.first_page') }}</span>
              </span>
              <span class="text-slate-600">➔</span>
              <span>{{ ep.next_file ? $t('memory.next_page') : $t('memory.current_page') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column: Markdown File Viewer -->
      <div class="flex-1 flex flex-col overflow-hidden bg-[#080b14]">
        <div v-if="selectedEpisode" class="flex-1 flex flex-col overflow-hidden">
          <!-- Top Bar of Selected Record -->
          <div class="p-3.5 border-b border-[#1a2139] bg-[#0c0f1e] flex flex-wrap items-center justify-between gap-3 shrink-0">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 rounded-xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-300 font-mono font-bold text-xs shrink-0">
                #{{ selectedEpisode.index }}
              </div>
              <div>
                <h4 class="text-xs font-bold text-slate-100 flex items-center gap-1.5 font-mono">
                  <span>{{ selectedEpisode.file_name }}</span>
                </h4>
                <div class="text-[10px] text-slate-400 flex items-center gap-2">
                  <span>{{ selectedEpisode.date_time }}</span>
                  <span>•</span>
                  <span class="font-mono text-indigo-300">{{ selectedEpisode.id }}</span>
                </div>
              </div>
            </div>

            <!-- Log Actions and Navigation -->
            <div class="flex items-center gap-2">
              <!-- Previous / Next Navigation -->
              <div class="flex items-center bg-[#101424] p-0.5 rounded-xl border border-[#202740]">
                <button
                  @click="selectEpisodeByFileName(selectedEpisode.prev_file)"
                  :disabled="!selectedEpisode.prev_file"
                  class="px-2.5 py-1 rounded-lg text-xs font-medium flex items-center gap-1 transition-all cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed hover:bg-white/10 text-slate-300"
                  :title="$t('memory.nav_prev_page_tooltip')"
                >
                  <ArrowLeft class="w-3 h-3" />
                  <span class="hidden md:inline">{{ $t('memory.prev') }}</span>
                </button>
                <button
                  @click="selectEpisodeByFileName(selectedEpisode.next_file)"
                  :disabled="!selectedEpisode.next_file"
                  class="px-2.5 py-1 rounded-lg text-xs font-medium flex items-center gap-1 transition-all cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed hover:bg-white/10 text-slate-300"
                  :title="$t('memory.nav_next_page_tooltip')"
                >
                  <span class="hidden md:inline">{{ $t('memory.next') }}</span>
                  <ArrowRight class="w-3 h-3" />
                </button>
              </div>

              <!-- Visual / Raw Switcher -->
              <div class="flex items-center bg-[#101424] p-0.5 rounded-xl border border-[#202740]">
                <button
                  @click="episodeViewMode = 'visual'"
                  class="px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer"
                  :class="episodeViewMode === 'visual' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'"
                >
                  <FileText class="w-3.5 h-3.5 inline mr-1" />
                  <span>{{ $t('memory.visual_mode') }}</span>
                </button>
                <button
                  @click="episodeViewMode = 'raw'"
                  class="px-2.5 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer"
                  :class="episodeViewMode === 'raw' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'"
                >
                  <Code2 class="w-3.5 h-3.5 inline mr-1" />
                  <span>{{ $t('memory.raw_mode') }}</span>
                </button>
              </div>

              <!-- Copy Markdown -->
              <button
                @click="copyEpisodeMarkdown"
                class="px-2.5 py-1.5 rounded-xl bg-[#14192b] hover:bg-[#1e253e] border border-[#242e4d] text-slate-300 hover:text-white text-xs font-medium flex items-center gap-1.5 transition-all cursor-pointer active:scale-95"
                :title="$t('memory.copy_markdown_tooltip')"
              >
                <Check v-if="isCopied" class="w-3.5 h-3.5 text-emerald-400" />
                <Copy v-else class="w-3.5 h-3.5 text-slate-400" />
                <span>{{ isCopied ? $t('common.copied') : $t('common.copy') }}</span>
              </button>

              <!-- Open in Finder -->
              <button
                @click="openEpisodesFolder"
                class="p-1.5 rounded-xl bg-[#14192b] hover:bg-indigo-600/30 border border-[#242e4d] hover:border-indigo-500/50 text-indigo-300 hover:text-white transition-all cursor-pointer active:scale-95"
                :title="$t('memory.open_diary_folder_tooltip')"
              >
                <FolderOpen class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          <!-- Log Record Content -->
          <div class="flex-1 overflow-y-auto p-6 space-y-6">
            <!-- Visual Mode -->
            <div v-if="episodeViewMode === 'visual'" class="max-w-4xl mx-auto space-y-5">
              <!-- Log Chain Metadata -->
              <div class="p-4 rounded-2xl bg-[#0f1324] border border-[#1e2642] shadow-sm grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                <div>
                  <span class="text-[10px] text-slate-500 uppercase tracking-wider font-bold block mb-1">{{ $t('memory.prev') }}:</span>
                  <button
                    v-if="selectedEpisode.prev_file"
                    @click="selectEpisodeByFileName(selectedEpisode.prev_file)"
                    class="text-indigo-400 hover:text-indigo-300 font-mono text-[11px] flex items-center gap-1 cursor-pointer underline underline-offset-2"
                  >
                    <ArrowLeft class="w-3 h-3" />
                    <span>{{ selectedEpisode.prev_file }}</span>
                  </button>
                  <span v-else class="text-slate-500 text-[11px] italic">{{ $t('memory.start_of_diary') }}</span>
                </div>
                <div>
                  <span class="text-[10px] text-slate-500 uppercase tracking-wider font-bold block mb-1">{{ $t('memory.next') }}:</span>
                  <button
                    v-if="selectedEpisode.next_file"
                    @click="selectEpisodeByFileName(selectedEpisode.next_file)"
                    class="text-indigo-400 hover:text-indigo-300 font-mono text-[11px] flex items-center gap-1 cursor-pointer underline underline-offset-2"
                  >
                    <span>{{ selectedEpisode.next_file }}</span>
                    <ArrowRight class="w-3 h-3" />
                  </button>
                  <span v-else class="text-amber-400/80 text-[11px] italic">{{ $t('memory.current_page_waiting') }}</span>
                </div>
              </div>

              <!-- Learned Memories (if any) -->
              <div
                v-if="selectedEpisode.learned_memories.length > 0"
                class="p-4 rounded-2xl bg-emerald-950/20 border border-emerald-500/30 space-y-2 shadow-sm"
              >
                <div class="text-[11px] font-bold text-emerald-400 uppercase tracking-wider flex items-center gap-1.5">
                  <Brain class="w-4 h-4" />
                  <span>{{ $t('memory.facts_registered_page', { count: selectedEpisode.learned_memories.length }) }}</span>
                </div>
                <div class="flex flex-wrap gap-1.5 pt-1">
                  <div
                    v-for="(mem, mi) in selectedEpisode.learned_memories"
                    :key="mi"
                    class="px-2.5 py-1 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-emerald-200 font-mono text-xs"
                  >
                    {{ mem }}
                  </div>
                </div>
              </div>

              <!-- User Message -->
              <div class="p-5 rounded-2xl bg-[#0f1324] border border-[#1e2642] space-y-2 shadow-sm">
                <div class="flex items-center gap-2 text-xs font-bold text-slate-300">
                  <span class="w-6 h-6 rounded-full bg-slate-700 flex items-center justify-center text-xs">👤</span>
                  <span>{{ $t('memory.user_message_title') }}</span>
                </div>
                <div class="text-slate-100 text-sm leading-relaxed whitespace-pre-wrap pl-8">
                  {{ selectedEpisode.user_message }}
                </div>
              </div>

              <!-- AI Response -->
              <div class="p-5 rounded-2xl bg-[#0e1428] border border-indigo-500/30 space-y-3 shadow-md">
                <div class="flex items-center gap-2 text-xs font-bold text-indigo-300">
                  <span class="w-6 h-6 rounded-full bg-indigo-600 flex items-center justify-center text-xs">🤖</span>
                  <span>{{ $t('memory.ai_response_title') }}</span>
                </div>
                <div
                  class="prose prose-invert prose-sm max-w-none text-slate-200 text-sm leading-relaxed pl-8"
                  v-html="renderMarkdown(selectedEpisode.ai_response)"
                ></div>
              </div>
            </div>

            <!-- Raw Markdown Mode -->
            <div v-else class="max-w-4xl mx-auto">
              <div class="p-4 rounded-2xl bg-[#0a0d18] border border-[#1b2238] font-mono text-xs text-slate-300 whitespace-pre-wrap leading-relaxed overflow-x-auto shadow-inner">
                {{ selectedEpisode.raw_markdown }}
              </div>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="flex-1 flex flex-col items-center justify-center p-8 text-center text-slate-500 space-y-3">
          <BookOpen class="w-12 h-12 text-slate-600" />
          <div class="text-sm font-bold text-slate-300">{{ $t('memory.no_page_selected_title') }}</div>
          <p class="text-xs text-slate-500 max-w-sm">
            {{ $t('memory.no_page_selected_desc') }}
          </p>
        </div>
      </div>
    </div>

    <!-- Modal: Total Memory Reset Confirmation -->
    <div
      v-if="isConfirmResetModalOpen"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0f1220] border border-rose-500/30 rounded-2xl max-w-md w-full p-6 space-y-4 shadow-2xl">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-rose-500/20 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0">
            <Trash2 class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-100">{{ $t('memory.reset_confirm_title') }}</h3>
            <p class="text-xs text-slate-400">{{ $t('memory.reset_confirm_desc') }}</p>
          </div>
        </div>

        <p class="text-xs text-slate-300 leading-relaxed bg-[#14192b] p-3 rounded-xl border border-[#202740]">
          {{ $t('memory.reset_confirm_warning', { path: '~/.atena/brain/' }) }}
        </p>

        <div class="flex justify-end gap-2 pt-2">
          <button
            @click="isConfirmResetModalOpen = false"
            class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white transition-all cursor-pointer"
          >{{ $t('common.cancel') }}</button>
          <button
            @click="handleConfirmClearAll"
            class="px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold shadow-lg shadow-rose-600/25 transition-all cursor-pointer"
          >
            {{ $t('memory.btn_confirm_clear_all') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Delete Node Confirmation -->
    <div
      v-if="isConfirmDeleteNodeModalOpen && selectedNode"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0f1220] border border-rose-500/30 rounded-2xl max-w-md w-full p-6 space-y-4 shadow-2xl">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-rose-500/20 border border-rose-500/30 flex items-center justify-center text-rose-400 shrink-0">
            <Trash2 class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-100">{{ $t('memory.delete_node_confirm_title') }}</h3>
            <p class="text-xs text-slate-400">{{ $t('memory.delete_node_confirm_desc', { label: selectedNode.label }) }}</p>
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2">
          <button
            @click="isConfirmDeleteNodeModalOpen = false"
            class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white transition-all cursor-pointer"
          >{{ $t('common.cancel') }}</button>
          <button
            @click="handleConfirmDeleteNode"
            class="px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold shadow-lg shadow-rose-600/25 transition-all cursor-pointer"
          >
            {{ $t('memory.btn_forget_node') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Optimization & Pruning Report -->
    <div
      v-if="sleepReport"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0f1220] border border-[#242c48] rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <div class="flex items-center justify-between border-b border-[#202740] pb-3">
          <div class="flex items-center gap-2.5">
            <div class="w-9 h-9 rounded-xl bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-300">
              <Scissors class="w-5 h-5" />
            </div>
            <div>
              <h3 class="text-sm font-bold text-slate-100">{{ $t('memory.prune_report_title') }}</h3>
              <p class="text-[11px] text-slate-400">{{ $t('memory.prune_report_subtitle') }}</p>
            </div>
          </div>
          <button @click="sleepReport = null" class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10">
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-3 gap-2 text-center">
          <div class="p-2.5 rounded-xl bg-[#14192b] border border-[#202740]">
            <div class="text-[10px] text-slate-400 font-medium">{{ $t('memory.report_nodes_in_graph') }}</div>
            <div class="text-base font-bold font-mono text-slate-100 mt-0.5">{{ graphData.nodes.length }}</div>
          </div>
          <div class="p-2.5 rounded-xl bg-amber-950/20 border border-amber-500/30">
            <div class="text-[10px] text-amber-400 font-medium">{{ $t('memory.report_synapses_pruned') }}</div>
            <div class="text-base font-bold font-mono text-amber-300 mt-0.5">{{ sleepReport.synapses_pruned }}</div>
          </div>
          <div class="p-2.5 rounded-xl bg-indigo-950/20 border border-indigo-500/30">
            <div class="text-[10px] text-indigo-400 font-medium">{{ $t('memory.report_duplicates_unified') }}</div>
            <div class="text-base font-bold font-mono text-indigo-300 mt-0.5">{{ sleepReport.synapses_reinforced }}</div>
          </div>
          <div class="p-2.5 rounded-xl bg-[#14192b] border border-[#202740]">
            <div class="text-[10px] text-slate-400 font-medium">{{ $t('memory.report_orphans_invalid') }}</div>
            <div class="text-base font-bold font-mono text-slate-300 mt-0.5">{{ sleepReport.noise_discarded }}</div>
          </div>
          <div class="p-2.5 rounded-xl bg-purple-950/20 border border-purple-500/30 col-span-2">
            <div class="text-[10px] text-purple-400 font-medium">{{ $t('memory.report_active_connections') }}</div>
            <div class="text-base font-bold font-mono text-purple-300 mt-0.5">{{ graphData.edges.length }}</div>
          </div>
        </div>

        <!-- Details Log -->
        <div class="space-y-1.5">
          <div class="text-[11px] font-bold text-slate-300 uppercase tracking-wider">{{ $t('memory.report_actions_executed') }}</div>
          <div class="max-h-40 overflow-y-auto space-y-1 p-2.5 rounded-xl bg-[#121626] border border-[#202740] text-xs font-mono">
            <div v-for="(detail, i) in sleepReport.details" :key="i" class="text-slate-300">
              {{ detail }}
            </div>
            <div v-if="!sleepReport.details || sleepReport.details.length === 0" class="text-slate-500 text-center py-2">
              {{ $t('memory.report_already_optimized') }}
            </div>
          </div>
        </div>

        <div class="flex justify-end pt-2">
          <button
            @click="sleepReport = null"
            class="px-4 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold shadow-lg shadow-purple-600/20 cursor-pointer"
          >
            {{ $t('memory.report_finish_btn') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Teach Manual Fact -->
    <div
      v-if="isLearnModalOpen"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0f1220] border border-[#242c48] rounded-2xl max-w-md w-full p-5 space-y-4 shadow-2xl">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Brain class="w-5 h-5 text-indigo-400" />
            <h3 class="text-sm font-bold text-slate-100">{{ $t('memory.learn_fact_title') }}</h3>
          </div>
          <button @click="isLearnModalOpen = false" class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10 cursor-pointer">
            <X class="w-4 h-4" />
          </button>
        </div>

        <p class="text-xs text-slate-400 leading-relaxed">
          {{ $t('memory.learn_fact_desc') }}
        </p>

        <textarea
          v-model="newFactText"
          rows="3"
          :placeholder="$t('memory.learn_fact_placeholder')"
          @keydown.enter.ctrl="handleSubmitFact"
          @keydown.enter.meta="handleSubmitFact"
          class="w-full p-3 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-200 placeholder-slate-500 focus:border-indigo-500 focus:outline-none resize-none leading-relaxed"
        ></textarea>

        <!-- Memory Scope Selector (Global vs Private) -->
        <div class="space-y-2 pt-1">
          <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider">{{ $t('memory.learn_fact_scope_label') }}</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              @click="newFactScope = 'global'"
              class="flex items-center gap-2 p-2.5 rounded-xl border text-left transition-all cursor-pointer"
              :class="newFactScope === 'global' ? 'bg-indigo-600/20 border-indigo-500 text-indigo-200 shadow-sm' : 'bg-[#14192b] border-[#242e4d] text-slate-400 hover:border-slate-600'"
            >
              <Globe class="w-4 h-4 text-emerald-400 shrink-0" />
              <div>
                <div class="text-xs font-bold text-slate-100">{{ $t('memory.global_memory') }}</div>
                <div class="text-[9.5px] text-slate-400">{{ $t('memory.all_sessions') }}</div>
              </div>
            </button>
            <button
              type="button"
              @click="newFactScope = 'private'"
              class="flex items-center gap-2 p-2.5 rounded-xl border text-left transition-all cursor-pointer"
              :class="newFactScope === 'private' ? 'bg-indigo-600/20 border-indigo-500 text-indigo-200 shadow-sm' : 'bg-[#14192b] border-[#242e4d] text-slate-400 hover:border-slate-600'"
            >
              <Lock class="w-4 h-4 text-amber-400 shrink-0" />
              <div>
                <div class="text-xs font-bold text-slate-100">{{ $t('memory.private_memory') }}</div>
                <div class="text-[9.5px] text-slate-400">{{ $t('memory.specific_session') }}</div>
              </div>
            </button>
          </div>

          <div v-if="newFactScope === 'private'" class="pt-1">
            <label class="block text-[10px] font-mono text-slate-400 mb-1">{{ $t('memory.session_id_input_label') }}</label>
            <input
              v-model="newFactSessionId"
              type="text"
              :placeholder="$t('memory.session_id_placeholder')"
              class="w-full px-3 py-1.5 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-200 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-mono"
            />
          </div>
        </div>

        <!-- Error or Alert Feedback -->
        <div v-if="learnFactError" class="p-2.5 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
          <div class="text-[11px] text-rose-200 leading-tight">
            {{ learnFactError }}
          </div>
        </div>

        <div class="flex justify-between items-center pt-2">
          <span class="text-[10px] text-slate-500 font-mono hidden sm:inline">{{ $t('memory.ctrl_enter_save') }}</span>
          <div class="flex items-center gap-2 ml-auto">
            <button
              @click="isLearnModalOpen = false"
              class="px-4 py-2 rounded-xl text-xs text-slate-400 hover:text-white cursor-pointer"
            >{{ $t('common.cancel') }}</button>
            <button
              @click="handleSubmitFact"
              :disabled="!newFactText.trim() || isSubmittingFact"
              class="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold shadow-lg shadow-indigo-600/20 cursor-pointer active:scale-95 transition-all"
            >
              <Loader2 v-if="isSubmittingFact" class="w-3.5 h-3.5 animate-spin" />
              <span>{{ isSubmittingFact ? $t('memory.memorizing') : $t('memory.memorize_fact_btn') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal: Teach / Refine Manual Skill -->
    <div
      v-if="isNewSkillModalOpen"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0f1220] border border-[#242c48] rounded-2xl max-w-3xl w-full p-5 space-y-4 shadow-2xl max-h-[90vh] flex flex-col">
        <div class="flex items-center justify-between shrink-0">
          <div class="flex items-center gap-2">
            <Wrench class="w-5 h-5 text-indigo-400" />
            <h3 class="text-sm font-bold text-slate-100">
              {{ editingSkillId ? $t('memory.refine_skill_title') : $t('memory.new_skill_title') }}
            </h3>
          </div>
          <div class="flex items-center gap-2">
            <button
              v-if="editingSkillId"
              @click="handleOpenSkillFolder(editingSkillId)"
              class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#151b2e] hover:bg-[#1e2640] border border-[#242e4d] text-slate-300 text-xs font-semibold cursor-pointer transition-all"
              :title="$t('memory.open_skill_folder_tooltip')"
            >
              <Folder class="w-3.5 h-3.5 text-indigo-400" />
              <span>{{ $t('memory.open_skills_folder') }}</span>
            </button>
            <button @click="isNewSkillModalOpen = false" class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10">
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        <p class="text-xs text-slate-400 leading-relaxed shrink-0">
          {{ $t('memory.skill_modal_desc') }}
        </p>

        <div class="space-y-4 overflow-y-auto pr-2 flex-1 min-h-0 custom-scrollbar">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider mb-1">{{ $t('memory.skill_name_label') }}</label>
              <input
                v-model="newSkillForm.name"
                type="text"
                :placeholder="$t('memory.skill_name_placeholder')"
                class="w-full px-3 py-2 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-medium"
              />
            </div>

            <div>
              <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider mb-1">{{ $t('memory.skill_triggers_label') }}</label>
              <input
                v-model="newSkillForm.triggers"
                type="text"
                :placeholder="$t('memory.skill_triggers_placeholder')"
                class="w-full px-3 py-2 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-medium"
              />
            </div>
          </div>

          <div>
            <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider mb-1">{{ $t('memory.skill_desc_label') }}</label>
            <input
              v-model="newSkillForm.description"
              type="text"
              :placeholder="$t('memory.skill_desc_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#14192b] border border-[#242e4d] text-xs text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-medium"
            />
          </div>

          <!-- Permission Mode Selector (Ask vs Auto) -->
          <div class="p-3.5 rounded-xl bg-[#14192b] border border-[#242e4d] flex items-center justify-between gap-4">
            <div class="min-w-0 flex-1">
              <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider">{{ $t('memory.skill_perm_title') }}</label>
              <p class="text-[10.5px] text-slate-400 mt-0.5">{{ $t('memory.skill_perm_desc') }}</p>
            </div>
            <div class="flex items-center gap-1.5 p-1 bg-[#0e1220] rounded-lg border border-[#202740] shrink-0">
              <button
                type="button"
                @click="newSkillForm.permission_mode = 'ask'"
                :class="[
                  'px-2.5 py-1 rounded-md text-[10.5px] font-semibold transition-all cursor-pointer flex items-center gap-1',
                  newSkillForm.permission_mode !== 'auto'
                    ? 'bg-indigo-600 text-white shadow-xs'
                    : 'text-slate-400 hover:text-slate-200'
                ]"
              >
                <ShieldCheck class="w-3 h-3" />
                <span>{{ $t('memory.skill_perm_ask') }}</span>
              </button>
              <button
                type="button"
                @click="newSkillForm.permission_mode = 'auto'"
                :class="[
                  'px-2.5 py-1 rounded-md text-[10.5px] font-semibold transition-all cursor-pointer flex items-center gap-1',
                  newSkillForm.permission_mode === 'auto'
                    ? 'bg-amber-600 text-white shadow-xs'
                    : 'text-slate-400 hover:text-slate-200'
                ]"
              >
                <Zap class="w-3 h-3" />
                <span>{{ $t('memory.skill_perm_auto') }}</span>
              </button>
            </div>
          </div>

          <!-- Structured Step-by-Step Flow with Command / Script Fields -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider">{{ $t('memory.steps_flow_label') }}</label>
              <button
                type="button"
                @click="addFormStep"
                class="flex items-center gap-1 text-[11px] font-semibold text-indigo-400 hover:text-indigo-300 transition-colors cursor-pointer"
              >
                <Plus class="w-3 h-3" />
                <span>{{ $t('memory.add_step_btn') }}</span>
              </button>
            </div>

            <div class="space-y-2.5">
              <div
                v-for="(step, sIdx) in formSteps"
                :key="sIdx"
                class="p-3 rounded-xl bg-[#14192b] border border-[#202740] space-y-2 relative group"
              >
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1.5 text-xs font-bold text-slate-300">
                    <span class="w-4 h-4 rounded-full bg-indigo-600/40 text-indigo-300 flex items-center justify-center text-[10px] font-mono">
                      {{ sIdx + 1 }}
                    </span>
                    <span>{{ $t('memory.step_instruction_label') }}</span>
                  </div>
                  <button
                    v-if="formSteps.length > 1"
                    type="button"
                    @click="removeFormStep(sIdx)"
                    class="text-slate-500 hover:text-rose-400 p-1 rounded transition-colors cursor-pointer"
                    :title="$t('memory.remove_step_tooltip')"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>

                <input
                  v-model="step.instruction"
                  type="text"
                  :placeholder="$t('memory.skill_steps_placeholder')"
                  class="w-full px-3 py-1.5 rounded-lg bg-[#0d111e] border border-[#242e4d] text-xs text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none font-medium"
                />

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1">
                  <div>
                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-0.5">{{ $t('memory.step_command_label') }}</label>
                    <div class="flex items-center gap-1">
                      <div class="relative flex-1">
                        <Terminal class="w-3 h-3 text-slate-500 absolute left-2 top-1/2 -translate-y-1/2" />
                        <input
                          v-model="step.command"
                          type="text"
                          :placeholder="$t('memory.step_command_placeholder')"
                          class="w-full pl-7 pr-2 py-1 rounded-lg bg-[#0b0e18] border border-[#1e2640] text-[11px] font-mono text-emerald-300 placeholder-slate-600 focus:border-emerald-500 focus:outline-none"
                        />
                      </div>
                      <button
                        v-if="step.command?.trim()"
                        type="button"
                        @click="handleRunStepCommand(editingSkillId, step.command!)"
                        class="px-2 py-1 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 text-[10px] font-semibold flex items-center gap-1 cursor-pointer transition-all shrink-0 active:scale-95"
                        :title="$t('memory.test_step_btn')"
                      >
                        <Play class="w-2.5 h-2.5" />
                        <span>{{ $t('memory.test_step_btn') }}</span>
                      </button>
                    </div>
                  </div>

                  <div>
                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-0.5">{{ $t('memory.step_script_label') }}</label>
                    <div class="relative">
                      <Code2 class="w-3 h-3 text-slate-500 absolute left-2 top-1/2 -translate-y-1/2" />
                      <input
                        v-model="step.script_file"
                        type="text"
                        :placeholder="$t('memory.step_script_placeholder')"
                        class="w-full pl-7 pr-2 py-1 rounded-lg bg-[#0b0e18] border border-[#1e2640] text-[11px] font-mono text-sky-300 placeholder-slate-600 focus:border-sky-500 focus:outline-none"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-if="editingSkillId">
            <label class="block text-[11px] font-bold text-purple-300 uppercase tracking-wider mb-1">{{ $t('memory.skill_refinement_note_label') }}</label>
            <input
              v-model="newSkillForm.refinement_note"
              type="text"
              :placeholder="$t('memory.skill_refinement_note_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#14192b] border border-purple-500/40 text-xs text-purple-200 placeholder-slate-500 focus:border-purple-500 focus:outline-none font-medium"
            />
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-[#1e2640] shrink-0">
          <button
            @click="isNewSkillModalOpen = false"
            class="px-3.5 py-1.5 rounded-xl bg-[#14192b] hover:bg-[#1c233d] border border-[#222b48] text-slate-300 text-xs font-semibold cursor-pointer"
          >{{ $t('common.cancel') }}</button>
          <button
            @click="handleSaveSkill"
            :disabled="!newSkillForm.name.trim() || formSteps.every(s => !s.instruction.trim())"
            class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold shadow-lg shadow-indigo-600/20 cursor-pointer flex items-center gap-1.5"
          >
            <CheckCircle2 class="w-3.5 h-3.5" />
            <span>{{ editingSkillId ? $t('memory.save_refinement_btn') : $t('memory.save_skill_btn') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Terminal / Script Output -->
    <div
      v-if="isTerminalModalOpen"
      class="fixed inset-0 bg-black/85 backdrop-blur-md z-[60] flex items-center justify-center p-4 animate-in fade-in duration-150"
    >
      <div class="bg-[#0b0e17] border border-[#1e2744] rounded-2xl max-w-2xl w-full p-5 space-y-4 shadow-2xl flex flex-col max-h-[85vh]">
        <!-- Terminal Header -->
        <div class="flex items-center justify-between pb-3 border-b border-[#1b2238] shrink-0">
          <div class="flex items-center gap-2.5">
            <div class="w-7 h-7 rounded-lg bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
              <Terminal class="w-4 h-4" />
            </div>
            <div>
              <h3 class="text-xs font-bold text-slate-100 flex items-center gap-2">
                {{ $t('memory.terminal_title') }}
                <span
                  v-if="terminalRunning"
                  class="px-2 py-0.5 rounded-full text-[10px] font-mono bg-amber-500/20 text-amber-300 border border-amber-500/30 animate-pulse"
                >
                  {{ $t('memory.terminal_status_running') }}
                </span>
                <span
                  v-else-if="terminalOutput?.success"
                  class="px-2 py-0.5 rounded-full text-[10px] font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
                >
                  {{ $t('memory.terminal_status_success', { code: terminalOutput.exitCode ?? 0 }) }}
                </span>
                <span
                  v-else
                  class="px-2 py-0.5 rounded-full text-[10px] font-mono bg-rose-500/20 text-rose-300 border border-rose-500/30"
                >
                  {{ $t('memory.terminal_status_error', { code: terminalOutput?.exitCode ?? -1 }) }}
                </span>
              </h3>
              <p class="text-[11px] font-mono text-slate-400 truncate max-w-md">
                $ {{ terminalOutput?.commandName }}
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <span v-if="terminalOutput?.durationMs" class="text-[10.5px] font-mono text-slate-500">
              {{ $t('memory.terminal_duration', { ms: terminalOutput.durationMs }) }}
            </span>
            <button
              @click="copyTerminalOutput"
              class="p-1.5 rounded-lg bg-[#14192b] hover:bg-[#1e2640] border border-[#222b48] text-slate-300 hover:text-white transition-all cursor-pointer"
              :title="$t('memory.terminal_copy')"
            >
              <Check v-if="terminalCopied" class="w-3.5 h-3.5 text-emerald-400" />
              <Copy v-else class="w-3.5 h-3.5" />
            </button>
            <button
              @click="isTerminalModalOpen = false"
              class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Destructive Command Alert if blocked -->
        <div
          v-if="terminalOutput?.isBlocked"
          class="p-3.5 rounded-xl bg-rose-950/40 border border-rose-500/40 flex items-start gap-3 text-rose-200 text-xs shrink-0"
        >
          <AlertTriangle class="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
          <div>
            <div class="font-bold text-rose-300">{{ $t('memory.destructive_warning_title') }}</div>
            <p class="mt-0.5 text-rose-200/90 leading-relaxed">{{ $t('memory.destructive_warning_desc') }}</p>
          </div>
        </div>

        <!-- Console Output Window -->
        <div class="flex-1 bg-[#05070d] rounded-xl border border-[#182038] p-3 font-mono text-xs overflow-y-auto space-y-3 min-h-[180px] select-text">
          <div v-if="terminalRunning" class="flex items-center gap-2 text-slate-400">
            <Loader2 class="w-4 h-4 animate-spin text-indigo-400" />
            <span>{{ $t('memory.terminal_status_running') }}</span>
          </div>

          <div v-if="terminalOutput?.stdout" class="space-y-1">
            <div class="text-[10px] uppercase font-bold text-emerald-500/70 tracking-wider">
              {{ $t('memory.terminal_stdout') }}
            </div>
            <pre class="text-slate-200 whitespace-pre-wrap leading-relaxed">{{ terminalOutput.stdout }}</pre>
          </div>

          <div v-if="terminalOutput?.stderr" class="space-y-1">
            <div class="text-[10px] uppercase font-bold text-rose-500/70 tracking-wider">
              {{ $t('memory.terminal_stderr') }}
            </div>
            <pre class="text-rose-300 whitespace-pre-wrap leading-relaxed">{{ terminalOutput.stderr }}</pre>
          </div>

          <div v-if="!terminalRunning && !terminalOutput?.stdout && !terminalOutput?.stderr" class="text-slate-500 italic">
            (Empty output)
          </div>
        </div>

        <!-- Terminal Footer -->
        <div class="flex justify-end pt-2 shrink-0">
          <button
            @click="isTerminalModalOpen = false"
            class="px-4 py-1.5 rounded-xl bg-[#14192b] hover:bg-[#1c233d] border border-[#222b48] text-slate-300 text-xs font-semibold cursor-pointer"
          >
            {{ $t('memory.terminal_close') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { saveTextFile } from '~/utils/exportMarkdown'
import {
  Brain,
  Zap,
  ShieldCheck,
  Share2,
  CircleDot,
  HardDrive,
  RefreshCw,
  Plus,
  Trash2,
  Sparkles,
  Search,
  Maximize2,
  Radio,
  Play,
  X,
  Box,
  MapPin,
  Tag,
  Activity,
  Moon,
  AlertTriangle,
  CheckCircle2,
  Loader2,
  Wrench,
  ListOrdered,
  Edit3,
  History,
  Scissors,
  Clock,
  FolderOpen,
  ScrollText,
  BookOpen,
  FileText,
  Link2,
  Copy,
  Check,
  ArrowLeft,
  ArrowRight,
  Code2,
  MessageSquare,
  Download,
  Lock,
  Globe,
  FileJson,
  Filter,
  Network,
  Folder,
  Terminal
} from 'lucide-vue-next'
import MarkdownIt from 'markdown-it'
import { useAppLocale } from '../../composables/useLocale'
import type { AppConfig } from '../../types'

export interface GraphNode {
  id: number
  label: string
  type_flag: number | string
  valence?: number
  session_id?: string | null
  created_at?: number
}

export interface GraphEdge {
  source_id: number
  target_id: number
  relation_type: number | string
  weight?: number
  access_count?: number
  created_at?: number
  last_accessed?: number
}

export interface PhysicsNode {
  id: number
  label: string
  type_flag: number | string
  valence: number
  session_id: string | null
  x: number
  y: number
  vx: number
  vy: number
  radius: number
  pulsePhase?: number
  excitation?: number
}

export interface EpisodeItem {
  file_name: string
  timestamp?: number
  user_message?: string
  ai_response?: string
  learned_memories: string[]
  raw_markdown: string
  [key: string]: any
}

export interface SkillStepItem {
  order: number
  instruction: string
  tool_name?: string | null
  command?: string | null
  script_file?: string | null
  cwd?: string | null
  timeout_ms?: number | null
}

export interface SkillItem {
  id: string
  name: string
  description: string
  triggers: string[]
  steps: SkillStepItem[]
  version: number
  executions_count: number
  success_count: number
  last_refined_at: number
  refinement_notes: string[]
  folder_path?: string | null
  scripts?: string[]
  env_vars?: Record<string, string> | null
  [key: string]: any
}

export interface SleepReport {
  timestamp: number | string
  events_processed: number
  facts_consolidated: number
  rules_created: number
  synapses_pruned: number
  synapses_reinforced: number
  noise_discarded: number
  positive_count: number
  negative_count: number
  details?: string[]
}

export interface MemoryStats {
  total_nodes: number
  total_edges: number
  cache_size: number
  cache_hits: number
  cache_misses: number
  cache_hit_rate: number
}

const md = new MarkdownIt({ html: true, breaks: true, linkify: true })
const renderMarkdown = (content?: string | null): string => {
  if (!content) return ''
  return md.render(content)
}

const { t } = useAppLocale()

const props = withDefaults(
  defineProps<{
    isActive?: boolean
    config?: AppConfig
  }>(),
  {
    isActive: true,
    config: () => ({
      enable_cognitive_memory: true,
      enable_facts_memory: true,
      enable_skills_memory: true,
      enable_episodic_memory: true
    })
  }
)

const emit = defineEmits<{
  (e: 'saveConfig', config: AppConfig): void
}>()

const isFactsActive = computed(() => props.config?.enable_facts_memory !== false)
const isSkillsActive = computed(() => props.config?.enable_skills_memory !== false)
const isEpisodesActive = computed(() => props.config?.enable_episodic_memory !== false)

const isCurrentLayerActive = computed(() => {
  if (activeMemoryTab.value === 'graph') return isFactsActive.value
  if (activeMemoryTab.value === 'skills') return isSkillsActive.value
  if (activeMemoryTab.value === 'episodes') return isEpisodesActive.value
  return true
})

const currentLayerPausedMessage = computed(() => {
  if (activeMemoryTab.value === 'graph') return t('memory.facts_paused_banner')
  if (activeMemoryTab.value === 'skills') return t('memory.skills_paused_banner')
  if (activeMemoryTab.value === 'episodes') return t('memory.episodes_paused_banner')
  return ''
})

const toggleCurrentLayer = () => {
  if (!props.config) return
  if (activeMemoryTab.value === 'graph') {
    props.config.enable_facts_memory = !isFactsActive.value
  } else if (activeMemoryTab.value === 'skills') {
    props.config.enable_skills_memory = !isSkillsActive.value
  } else if (activeMemoryTab.value === 'episodes') {
    props.config.enable_episodic_memory = !isEpisodesActive.value
  }
  emit('saveConfig', props.config)
}

const canvasRef = ref<HTMLCanvasElement | null>(null)
const canvasContainerRef = ref<HTMLDivElement | null>(null)
const isDarkMode = inject<Ref<boolean>>('isDarkMode', ref(true))

const checkIsDark = (): boolean => {
  if (typeof document !== 'undefined' && document.documentElement) {
    return !document.documentElement.classList.contains('light')
  }
  return isDarkMode?.value ?? true
}
const isLoading = ref(true)
const isRefreshing = ref(false)
const isActivating = ref(false)
const isOptimizingGraph = ref(false)
const isLearnModalOpen = ref(false)
const isSubmittingFact = ref(false)
const learnFactError = ref<string | null>(null)
const isNewSkillModalOpen = ref(false)
const isConfirmResetModalOpen = ref(false)
const isConfirmDeleteNodeModalOpen = ref(false)

const showActivationPanel = ref(false)
const activeMemoryTab = ref<'graph' | 'skills' | 'episodes'>('graph')
const skillsList = ref<SkillItem[]>([])
const editingSkillId = ref<string | null>(null)
const skillSearchQuery = ref('')

// Episodic Chain States
const episodesList = ref<EpisodeItem[]>([])
const selectedEpisode = ref<EpisodeItem | null>(null)
const episodeSearchQuery = ref('')
const episodeViewMode = ref<'visual' | 'raw'>('visual')
const isEpisodesLoading = ref(false)
const isCopied = ref(false)
const newSkillForm = ref({
  name: '',
  description: '',
  triggers: '',
  steps: '',
  refinement_note: '',
  permission_mode: 'ask'
})

interface FormStepItem {
  instruction: string
  command?: string
  script_file?: string
}

const formSteps = ref<FormStepItem[]>([])

const isTerminalModalOpen = ref(false)
const terminalRunning = ref(false)
const terminalOutput = ref<{
  commandName: string
  success: boolean
  exitCode: number | null
  stdout: string
  stderr: string
  durationMs: number
  isBlocked?: boolean
} | null>(null)
const terminalCopied = ref(false)

const searchQuery = ref('')
const scopeFilter = ref<'all' | 'global' | 'private'>('all')
const valenceFilter = ref<'all' | 'positive' | 'neutral' | 'negative'>('all')
const typeFilter = ref<string>('all')
const showFiltersBar = ref(false)

const newFactScope = ref<'global' | 'private'>('global')
const newFactSessionId = ref('')

const isExporting = ref(false)
const showExportMenu = ref(false)

const isFilterActive = computed(() => {
  return Boolean(
    searchQuery.value.trim() ||
    scopeFilter.value !== 'all' ||
    valenceFilter.value !== 'all' ||
    typeFilter.value !== 'all'
  )
})

const clearAllFilters = () => {
  searchQuery.value = ''
  scopeFilter.value = 'all'
  valenceFilter.value = 'all'
  typeFilter.value = 'all'
}

const matchingNodeIds = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  const terms = query ? query.split(/\s+/).filter(Boolean) : []
  const matches = new Set<number>()

  for (const node of graphData.value.nodes) {
    // 1. Scope filter
    if (scopeFilter.value === 'global' && node.session_id) continue
    if (scopeFilter.value === 'private' && !node.session_id) continue

    // 2. Valence filter
    if (valenceFilter.value === 'positive' && node.valence !== 1) continue
    if (valenceFilter.value === 'neutral' && node.valence !== 0) continue
    if (valenceFilter.value === 'negative' && node.valence !== -1) continue

    // 3. Type filter
    if (typeFilter.value !== 'all') {
      const typeName = getNodeTypeName(node.type_flag).toLowerCase()
      if (!typeName.includes(typeFilter.value.toLowerCase())) continue
    }

    // 4. Text search with multi-term support
    if (terms.length > 0) {
      const label = (node.label || '').toLowerCase()
      const typeName = getNodeTypeName(node.type_flag).toLowerCase()
      const sessionStr = (node.session_id || '').toLowerCase()
      const allText = `${label} ${typeName} ${sessionStr}`
      if (!terms.every((t) => allText.includes(t))) continue
    }

    matches.add(node.id)
  }

  return matches
})

const activationAnchor = ref('')
const synthesizedContext = ref('')
const newFactText = ref('')
const selectedNode = ref<GraphNode | null>(null)
const sleepReport = ref<SleepReport | null>(null)
const vigiliaBuffer = ref<any[]>([])

const stats = ref<MemoryStats>({
  total_nodes: 0,
  total_edges: 0,
  cache_size: 0,
  cache_hits: 0,
  cache_misses: 0,
  cache_hit_rate: 0
})

const graphData = ref<{ nodes: GraphNode[]; edges: GraphEdge[] }>({
  nodes: [],
  edges: []
})

// Physics simulation variables
let animationFrameId: number | null = null
let nodesPhysics: PhysicsNode[] = []
let zoom = 1
let panX = 0
let panY = 0
let isDragging = false
let dragTargetNode: PhysicsNode | null = null
let dragStartPos = { x: 0, y: 0 }
let resizeObserver: ResizeObserver | null = null
let animTime = 0
const hoveredNodeId = ref<number | null>(null)

interface AmbientParticle {
  x: number
  y: number
  radius: number
  alpha: number
  vx: number
  vy: number
  color: string
  phase: number
}
let ambientParticles: AmbientParticle[] = []

const initAmbientParticles = (width: number, height: number) => {
  const colors = ['#6366f1', '#a855f7', '#14b8a6', '#38bdf8', '#818cf8', '#ec4899']
  ambientParticles = Array.from({ length: 40 }, () => ({
    x: (Math.random() - 0.5) * Math.max(width, 800) * 2.5,
    y: (Math.random() - 0.5) * Math.max(height, 600) * 2.5,
    radius: 0.8 + Math.random() * 1.6,
    alpha: 0.15 + Math.random() * 0.45,
    vx: (Math.random() - 0.5) * 0.25,
    vy: (Math.random() - 0.5) * 0.25,
    color: colors[Math.floor(Math.random() * colors.length)] || '#6366f1',
    phase: Math.random() * Math.PI * 2
  }))
}

// Palette
const getNodeColor = (type: number | string, valence: number = 0): string => {
  if (valence === 1) return '#10b981' // Positive Reward (Emerald)
  if (valence === -1 || type === 5 || type === 'RuleOrAlert') return '#f43f5e' // Negative / Rule (Rose)
  if (type === 1 || type === 'Object') return '#6366f1' // Object (Indigo)
  if (type === 2 || type === 'Action') return '#f59e0b' // Action (Amber)
  if (type === 3 || type === 'Container') return '#14b8a6' // Container (Teal)
  if (type === 4 || type === 'Attribute') return '#ec4899' // Attribute (Pink)
  return '#818cf8'
}

const getNodeTypeName = (type: number | string): string => {
  if (type === 1 || type === 'Object') return t('memory.node_type_object')
  if (type === 2 || type === 'Action') return t('memory.node_type_action')
  if (type === 3 || type === 'Container') return t('memory.node_type_container')
  if (type === 4 || type === 'Attribute') return t('memory.node_type_attribute')
  if (type === 5 || type === 'RuleOrAlert') return t('memory.node_type_rule')
  return t('memory.node_type_concept')
}

const getNodeIcon = (type: number | string, valence: number = 0) => {
  if (valence === 1) return CheckCircle2
  if (valence === -1 || type === 5 || type === 'RuleOrAlert') return AlertTriangle
  if (type === 1 || type === 'Object') return Box
  if (type === 2 || type === 'Action') return Activity
  if (type === 3 || type === 'Container') return MapPin
  if (type === 4 || type === 'Attribute') return Tag
  return CircleDot
}

const getRelationName = (rel: number | string): string => {
  if (rel === 0xA1 || rel === 'StoredWith') return t('memory.relation_stored_with')
  if (rel === 0xA2 || rel === 'LocatedIn') return t('memory.relation_located_in')
  if (rel === 0xA3 || rel === 'HasProperty') return t('memory.relation_has_property')
  if (rel === 0xA4 || rel === 'AvoidAction') return t('memory.relation_avoid_action')
  return t('memory.relation_connected_to')
}

const getNodeLabel = (id: number): string => {
  const n = graphData.value.nodes.find((item) => item.id === id)
  return n ? n.label : `ID #${id}`
}

const selectedNodeEdges = computed(() => {
  if (!selectedNode.value) return []
  return graphData.value.edges.filter((e) => e.source_id === selectedNode.value!.id)
})

// Suggested real nodes from the current memory graph for 1-click spreading activation test
const suggestedNodes = computed(() => {
  if (!graphData.value.nodes || graphData.value.nodes.length === 0) return []
  const nodes = [...graphData.value.nodes]
  // Prioritize entities/concepts, then other nodes
  const entities = nodes.filter((n) => n.type_flag === 1 || n.type_flag === 'Object' || n.type_flag === 'Entity' || n.type_flag === 0x01)
  const others = nodes.filter((n) => n.type_flag !== 1 && n.type_flag !== 'Object' && n.type_flag !== 'Entity' && n.type_flag !== 0x01)
  return [...entities, ...others].slice(0, 4)
})

const suggestedPlaceholder = computed(() => {
  if (suggestedNodes.value.length > 0) {
    return t('memory.suggested_placeholder_prefix', { examples: suggestedNodes.value.map((n) => n.label).join(', ') })
  }
  return t('memory.suggested_placeholder_fallback')
})

const testWithNode = (label: string) => {
  if (!label) return
  showActivationPanel.value = true
  activationAnchor.value = label
  handleTestActivation()
}

// Episodic Chain Methods & Filters (.md)
const filteredEpisodes = computed(() => {
  if (!episodeSearchQuery.value.trim()) return episodesList.value
  const q = episodeSearchQuery.value.toLowerCase()
  return episodesList.value.filter(ep =>
    (ep.user_message && ep.user_message.toLowerCase().includes(q)) ||
    (ep.ai_response && ep.ai_response.toLowerCase().includes(q)) ||
    (ep.file_name && ep.file_name.toLowerCase().includes(q)) ||
    (ep.learned_memories && ep.learned_memories.some(m => m.toLowerCase().includes(q)))
  )
})

const fetchEpisodes = async () => {
  isEpisodesLoading.value = true
  try {
    const list = await invoke<EpisodeItem[]>('episodes_get_all')
    episodesList.value = list || []
    if (episodesList.value.length > 0) {
      if (!selectedEpisode.value || !episodesList.value.some(e => e.file_name === selectedEpisode.value!.file_name)) {
        selectedEpisode.value = episodesList.value[0] || null
      } else {
        const current = episodesList.value.find(e => e.file_name === selectedEpisode.value!.file_name)
        if (current) selectedEpisode.value = current
      }
    }
  } catch (err) {
    console.error('Failed to fetch episodes:', err)
  } finally {
    isEpisodesLoading.value = false
  }
}

const openEpisodesFolder = async () => {
  try {
    await invoke('episodes_open_folder')
  } catch (err) {
    console.error('Failed to open episodes folder:', err)
  }
}

const selectEpisodeByFileName = (fileName: string) => {
  if (!fileName) return
  const ep = episodesList.value.find(e => e.file_name === fileName)
  if (ep) {
    selectedEpisode.value = ep
  }
}

const copyEpisodeMarkdown = () => {
  if (!selectedEpisode.value) return
  navigator.clipboard.writeText(selectedEpisode.value.raw_markdown)
  isCopied.value = true
  setTimeout(() => {
    isCopied.value = false
  }, 2000)
}

const formatTimestamp = (ts?: number | null): string => {
  if (!ts) return t('memory.no_date')
  const d = new Date(ts * 1000)
  if (isNaN(d.getTime())) return t('memory.invalid_date')
  const today = new Date()
  const isToday = d.getDate() === today.getDate() && d.getMonth() === today.getMonth() && d.getFullYear() === today.getFullYear()
  const timePart = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  if (isToday) {
    return t('memory.today_at', { time: timePart })
  }
  const datePart = d.toLocaleDateString([], { day: '2-digit', month: '2-digit', year: 'numeric' })
  return t('memory.date_at', { date: datePart, time: timePart })
}

const formatCleanText = (text?: string | null): string => {
  if (!text) return ''
  let s = text
  // 1. Remove code blocks
  s = s.split('```').filter((_, i) => i % 2 === 0).join(' ')
  // 2. Remove markdown symbols (|), (#), (*), (`), (_), (~)
  s = s.replace(/[#|*`~_]/g, ' ')
  // 3. Remove extraneous quotes
  s = s.replace(/["“”«»]/g, ' ')
  // 4. Collapse multiple spaces and trim
  s = s.replace(/\s+/g, ' ').trim()
  // 5. Fix spaces before punctuation
  s = s.replace(/\s+([,.!?:;])/g, '$1')
  return s
}

// =========================================================================
// Procedural Skills (Skills & Refinement)
// =========================================================================

const fetchSkills = async () => {
  try {
    const res = await invoke<SkillItem[]>('skills_get_all')
    if (res) {
      skillsList.value = res
    }
  } catch (err) {
    console.error('Failed to fetch skills:', err)
  }
}

const openNewSkillModal = (skill: SkillItem | null = null) => {
  if (skill) {
    editingSkillId.value = skill.id
    newSkillForm.value = {
      name: skill.name,
      description: skill.description,
      triggers: skill.triggers.join(', '),
      steps: '',
      refinement_note: '',
      permission_mode: skill.permission_mode || 'ask'
    }
    formSteps.value = (skill.steps || []).map(s => ({
      instruction: s.instruction,
      command: s.command || '',
      script_file: s.script_file || '',
    }))
    if (formSteps.value.length === 0) {
      formSteps.value = [{ instruction: '', command: '', script_file: '' }]
    }
  } else {
    editingSkillId.value = null
    newSkillForm.value = {
      name: '',
      description: '',
      triggers: '',
      steps: '',
      refinement_note: '',
      permission_mode: 'ask'
    }
    formSteps.value = [{ instruction: '', command: '', script_file: '' }]
  }
  isNewSkillModalOpen.value = true
}

const toggleSkillPermission = async (skill: SkillItem) => {
  const newMode = skill.permission_mode === 'auto' ? 'ask' : 'auto'
  try {
    await invoke('skills_set_permission_mode', {
      skillId: skill.id,
      permissionMode: newMode
    })
    skill.permission_mode = newMode
  } catch (err) {
    console.error('Failed to toggle skill permission:', err)
  }
}

const addFormStep = () => {
  formSteps.value.push({ instruction: '', command: '', script_file: '' })
}

const removeFormStep = (idx: number) => {
  if (formSteps.value.length > 1) {
    formSteps.value.splice(idx, 1)
  }
}

const handleSaveSkill = async () => {
  try {
    const triggers = newSkillForm.value.triggers.split(',').map(s => s.trim()).filter(Boolean)
    const validSteps = formSteps.value.filter(s => s.instruction.trim().length > 0)
    if (!newSkillForm.value.name.trim() || validSteps.length === 0) return

    const detailedSteps = validSteps.map((s, idx) => ({
      order: idx + 1,
      instruction: s.instruction.trim(),
      tool_name: null,
      command: s.command?.trim() ? s.command.trim() : null,
      script_file: s.script_file?.trim() ? s.script_file.trim() : null,
      cwd: null,
      timeout_ms: null
    }))

    const rawSteps = detailedSteps.map(s => s.instruction)

    await invoke('skills_save_manual', {
      id: editingSkillId.value,
      name: newSkillForm.value.name,
      description: newSkillForm.value.description,
      triggers,
      steps: rawSteps,
      stepsDetailed: detailedSteps,
      refinementNote: newSkillForm.value.refinement_note || null,
      envVars: null,
      permissionMode: newSkillForm.value.permission_mode || 'ask'
    })
    isNewSkillModalOpen.value = false
    await fetchSkills()
  } catch (err) {
    console.error('Failed to save skill:', err)
  }
}

const handleOpenSkillsFolder = async () => {
  try {
    await invoke('skills_open_folder', { skillId: null })
  } catch (err) {
    console.error('Failed to open skills folder:', err)
  }
}

const handleOpenSkillFolder = async (skillId: string) => {
  try {
    await invoke('skills_open_folder', { skillId })
  } catch (err) {
    console.error('Failed to open skill folder:', err)
  }
}

const handleRunStepCommand = async (skillId: string | null, cmd: string) => {
  if (!cmd || !cmd.trim()) return
  terminalRunning.value = true
  isTerminalModalOpen.value = true
  terminalOutput.value = {
    commandName: cmd,
    success: false,
    exitCode: null,
    stdout: '',
    stderr: '',
    durationMs: 0
  }
  try {
    const res = await invoke<any>('skills_run_command', {
      skillId,
      command: cmd.trim(),
      cwd: null,
      timeoutMs: 30000
    })
    terminalOutput.value = {
      commandName: cmd,
      success: res.success,
      exitCode: res.exit_code,
      stdout: res.stdout || '',
      stderr: res.stderr || '',
      durationMs: res.duration_ms || 0
    }
  } catch (err: any) {
    const msg = String(err)
    const isDestructive = msg.includes('Destructive') || msg.includes('safety') || msg.includes('rejected')
    terminalOutput.value = {
      commandName: cmd,
      success: false,
      exitCode: -1,
      stdout: '',
      stderr: msg,
      durationMs: 0,
      isBlocked: isDestructive
    }
  } finally {
    terminalRunning.value = false
  }
}

const handleRunSkillScript = async (skillId: string, scriptName: string) => {
  if (!scriptName) return
  terminalRunning.value = true
  isTerminalModalOpen.value = true
  const displayCmd = `scripts/${scriptName}`
  terminalOutput.value = {
    commandName: displayCmd,
    success: false,
    exitCode: null,
    stdout: '',
    stderr: '',
    durationMs: 0
  }
  try {
    const res = await invoke<any>('skills_run_script', {
      skillId,
      scriptName,
      args: [],
      cwd: null,
      timeoutMs: 30000
    })
    terminalOutput.value = {
      commandName: displayCmd,
      success: res.success,
      exitCode: res.exit_code,
      stdout: res.stdout || '',
      stderr: res.stderr || '',
      durationMs: res.duration_ms || 0
    }
  } catch (err: any) {
    terminalOutput.value = {
      commandName: displayCmd,
      success: false,
      exitCode: -1,
      stdout: '',
      stderr: String(err),
      durationMs: 0
    }
  } finally {
    terminalRunning.value = false
  }
}

const copyTerminalOutput = () => {
  if (!terminalOutput.value) return
  const full = `${terminalOutput.value.commandName}\n\nSTDOUT:\n${terminalOutput.value.stdout}\n\nSTDERR:\n${terminalOutput.value.stderr}`
  navigator.clipboard.writeText(full)
  terminalCopied.value = true
  setTimeout(() => {
    terminalCopied.value = false
  }, 2000)
}

const handleDeleteSkill = async (id: string) => {
  try {
    await invoke('skills_delete', { id })
    await fetchSkills()
  } catch (err) {
    console.error('Failed to delete skill:', err)
  }
}

const filteredSkills = computed(() => {
  if (!skillSearchQuery.value.trim()) return skillsList.value
  const q = skillSearchQuery.value.toLowerCase()
  return skillsList.value.filter(s =>
    s.name.toLowerCase().includes(q) ||
    s.description.toLowerCase().includes(q) ||
    s.triggers.some(t => t.toLowerCase().includes(q))
  )
})

// Fetch full graph & vigilia buffer from Rust backend
const fetchGraph = async () => {
  isRefreshing.value = true
  try {
    const [res, vigiliaRes] = await Promise.all([
      invoke<{ stats: MemoryStats; nodes: GraphNode[]; edges: GraphEdge[] }>('memory_get_full_graph'),
      invoke<any[]>('memory_get_vigilia_buffer').catch(() => [])
    ])
    if (res) {
      stats.value = res.stats || stats.value
      graphData.value.nodes = res.nodes || []
      graphData.value.edges = res.edges || []
      syncPhysicsNodes()
    }
    if (vigiliaRes) {
      vigiliaBuffer.value = vigiliaRes
    }
  } catch (err) {
    console.error('Failed to fetch memory graph:', err)
  } finally {
    isLoading.value = false
    isRefreshing.value = false
  }
}

const handleOptimizeAndPruneGraph = async () => {
  isOptimizingGraph.value = true
  try {
    const report = await invoke<any>('memory_optimize_and_prune')
    sleepReport.value = {
      timestamp: report.timestamp,
      events_processed: 0,
      facts_consolidated: 0,
      rules_created: 0,
      synapses_pruned: report.synapses_pruned,
      synapses_reinforced: report.duplicates_removed,
      noise_discarded: report.corrupted_removed + report.orphan_nodes_pruned,
      positive_count: 0,
      negative_count: 0,
      details: report.details
    }
    await fetchGraph()
  } catch (err) {
    console.error(`Failed to optimize graph: ${err}`)
  } finally {
    isOptimizingGraph.value = false
  }
}

// Sync physics state with updated backend nodes
const syncPhysicsNodes = () => {
  const existingMap = new Map(nodesPhysics.map((p) => [p.id, p]))
  const rect = canvasContainerRef.value?.getBoundingClientRect()
  const width = (rect && rect.width > 0) ? rect.width : 800
  const height = (rect && rect.height > 0) ? rect.height : 600

  if (ambientParticles.length === 0) {
    initAmbientParticles(width, height)
  }

  nodesPhysics = graphData.value.nodes.map((node, i) => {
    const existing = existingMap.get(node.id)
    if (existing) {
      existing.label = node.label
      existing.type_flag = node.type_flag
      existing.valence = node.valence || 0
      existing.session_id = node.session_id || null
      return existing
    }
    const count = Math.max(1, graphData.value.nodes.length)
    const angle = (i / count) * Math.PI * 2
    const radius = 130 + (i % 4) * 60
    return {
      id: node.id,
      label: node.label,
      type_flag: node.type_flag,
      valence: node.valence || 0,
      session_id: node.session_id || null,
      x: width / 2 + Math.cos(angle) * radius,
      y: height / 2 + Math.sin(angle) * radius,
      vx: 0,
      vy: 0,
      radius: 20 + Math.min(6, node.label.length * 0.2),
      pulsePhase: Math.random() * Math.PI * 2,
      excitation: 0
    }
  })
}

// Physics simulation loop (Force-Directed with Spacing & Central Gravity)
const simulatePhysics = () => {
  if (!canvasContainerRef.value || !props.isActive || activeMemoryTab.value !== 'graph') return
  const rect = canvasContainerRef.value.getBoundingClientRect()
  if (rect.width <= 0 || rect.height <= 0) return

  const width = rect.width
  const height = rect.height
  const centerX = width / 2
  const centerY = height / 2

  const kRepulse = 5200
  const kSpring = 0.032
  const springLength = 175
  const damping = 0.80
  const centerPull = 0.005

  // 1. Repulsion between all nodes + Central Gravity Pull + Anti-Collision
  for (let i = 0; i < nodesPhysics.length; i++) {
    const a = nodesPhysics[i]
    if (!a) continue
    a.vx += (centerX - a.x) * centerPull
    a.vy += (centerY - a.y) * centerPull

    for (let j = i + 1; j < nodesPhysics.length; j++) {
      const b = nodesPhysics[j]
      if (!b) continue
      const dx = b.x - a.x
      const dy = b.y - a.y
      const dist = Math.hypot(dx, dy) || 1
      const minDist = a.radius + b.radius + 55

      if (dist < 500) {
        let force = Math.min(16, kRepulse / (dist * dist))
        if (dist < minDist) {
          // Extra push for collisions
          force += (minDist - dist) * 0.18
        }
        const fx = (dx / dist) * force
        const fy = (dy / dist) * force
        a.vx -= fx
        a.vy -= fy
        b.vx += fx
        b.vy += fy
      }
    }
  }

  // 2. Spring attraction along edges
  for (const edge of graphData.value.edges) {
    const a = nodesPhysics.find((n) => n.id === edge.source_id)
    const b = nodesPhysics.find((n) => n.id === edge.target_id)
    if (a && b) {
      const dx = b.x - a.x
      const dy = b.y - a.y
      const dist = Math.hypot(dx, dy) || 1
      const displacement = dist - springLength
      const force = displacement * kSpring * Math.min(2, Math.max(0.6, edge.weight || 1.0))
      const fx = (dx / dist) * force
      const fy = (dy / dist) * force
      a.vx += fx
      a.vy += fy
      b.vx -= fx
      b.vy -= fy
    }
  }

  // 3. Apply velocity and damping with speed bounds
  for (const node of nodesPhysics) {
    if (!node || node === dragTargetNode) continue
    node.vx = Math.max(-8, Math.min(8, node.vx * damping))
    node.vy = Math.max(-8, Math.min(8, node.vy * damping))
    node.x += node.vx
    node.y += node.vy
  }
}

// Render canvas with glowing bio-neurons, curved synaptic axons, action potential sparks, and floating labels
const drawCanvas = () => {
  const canvas = canvasRef.value
  if (!canvas || !canvasContainerRef.value || !props.isActive || activeMemoryTab.value !== 'graph') return
  const rect = canvasContainerRef.value.getBoundingClientRect()
  if (rect.width <= 0 || rect.height <= 0) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return
  const width = rect.width
  const height = rect.height
  const dpr = window.devicePixelRatio || 1
  const isDark = checkIsDark()

  // Set Retina transformation and clear buffer
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, width, height)

  // 0. Neural Space Background (Deep bio-space in dark mode, clean luminous bio-lab in light mode)
  if (isDark) {
    const bgGrad = ctx.createRadialGradient(width / 2, height / 2, 40, width / 2, height / 2, Math.max(width, height) * 0.85)
    bgGrad.addColorStop(0, '#0d1326')
    bgGrad.addColorStop(0.55, '#070914')
    bgGrad.addColorStop(1, '#04050b')
    ctx.fillStyle = bgGrad
    ctx.fillRect(0, 0, width, height)
  } else {
    const bgGrad = ctx.createRadialGradient(width / 2, height / 2, 40, width / 2, height / 2, Math.max(width, height) * 0.9)
    bgGrad.addColorStop(0, '#ffffff')
    bgGrad.addColorStop(0.55, '#f8fafc')
    bgGrad.addColorStop(1, '#edf2f7')
    ctx.fillStyle = bgGrad
    ctx.fillRect(0, 0, width, height)
  }

  ctx.save()
  ctx.translate(panX, panY)
  ctx.scale(zoom, zoom)

  // Draw subtle bio-matrix grid points in visible world bounds
  const startX = Math.floor((-panX / zoom - 200) / 45) * 45
  const endX = Math.ceil(((width - panX) / zoom + 200) / 45) * 45
  const startY = Math.floor((-panY / zoom - 200) / 45) * 45
  const endY = Math.ceil(((height - panY) / zoom + 200) / 45) * 45

  ctx.fillStyle = isDark ? 'rgba(99, 102, 241, 0.08)' : 'rgba(100, 116, 139, 0.22)'
  for (let x = startX; x < endX; x += 45) {
    for (let y = startY; y < endY; y += 45) {
      ctx.beginPath()
      ctx.arc(x, y, 1.0, 0, Math.PI * 2)
      ctx.fill()
    }
  }

  // Draw ambient drifting synaptic dust (neurotransmitters)
  for (const p of ambientParticles) {
    p.x += p.vx
    p.y += p.vy
    if (p.x < startX) p.x = endX
    if (p.x > endX) p.x = startX
    if (p.y < startY) p.y = endY
    if (p.y > endY) p.y = startY

    ctx.beginPath()
    ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2)
    ctx.fillStyle = p.color
    ctx.globalAlpha = p.alpha * (isDark ? (0.6 + Math.sin(animTime * 1.5 + p.phase) * 0.4) : 0.35)
    ctx.fill()
  }
  ctx.globalAlpha = 1.0

  // 1. Draw Synaptic Axons (Curved fibers, myelin sheath, action potentials)
  const filterActive = isFilterActive.value
  const matches = matchingNodeIds.value

  const connectedEdgeIndices = new Set<number>()
  const connectedNeighborNodeIds = new Set<number>()

  if (filterActive) {
    graphData.value.edges.forEach((edge, idx) => {
      if (matches.has(edge.source_id) || matches.has(edge.target_id)) {
        connectedEdgeIndices.add(idx)
        connectedNeighborNodeIds.add(edge.source_id)
        connectedNeighborNodeIds.add(edge.target_id)
      }
    })
  }

  // Highlight neighbors of hovered node
  const hoveredNeighbors = new Set<number>()
  if (hoveredNodeId.value !== null) {
    graphData.value.edges.forEach((edge) => {
      if (edge.source_id === hoveredNodeId.value) hoveredNeighbors.add(edge.target_id)
      if (edge.target_id === hoveredNodeId.value) hoveredNeighbors.add(edge.source_id)
    })
  }

  graphData.value.edges.forEach((edge, idx) => {
    const a = nodesPhysics.find((n) => n.id === edge.source_id)
    const b = nodesPhysics.find((n) => n.id === edge.target_id)
    if (!a || !b) return

    const isDirectMatchEdge = filterActive && connectedEdgeIndices.has(idx)
    const isSelected = selectedNode.value?.id === a.id || selectedNode.value?.id === b.id
    const isHovered = hoveredNodeId.value === a.id || hoveredNodeId.value === b.id
    const isHighlighted = isSelected || isHovered || isDirectMatchEdge

    // Subtle biological curvature
    const dx = b.x - a.x
    const dy = b.y - a.y
    const dist = Math.hypot(dx, dy) || 1
    const curveSign = (idx % 2 === 0) ? 1 : -1
    const curveFactor = Math.min(22, Math.max(6, dist * 0.08)) * curveSign
    const nx = -dy / dist
    const ny = dx / dist
    const cpX = (a.x + b.x) / 2 + nx * curveFactor
    const cpY = (a.y + b.y) / 2 + ny * curveFactor

    const isAvoid = edge.relation_type === 0xA4 || edge.relation_type === 'AvoidAction'
    const colA = getNodeColor(a.type_flag, a.valence)
    const colB = getNodeColor(b.type_flag, b.valence)

    if ((filterActive && !isDirectMatchEdge) || (hoveredNodeId.value !== null && !isHovered && !isSelected)) {
      ctx.globalAlpha = isDark ? 0.12 : 0.16
    } else {
      ctx.globalAlpha = 1.0
    }

    // 1) Outer Myelin Sheath / Synaptic Glow
    ctx.beginPath()
    ctx.moveTo(a.x, a.y)
    ctx.quadraticCurveTo(cpX, cpY, b.x, b.y)
    ctx.lineWidth = isHighlighted ? 6 : Math.min(5, Math.max(2.5, (edge.weight || 1) * 1.6))
    if (isAvoid) {
      ctx.strokeStyle = isHighlighted ? 'rgba(244, 63, 94, 0.45)' : 'rgba(244, 63, 94, 0.2)'
      ctx.setLineDash([6, 6])
    } else {
      ctx.setLineDash([])
      const gradGlow = ctx.createLinearGradient(a.x, a.y, b.x, b.y)
      const alphaHex = isDark ? (isHighlighted ? '60' : '25') : (isHighlighted ? '45' : '18')
      gradGlow.addColorStop(0, `${colA}${alphaHex}`)
      gradGlow.addColorStop(1, `${colB}${alphaHex}`)
      ctx.strokeStyle = gradGlow
    }
    ctx.stroke()

    // 2) Inner Neuro-Filament Core
    ctx.beginPath()
    ctx.moveTo(a.x, a.y)
    ctx.quadraticCurveTo(cpX, cpY, b.x, b.y)
    ctx.lineWidth = isHighlighted ? 2.5 : Math.min(2.2, Math.max(1.0, (edge.weight || 1) * 1.1))
    if (isAvoid) {
      ctx.strokeStyle = isHighlighted ? '#f43f5e' : (isDark ? 'rgba(244, 63, 94, 0.7)' : '#fb7185')
      ctx.setLineDash([4, 4])
    } else {
      ctx.setLineDash([])
      const gradCore = ctx.createLinearGradient(a.x, a.y, b.x, b.y)
      gradCore.addColorStop(0, isHighlighted ? (isDark ? '#ffffff' : colA) : colA)
      gradCore.addColorStop(1, isHighlighted ? colB : `${colB}dd`)
      ctx.strokeStyle = gradCore
    }
    ctx.stroke()
    ctx.setLineDash([])

    // 3) Synaptic Boutons (Terminal bulbs where axon connects to soma)
    const tBoutonA = 0.08
    const omtA = 1 - tBoutonA
    const bAx = omtA * omtA * a.x + 2 * omtA * tBoutonA * cpX + tBoutonA * tBoutonA * b.x
    const bAy = omtA * omtA * a.y + 2 * omtA * tBoutonA * cpY + tBoutonA * tBoutonA * b.y

    const tBoutonB = 0.92
    const omtB = 1 - tBoutonB
    const bBx = omtB * omtB * a.x + 2 * omtB * tBoutonB * cpX + tBoutonB * tBoutonB * b.x
    const bBy = omtB * omtB * a.y + 2 * omtB * tBoutonB * cpY + tBoutonB * tBoutonB * b.y

    ctx.beginPath()
    ctx.arc(bAx, bAy, 2.2, 0, Math.PI * 2)
    ctx.fillStyle = colA
    ctx.fill()

    ctx.beginPath()
    ctx.arc(bBx, bBy, 2.2, 0, Math.PI * 2)
    ctx.fillStyle = colB
    ctx.fill()

    // 4) Live Action Potential Sparks (Electric Impulses firing along the axon)
    const sparkSpeed = 0.35 + ((idx * 23) % 40) / 100
    const t1 = ((animTime * sparkSpeed) + (idx * 0.23)) % 1

    const omt1 = 1 - t1
    const spX = omt1 * omt1 * a.x + 2 * omt1 * t1 * cpX + t1 * t1 * b.x
    const spY = omt1 * omt1 * a.y + 2 * omt1 * t1 * cpY + t1 * t1 * b.y

    // Fading electric spark trail
    const tTail = Math.max(0, t1 - 0.06)
    const omtT = 1 - tTail
    const tailX = omtT * omtT * a.x + 2 * omtT * tTail * cpX + tTail * tTail * b.x
    const tailY = omtT * omtT * a.y + 2 * omtT * tTail * cpY + tTail * tTail * b.y

    ctx.beginPath()
    ctx.moveTo(tailX, tailY)
    ctx.lineTo(spX, spY)
    ctx.lineWidth = isHighlighted ? 3.5 : 2.5
    ctx.strokeStyle = isDark
      ? (isHighlighted ? '#ffffff' : (isAvoid ? '#fca5a5' : '#7dd3fc'))
      : (isHighlighted ? '#1e1b4b' : (isAvoid ? '#e11d48' : '#0284c7'))
    ctx.stroke()

    // Spark glowing head
    ctx.beginPath()
    ctx.arc(spX, spY, isHighlighted ? 3.2 : 2.4, 0, Math.PI * 2)
    ctx.fillStyle = isDark ? '#ffffff' : (isAvoid ? '#e11d48' : '#0284c7')
    ctx.shadowColor = isAvoid ? '#f43f5e' : (isDark ? '#38bdf8' : '#6366f1')
    ctx.shadowBlur = isDark ? 10 : 6
    ctx.fill()
    ctx.shadowBlur = 0

    // Action potential arrives at target node -> triggers micro excitation
    if (t1 > 0.94 && b.excitation !== undefined) {
      b.excitation = Math.max(b.excitation || 0, 0.4)
    }

    // 5) Relation badge (when highlighted or hovered)
    if (isHighlighted) {
      ctx.save()
      const midX = cpX
      const midY = cpY
      const relName = getRelationName(edge.relation_type)
      ctx.font = 'bold 9px monospace'
      const textWidth = ctx.measureText(relName).width
      const badgeW = textWidth + 14
      const badgeH = 18

      ctx.fillStyle = isDark ? 'rgba(8, 12, 24, 0.92)' : 'rgba(255, 255, 255, 0.96)'
      ctx.strokeStyle = isAvoid ? '#f43f5e' : (isDark ? '#6366f1' : '#4f46e5')
      ctx.lineWidth = 1.2
      ctx.shadowColor = isDark ? (isAvoid ? 'rgba(244, 63, 94, 0.5)' : 'rgba(99, 102, 241, 0.5)') : 'rgba(0, 0, 0, 0.12)'
      ctx.shadowBlur = isDark ? 8 : 4
      ctx.beginPath()
      ctx.roundRect(midX - badgeW / 2, midY - badgeH / 2, badgeW, badgeH, 5)
      ctx.fill()
      ctx.stroke()
      ctx.shadowBlur = 0

      ctx.fillStyle = isDark ? '#f8fafc' : '#1e293b'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText(relName, midX, midY)
      ctx.restore()
    }
  })

  // 2. Draw Biological Neurons (Soma, Perisomatic Halo, Receptors, Nucleus & Floating Labels)
  for (const node of nodesPhysics) {
    const isSelected = selectedNode.value?.id === node.id
    const isHovered = hoveredNodeId.value === node.id
    const isDirectMatch = filterActive && matches.has(node.id)
    const isNeighbor = filterActive && connectedNeighborNodeIds.has(node.id)
    const isHoverNeighbor = hoveredNodeId.value !== null && hoveredNeighbors.has(node.id)
    const isHighlighted = isSelected || isHovered || isDirectMatch || isHoverNeighbor

    // Excitation decay
    if (node.excitation) {
      node.excitation *= 0.93
      if (node.excitation < 0.02) node.excitation = 0
    }
    const excitation = node.excitation || 0

    if (filterActive) {
      if (isDirectMatch) ctx.globalAlpha = 1.0
      else if (isNeighbor) ctx.globalAlpha = 0.70
      else ctx.globalAlpha = isDark ? 0.12 : 0.18
    } else if (hoveredNodeId.value !== null && !isHighlighted) {
      ctx.globalAlpha = isDark ? 0.35 : 0.40
    } else {
      ctx.globalAlpha = 1.0
    }

    const color = getNodeColor(node.type_flag, node.valence)
    const pulse = Math.sin(animTime * 2.5 + (node.pulsePhase || 0)) * 2.0 + excitation * 6

    // 1) Outer Bioluminescent Aura (Perisomatic Halo)
    const auraRadius = node.radius + (isHighlighted ? 18 : 10) + pulse
    const auraGrad = ctx.createRadialGradient(node.x, node.y, node.radius * 0.7, node.x, node.y, auraRadius)
    if (isDark) {
      auraGrad.addColorStop(0, isHighlighted ? `${color}66` : `${color}33`)
      auraGrad.addColorStop(0.7, `${color}12`)
      auraGrad.addColorStop(1, `${color}00`)
    } else {
      auraGrad.addColorStop(0, isHighlighted ? `${color}40` : `${color}20`)
      auraGrad.addColorStop(0.7, `${color}08`)
      auraGrad.addColorStop(1, `${color}00`)
    }
    ctx.beginPath()
    ctx.arc(node.x, node.y, auraRadius, 0, Math.PI * 2)
    ctx.fillStyle = auraGrad
    ctx.fill()

    // 2) Cytoplasm / Cell Body
    const cytoGrad = ctx.createRadialGradient(node.x - node.radius * 0.25, node.y - node.radius * 0.25, 2, node.x, node.y, node.radius)
    if (isDark) {
      cytoGrad.addColorStop(0, `${color}99`)
      cytoGrad.addColorStop(0.45, `${color}40`)
      cytoGrad.addColorStop(1, '#090d1a')
    } else {
      cytoGrad.addColorStop(0, '#ffffff')
      cytoGrad.addColorStop(0.5, `${color}25`)
      cytoGrad.addColorStop(1, `${color}55`)
    }
    ctx.beginPath()
    ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2)
    ctx.fillStyle = cytoGrad
    ctx.fill()

    // 3) Cell Membrane (Glow boundary)
    ctx.beginPath()
    ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2)
    ctx.strokeStyle = isHighlighted ? (isDark ? '#ffffff' : color) : color
    ctx.lineWidth = isHighlighted ? (isDark ? 2.8 : 3.0) : 2.0
    ctx.shadowColor = isDark ? (isHighlighted ? '#ffffff' : color) : `${color}80`
    ctx.shadowBlur = isDark ? (isHighlighted ? 16 : 8) : (isHighlighted ? 8 : 3)
    ctx.stroke()
    ctx.shadowBlur = 0

    // 4) Micro Dendritic Receptors (6 tiny sensory buds on the membrane)
    for (let k = 0; k < 6; k++) {
      const theta = (k / 6) * Math.PI * 2 + animTime * 0.08
      const rx = node.x + Math.cos(theta) * (node.radius + 1.2)
      const ry = node.y + Math.sin(theta) * (node.radius + 1.2)
      ctx.beginPath()
      ctx.arc(rx, ry, 1.2, 0, Math.PI * 2)
      ctx.fillStyle = isDark ? (isHighlighted ? '#ffffff' : `${color}dd`) : color
      ctx.fill()
    }

    // 5) Nucleus (Central Glowing Core)
    const nucRadius = 4.5 + excitation * 2
    const nucGrad = ctx.createRadialGradient(node.x, node.y, 0, node.x, node.y, nucRadius)
    nucGrad.addColorStop(0, '#ffffff')
    nucGrad.addColorStop(0.6, color)
    nucGrad.addColorStop(1, `${color}00`)
    ctx.beginPath()
    ctx.arc(node.x, node.y, nucRadius, 0, Math.PI * 2)
    ctx.fillStyle = nucGrad
    ctx.fill()

    // 6) Valence Indicator (Top-right of soma)
    if (node.valence !== 0) {
      const vx = node.x + node.radius * 0.72
      const vy = node.y - node.radius * 0.72
      ctx.beginPath()
      ctx.arc(vx, vy, 6, 0, Math.PI * 2)
      ctx.fillStyle = node.valence === 1 ? '#10b981' : '#f43f5e'
      ctx.shadowColor = node.valence === 1 ? '#10b981' : '#f43f5e'
      ctx.shadowBlur = isDark ? 6 : 3
      ctx.fill()
      ctx.shadowBlur = 0
      ctx.strokeStyle = isDark ? '#060812' : '#ffffff'
      ctx.lineWidth = 1.5
      ctx.stroke()

      ctx.fillStyle = '#ffffff'
      ctx.font = 'bold 8px monospace'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText(node.valence === 1 ? '+' : '!', vx, vy)
    }

    // 7) Private Session Lock Indicator (Top-left of soma)
    if (node.session_id) {
      const lx = node.x - node.radius * 0.72
      const ly = node.y - node.radius * 0.72
      ctx.beginPath()
      ctx.arc(lx, ly, 5.5, 0, Math.PI * 2)
      ctx.fillStyle = '#f59e0b'
      ctx.shadowColor = '#f59e0b'
      ctx.shadowBlur = isDark ? 6 : 3
      ctx.fill()
      ctx.shadowBlur = 0
      ctx.strokeStyle = isDark ? '#060812' : '#ffffff'
      ctx.lineWidth = 1.5
      ctx.stroke()
    }

    // 8) Floating Label Capsule (Placed below the neuron soma)
    const labelY = node.y + node.radius + 15
    const rawLabel = node.session_id ? `🔒 ${node.label}` : node.label
    const displayLabel = rawLabel.length > 28 ? `${rawLabel.slice(0, 26)}...` : rawLabel
    ctx.font = isHighlighted ? 'bold 10.5px Inter, -apple-system, sans-serif' : '500 10px Inter, -apple-system, sans-serif'
    const textWidth = ctx.measureText(displayLabel).width
    const pillW = textWidth + 14
    const pillH = 19

    if (isDark) {
      ctx.fillStyle = isHighlighted ? 'rgba(16, 22, 44, 0.95)' : 'rgba(9, 13, 26, 0.88)'
      ctx.strokeStyle = isHighlighted ? '#ffffff' : `${color}66`
      ctx.shadowColor = isHighlighted ? 'rgba(0,0,0,0.8)' : 'rgba(0,0,0,0.4)'
    } else {
      ctx.fillStyle = isHighlighted ? '#ffffff' : 'rgba(255, 255, 255, 0.95)'
      ctx.strokeStyle = isHighlighted ? color : `${color}70`
      ctx.shadowColor = isHighlighted ? 'rgba(0,0,0,0.18)' : 'rgba(0,0,0,0.08)'
    }
    ctx.lineWidth = isHighlighted ? 1.4 : 1
    ctx.shadowBlur = isHighlighted ? 8 : 4
    ctx.beginPath()
    ctx.roundRect(node.x - pillW / 2, labelY - pillH / 2, pillW, pillH, 6)
    ctx.fill()
    ctx.stroke()
    ctx.shadowBlur = 0

    ctx.fillStyle = isDark
      ? (isHighlighted ? '#ffffff' : '#f1f5f9')
      : (isHighlighted ? '#0f172a' : '#1e293b')
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText(displayLabel, node.x, labelY)
  }

  ctx.globalAlpha = 1.0
  ctx.restore()
}

const renderLoop = () => {
  if (props.isActive && activeMemoryTab.value === 'graph') {
    animTime += 0.02
    simulatePhysics()
    drawCanvas()
  }
  animationFrameId = requestAnimationFrame(renderLoop)
}

// Mouse events for canvas interaction
const getCanvasCoords = (e: MouseEvent) => {
  if (!canvasRef.value) return { worldX: 0, worldY: 0, screenX: 0, screenY: 0 }
  const rect = canvasRef.value.getBoundingClientRect()
  const screenX = e.clientX - rect.left
  const screenY = e.clientY - rect.top
  const worldX = (screenX - panX) / zoom
  const worldY = (screenY - panY) / zoom
  return { worldX, worldY, screenX, screenY }
}

const handleMouseDown = (e: MouseEvent) => {
  const { worldX, worldY } = getCanvasCoords(e)
  const clickedNode = nodesPhysics.find((n) => {
    const distSoma = Math.hypot(n.x - worldX, n.y - worldY)
    if (distSoma <= n.radius + 8) return true
    const labelY = n.y + n.radius + 15
    return Math.abs(worldX - n.x) <= 60 && Math.abs(worldY - labelY) <= 12
  })

  if (clickedNode) {
    dragTargetNode = clickedNode
    clickedNode.excitation = 1.0
    selectedNode.value = graphData.value.nodes.find((n) => n.id === clickedNode.id) || null
    if (canvasRef.value) canvasRef.value.style.cursor = 'grabbing'
  } else {
    isDragging = true
    dragStartPos = { x: e.clientX - panX, y: e.clientY - panY }
    if (canvasRef.value) canvasRef.value.style.cursor = 'grabbing'
  }
}

const handleMouseMove = (e: MouseEvent) => {
  const { worldX, worldY } = getCanvasCoords(e)
  if (dragTargetNode) {
    dragTargetNode.x = worldX
    dragTargetNode.y = worldY
    dragTargetNode.vx = 0
    dragTargetNode.vy = 0
  } else if (isDragging) {
    panX = e.clientX - dragStartPos.x
    panY = e.clientY - dragStartPos.y
  } else {
    const hitNode = nodesPhysics.find((n) => {
      const distSoma = Math.hypot(n.x - worldX, n.y - worldY)
      if (distSoma <= n.radius + 6) return true
      const labelY = n.y + n.radius + 15
      return Math.abs(worldX - n.x) <= 60 && Math.abs(worldY - labelY) <= 12
    })
    hoveredNodeId.value = hitNode ? hitNode.id : null
    if (canvasRef.value) {
      canvasRef.value.style.cursor = hitNode ? 'pointer' : 'grab'
    }
  }
}

const handleMouseUp = () => {
  dragTargetNode = null
  isDragging = false
  if (canvasRef.value) {
    canvasRef.value.style.cursor = hoveredNodeId.value ? 'pointer' : 'grab'
  }
}

const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9
  zoom = Math.min(3, Math.max(0.3, zoom * zoomFactor))
}

const centerGraph = () => {
  zoom = 1
  panX = 0
  panY = 0
}

// User Actions
const handleTestActivation = async () => {
  if (!activationAnchor.value.trim()) return
  isActivating.value = true
  try {
    const res = await invoke<any>('memory_query', {
      startLabel: activationAnchor.value.trim(),
      maxDepth: 3
    })
    if (res) {
      synthesizedContext.value = res.llm_context || t('memory.no_association_found')
      stats.value = res.stats || stats.value
      if (res.paths && res.paths.length > 0) {
        for (const path of res.paths) {
          for (const step of path.steps || []) {
            const pNode = nodesPhysics.find(
              (n) => n.label.toLowerCase() === (step.node_label || '').toLowerCase()
            )
            if (pNode) {
              pNode.excitation = 1.0
            }
          }
        }
        if (res.paths[0].steps && res.paths[0].steps.length > 0) {
          const rootLabel = res.paths[0].steps[0].node_label
          const match = graphData.value.nodes.find(
            (n) => n.label.toLowerCase() === rootLabel.toLowerCase()
          )
          if (match) {
            selectedNode.value = match
          }
        }
      } else {
        const match = graphData.value.nodes.find(
          (n) => n.label.toLowerCase() === activationAnchor.value.trim().toLowerCase()
        )
        if (match) {
          selectedNode.value = match
          const pNode = nodesPhysics.find((n) => n.id === match.id)
          if (pNode) pNode.excitation = 1.0
        }
      }
    }
  } catch (err) {
    synthesizedContext.value = `Erro: ${err}`
  } finally {
    isActivating.value = false
  }
}

const openLearnModal = () => {
  newFactText.value = ''
  newFactScope.value = 'global'
  newFactSessionId.value = ''
  learnFactError.value = null
  isLearnModalOpen.value = true
}

const handleSubmitFact = async () => {
  if (!newFactText.value.trim() || isSubmittingFact.value) return
  isSubmittingFact.value = true
  learnFactError.value = null
  try {
    const sessionId = newFactScope.value === 'private'
      ? (newFactSessionId.value.trim() || 'sessao-manual')
      : null

    const facts = await invoke<any[]>('memory_learn_text', {
      text: newFactText.value.trim(),
      sessionId
    })
    if (facts && facts.length > 0) {
      newFactText.value = ''
      isLearnModalOpen.value = false
      await fetchGraph()
    } else {
      learnFactError.value = t('memory.learn_fact_empty_error')
    }
  } catch (err) {
    console.error(`Failed to memorize fact: ${err}`)
    learnFactError.value = `${t('common.error')}: ${err}`
  } finally {
    isSubmittingFact.value = false
  }
}

const handleToggleNodeScope = async (node: GraphNode) => {
  if (!node) return
  const targetScope = node.session_id ? null : 'sessao-principal'
  try {
    await invoke('memory_update_node_scope', {
      id: node.id,
      sessionId: targetScope
    })
    node.session_id = targetScope
    if (selectedNode.value && selectedNode.value.id === node.id) {
      selectedNode.value.session_id = targetScope
    }
    await fetchGraph()
  } catch (err) {
    console.error('Failed to update node scope:', err)
  }
}

const handleExportGraph = async (format: string = 'json') => {
  showExportMenu.value = false
  isExporting.value = true
  try {
    const data = await invoke<string>('memory_export_graph', { format })
    const now = new Date().toISOString().slice(0, 10)
    const isMd = format === 'markdown' || format === 'md'
    const ext = isMd ? 'md' : 'json'
    await saveTextFile({
      content: data,
      filename: `atena-grafo-memoria-${now}.${ext}`,
      title: t('memory.export_dialog_title', { ext: ext.toUpperCase() }),
      filters: [
        { name: isMd ? 'Markdown (*.md)' : 'JSON (*.json)', extensions: [ext] },
        { name: t('memory.all_files_filter'), extensions: ['*'] }
      ]
    })
  } catch (err) {
    console.error('Failed to export graph:', err)
  } finally {
    isExporting.value = false
  }
}

const handleReinforceSelected = async () => {
  if (!selectedNode.value) return
  for (const edge of selectedNodeEdges.value) {
    try {
      await invoke('memory_reinforce', {
        sourceId: edge.source_id,
        targetId: edge.target_id
      })
    } catch (e) {
      console.error(e)
    }
  }
  await fetchGraph()
}

const handleConfirmDeleteNode = async () => {
  if (!selectedNode.value) return
  try {
    await invoke('memory_delete_node', { id: selectedNode.value.id })
    selectedNode.value = null
    isConfirmDeleteNodeModalOpen.value = false
    await fetchGraph()
  } catch (err) {
    console.error(`Failed to delete node: ${err}`)
  }
}

const handleApplyDecay = async () => {
  try {
    await invoke('memory_apply_decay', { decayFactor: 0.05 })
    await fetchGraph()
  } catch (err) {
    console.error(err)
  }
}

const handleConfirmClearAll = async () => {
  try {
    await invoke('memory_clear_all')
    selectedNode.value = null
    synthesizedContext.value = ''
    vigiliaBuffer.value = []
    isConfirmResetModalOpen.value = false
    await fetchGraph()
  } catch (err) {
    console.error(`Failed to clear memory: ${err}`)
  }
}

// Auto-resize canvas with Retina High-DPI support
const handleResize = () => {
  const canvas = canvasRef.value
  const container = canvasContainerRef.value
  if (!canvas || !container) return

  const rect = container.getBoundingClientRect()
  if (rect.width <= 0 || rect.height <= 0) return

  const dpr = window.devicePixelRatio || 1
  canvas.width = Math.floor(rect.width * dpr)
  canvas.height = Math.floor(rect.height * dpr)
  canvas.style.width = `${rect.width}px`
  canvas.style.height = `${rect.height}px`

  const ctx = canvas.getContext('2d')
  if (ctx) {
    ctx.resetTransform?.()
    ctx.scale(dpr, dpr)
  }
}

// Watcher to adjust canvas when memory screen and facts tab become visible again
watch(
  () => [props.isActive, activeMemoryTab.value] as const,
  ([isScreenActive, tab]) => {
    if (isScreenActive && tab === 'graph') {
      nextTick(() => {
        handleResize()
        if (nodesPhysics.length === 0 && graphData.value.nodes.length > 0) {
          syncPhysicsNodes()
        }
      })
    }
  }
)

let autoRefreshInterval: any = null

onMounted(async () => {
  nextTick(() => {
    handleResize()
  })
  if (canvasContainerRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0 && entry.contentRect.height > 0) {
          handleResize()
        }
      }
    })
    resizeObserver.observe(canvasContainerRef.value)
  }
  window.addEventListener('resize', handleResize)
  await Promise.all([
    fetchGraph(),
    fetchSkills(),
    fetchEpisodes()
  ])
  renderLoop()

  // Polling to keep real-time sync with ongoing chat learnings, vigilia & episodes
  autoRefreshInterval = setInterval(() => {
    if (props.isActive) {
      fetchGraph()
      fetchSkills()
      fetchEpisodes()
    }
  }, 4000)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  if (resizeObserver) resizeObserver.disconnect()
  if (animationFrameId) cancelAnimationFrame(animationFrameId)
  if (autoRefreshInterval) clearInterval(autoRefreshInterval)
})
</script>
