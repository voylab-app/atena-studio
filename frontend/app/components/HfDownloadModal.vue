<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 md:p-6 bg-black/80 backdrop-blur-md animate-fade-in select-none">
    <!-- Modal Card Container -->
    <div
      class="w-full max-w-6xl h-[92vh] max-h-[920px] bg-[#0c0f18] border border-[#1f263d] rounded-3xl shadow-2xl flex flex-col overflow-hidden text-slate-100 animate-scale-up"
      @click.stop
    >
      <!-- MODAL TOP BAR / HEADER -->
      <div class="px-6 py-4 bg-[#101424] border-b border-[#1c2237] flex items-center justify-between flex-shrink-0 gap-4">
        <!-- Title & Subtitle -->
        <div class="flex items-center gap-3.5">
          <div class="w-10 h-10 rounded-2xl bg-gradient-to-tr from-amber-500/20 via-orange-500/20 to-indigo-500/20 border border-amber-500/30 flex items-center justify-center text-amber-400 shadow-md">
            <span class="text-xl">🤗</span>
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-base font-bold text-slate-100 tracking-tight">{{ $t('modals.hf.title') }}</h3>
              <span class="text-[10.5px] px-2 py-0.5 rounded-full bg-indigo-500/15 text-indigo-300 border border-indigo-500/30 font-semibold font-mono">
                {{ supportsMlx ? 'GGUF & MLX' : 'GGUF' }}
              </span>
            </div>
            <p class="text-xs text-slate-400 mt-0.5">
              {{ $t('modals.hf.subtitle') }}
            </p>
          </div>
        </div>

        <!-- Navigation Tabs & Actions -->
        <div class="flex items-center gap-3">
          <!-- Folder Quick Bar -->
          <div class="hidden md:flex items-center gap-2 bg-[#141828] px-3 py-1.5 rounded-xl border border-[#20273f] text-xs">
            <Folder class="w-3.5 h-3.5 text-indigo-400 flex-shrink-0" />
            <span class="text-slate-400 text-[11px]">{{ $t('modals.hf.save_label') }}</span>
            <select
              v-if="availableDirs.length > 1"
              :value="currentDestination"
              @change="handleDestinationSelect(($event.target as HTMLSelectElement).value)"
              class="bg-[#0e111c] border border-[#22293e] text-slate-200 text-[11px] font-mono rounded-lg px-2 py-0.5 focus:outline-none focus:border-indigo-500 max-w-[170px] lg:max-w-[240px] truncate cursor-pointer"
              :title="$t('modals.hf.save_tooltip')"
            >
              <option v-for="d in availableDirs" :key="d" :value="d">
                {{ d }}
              </option>
            </select>
            <span v-else class="font-mono text-[11px] text-slate-300 truncate max-w-[140px] lg:max-w-[200px]" :title="currentDestination">
              {{ currentDestination || $t('modals.hf.not_selected') }}
            </span>
            <button
              @click="$emit('selectFolder')"
              class="px-2 py-0.5 rounded-md bg-[#1d233a] hover:bg-[#272f4e] border border-[#2b3554] text-[10.5px] font-semibold text-indigo-300 hover:text-white transition-all cursor-pointer"
              :title="$t('modals.hf.change_folder_tooltip')"
            >
              {{ $t('modals.hf.change_folder') }}
            </button>
          </div>

          <!-- Tab Selector -->
          <div class="flex items-center gap-1 bg-[#141828] p-1 rounded-xl border border-[#20273f]">
            <button
              @click="activeTab = 'explore'"
              :class="[
                'px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer',
                activeTab === 'explore'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-slate-200'
              ]"
            >
              <Search class="w-3.5 h-3.5" />
              <span>{{ $t('modals.hf.tab_explore') }}</span>
            </button>

            <button
              @click="activeTab = 'downloads'"
              :class="[
                'px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer relative',
                activeTab === 'downloads'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-slate-200'
              ]"
            >
              <Download class="w-3.5 h-3.5" />
              <span>{{ $t('modals.hf.tab_downloads') }}</span>
              <span
                v-if="runningDownloadsCount > 0"
                class="px-1.5 py-0.2 bg-amber-400 text-black font-extrabold text-[10px] rounded-full animate-pulse"
              >
                {{ runningDownloadsCount }}
              </span>
            </button>
          </div>

          <!-- Close Modal -->
          <button
            @click="$emit('close')"
            class="p-2 rounded-xl bg-[#141828] hover:bg-[#20273f] text-slate-400 hover:text-white border border-[#20273f] transition-all cursor-pointer"
            :title="$t('modals.hf.close_tooltip')"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- MAIN TAB 1: EXPLORE & SEARCH (2-COLUMN LM-STUDIO LAYOUT) -->
      <div v-if="activeTab === 'explore'" class="flex-1 flex overflow-hidden">
        
        <!-- LEFT COLUMN: Search & Models List -->
        <div class="w-full md:w-[380px] lg:w-[420px] flex-shrink-0 border-r border-[#1a2035] flex flex-col bg-[#0d101a] overflow-hidden">
          
          <!-- Top Controls: Search Bar -->
          <div class="p-3.5 bg-[#0e121e] border-b border-[#191f33] space-y-2.5 flex-shrink-0">
            <!-- Search Input -->
            <div class="relative">
              <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                v-model="searchQuery"
                @input="handleSearchInput"
                :placeholder="$t('modals.hf.search_placeholder')"
                class="w-full bg-[#141828] border border-[#222941] rounded-xl pl-9 pr-8 py-2 text-xs text-slate-100 placeholder-slate-400 outline-none focus:border-indigo-500 transition-all font-sans"
              />
              <button
                v-if="searchQuery"
                @click="searchQuery = ''; handleSearch()"
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white p-0.5 cursor-pointer"
              >
                <X class="w-3.5 h-3.5" />
              </button>
            </div>

            <!-- Filters Bar: Format & Sort & Incompatibility -->
            <div class="flex items-center justify-between gap-1.5 flex-wrap text-xs">
              <!-- Format Filter Pills -->
              <div class="flex items-center gap-1 bg-[#141828] p-0.5 rounded-lg border border-[#20273f]">
                <button
                  v-for="filter in formatFilters"
                  :key="filter.id"
                  @click="selectedFormat = filter.id; handleSearch()"
                  :class="[
                    'px-2 py-0.5 rounded-md text-[11px] font-semibold transition-all cursor-pointer',
                    selectedFormat === filter.id
                      ? 'bg-indigo-600 text-white shadow-xs'
                      : 'text-slate-400 hover:text-slate-200'
                  ]"
                >
                  {{ filter.label }}
                </button>
              </div>

              <!-- Sort Dropdown -->
              <div class="flex items-center gap-1 bg-[#141828] px-2 py-1 rounded-lg border border-[#20273f]">
                <ArrowUpDown class="w-3 h-3 text-indigo-400" />
                <select
                  v-model="selectedSort"
                  @change="handleSearch"
                  class="bg-transparent text-[11px] font-semibold text-slate-300 outline-none cursor-pointer pr-1 !bg-none"
                >
                  <option value="trendingScore" class="bg-[#0e121e] text-slate-200">Trending</option>
                  <option value="downloads" class="bg-[#0e121e] text-slate-200">{{ $t('modals.hf.sort_downloads') }}</option>
                  <option value="likes" class="bg-[#0e121e] text-slate-200">{{ $t('modals.hf.sort_likes') }}</option>
                  <option value="lastModified" class="bg-[#0e121e] text-slate-200">{{ $t('modals.hf.sort_recent') }}</option>
                </select>
              </div>
            </div>

            <!-- Destaques / Staff Picks Row -->
            <div class="flex items-center justify-between gap-2 pt-0.5">
              <div class="flex items-center gap-1 overflow-x-auto pb-0.5 no-scrollbar flex-1">
                <span class="text-[10.5px] text-slate-400 font-semibold flex items-center gap-1 flex-shrink-0">
                  <Sparkles class="w-3 h-3 text-amber-400" />
                  <span>{{ $t('modals.hf.highlights') }}</span>
                </span>
                <button
                  v-for="preset in curatedPresets"
                  :key="preset.id"
                  @click="applyPreset(preset)"
                  :class="[
                    'px-2 py-0.5 rounded-md border text-[10.5px] font-medium transition-all flex items-center gap-1 flex-shrink-0 cursor-pointer',
                    selectedModelDetail?.id === preset.repo
                      ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200'
                      : 'bg-[#141828] hover:bg-[#1a2034] border-[#222941] text-slate-300 hover:text-white'
                  ]"
                >
                  <span>{{ preset.label }}</span>
                </button>
              </div>

              <!-- Incompatibility filter toggle -->
              <button
                type="button"
                @click="hideIncompatible = !hideIncompatible"
                :class="[
                  'flex items-center gap-1 px-2 py-0.5 rounded-md border text-[10.5px] font-semibold transition-all cursor-pointer select-none flex-shrink-0',
                  hideIncompatible
                    ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/40 shadow-xs'
                    : 'bg-[#141828] text-slate-400 border-[#20273f] hover:text-slate-200'
                ]"
                :title="$t('modals.hf.hide_incompatible_tooltip', { ram: props.hardware?.total_ram_gb || 16 })"
              >
                <ShieldCheck class="w-3 h-3" :class="hideIncompatible ? 'text-emerald-400' : 'text-slate-500'" />
                <span>{{ props.hardware?.total_ram_gb || 16 }}G</span>
                <span
                  v-if="hideIncompatible && hiddenIncompatibleCount > 0"
                  class="px-1 rounded-full bg-emerald-500/25 text-emerald-300 text-[9px] font-mono font-bold"
                >
                  -{{ hiddenIncompatibleCount }}
                </span>
              </button>
            </div>
          </div>

          <!-- Models List (Master) -->
          <div class="flex-1 overflow-y-auto p-2.5 space-y-1.5">
            <!-- Searching spinner state -->
            <div v-if="isSearching" class="h-48 flex flex-col items-center justify-center space-y-2 text-slate-400">
              <Loader2 class="w-6 h-6 text-indigo-400 animate-spin" />
              <span class="text-xs">{{ $t('modals.hf.searching') }}</span>
            </div>

            <!-- Empty Results State -->
            <div v-else-if="displayedResults.length === 0" class="h-48 flex flex-col items-center justify-center text-center p-4 text-slate-400 space-y-1.5">
              <Box class="w-8 h-8 text-slate-500" />
              <p class="text-xs font-semibold text-slate-300">{{ $t('modals.hf.no_models_found') }}</p>
              <p v-if="hiddenIncompatibleCount > 0" class="text-[11px] text-slate-400">
                {{ $t('modals.hf.hidden_incompatible_count', { count: hiddenIncompatibleCount }) }}
                <button @click="hideIncompatible = false" class="text-indigo-400 underline font-semibold ml-1 cursor-pointer">
                  {{ $t('modals.hf.show_all') }}
                </button>
              </p>
            </div>

            <!-- Model Cards -->
            <div
              v-else
              v-for="model in displayedResults"
              :key="model.id"
              @click="inspectModel(model.id)"
              :class="[
                'p-3 rounded-2xl border transition-all cursor-pointer flex items-start gap-3 group relative select-none',
                selectedModelDetail?.id === model.id
                  ? 'bg-indigo-600 text-white border-indigo-500 shadow-md shadow-indigo-600/20'
                  : 'bg-[#111524] hover:bg-[#151a2d] border-[#1c2237] hover:border-[#2b3552] text-slate-200'
              ]"
            >
              <!-- Brand / Provider Icon -->
              <div
                :class="[
                  'model-avatar-badge w-9 h-9 rounded-xl flex items-center justify-center shrink-0 shadow-sm border overflow-hidden p-1 transition-all',
                  getModelAvatar(model)
                    ? 'bg-white border-slate-200 dark:border-slate-300/20 shadow-sm'
                    : selectedModelDetail?.id === model.id
                      ? 'bg-white border-white shadow-md ring-2 ring-white/30'
                      : 'bg-slate-100 dark:bg-[#161b2e] border-slate-200 dark:border-[#252d48]'
                ]"
              >
                <!-- Authentic image avatar if available -->
                <img
                  v-if="getModelAvatar(model)"
                  :src="getModelAvatar(model)"
                  :alt="model.author || 'Avatar'"
                  class="w-full h-full object-contain rounded-lg transition-opacity duration-300"
                  loading="lazy"
                  @error="markAvatarFailed(getModelAvatar(model))"
                />
                <!-- Initial Letter Fallback -->
                <span
                  v-else
                  :class="[
                    'text-xs font-bold uppercase',
                    selectedModelDetail?.id === model.id ? 'text-indigo-600' : (model.is_mlx ? 'text-purple-500 dark:text-purple-300' : 'text-cyan-500 dark:text-cyan-300')
                  ]"
                >
                  {{ (model.author || model.id.split('/')[0] || '?').substring(0, 2) }}
                </span>
              </div>

              <!-- Model Details in Card -->
              <div class="min-w-0 flex-1 space-y-1">
                <!-- Title & Verified Badge -->
                <div class="flex items-center justify-between gap-1.5">
                  <div class="flex items-center gap-1.5 truncate">
                    <span
                      :class="[
                        'text-xs font-bold truncate',
                        selectedModelDetail?.id === model.id ? 'text-white' : 'text-slate-100 group-hover:text-indigo-300'
                      ]"
                      :title="model.id"
                    >
                      {{ getFriendlyModelName(model.id, model) }}
                    </span>
                    <CheckCircle2
                      class="w-3 h-3 flex-shrink-0"
                      :class="selectedModelDetail?.id === model.id ? 'text-white' : 'text-indigo-400'"
                    />
                  </div>

                  <!-- Installed Badge if model is in local catalog -->
                  <span
                    v-if="findLocalModel(model)"
                    :class="[
                      'px-1.5 py-0.2 rounded text-[9.5px] font-bold flex-shrink-0 flex items-center gap-0.5',
                      selectedModelDetail?.id === model.id
                        ? 'bg-emerald-400 text-slate-950 shadow-sm'
                        : 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
                    ]"
                    :title="$t('modals.hf.in_catalog_tooltip')"
                  >
                    <Check class="w-2.5 h-2.5 stroke-[3]" />
                    <span>{{ $t('modals.hf.in_catalog_badge') }}</span>
                  </span>

                  <!-- Param Size Badge -->
                  <span
                    v-if="model.param_size"
                    :class="[
                      'px-1.5 py-0.2 rounded text-[9.5px] font-mono font-bold flex-shrink-0',
                      selectedModelDetail?.id === model.id
                        ? 'bg-white/20 text-white'
                        : 'bg-teal-500/15 text-teal-300 border border-teal-500/30'
                    ]"
                  >
                    {{ model.param_size }}
                  </span>
                </div>

                <!-- Subtitle / Author & Format -->
                <p
                  :class="[
                    'text-[10.5px] truncate',
                    selectedModelDetail?.id === model.id ? 'text-white/90' : 'text-slate-400'
                  ]"
                >
                  {{ $t('modals.hf.by_author', { author: model.author || model.id.split('/')[0] }) }}
                  <span :class="getModelFormatClass(model, selectedModelDetail?.id === model.id)">{{ getModelFormatLabel(model) }}</span>
                  <template v-if="getModelCardSize(model)">
                    • <span class="font-mono">{{ getModelCardSize(model) }}</span>
                  </template>
                </p>

                <!-- Capability Badges + Metrics Row -->
                <div class="flex items-center justify-between gap-2 pt-0.5">
                  <!-- Capability Icons -->
                  <div class="flex items-center gap-1.5">
                    <span
                      v-if="model.supports_vision"
                      :title="$t('modals.hf.vision_support_tooltip')"
                      :class="selectedModelDetail?.id === model.id ? 'text-cyan-200' : 'text-cyan-400'"
                    >
                      <Eye class="w-3.5 h-3.5" />
                    </span>
                    <span
                      v-if="model.supports_tools"
                      :title="$t('modals.hf.tools_mcp_support')"
                      :class="selectedModelDetail?.id === model.id ? 'text-emerald-200' : 'text-emerald-400'"
                    >
                      <Wrench class="w-3.5 h-3.5" />
                    </span>
                    <span
                      v-if="model.supports_thinking"
                      :title="$t('modals.hf.thinking_model_tooltip')"
                      :class="selectedModelDetail?.id === model.id ? 'text-purple-200' : 'text-purple-400'"
                    >
                      <Brain class="w-3.5 h-3.5" />
                    </span>
                  </div>

                  <!-- Downloads & Likes -->
                  <div
                    :class="[
                      'flex items-center gap-2 text-[10px] font-mono',
                      selectedModelDetail?.id === model.id ? 'text-white/90' : 'text-slate-400'
                    ]"
                  >
                    <span class="flex items-center gap-0.5">
                      <Download class="w-2.5 h-2.5" />
                      {{ formatNumber(model.downloads) }}
                    </span>
                    <span class="flex items-center gap-0.5">
                      <span>❤️</span>
                      {{ formatNumber(model.likes) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- RIGHT COLUMN: Model Detail Inspector & Download Action Pane -->
        <div class="flex-1 bg-[#0a0d16] flex flex-col overflow-y-auto p-5 lg:p-7 space-y-6">
          
          <!-- State: Loading Model Detail -->
          <div v-if="isLoadingDetail" class="h-full flex flex-col items-center justify-center space-y-3 text-slate-400">
            <Loader2 class="w-8 h-8 text-indigo-400 animate-spin" />
            <p class="text-xs font-semibold text-slate-300">{{ $t('modals.hf.loading_details') }}</p>
          </div>

          <!-- State: Model Selected -->
          <template v-else-if="selectedModelDetail">
            
            <!-- Model Header & Actions Bar -->
            <div class="flex items-start justify-between gap-4 flex-wrap">
              <div class="flex items-start gap-3.5 min-w-0 max-w-2xl">
                <div
                  :class="[
                    'w-12 h-12 rounded-2xl flex items-center justify-center shrink-0 shadow-md border overflow-hidden p-1.5',
                    getModelAvatar(selectedModelDetail) ? 'bg-white' : 'bg-white dark:bg-[#161b2e]',
                    selectedModelDetail.is_mlx
                      ? 'border-purple-500/25'
                      : 'border-cyan-500/25'
                  ]"
                >
                  <img
                    v-if="getModelAvatar(selectedModelDetail)"
                    :src="getModelAvatar(selectedModelDetail)"
                    :alt="selectedModelDetail.author || 'Logo'"
                    class="w-full h-full object-contain rounded-xl transition-opacity duration-300"
                    loading="lazy"
                    @error="markAvatarFailed(getModelAvatar(selectedModelDetail))"
                  />
                  <span
                    v-else
                    :class="[
                      'text-sm font-bold uppercase',
                      selectedModelDetail.is_mlx ? 'text-purple-600 dark:text-purple-300' : 'text-cyan-600 dark:text-cyan-300'
                    ]"
                  >
                    {{ getAuthorInitials(selectedModelDetail) }}
                  </span>
                </div>

              <div class="space-y-1.5 min-w-0">
                <!-- Repo ID & Verified Badge -->
                <div class="flex items-center gap-2.5 flex-wrap">
                  <h3 class="text-lg font-bold text-slate-100 tracking-tight flex items-center gap-1.5 min-w-0" :title="selectedModelDetail.id">
                    <span class="truncate">{{ getFriendlyModelName(selectedModelDetail.id, selectedModelDetail) }}</span>
                    <CheckCircle2 class="w-4 h-4 text-indigo-400 flex-shrink-0" />
                  </h3>

                  <!-- Copy Button -->
                  <button
                    @click="copyRepoId(selectedModelDetail.id)"
                    class="p-1.5 rounded-lg bg-[#141828] hover:bg-[#1e243c] border border-[#222941] text-slate-400 hover:text-white transition-all cursor-pointer"
                    :title="$t('modals.hf.copy_repo_id')"
                  >
                    <Check v-if="copiedId" class="w-3.5 h-3.5 text-emerald-400" />
                    <Copy v-else class="w-3.5 h-3.5" />
                  </button>

                  <!-- External HF Link (Native Browser Opener) -->
                  <button
                    type="button"
                    @click="openHfUrl(selectedModelDetail.id)"
                    class="flex items-center gap-1 text-xs text-indigo-400 hover:text-indigo-300 px-2.5 py-1 rounded-lg bg-[#141828] hover:bg-[#1e243c] border border-[#222941] transition-all cursor-pointer shadow-xs"
                    :title="$t('modals.hf.open_hf_tooltip')"
                  >
                    <span>{{ $t('modals.hf.view_on_hf') }}</span>
                    <ExternalLink class="w-3 h-3" />
                  </button>
                </div>

                <!-- Stats & Author Sub-bar -->
                <div class="flex items-center gap-3 text-xs text-slate-400 flex-wrap">
                  <span class="px-2 py-0.5 rounded-md bg-[#141828] border border-[#222941] font-mono text-[10.5px] text-slate-400 max-w-[360px] truncate" :title="selectedModelDetail.id">
                    {{ selectedModelDetail.id }}
                  </span>
                  <span>•</span>
                  <span class="flex items-center gap-1 text-slate-300">
                    <Download class="w-3.5 h-3.5 text-indigo-400" />
                    <strong class="font-mono">{{ formatNumber(selectedModelDetail.downloads) }}</strong> {{ $t('modals.hf.downloads_count', { count: '' }).trim() }}
                  </span>
                  <span>•</span>
                  <span class="flex items-center gap-1 text-slate-300">
                    <span>❤️</span>
                    <strong class="font-mono">{{ formatNumber(selectedModelDetail.likes) }}</strong> {{ $t('modals.hf.likes_count', { count: '' }).trim() }}
                  </span>
                  <span>•</span>
                  <span>{{ $t('modals.hf.by_author', { author: selectedModelDetail.author || 'HuggingFace' }) }}</span>
                </div>
              </div>
              </div>

              <!-- Format & Gated Badges -->
              <div class="flex items-center gap-2 flex-shrink-0">
                <span
                  v-if="selectedModelDetail.is_mlx"
                  class="px-3 py-1 rounded-xl text-xs font-bold uppercase border shadow-sm bg-purple-500/15 text-purple-300 border-purple-500/30"
                >
                  MLX (Apple Silicon)
                </span>
                <span
                  v-else-if="selectedModelDetail.is_gguf || ggufFilesList.length > 0"
                  class="px-3 py-1 rounded-xl text-xs font-bold uppercase border shadow-sm bg-cyan-500/15 text-cyan-300 border-cyan-500/30"
                >
                  GGUF
                </span>
                <span
                  v-else
                  class="px-3 py-1 rounded-xl text-xs font-bold border shadow-sm bg-amber-500/15 text-amber-300 border-amber-500/30"
                >
                  Safetensors (Base)
                </span>

                <span
                  v-if="selectedModelDetail.is_gated"
                  class="px-2.5 py-1 rounded-xl text-xs font-bold border shadow-sm bg-rose-500/15 text-rose-300 border-rose-500/30 flex items-center gap-1.5"
                  :title="$t('modals.hf.gated_tooltip')"
                >
                  <Lock class="w-3.5 h-3.5 text-rose-400" />
                  <span>{{ $t('modals.hf.gated_badge') }}</span>
                </span>
              </div>
            </div>

            <!-- Model Specs / Capabilities Badges -->
            <div class="flex flex-wrap gap-2 pt-1 border-t border-[#181e30] pt-4">
              <!-- Thinking -->
              <span
                v-if="selectedModelDetail.supports_thinking"
                class="px-2.5 py-1 rounded-lg bg-purple-500/15 border border-purple-500/30 text-xs font-semibold text-purple-300 flex items-center gap-1.5 shadow-xs"
              >
                <Brain class="w-3.5 h-3.5 text-purple-400" />
                <span>{{ $t('modals.hf.tag_reasoning') }}</span>
              </span>

              <!-- Vision -->
              <span
                v-if="selectedModelDetail.supports_vision"
                class="px-2.5 py-1 rounded-lg bg-cyan-500/15 border border-cyan-500/30 text-xs font-semibold text-cyan-300 flex items-center gap-1.5 shadow-xs"
              >
                <Eye class="w-3.5 h-3.5 text-cyan-400" />
                <span>{{ $t('modals.hf.tag_vision') }}</span>
              </span>

              <!-- Tools -->
              <span
                v-if="selectedModelDetail.supports_tools"
                class="px-2.5 py-1 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-xs font-semibold text-emerald-300 flex items-center gap-1.5 shadow-xs"
              >
                <Wrench class="w-3.5 h-3.5 text-emerald-400" />
                <span>{{ $t('modals.hf.tools_mcp_support') }}</span>
              </span>

              <!-- Text only fallback -->
              <span
                v-if="!selectedModelDetail.supports_vision"
                class="px-2.5 py-1 rounded-lg bg-[#141828] border border-[#20273f] text-xs text-slate-300 flex items-center gap-1.5"
              >
                <FileText class="w-3.5 h-3.5 text-slate-400" />
                <span>{{ $t('modals.hf.text_processing') }}</span>
              </span>
            </div>

            <!-- DOWNLOAD OPTIONS BOX (CARD) -->
            <div class="p-5 rounded-3xl bg-gradient-to-b from-[#121629] to-[#0e1120] border border-indigo-500/30 shadow-xl space-y-4">
              
              <div class="flex items-center justify-between flex-wrap gap-2">
                <div class="flex items-center gap-2">
                  <Download class="w-4 h-4 text-indigo-400" />
                  <h4 class="text-sm font-bold text-slate-100 uppercase tracking-wider">
                    {{ $t('modals.hf.tab_download_options') }}
                  </h4>
                </div>

                <!-- Hardware Compatibility Pill -->
                <div
                  v-if="isCurrentModelMemoryFit"
                  class="px-3 py-1 rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 text-xs font-semibold flex items-center gap-1.5 shadow-xs"
                >
                  <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
                  <span>{{ $t('modals.hf.compatible_badge', { ram: props.hardware?.total_ram_gb || 16 }) }}</span>
                </div>
                <div
                  v-else
                  class="px-3 py-1 rounded-xl bg-rose-500/15 border border-rose-500/30 text-rose-300 text-xs font-semibold flex items-center gap-1.5 shadow-xs"
                >
                  <AlertTriangle class="w-3.5 h-3.5 text-rose-400" />
                  <span>{{ $t('modals.hf.insufficient_ram_badge', { required: requiredRamEstimateGb, total: props.hardware?.total_ram_gb || 16 }) }}</span>
                </div>
              </div>

              <!-- Already Installed on Disk Banner -->
              <div
                v-if="isCurrentModelInstalled"
                class="p-4 rounded-2xl bg-gradient-to-r from-emerald-500/15 via-emerald-500/10 to-teal-500/15 border border-emerald-500/30 flex items-center justify-between gap-3 text-xs shadow-md"
              >
                <div class="flex items-center gap-3 min-w-0">
                  <div class="w-8 h-8 rounded-xl bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400 flex-shrink-0 shadow-sm">
                    <CheckCircle2 class="w-4 h-4" />
                  </div>
                  <div class="min-w-0">
                    <div class="flex items-center gap-2">
                      <p class="font-bold text-emerald-300 text-xs">{{ $t('modals.hf.model_on_disk_title') }}</p>
                      <span class="px-1.5 py-0.2 rounded text-[10px] font-mono bg-emerald-500/20 text-emerald-200 border border-emerald-500/40 font-semibold">
                        {{ $t('modals.hf.ready_for_use') }}
                      </span>
                    </div>
                    <p class="text-[11px] text-slate-400 truncate max-w-md font-mono mt-0.5" :title="contractUserPath(installedLocalModel?.local_path) || ''">
                      {{ contractUserPath(installedLocalModel?.local_path) || $t('modals.hf.available_local') }}
                    </p>
                  </div>
                </div>
                <button
                  @click="handleUseLocalModel(installedLocalModel)"
                  class="px-3.5 py-1.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-extrabold text-xs flex items-center gap-1.5 shadow-lg shadow-emerald-500/25 transition-all active:scale-95 cursor-pointer flex-shrink-0"
                >
                  <Sparkles class="w-3.5 h-3.5" />
                  <span>{{ $t('modals.hf.use_now') }}</span>
                </button>
              </div>

              <!-- MLX Platform Incompatibility Warning -->
              <div v-if="selectedModelDetail.is_mlx && !supportsMlx" class="p-3.5 rounded-2xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2.5">
                <AlertTriangle class="w-4 h-4 text-rose-400 flex-shrink-0" />
                <span v-html="$t('modals.hf.mlx_incompatible_warning')"></span>
              </div>

              <!-- GGUF Quants Selection -->
              <div v-if="!selectedModelDetail.is_mlx && ggufFilesList.length > 0" class="space-y-2">
                <p class="text-xs text-slate-400">
                  {{ $t('modals.hf.select_quant_file') }}
                </p>

                <!-- Quants list (model files only, excluding mmproj auxiliaries) -->
                <div class="space-y-2 max-h-64 overflow-y-auto pr-1">
                  <div
                    v-for="file in ggufModelFilesList"
                    :key="file.path"
                    @click="toggleGgufFile(file.path)"
                    :class="[
                      'p-3.5 rounded-2xl border transition-all flex items-center justify-between cursor-pointer select-none',
                      selectedFiles.includes(file.path)
                        ? 'bg-indigo-600/20 border-indigo-500 shadow-sm shadow-indigo-600/20'
                        : 'bg-[#141829] border-[#1e253d] hover:bg-[#181d33] hover:border-slate-600'
                    ]"
                  >
                    <div class="flex items-center gap-3 truncate pr-2">
                      <!-- Checkbox indicator -->
                      <div
                        :class="[
                          'w-4 h-4 rounded-md border flex items-center justify-center transition-all flex-shrink-0',
                          selectedFiles.includes(file.path)
                            ? 'bg-indigo-600 border-indigo-500 text-white'
                            : 'border-slate-600 bg-[#181d2e]'
                        ]"
                      >
                        <Check v-if="selectedFiles.includes(file.path)" class="w-3 h-3 stroke-[3]" />
                      </div>

                      <div class="truncate">
                        <div class="flex items-center gap-2">
                          <span class="text-xs font-bold text-slate-100 truncate" :title="file.path">{{ getFriendlyFileName(file.path, selectedModelDetail.id) }}</span>
                          <span
                            v-if="file.quant_type"
                            class="px-1.5 py-0.2 rounded bg-[#1e253c] border border-[#2b3452] text-[10px] font-mono text-indigo-300 font-bold"
                          >
                            {{ file.quant_type }}
                          </span>
                          <span
                            v-if="isRecommendedQuant(file.path)"
                            class="px-2 py-0.5 rounded-md bg-emerald-500/15 border border-emerald-500/30 text-[9.5px] font-bold text-emerald-400"
                          >
                            {{ $t('modals.hf.recommended') }}
                          </span>
                        </div>
                      </div>
                    </div>

                    <!-- Size -->
                    <div class="flex items-center gap-3 flex-shrink-0">
                      <span class="text-xs font-mono font-bold text-slate-200">
                        {{ file.size_bytes ? formatBytes(file.size_bytes) : $t('modals.hf.dynamic_size') }}
                      </span>
                    </div>
                  </div>
                </div>

                <!-- Auxiliary mmproj file (auto-included for vision models) -->
                <div v-if="ggufMmprojFile" class="mt-3">
                  <p class="text-[11px] text-slate-500 mb-1.5 flex items-center gap-1">
                    <Eye class="w-3 h-3 text-cyan-400" />
                    <span>{{ $t('modals.hf.multimodal_companion_file') }}</span>
                  </p>
                  <div
                    class="p-3 rounded-2xl border transition-all flex items-center justify-between select-none"
                    :class="selectedFiles.includes(ggufMmprojFile.path)
                      ? 'bg-cyan-600/10 border-cyan-500/40'
                      : 'bg-[#141829] border-[#1e253d] opacity-50'"
                  >
                    <div class="flex items-center gap-3 truncate pr-2">
                      <div
                        :class="[
                          'w-4 h-4 rounded-md border flex items-center justify-center transition-all flex-shrink-0',
                          selectedFiles.includes(ggufMmprojFile.path)
                            ? 'bg-cyan-600 border-cyan-500 text-white'
                            : 'border-slate-600 bg-[#181d2e]'
                        ]"
                      >
                        <Check v-if="selectedFiles.includes(ggufMmprojFile.path)" class="w-3 h-3 stroke-[3]" />
                      </div>
                      <div class="truncate">
                        <div class="flex items-center gap-2">
                          <span class="text-xs font-bold text-slate-100 truncate" :title="ggufMmprojFile.path">{{ getFriendlyFileName(ggufMmprojFile.path, selectedModelDetail.id) }}</span>
                          <span class="px-1.5 py-0.2 rounded bg-cyan-500/15 border border-cyan-500/30 text-[10px] font-mono text-cyan-300 font-bold">{{ $t('modals.hf.badge_vision') }}</span>
                          <span v-if="ggufMmprojFile.quant_type" class="px-1.5 py-0.2 rounded bg-[#1e253c] border border-[#2b3452] text-[10px] font-mono text-indigo-300 font-bold">
                            {{ ggufMmprojFile.quant_type }}
                          </span>
                        </div>
                      </div>
                    </div>
                    <span class="text-xs font-mono font-bold text-slate-200 flex-shrink-0">
                      {{ ggufMmprojFile.size_bytes ? formatBytes(ggufMmprojFile.size_bytes) : $t('modals.hf.dynamic_size') }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Safetensors / PyTorch Base Repository (No direct GGUF in repo) -->
              <div v-else-if="!selectedModelDetail.is_mlx" class="p-5 rounded-2xl bg-gradient-to-b from-amber-500/10 to-transparent border border-amber-500/25 space-y-4">
                <div class="flex items-start gap-3.5">
                  <div class="w-10 h-10 rounded-xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center flex-shrink-0 text-amber-400">
                    <Info class="w-5 h-5" />
                  </div>
                  <div class="space-y-1.5 min-w-0 flex-1">
                    <h4 class="text-sm font-bold text-amber-200 flex items-center gap-2">
                      <span>{{ $t('modals.hf.safetensors_repo_title') }}</span>
                      <span v-if="selectedModelDetail.is_gated" class="px-2 py-0.2 rounded text-[10px] font-mono bg-rose-500/20 text-rose-300 border border-rose-500/40">
                        {{ $t('modals.hf.gated_badge') }}
                      </span>
                    </h4>
                    <p class="text-xs text-slate-300 leading-relaxed" v-html="$t('modals.hf.safetensors_desc')"></p>
                    <p class="text-xs text-indigo-300 font-medium" v-html="$t('modals.hf.safetensors_hint')"></p>
                  </div>
                </div>

                <!-- Action row: Find GGUF version -->
                <div class="pt-2 flex flex-wrap items-center gap-3">
                  <button
                    type="button"
                    @click="searchGgufAlternative(selectedModelDetail.id)"
                    class="px-4 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-bold transition-all shadow-md shadow-indigo-600/30 flex items-center gap-2 cursor-pointer active:scale-95"
                  >
                    <Search class="w-4 h-4" />
                    <span>{{ $t('modals.hf.find_gguf_versions', { name: getShortModelName(selectedModelDetail.id) }) }}</span>
                  </button>

                  <button
                    type="button"
                    @click="openHfUrl(selectedModelDetail.id)"
                    class="px-3.5 py-2.5 rounded-xl bg-[#161b2c] hover:bg-[#1d243b] border border-[#273252] text-slate-300 hover:text-white text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer"
                  >
                    <span>{{ $t('modals.hf.view_on_hf') }}</span>
                    <ExternalLink class="w-3.5 h-3.5" />
                  </button>
                </div>

                <!-- Inspect/Download raw safetensors files if available -->
                <div v-if="safetensorsFilesList.length > 0" class="pt-3 border-t border-amber-500/15">
                  <details class="group">
                    <summary class="text-xs font-semibold text-slate-400 hover:text-slate-200 flex items-center gap-1.5 cursor-pointer select-none">
                      <span>{{ $t('modals.hf.view_raw_safetensors', { count: safetensorsFilesList.length }) }}</span>
                    </summary>
                    <div class="mt-2.5 space-y-1.5 max-h-44 overflow-y-auto pr-1">
                      <div
                        v-for="file in safetensorsFilesList"
                        :key="file.path"
                        @click="toggleFile(file.path)"
                        class="p-2.5 rounded-xl border transition-all flex items-center justify-between cursor-pointer select-none"
                        :class="selectedFiles.includes(file.path) ? 'bg-amber-500/10 border-amber-500/40 text-amber-200' : 'bg-[#101424] border-[#1b2237] text-slate-400 hover:text-slate-200'"
                      >
                        <div class="flex items-center gap-2 truncate pr-2">
                          <div
                            class="w-4 h-4 rounded border flex items-center justify-center transition-all flex-shrink-0"
                            :class="selectedFiles.includes(file.path) ? 'bg-amber-500 border-amber-400 text-black' : 'border-slate-600 bg-[#15192a]'"
                          >
                            <Check v-if="selectedFiles.includes(file.path)" class="w-2.5 h-2.5 stroke-[3]" />
                          </div>
                          <span class="truncate font-mono text-xs" :title="file.path">{{ getFriendlyFileName(file.path, selectedModelDetail.id) }}</span>
                        </div>
                        <span class="text-[11px] font-mono text-slate-400 shrink-0">
                          {{ file.size_bytes ? formatBytes(file.size_bytes) : $t('modals.hf.dynamic_size') }}
                        </span>
                      </div>
                    </div>
                  </details>
                </div>
              </div>

              <!-- MLX Bundle Files Selection -->
              <div v-else class="p-4 rounded-2xl bg-[#141829] border border-[#1f253d] space-y-2">
                <div class="flex items-center justify-between text-xs">
                  <span class="text-slate-200 font-semibold">{{ $t('modals.hf.mlx_bundle_title') }}</span>
                  <span class="font-mono text-indigo-300 font-bold">{{ $t('modals.hf.files_count', { count: selectedModelDetail.files.length }) }}</span>
                </div>
                <div class="flex flex-wrap gap-1 max-h-32 overflow-y-auto p-2 bg-[#0e111c] rounded-xl border border-[#1a1f33]">
                  <span
                    v-for="file in selectedModelDetail.files"
                    :key="file.path"
                    class="px-2 py-0.5 rounded-md bg-[#171c2d] border border-[#22293e] text-[10.5px] font-mono text-slate-300 flex items-center gap-1.5"
                  >
                    <span :title="file.path">{{ getFriendlyFileName(file.path, selectedModelDetail.id) }}</span>
                    <span v-if="file.size_bytes" class="text-slate-500 text-[9px]">{{ formatBytes(file.size_bytes) }}</span>
                  </span>
                </div>
              </div>

              <!-- Bottom Action Button & Destination Path -->
              <div class="pt-4 border-t border-[#1f253f] flex items-center justify-between gap-4 flex-wrap">
                <div class="flex items-center gap-2 text-xs truncate">
                  <Folder class="w-4 h-4 text-indigo-400 flex-shrink-0" />
                  <span class="text-slate-400">{{ $t('modals.hf.save_label') }}</span>
                  <select
                    v-if="availableDirs.length > 1"
                    :value="currentDestination"
                    @change="handleDestinationSelect(($event.target as HTMLSelectElement).value)"
                    class="bg-[#0e111c] border border-[#22293e] text-slate-200 text-xs font-mono rounded-lg px-2 py-1 focus:outline-none focus:border-indigo-500 max-w-xs truncate cursor-pointer"
                    :title="$t('modals.hf.save_tooltip')"
                  >
                    <option v-for="d in availableDirs" :key="d" :value="d">
                      {{ d }}
                    </option>
                  </select>
                  <span v-else class="font-mono text-slate-200 truncate max-w-xs" :title="currentDestination">
                    {{ currentDestination || $t('modals.hf.not_selected') }}
                  </span>
                  <button
                    @click="$emit('selectFolder')"
                    class="px-2 py-0.5 rounded bg-[#1c2238] hover:bg-[#252e4c] border border-[#293454] text-[11px] font-medium text-indigo-300 hover:text-white transition-all cursor-pointer"
                    :title="$t('modals.hf.change_folder_tooltip')"
                  >
                    {{ $t('modals.hf.change_folder') }}
                  </button>
                </div>

                <!-- If already installed: Show 'Carregar Modelo' button and secondary 'Baixar Novamente' -->
                <div v-if="isCurrentModelInstalled" class="flex items-center gap-2.5 flex-wrap">
                  <button
                    @click="handleUseLocalModel(installedLocalModel)"
                    class="px-6 py-2.5 rounded-2xl bg-gradient-to-r from-emerald-600 via-emerald-500 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-slate-950 font-extrabold text-xs flex items-center gap-2 shadow-xl shadow-emerald-500/30 transition-all active:scale-95 cursor-pointer"
                  >
                    <CheckCircle2 class="w-4 h-4 text-slate-950" />
                    <span>{{ $t('modals.hf.load_local_btn') }}</span>
                  </button>

                  <button
                    @click="triggerDownload"
                    :disabled="selectedFiles.length === 0 || isStartingDownload"
                    class="px-4 py-2.5 rounded-2xl bg-[#171c2f] hover:bg-[#202742] border border-[#273255] text-slate-300 hover:text-white text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer"
                    :title="$t('modals.hf.download_again_tooltip')"
                  >
                    <Download class="w-3.5 h-3.5 text-slate-400" />
                    <span>{{ $t('modals.hf.download_again') }}</span>
                  </button>
                </div>

                <!-- Primary Action Button: Download if files available, or Search GGUF if Safetensors without selection -->
                <button
                  v-else-if="selectedModelDetail.is_mlx || ggufFilesList.length > 0 || selectedFiles.length > 0"
                  @click="triggerDownload"
                  :disabled="selectedFiles.length === 0 || isStartingDownload || (selectedModelDetail.is_mlx && !supportsMlx)"
                  class="px-6 py-2.5 rounded-2xl bg-gradient-to-r from-indigo-600 via-indigo-500 to-indigo-600 hover:from-indigo-500 hover:to-indigo-400 text-white text-xs font-bold flex items-center gap-2 shadow-xl shadow-indigo-600/30 transition-all active:scale-95 disabled:opacity-50 cursor-pointer"
                >
                  <Download class="w-4 h-4" />
                  <span>{{ (selectedModelDetail.is_mlx && !supportsMlx) ? $t('modals.hf.incompatible_platform') : (isStartingDownload ? $t('modals.hf.starting_download') : $t('modals.hf.download_model_btn', { size: selectedFilesTotalBytes > 0 ? formatBytes(selectedFilesTotalBytes) : $t('modals.hf.download_selected') })) }}</span>
                </button>
                <button
                  v-else
                  @click="searchGgufAlternative(selectedModelDetail.id)"
                  class="px-6 py-2.5 rounded-2xl bg-gradient-to-r from-indigo-600 via-indigo-500 to-indigo-600 hover:from-indigo-500 hover:to-indigo-400 text-white text-xs font-bold flex items-center gap-2 shadow-xl shadow-indigo-600/30 transition-all active:scale-95 cursor-pointer"
                >
                  <Search class="w-4 h-4" />
                  <span>{{ $t('modals.hf.search_gguf_versions_btn') }}</span>
                </button>
              </div>
            </div>

            <!-- README / MODEL CARD SECTION (LM-STUDIO STYLE) -->
            <div class="space-y-3 pt-2">
              <div class="flex items-center justify-between">
                <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider flex items-center gap-2">
                  <FileText class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('modals.hf.readme_title') }}</span>
                </h4>
                <button
                  v-if="selectedModelDetail.readme"
                  @click="isReadmeExpanded = !isReadmeExpanded"
                  class="text-[11px] text-indigo-400 hover:text-indigo-300 font-medium cursor-pointer"
                >
                  {{ isReadmeExpanded ? ($t('common.collapse') || 'Recolher') : ($t('common.expand') || 'Expandir') }}
                </button>
              </div>

              <!-- Formatted Markdown Container -->
              <div
                class="p-5 rounded-2xl bg-[#0e111d] border border-[#1b2136] text-xs text-slate-300 leading-relaxed overflow-x-auto transition-all"
                :class="isReadmeExpanded ? 'max-h-none' : 'max-h-96 overflow-y-auto'"
              >
                <!-- Rendered Markdown -->
                <div
                  v-if="renderedReadme"
                  @click="handleReadmeClick"
                  @error.capture="handleReadmeImageError"
                  class="prose prose-invert prose-xs max-w-none font-sans select-text space-y-3 [&_h1]:text-base [&_h1]:font-bold [&_h1]:text-slate-100 [&_h2]:text-sm [&_h2]:font-bold [&_h2]:text-slate-200 [&_h3]:text-xs [&_h3]:font-semibold [&_h3]:text-indigo-300 [&_p]:text-slate-300 [&_p]:leading-relaxed [&_code]:bg-[#161a2b] [&_code]:text-indigo-300 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:rounded [&_code]:font-mono [&_pre]:bg-[#080a11] [&_pre]:p-3 [&_pre]:rounded-xl [&_pre]:border [&_pre]:border-[#1c2238] [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:list-decimal [&_ol]:pl-5 [&_a]:text-indigo-400 [&_a]:underline [&_img]:max-w-full [&_img]:h-auto [&_img]:rounded-xl [&_img]:my-3 [&_img]:border [&_img]:border-[#1c2238]/60 [&_table]:max-w-full [&_table]:border-collapse [&_table]:my-3 [&_th]:border [&_th]:border-[#222941] [&_th]:p-2 [&_th]:bg-[#141828] [&_td]:border [&_td]:border-[#1c2238] [&_td]:p-2"
                  v-html="renderedReadme"
                />

                <!-- Fallback to description if no README -->
                <div v-else-if="selectedModelDetail.description" class="text-slate-300 leading-relaxed whitespace-pre-wrap">
                  {{ selectedModelDetail.description }}
                </div>

                <div v-else class="text-slate-500 italic">
                  {{ $t('modals.hf.no_readme') }}
                </div>
              </div>
            </div>

            <!-- Tags Section -->
            <div v-if="selectedModelDetail.tags && selectedModelDetail.tags.length > 0" class="space-y-2 pt-2">
              <span class="text-xs font-bold text-slate-400 uppercase tracking-wider">{{ $t('modals.hf.repo_tags') }}</span>
              <div class="flex flex-wrap gap-1.5">
                <span
                  v-for="tag in selectedModelDetail.tags.slice(0, 16)"
                  :key="tag"
                  class="px-2.5 py-0.5 rounded-lg bg-[#111524] border border-[#1d2338] text-[11px] text-slate-400 font-mono"
                >
                  {{ tag }}
                </span>
              </div>
            </div>

          </template>

          <!-- State: No Model Selected Initial Prompt -->
          <div v-else class="h-full flex flex-col items-center justify-center text-center space-y-3 text-slate-400">
            <Box class="w-12 h-12 text-slate-600" />
            <p class="text-sm font-bold text-slate-300">{{ $t('modals.hf.select_model_title') }}</p>
            <p class="text-xs text-slate-500 max-w-sm">
              {{ $t('modals.hf.select_model_desc') }}
            </p>
          </div>
        </div>
      </div>

      <!-- MAIN TAB 2: ACTIVE & COMPLETED DOWNLOADS -->
      <div v-else-if="activeTab === 'downloads'" class="flex-1 flex flex-col overflow-hidden p-6">
        <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
          <div>
            <h4 class="text-sm font-bold text-slate-100 flex items-center gap-2">
              <Download class="w-4 h-4 text-indigo-400" />
              <span>{{ $t('modals.hf.downloads_manager') }}</span>
            </h4>
            <p class="text-xs text-slate-400 mt-0.5">
              Acompanhe downloads em andamento em tempo real com velocidade e estimativa de tempo.
            </p>
          </div>

          <div class="flex items-center gap-2">
            <!-- Filter Pills (Active vs All) -->
            <div class="flex items-center gap-1 bg-[#141826] p-1 rounded-xl border border-[#20273c] text-xs">
              <button
                @click="downloadFilter = 'active'"
                :class="[
                  'px-3 py-1 rounded-lg font-semibold transition-all cursor-pointer',
                  downloadFilter === 'active'
                    ? 'bg-indigo-600 text-white shadow-xs'
                    : 'text-slate-400 hover:text-slate-200'
                ]"
              >
                <span>{{ $t('modals.hf.tab_active_downloads', { count: runningDownloadsCount }) }}</span>
              </button>
              <button
                @click="downloadFilter = 'all'"
                :class="[
                  'px-3 py-1 rounded-lg font-semibold transition-all cursor-pointer',
                  downloadFilter === 'all'
                    ? 'bg-indigo-600 text-white shadow-xs'
                    : 'text-slate-400 hover:text-slate-200'
                ]"
              >
                <span>{{ $t('modals.hf.tab_all_history', { count: finishedDownloadsCount }) }}</span>
              </button>
            </div>

            <!-- Clear Finished/Cancelled Button -->
            <button
              v-if="finishedDownloadsCount > 0"
              @click="clearFinishedDownloads"
              class="px-3 py-1.5 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/25 text-xs text-rose-300 hover:text-rose-200 flex items-center gap-1.5 transition-all cursor-pointer"
              :title="$t('modals.hf.clear_completed_tooltip')"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>{{ $t('modals.hf.clear_completed') }}</span>
            </button>

            <button
              @click="fetchDownloads"
              class="px-3 py-1.5 rounded-xl bg-[#141826] hover:bg-[#1a2034] border border-[#20273c] text-xs text-slate-300 hover:text-white flex items-center gap-1.5 transition-all cursor-pointer"
            >
              <RefreshCw class="w-3.5 h-3.5 text-indigo-400" />
              <span>{{ $t('common.refresh') || 'Atualizar' }}</span>
            </button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto space-y-3 pr-1">
          <div v-if="displayedDownloads.length === 0" class="h-64 flex flex-col items-center justify-center text-center text-slate-400 space-y-2">
            <Download class="w-10 h-10 text-slate-600" />
            <p class="text-sm font-semibold text-slate-300">
              {{ downloadFilter === 'active' ? $t('modals.hf.empty_active_downloads') : $t('modals.hf.empty_history') }}
            </p>
            <p class="text-xs text-slate-500 max-w-sm">
              {{ downloadFilter === 'active' ? $t('modals.hf.empty_active_downloads_hint') : $t('modals.hf.empty_history_hint') }}
            </p>
          </div>

          <div
            v-for="task in displayedDownloads"
            :key="task.task_id"
            class="p-4 rounded-3xl bg-[#111524] border border-[#1e2439] space-y-3 shadow-md group"
          >
            <!-- Task Header -->
            <div class="flex items-start justify-between gap-3">
              <div class="space-y-0.5 truncate">
                <div class="flex items-center gap-2">
                  <h5 class="text-xs font-bold text-slate-100 truncate">
                    {{ getFriendlyModelName(task.title || task.repo_id) }}
                  </h5>
                  <span
                    :class="[
                      'px-2 py-0.5 rounded-md text-[9.5px] font-bold uppercase border',
                      task.status === 'downloading'
                        ? 'bg-amber-500/15 text-amber-300 border-amber-500/30 animate-pulse'
                        : task.status === 'completed'
                        ? 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30'
                        : task.status === 'cancelled'
                        ? 'bg-slate-500/15 text-slate-400 border-slate-500/30'
                        : 'bg-rose-500/15 text-rose-300 border-rose-500/30'
                    ]"
                  >
                    {{ task.status === 'downloading' ? $t('modals.hf.status_downloading') : task.status === 'completed' ? $t('modals.hf.status_completed') : task.status === 'cancelled' ? $t('modals.hf.status_cancelled') : $t('modals.hf.status_error') }}
                  </span>
                </div>
                <p class="text-[11px] text-slate-400 font-mono truncate flex items-center gap-1.5">
                  <span class="text-slate-300 font-semibold">{{ task.repo_id }}</span>
                  <span class="text-slate-600">•</span>
                  <span>{{ task.file_name }}</span>
                </p>
              </div>

              <!-- Cancel / Action Button -->
              <div class="flex items-center gap-2 flex-shrink-0">
                <button
                  v-if="task.status === 'downloading'"
                  @click="cancelDownload(task)"
                  class="px-3 py-1 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 border border-rose-500/30 text-rose-300 text-xs font-semibold transition-all flex items-center gap-1 cursor-pointer"
                  :title="$t('modals.hf.cancel_download_tooltip')"
                >
                  <StopCircle class="w-3.5 h-3.5 text-rose-400" />
                  <span>{{ $t('modals.hf.cancel_download') }}</span>
                </button>

                <button
                  v-else-if="task.status === 'completed'"
                  @click="$emit('modelDownloaded'); $emit('close')"
                  class="px-3 py-1 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold transition-all flex items-center gap-1 shadow-sm cursor-pointer"
                >
                  <CheckCircle2 class="w-3.5 h-3.5" />
                  <span>{{ $t('modals.hf.ready_in_catalog') }}</span>
                </button>

                <button
                  v-else-if="task.status === 'error'"
                  @click="retryDownload(task)"
                  class="px-3 py-1 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 border border-indigo-500/40 text-indigo-200 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer shadow-xs"
                  :title="$t('modals.hf.retry_download_tooltip')"
                >
                  <RefreshCw class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('common.retry') || 'Retomar' }}</span>
                </button>

                <button
                  v-if="task.status !== 'downloading'"
                  @click="dismissTask(task.task_id)"
                  class="p-1.5 rounded-xl bg-[#151928] hover:bg-rose-500/20 text-slate-400 hover:text-rose-300 border border-[#20273c] transition-colors cursor-pointer"
                  title="Remover da lista"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            <!-- Progress Bar -->
            <div class="space-y-1.5">
              <div class="w-full h-2 bg-[#090b12] rounded-full overflow-hidden border border-[#1e2439]">
                <div
                  class="h-full rounded-full transition-all duration-300"
                  :class="task.status === 'completed' ? 'bg-emerald-500' : task.status === 'cancelled' ? 'bg-slate-600' : task.status === 'error' ? 'bg-rose-500' : 'bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 animate-pulse'"
                  :style="{ width: `${task.progress_pct}%` }"
                ></div>
              </div>

              <!-- Telemetry metrics row -->
              <div class="flex items-center justify-between text-[11px] text-slate-400 font-mono">
                <div class="flex items-center gap-3">
                  <span>{{ formatBytes(task.downloaded_bytes) }} / {{ task.total_bytes ? formatBytes(task.total_bytes) : 'Calculando...' }}</span>
                  <span class="font-bold text-indigo-300">({{ task.progress_pct.toFixed(1) }}%)</span>
                </div>

                <div class="flex items-center gap-3">
                  <span v-if="task.status === 'downloading' && task.speed_bytes_per_sec > 0" class="text-amber-300 font-semibold">
                    {{ formatSpeed(task.speed_bytes_per_sec) }}
                  </span>
                  <span v-if="task.status === 'downloading' && task.eta_seconds" class="text-slate-400">
                    ETA: {{ formatEta(task.eta_seconds) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Destination info -->
            <div class="text-[10.5px] text-slate-500 font-mono truncate flex items-center gap-1">
              <Folder class="w-3 h-3 text-slate-600 flex-shrink-0" />
              <span class="truncate">{{ task.destination_path }}</span>
            </div>

            <!-- Error message if any -->
            <p v-if="task.error_message" class="text-xs text-rose-400 bg-rose-500/10 p-2.5 rounded-xl border border-rose-500/20">
              ⚠️ {{ task.error_message }}
            </p>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, inject, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import MarkdownIt from 'markdown-it'
import { addNotification } from '~/utils/notifications'
import { resolveModelLogo } from '../composables/useModelLogo'
import { useAppLocale } from '../composables/useLocale'
import { contractUserPath } from '~/utils/pathUtils'
import type { HardwareInfo, Model } from '~/types'

const supportsMlx = inject<ComputedRef<boolean>>('supportsMlx', computed(() => true))
const { t } = useAppLocale()
import {
  Search,
  Download,
  Folder,
  RefreshCw,
  X,
  Check,
  Sparkles,
  ExternalLink,
  StopCircle,
  CheckCircle2,
  Box,
  Loader2,
  Brain,
  Eye,
  FileText,
  Wrench,
  AlertTriangle,
  Trash2,
  ArrowUpDown,
  ShieldCheck,
  Copy,
  Lock,
  Info
} from 'lucide-vue-next'

const md = new (MarkdownIt as any)({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true
})

const props = defineProps<{
  modelsDir?: string
  modelsDirs?: string[]
  hardware?: HardwareInfo | any
  localModels?: Model[]
}>()

const emit = defineEmits<{
  close: []
  selectFolder: []
  setDestination: [val: string]
  modelDownloaded: [model?: any]
  loadModel: [model: any]
}>()

const availableDirs = computed(() => {
  const dirs = [...(props.modelsDirs || [])]
  if (props.modelsDir && !dirs.some((d) => contractUserPath(d) === contractUserPath(props.modelsDir))) {
    dirs.unshift(props.modelsDir)
  }
  return Array.from(new Set(dirs.filter(Boolean).map(contractUserPath)))
})

const currentDestination = computed(() => {
  const dest = props.modelsDir || (props.modelsDirs && props.modelsDirs[0]) || ''
  return contractUserPath(dest)
})

const handleDestinationSelect = (val: string) => {
  if (val) {
    emit('setDestination', val)
  }
}

const activeTab = ref('explore')
const downloadFilter = ref('active') // 'active' | 'all'
const searchQuery = ref('')
const selectedFormat = ref('all')
const selectedSort = ref('trendingScore')
const hideIncompatible = ref(true)
const isSearching = ref(false)
const searchResults = ref<any[]>([])
const avatarCache = ref<Map<string, string>>(new Map())
const failedAvatars = ref<Set<string>>(new Set())

const markAvatarFailed = (url?: string | null) => {
  if (url) {
    failedAvatars.value.add(url)
  }
}

const getModelAvatar = (model: any): string | undefined => {
  if (!model) return undefined
  const logo = resolveModelLogo(model, avatarCache.value)
  if (logo && !failedAvatars.value.has(logo)) return logo
  return undefined
}

const selectedModelDetail = ref<any>(null)
const isLoadingDetail = ref(false)
const selectedFiles = ref<string[]>([])
const isStartingDownload = ref(false)
const copiedId = ref(false)
const isReadmeExpanded = ref(false)

const downloads = ref<any[]>([])
const notifiedTaskIds = new Set()
let downloadsTimer: ReturnType<typeof setInterval> | null = null

const allFormatFilters = computed(() => [
  { id: 'all', label: t('models.filter_all') },
  { id: 'gguf', label: 'GGUF' },
  { id: 'mlx', label: 'MLX (Apple Silicon)' }
])

const formatFilters = computed(() => {
  if (supportsMlx.value) {
    return allFormatFilters.value
  }
  return allFormatFilters.value.filter(f => f.id !== 'mlx')
})

const allCuratedPresets = [
  { id: 'qwen25-7b-mlx', label: 'Qwen 2.5 7B', repo: 'mlx-community/Qwen2.5-7B-Instruct-4bit', format: 'MLX' },
  { id: 'llama32-3b-gguf', label: 'Llama 3.2 3B', repo: 'bartowski/Llama-3.2-3B-Instruct-GGUF', format: 'GGUF' },
  { id: 'deepseek-r1-8b', label: 'DeepSeek-R1 8B', repo: 'mlx-community/DeepSeek-R1-Distill-Llama-8B-4bit', format: 'MLX' },
  { id: 'mistral-nemo-12b', label: 'Mistral Nemo 12B', repo: 'bartowski/Mistral-Nemo-Instruct-2407-GGUF', format: 'GGUF' },
  { id: 'gemma2-9b-mlx', label: 'Gemma 2 9B', repo: 'mlx-community/gemma-2-9b-it-4bit', format: 'MLX' },
  { id: 'phi4-14b-gguf', label: 'Phi-4 14B', repo: 'bartowski/phi-4-GGUF', format: 'GGUF' },
  { id: 'qwen25-7b-gguf', label: 'Qwen 2.5 7B GGUF', repo: 'bartowski/Qwen2.5-7B-Instruct-GGUF', format: 'GGUF' }
]

const curatedPresets = computed(() => {
  if (supportsMlx.value) {
    return allCuratedPresets.filter(p => p.id !== 'qwen25-7b-gguf')
  }
  return allCuratedPresets.filter(p => p.format !== 'MLX')
})

const runningDownloadsCount = computed(() => {
  return downloads.value.filter((d) => d.status === 'downloading').length
})

const finishedDownloadsCount = computed(() => {
  return downloads.value.filter((d) => d.status !== 'downloading').length
})

const displayedDownloads = computed(() => {
  if (downloadFilter.value === 'active') {
    return downloads.value.filter((d) => d.status === 'downloading')
  }
  return downloads.value
})

// Helper: detect if a GGUF file is an auxiliary mmproj (multimodal projection) file
const isMmprojFile = (path: string): boolean => {
  const lower = (path || '').toLowerCase()
  return lower.includes('mmproj') || lower.includes('mm-proj') || lower.includes('vision-adapter') || lower.includes('projector')
}

// All GGUF files in the repository
const ggufFilesList = computed(() => {
  if (!selectedModelDetail.value || !selectedModelDetail.value.files) return []
  return selectedModelDetail.value.files.filter((f: any) => f.is_gguf || f.path.toLowerCase().endsWith('.gguf'))
})

// All Safetensors files in the repository
const safetensorsFilesList = computed(() => {
  if (!selectedModelDetail.value || !selectedModelDetail.value.files) return []
  return selectedModelDetail.value.files.filter((f: any) => f.is_safetensors || f.path.toLowerCase().endsWith('.safetensors'))
})

// Only the main model GGUF files (excludes mmproj/projector auxiliaries)
const ggufModelFilesList = computed(() => {
  return ggufFilesList.value.filter((f: any) => !isMmprojFile(f.path))
})

// The mmproj auxiliary file, if present
const ggufMmprojFile = computed(() => {
  return ggufFilesList.value.find((f: any) => isMmprojFile(f.path)) || null
})

const selectedFilesTotalBytes = computed(() => {
  if (!selectedModelDetail.value) return 0
  let total = 0
  for (const f of selectedModelDetail.value.files || []) {
    if (selectedFiles.value.includes(f.path)) {
      total += f.size_bytes || 0
    }
  }
  return total
})

const resolveHfUrl = (url: string, repoId: string, isMedia = true): string => {
  if (!url || !repoId) return url
  let trimmed = url.trim()

  // Keep absolute protocols, data URLs, anchors, mailto
  if (
    /^https?:\/\//i.test(trimmed) ||
    /^data:/i.test(trimmed) ||
    /^blob:/i.test(trimmed) ||
    /^mailto:/i.test(trimmed) ||
    trimmed.startsWith('#')
  ) {
    return trimmed
  }

  // Protocol-relative //huggingface.co
  if (trimmed.startsWith('//')) {
    return 'https:' + trimmed
  }

  // Remove leading ./ or /
  let cleanPath = trimmed.replace(/^\.?\//, '')

  // If it is a link to another Hugging Face model/author (e.g. "author/repo" without file extension)
  if (!isMedia && !/\.[a-zA-Z0-9]{1,5}$/.test(cleanPath) && /^[\w.-]+\/[\w.-]+$/.test(cleanPath)) {
    return `https://huggingface.co/${cleanPath}`
  }

  // Remove duplicate repoId if prefixed (e.g. "author/model/resolve/main/...")
  if (cleanPath.toLowerCase().startsWith(repoId.toLowerCase() + '/')) {
    cleanPath = cleanPath.substring(repoId.length + 1)
  }

  // Handle resolve/main or raw/main or blob/main prefix
  if (/^(?:resolve|raw|blob)\/main\//i.test(cleanPath)) {
    cleanPath = cleanPath.replace(/^(?:resolve|raw|blob)\/main\//i, '')
  }

  const endpoint = isMedia ? 'resolve' : 'blob'
  return `https://huggingface.co/${repoId}/${endpoint}/main/${cleanPath}`
}

const cleanReadmeContent = (raw: string, repoId: string): string => {
  if (!raw) return ''
  let text = raw.trim()
  if (text.startsWith('---')) {
    const endIdx = text.indexOf('---', 3)
    if (endIdx !== -1) {
      text = text.substring(endIdx + 3).trim()
    }
  }
  if (!repoId) return text

  // 1. Resolve relative HTML img/video/source/audio src attributes
  text = text.replace(
    /<(img|source|video|audio)([^>]*?)\ssrc\s*=\s*["']([^"']+)["']([^>]*?)>/gi,
    (_match: string, tag: string, before: string, src: string, after: string) => {
      const resolved = resolveHfUrl(src, repoId, true)
      const hasLoading = /loading\s*=/i.test(before + after)
      const loadingAttr = tag.toLowerCase() === 'img' && !hasLoading ? ' loading="lazy"' : ''
      return `<${tag}${before} src="${resolved}"${after}${loadingAttr}>`
    }
  )

  // 2. Resolve relative Markdown images: ![alt](url "title")
  text = text.replace(
    /!\[(.*?)\]\((?!https?:\/\/|data:|blob:|\/\/|#)([^)\s]+)(\s+["'][^"']*["'])?\)/gi,
    (_match: string, alt: string, url: string, title?: string) => {
      const resolved = resolveHfUrl(url, repoId, true)
      return `![${alt}](${resolved}${title || ''})`
    }
  )

  // 3. Resolve relative HTML a href attributes
  text = text.replace(
    /<a([^>]*?)\shref\s*=\s*["'](?!https?:\/\/|mailto:|#|\/\/)([^"']+)["']([^>]*?)>/gi,
    (_match: string, before: string, href: string, after: string) => {
      const resolved = resolveHfUrl(href, repoId, false)
      return `<a${before} href="${resolved}"${after}>`
    }
  )

  // 4. Resolve relative Markdown links: [text](url "title")
  text = text.replace(
    /(^|[^!])\[(.*?)\]\((?!https?:\/\/|mailto:|#|\/\/)([^)\s]+)(\s+["'][^"']*["'])?\)/gi,
    (_match: string, prefix: string, linkText: string, url: string, title?: string) => {
      const resolved = resolveHfUrl(url, repoId, false)
      return `${prefix}[${linkText}](${resolved}${title || ''})`
    }
  )

  return text
}

const renderedReadme = computed(() => {
  if (!selectedModelDetail.value?.readme) return ''
  try {
    const repoId = selectedModelDetail.value?.id || ''
    const cleaned = cleanReadmeContent(selectedModelDetail.value.readme, repoId)
    return md.render(cleaned)
  } catch (e) {
    console.error('Erro ao renderizar markdown do README:', e)
    return selectedModelDetail.value.readme
  }
})

const handleReadmeClick = (e: MouseEvent) => {
  const target = (e.target as HTMLElement)?.closest('a')
  if (target && target.href) {
    e.preventDefault()
    invoke('open_url', { url: target.href }).catch(() => {
      window.open(target.href, '_blank')
    })
  }
}

const handleReadmeImageError = (e: Event) => {
  const target = e.target as HTMLImageElement | null
  if (!target || target.tagName !== 'IMG') return

  const repoId = selectedModelDetail.value?.id || ''
  if (!repoId) {
    target.style.display = 'none'
    return
  }

  // If quantized repo doesn't have the assets folder, fallback to base model repo
  // e.g. "ornith-ai/Ornith-1.5-9B-GGUF" -> "ornith-ai/Ornith-1.5-9B"
  const baseRepoId = repoId.replace(/[-_](?:gguf|mlx|awq|gptq|exl2)$/i, '')
  const hasBaseCandidate = baseRepoId && baseRepoId !== repoId

  if (hasBaseCandidate && !target.dataset.triedBase && target.src.includes(`/${repoId}/`)) {
    target.dataset.triedBase = 'true'
    target.src = target.src.replace(`/${repoId}/`, `/${baseRepoId}/`)
    return
  }

  // Gracefully hide broken images to avoid unsightly missing icon placeholders
  target.style.display = 'none'
}

const parseModelParamNumber = (modelId?: string | null, paramSizeStr?: string | null): number | null => {
  if (paramSizeStr) {
    const num = parseFloat(paramSizeStr)
    if (!isNaN(num) && num > 0) return num
  }
  if (modelId) {
    // Look for matches like 31B, 32b, 7B, 12b, 405b
    const match = modelId.match(/[-_](\d+(?:\.\d+)?)[bB]/) || modelId.match(/(\d+(?:\.\d+)?)[bB]/)
    if (match && match[1]) {
      const num = parseFloat(match[1])
      if (!isNaN(num) && num > 0 && num < 1000) return num
    }
  }
  return null
}

const requiredRamEstimateGb = computed(() => {
  if (!selectedModelDetail.value) return 16

  // 1. If we have actual selected files or total model size, calculate based on real weight footprint
  const actualBytes = selectedFilesTotalBytes.value > 0
    ? selectedFilesTotalBytes.value
    : (selectedModelDetail.value.total_size_bytes || 0)

  if (actualBytes > 0) {
    const sizeGb = actualBytes / (1024 * 1024 * 1024)
    // Model in RAM + ~20% overhead for KV cache/context + 1 GB system margin
    return Math.max(4, Math.ceil(sizeGb * 1.2 + 1.0))
  }

  // 2. Fallback to parameter size if file size is completely unknown
  const paramNum = parseModelParamNumber(selectedModelDetail.value.id, null)
  if (paramNum) {
    if (paramNum >= 65) return 48
    if (paramNum >= 30) return 24
    if (paramNum >= 20) return 20
    if (paramNum >= 14) return 14
    if (paramNum >= 8) return 8
  }

  return 16
})

// Extract clean alphanumeric slug from model name/id
const extractModelSlug = (rawIdOrName: string): string => {
  if (!rawIdOrName) return ''
  const leaf = rawIdOrName.includes('/')
    ? rawIdOrName.split('/').slice(1).join('/')
    : (rawIdOrName.includes('___') ? rawIdOrName.split('___').slice(1).join('___') : rawIdOrName)

  return leaf
    .toLowerCase()
    .replace(/\.gguf$/i, '')
    .replace(/mlx/gi, '')
    .replace(/[^a-z0-9]/g, '')
}

// Check if a model is installed in local catalog (props.localModels)
const findLocalModel = (hfModelOrId: any, chosenFiles: string[] = []) => {
  if (!hfModelOrId || !props.localModels || props.localModels.length === 0) return null
  const hfId = typeof hfModelOrId === 'string' ? hfModelOrId : (hfModelOrId.id || '')
  const isMlx = typeof hfModelOrId === 'object'
    ? !!hfModelOrId.is_mlx
    : (hfId.toLowerCase().includes('mlx') || (chosenFiles.length > 1 && chosenFiles.some((f) => f && f.endsWith('.safetensors'))))

  const cleanHfId = hfId.toLowerCase().trim()
  const hfUnderscore = cleanHfId.replace(/\//g, '___')
  const hfSlug = extractModelSlug(cleanHfId)

  // 1. Direct match on id or local_path
  for (const lm of props.localModels) {
    if (!lm) continue
    const lmId = (lm.id || '').toLowerCase().trim()
    const lmUnderscore = lmId.replace(/\//g, '___')
    const lmPath = (lm.local_path || '').toLowerCase().trim()

    if (lmId === cleanHfId || lmUnderscore === hfUnderscore) return lm
    if (lmPath && (lmPath.endsWith('/' + cleanHfId) || lmPath.endsWith('/' + hfUnderscore))) return lm
  }

  // 2. GGUF single file match
  if (!isMlx && chosenFiles && chosenFiles.length > 0) {
    for (const fileName of chosenFiles) {
      if (!fileName || !fileName.endsWith('.gguf')) continue
      const lowerFile = fileName.toLowerCase()
      for (const lm of props.localModels) {
        if (!lm) continue
        const lmPath = (lm.local_path || '').toLowerCase()
        const lmId = (lm.id || '').toLowerCase()
        if (lmPath.endsWith('/' + lowerFile) || lmId.endsWith(lowerFile) || lmId === lowerFile) {
          return lm
        }
      }
    }
  }

  // 3. Robust slug match (e.g. mlx-community/gemma-4-12B-it-4bit vs lmstudio-community/gemma-4-12B-it-MLX-4bit)
  if (hfSlug) {
    for (const lm of props.localModels) {
      if (!lm) continue
      if (isMlx && lm.format !== 'mlx') continue
      if (!isMlx && lm.format === 'mlx') continue

      const lmSlug = extractModelSlug(lm.id || lm.name || '')
      const lmPathSlug = extractModelSlug(lm.local_path || '')

      if (lmSlug && (lmSlug === hfSlug || lmSlug.startsWith(hfSlug) || hfSlug.startsWith(lmSlug))) {
        return lm
      }
      if (lmPathSlug && (lmPathSlug === hfSlug || lmPathSlug.startsWith(hfSlug) || hfSlug.startsWith(lmPathSlug))) {
        return lm
      }
    }
  }

  return null
}

const diskModelMatch = ref<any>(null)
const isCheckingDisk = ref(false)

const checkDiskForCurrentModel = async () => {
  if (!selectedModelDetail.value) {
    diskModelMatch.value = null
    return
  }

  const memoryMatch = findLocalModel(selectedModelDetail.value, selectedFiles.value)
  if (memoryMatch) {
    diskModelMatch.value = memoryMatch
    return
  }

  try {
    isCheckingDisk.value = true
    const res = await invoke('check_hf_model_downloaded', {
      repoId: selectedModelDetail.value.id,
      files: selectedFiles.value,
      modelsDirs: availableDirs.value,
      isMlxBundle: !!selectedModelDetail.value.is_mlx
    })
    diskModelMatch.value = res || null
  } catch (err) {
    console.warn('Failed to check if model exists on disk:', err)
  } finally {
    isCheckingDisk.value = false
  }
}

const installedLocalModel = computed(() => {
  if (!selectedModelDetail.value) return null
  const mem = findLocalModel(selectedModelDetail.value, selectedFiles.value)
  if (mem) return mem
  if (diskModelMatch.value) return diskModelMatch.value
  return null
})

const isCurrentModelInstalled = computed(() => !!installedLocalModel.value)

const handleUseLocalModel = (model?: any) => {
  const m = model || installedLocalModel.value
  if (!m) return
  emit('loadModel', m)
  emit('close')
  addNotification({
    type: 'success',
    title: t('modals.hf.load_local_title'),
    message: t('modals.hf.load_local_msg', { name: m.name || m.id })
  })
}

watch(
  () => [selectedModelDetail.value?.id, selectedFiles.value],
  () => {
    checkDiskForCurrentModel()
  },
  { deep: true }
)

const isCurrentModelMemoryFit = computed(() => {
  const totalRam = props.hardware?.total_ram_gb || 16
  const maxSafeRamGb = Math.max(totalRam * 0.80, totalRam - 3.5)

  if (!selectedModelDetail.value) return true

  // 1. Exact selected quant size check (GGUF quant or MLX selected files)
  if (selectedFilesTotalBytes.value > 0) {
    const sizeGb = selectedFilesTotalBytes.value / (1024 * 1024 * 1024)
    return sizeGb <= maxSafeRamGb
  }

  // 2. MLX bundle size check
  if (selectedModelDetail.value.is_mlx && selectedModelDetail.value.total_size_bytes > 0) {
    const sizeGb = selectedModelDetail.value.total_size_bytes / (1024 * 1024 * 1024)
    return sizeGb <= maxSafeRamGb
  }

  // 3. If estimated size bytes exists
  if (selectedModelDetail.value.total_size_bytes > 0) {
    const sizeGb = selectedModelDetail.value.total_size_bytes / (1024 * 1024 * 1024)
    return sizeGb <= maxSafeRamGb
  }

  // 4. Fallback parameter check (only if sizes could not be determined at all)
  const paramNum = parseModelParamNumber(selectedModelDetail.value.id, null)
  if (paramNum !== null) {
    if (paramNum >= 65 && totalRam < 48) return false
    if (paramNum >= 28 && totalRam < 24) return false
    if (paramNum >= 20 && totalRam < 20) return false
    if (paramNum >= 15 && totalRam < 16) return false
  }

  return true
})

let searchTimeout: ReturnType<typeof setTimeout> | null = null
const handleSearchInput = () => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    handleSearch()
  }, 350)
}

const isModelExecutable = (model: any): boolean => {
  if (!model) return false

  // 1. Never show gated models (require terms agreement / HF auth token)
  if (model.is_gated) {
    return false
  }

  // 2. Must be runnable locally in Atena Studio (must be GGUF or MLX)
  if (!model.is_gguf && !model.is_mlx) {
    return false
  }

  // 3. Platform check: MLX is only runnable on Apple Silicon
  if (model.is_mlx && !model.is_gguf && !supportsMlx.value) {
    return false
  }

  return true
}

const isModelCompatible = (model: any): boolean => {
  if (!isModelExecutable(model)) return false

  // If the model is currently selected / being inspected, NEVER hide it from the list!
  if (selectedModelDetail.value?.id === model.id) return true

  // Hardware RAM capacity
  const totalRam = props.hardware?.total_ram_gb || 16
  const maxSafeModelRamGb = Math.max(totalRam * 0.80, totalRam - 3.5)

  // 1. If estimated size is known
  const sizeBytes = model.estimated_size_bytes

  if (sizeBytes && sizeBytes > 0) {
    const sizeGb = sizeBytes / (1024 * 1024 * 1024)
    return sizeGb <= maxSafeModelRamGb
  }

  // 2. Fallback parameter size check when byte size is unavailable
  const paramNum = parseModelParamNumber(model.id, model.param_size)
  if (paramNum !== null) {
    if (paramNum >= 65 && totalRam < 48) return false
    if (paramNum >= 28 && totalRam < 24) return false
    if (paramNum >= 20 && totalRam < 20) return false
    if (paramNum >= 15 && totalRam < 16) return false
  }

  return true
}

const getModelCardSize = (model: any) => {
  if (selectedModelDetail.value?.id === model.id && selectedFilesTotalBytes.value > 0) {
    return formatBytes(selectedFilesTotalBytes.value)
  }
  if (model.estimated_size_bytes) {
    return formatBytes(model.estimated_size_bytes)
  }
  return null
}

const displayedResults = computed(() => {
  const runnable = searchResults.value.filter((m) => isModelExecutable(m))
  if (!hideIncompatible.value) return runnable
  return runnable.filter((m) => isModelCompatible(m))
})

const hiddenIncompatibleCount = computed(() => {
  const runnable = searchResults.value.filter((m) => isModelExecutable(m))
  if (!hideIncompatible.value) return 0
  return runnable.length - displayedResults.value.length
})

const handleSearch = async () => {
  isSearching.value = true
  try {
    const list = await invoke('hf_search_models', {
      query: searchQuery.value,
      filterType: selectedFormat.value === 'all' ? null : selectedFormat.value,
      sort: selectedSort.value,
      limit: 40
    })
    searchResults.value = (list as any[]) || []
    selectedModelDetail.value = null

    // Background fetch avatars for model authors
    fetchAvatarsForResults(searchResults.value)
    
    // Auto-select first model if nothing is selected or if current model is not in displayed results
    if (displayedResults.value.length > 0 && (!selectedModelDetail.value || !displayedResults.value.some(m => m.id === selectedModelDetail.value?.id))) {
      inspectModel(displayedResults.value[0].id)
    }
  } catch (err) {
    console.error('Failed to search models:', err)
  } finally {
    isSearching.value = false
  }
}

const fetchAvatarsForResults = (models: any[]) => {
  if (!models || models.length === 0) return

  // 1. Extract unique authors that are not already cached
  const authorsToFetch = new Set<string>()
  for (const m of models) {
    const auth = (m.author || (m.id && m.id.split('/')[0]) || '').trim()
    if (auth && !avatarCache.value.has(auth.toLowerCase()) && !failedAvatars.value.has(auth.toLowerCase())) {
      authorsToFetch.add(auth)
    }
  }

  if (authorsToFetch.size === 0) return

  // 2. Mark pending so we don't duplicate fetches
  authorsToFetch.forEach((author) => {
    avatarCache.value.set(author.toLowerCase(), '')
  })

  // 3. Asynchronously fetch each author in the background without blocking
  authorsToFetch.forEach(async (author: string) => {
    try {
      const avatarUrl = await invoke('hf_get_author_avatar', { author })
      avatarCache.value.set(author.toLowerCase(), avatarUrl as string)
      if (avatarUrl && !failedAvatars.value.has(avatarUrl as string)) {
        for (const m of searchResults.value) {
          const mAuthor = (m.author || m.id.split('/')[0] || '').toLowerCase()
          if (mAuthor === author.toLowerCase()) {
            m.author_avatar_url = avatarUrl
          }
        }
        if (selectedModelDetail.value) {
          const selAuthor = (selectedModelDetail.value.author || selectedModelDetail.value.id.split('/')[0] || '').toLowerCase()
          if (selAuthor === author.toLowerCase() && !selectedModelDetail.value.author_avatar_url) {
            selectedModelDetail.value.author_avatar_url = avatarUrl
          }
        }
      }
    } catch {
      avatarCache.value.set(author.toLowerCase(), '')
    }
  })
}

const applyPreset = (preset: any) => {
  searchQuery.value = preset.repo
  inspectModel(preset.repo)
}

const inspectModel = async (repoId: string) => {
  isLoadingDetail.value = true
  selectedFiles.value = []
  isReadmeExpanded.value = false

  try {
    const detail = await invoke('hf_get_model_details', { repoId }) as any
    selectedModelDetail.value = detail

    if (detail?.author_avatar_url) {
      const auth = (detail.author || repoId.split('/')[0] || '').toLowerCase()
      avatarCache.value.set(auth, detail.author_avatar_url)
      const matching = searchResults.value.find(m => m.id === repoId)
      if (matching && !matching.author_avatar_url) {
        matching.author_avatar_url = detail.author_avatar_url
      }
    }

    if (detail?.is_mlx) {
      // Select all files for MLX bundle
      selectedFiles.value = (detail.files || []).map((f: any) => f.path)
    } else if (detail?.files) {
      // Pick best GGUF model file that fits within available RAM
      const ggufFiles = detail.files.filter((f: any) => (f.is_gguf || f.path.toLowerCase().endsWith('.gguf')) && !isMmprojFile(f.path))
      const mmprojFile = detail.files.find((f: any) => (f.is_gguf || f.path.toLowerCase().endsWith('.gguf')) && isMmprojFile(f.path))
      const recommended = getRecommendedGgufFile(detail.files)
      const picked: string[] = []
      if (recommended) {
        picked.push(recommended.path)
      } else if (ggufFiles.length > 0) {
        picked.push(ggufFiles[0].path)
      }
      // Auto-include mmproj auxiliary when present (required for vision models)
      if (mmprojFile && picked.length > 0) {
        picked.push(mmprojFile.path)
      }
      selectedFiles.value = picked
    }

    // Keep model card in searchResults synchronized with recommended file size
    const targetModel = searchResults.value.find((m) => m.id === repoId)
    if (targetModel && selectedFilesTotalBytes.value > 0) {
      targetModel.estimated_size_bytes = selectedFilesTotalBytes.value
    }
  } catch (err) {
    console.error('Erro ao inspecionar modelo:', err)
  } finally {
    isLoadingDetail.value = false
  }
}

const toggleGgufFile = (path: string) => {
  // Single quant selection for the model file, but auto-include mmproj if available
  const picked = [path]
  if (ggufMmprojFile.value) {
    picked.push(ggufMmprojFile.value.path)
  }
  selectedFiles.value = picked
}

const maxSafeModelRamBytes = computed(() => {
  const totalRam = props.hardware?.total_ram_gb || 16
  const safeGb = Math.max(totalRam * 0.80, totalRam - 3.5)
  return safeGb * 1024 * 1024 * 1024
})

const getRecommendedGgufFile = (files: any[]): any => {
  if (!files || files.length === 0) return null
  const ggufFiles = files.filter(
    (f: any) => (f.is_gguf || f.path.toLowerCase().endsWith('.gguf')) && !isMmprojFile(f.path)
  )
  if (ggufFiles.length === 0) return null

  const maxBytes = maxSafeModelRamBytes.value

  // Quant preferences ordered from highest quality down to lightweight:
  // When a model is small enough that Q8_0 or Q6_K fits comfortably in user RAM
  // (e.g. 4.07 GB Spark 4B on a 16 GB Mac), Q8_0 provides near-lossless precision!
  const quantOrder = [
    'Q8_0',
    'Q8_K',
    'Q8_1',
    'Q6_K',
    'Q5_K_M',
    'Q5_K_S',
    'Q5_0',
    'Q5_1',
    'Q4_K_M',
    'Q4_K_S',
    'Q4_K',
    'Q4_0',
    'Q4_1',
    'IQ4_NL',
    'IQ4_XS',
    'Q3_K_L',
    'Q3_K_M',
    'Q3_K_S',
    'IQ3_M',
    'IQ3_S',
    'IQ3_XS',
    'IQ3_XXS',
    'Q2_K',
    'IQ2_M',
    'IQ2_S',
    'IQ2_XS',
    'IQ2_XXS'
  ]

  // 1. Filter only the files that actually fit within the user's available RAM
  const fittingFiles = ggufFiles.filter((f: any) => {
    if (!f.size_bytes || f.size_bytes <= 0) return true
    return f.size_bytes <= maxBytes
  })

  // Recommendation must strictly prioritize quants that fit in RAM if any exist!
  const pool = fittingFiles.length > 0 ? fittingFiles : ggufFiles

  // 2. Look for best quantization in priority order among fitting files
  for (const q of quantOrder) {
    const found = pool.find((f: any) => {
      const qt = (f.quant_type || '').toUpperCase()
      const p = (f.path || '').toUpperCase()
      return qt === q || p.includes(q)
    })
    if (found) return found
  }

  // 3. Fallback: if fitting files exist, pick the largest fitting file;
  // otherwise, pick the smallest available file so user at least gets the lightest option
  if (fittingFiles.length > 0) {
    return [...fittingFiles].sort((a: any, b: any) => (b.size_bytes || 0) - (a.size_bytes || 0))[0]
  }

  return [...ggufFiles].sort((a: any, b: any) => (a.size_bytes || 0) - (b.size_bytes || 0))[0]
}

const isRecommendedQuant = (path: string): boolean => {
  if (!selectedModelDetail.value?.files) return false
  const rec = getRecommendedGgufFile(selectedModelDetail.value.files)
  return rec?.path === path
}

const copyRepoId = async (id: string) => {
  try {
    await navigator.clipboard.writeText(id)
    copiedId.value = true
    setTimeout(() => {
      copiedId.value = false
    }, 2000)
  } catch (err) {
    console.error('Falha ao copiar:', err)
  }
}

const openHfUrl = async (repoId: string) => {
  if (!repoId) return
  const url = `https://huggingface.co/${repoId}`
  try {
    await invoke('open_url', { url })
  } catch (err) {
    console.error('Failed to open URL via Tauri, falling back to window.open:', err)
    window.open(url, '_blank')
  }
}

const normalizeModelLabel = (value: string, options: Record<string, any> = {}): string => {
  if (!value) return ''
  let label = value.split('/').pop() || ''

  label = label
    .replace(/\.(gguf|safetensors|bin|json|txt|md)$/i, '')
    .replace(/(?:^|[-_])(gguf|mlx|safetensors|pytorch|transformers)(?:$|[-_])/gi, ' ')
    .replace(/(?:^|[-_])(?:qat|quantized|quant)(?:$|[-_])/gi, ' ')

  if (options.stripQuant !== false) {
    label = label
      .replace(/(?:^|[-_])(?:i?q\d(?:_[a-z0-9]+){0,3}|f16|bf16|fp16|f32|4bit|8bit)(?:$|[-_])/gi, ' ')
  }

  return label
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
    .split(' ')
    .map((part: string) => {
      if (!part) return part
      const upper = part.toUpperCase()
      if (/^(AI|GGUF|MLX|QAT|MTP|VLM|DPO|RLHF|IT|BF16|FP16|F16|F32)$/.test(upper)) return upper
      if (/^\d+(?:\.\d+)?[BMK]$/i.test(part)) return upper
      if (/^[A-Z0-9]{2,}$/.test(part)) return part
      return part.charAt(0).toUpperCase() + part.slice(1)
    })
    .join(' ')
}

const getFriendlyModelName = (fullId: string, model: any = null): string => {
  const label = normalizeModelLabel(fullId)
  return label || getShortModelName(fullId) || model?.id || ''
}

const getFriendlyFileName = (path: string, repoId = ''): string => {
  if (!path) return ''
  const fileName = path.split('/').pop() || ''
  const isVisionFile = isMmprojFile(fileName)
  const repoBase = repoId ? (repoId.split('/').pop() || '') : ''

  let cleaned = fileName
  if (repoBase && cleaned.toLowerCase().startsWith(repoBase.toLowerCase())) {
    cleaned = cleaned.slice(repoBase.length).replace(/^[-_.\s]+/, '')
  }

  const label = normalizeModelLabel(cleaned || fileName)
  if (isVisionFile) return label ? t('modals.hf.vision_file', { label }) : t('modals.hf.vision_file_generic')
  return label || fileName
}

const getAuthorInitials = (model: any): string => {
  const source = model?.author || model?.id?.split('/')[0] || 'HF'
  return source
    .replace(/[^a-z0-9]+/gi, ' ')
    .trim()
    .split(' ')
    .filter(Boolean)
    .slice(0, 2)
    .map((part: string) => part.charAt(0))
    .join('')
    .toUpperCase() || 'HF'
}

const getShortModelName = (fullId: string): string => {
  if (!fullId) return ''
  const parts = fullId.split('/')
  return (parts.length > 1 && parts[1]) ? parts[1] : fullId
}

const getModelFormatLabel = (model: any): string => {
  if (!model) return 'GGUF'
  if (model.is_mlx) return 'MLX'
  if (model.is_gguf) return 'GGUF'
  if (model.is_safetensors) return 'Safetensors'
  return 'Base'
}

const getModelFormatClass = (model: any, isSelected: boolean): string => {
  if (isSelected) return 'text-white font-bold'
  if (model.is_mlx) return 'text-purple-400 font-semibold'
  if (model.is_gguf) return 'text-cyan-400 font-semibold'
  if (model.is_safetensors) return 'text-amber-400 font-semibold'
  return 'text-slate-400 font-semibold'
}

const searchGgufAlternative = (repoId: string) => {
  const shortName = getShortModelName(repoId)
  searchQuery.value = shortName
  selectedFormat.value = 'gguf'
  handleSearch()
}

const toggleFile = (path: string) => {
  const idx = selectedFiles.value.indexOf(path)
  if (idx >= 0) {
    selectedFiles.value.splice(idx, 1)
  } else {
    selectedFiles.value.push(path)
  }
}

const triggerDownload = async () => {
  if (!selectedModelDetail.value || selectedFiles.value.length === 0) return
  isStartingDownload.value = true

  try {
    const friendlyTitle = getFriendlyModelName(selectedModelDetail.value.id, selectedModelDetail.value)
    const avatarUrl = getModelAvatar(selectedModelDetail.value) || selectedModelDetail.value.author_avatar_url || avatarCache.value.get((selectedModelDetail.value.author || selectedModelDetail.value.id.split('/')[0] || '').toLowerCase()) || null
    await invoke('start_hf_download', {
      repoId: selectedModelDetail.value.id,
      selectedFiles: selectedFiles.value,
      targetDir: currentDestination.value,
      isMlxBundle: selectedModelDetail.value.is_mlx,
      modelTitle: friendlyTitle,
      authorAvatarUrl: avatarUrl
    })

    // Switch to active downloads tab
    activeTab.value = 'downloads'
    downloadFilter.value = 'active'
    fetchDownloads()
  } catch (err) {
    console.error('Erro ao iniciar download:', err)
    alert(`Falha ao iniciar download: ${err}`)
  } finally {
    isStartingDownload.value = false
  }
}

const cancelDownload = async (task: any) => {
  try {
    await invoke('cancel_hf_download', { taskId: task.task_id })
    await invoke('dismiss_download_task', { taskId: task.task_id })

    addNotification({
      type: 'download_cancelled',
      title: t('modals.hf.download_cancelled_title'),
      message: t('modals.hf.download_cancelled_msg', { title: task.title || 'model' })
    })

    fetchDownloads()
  } catch (err) {
    console.error('Failed to cancel download:', err)
  }
}

const retryDownload = async (task: any) => {
  notifiedTaskIds.delete(task.task_id)
  try {
    await invoke('retry_hf_download', { taskId: task.task_id })
    fetchDownloads()
  } catch (err) {
    console.error('Failed to retry download:', err)
  }
}

const dismissTask = async (taskId: string) => {
  try {
    await invoke('dismiss_download_task', { taskId })
    fetchDownloads()
  } catch (err) {
    console.error('Failed to dismiss task:', err)
  }
}

const clearFinishedDownloads = async () => {
  try {
    await invoke('clear_finished_downloads')
    fetchDownloads()
  } catch (err) {
    console.error('Failed to clear downloads:', err)
  }
}

const fetchDownloads = async () => {
  try {
    const list = await invoke('get_active_downloads') as any[]
    downloads.value = list || []

    for (const d of downloads.value) {
      if (d.status === 'completed' && !notifiedTaskIds.has(d.task_id)) {
        notifiedTaskIds.add(d.task_id)
        addNotification({
          type: 'download_completed',
          title: t('modals.hf.download_complete_title'),
          message: t('modals.hf.download_complete_msg', { title: d.title })
        })
        emit('modelDownloaded')
      } else if (d.status === 'error' && !notifiedTaskIds.has(d.task_id)) {
        notifiedTaskIds.add(d.task_id)
        addNotification({
          type: 'download_error',
          title: t('modals.hf.download_error_title'),
          message: t('modals.hf.download_error_msg', { title: d.title, error: d.error_message || 'Unknown error' })
        })
      }
    }
  } catch (err) {
    // Silent
  }
}

const formatBytes = (bytes: number): string => {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatSpeed = (bytesPerSec: number): string => {
  if (!bytesPerSec) return '0 MB/s'
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`
}

const formatEta = (seconds: number): string => {
  if (!seconds || seconds <= 0) return '0s'
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

const formatNumber = (num: any) => {
  if (!num) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'k'
  return num.toString()
}

onMounted(() => {
  handleSearch()
  fetchDownloads()
  downloadsTimer = setInterval(fetchDownloads, 1000)
})

onUnmounted(() => {
  if (downloadsTimer) clearInterval(downloadsTimer)
  if (searchTimeout) clearTimeout(searchTimeout)
})
</script>
