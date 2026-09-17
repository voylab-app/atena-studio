<template>
  <div class="h-full flex flex-col bg-[#090a0f] p-6 overflow-hidden">
    <!-- Header & Controls -->
    <div class="flex flex-col gap-4 mb-5 select-none">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-bold text-slate-100 flex items-center gap-2.5">
            <div class="w-8 h-8 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shadow-sm">
              <Layers class="w-4 h-4" />
            </div>
            <span>{{ $t('models.title') }}</span>
          </h2>
          <div class="flex items-center gap-2 text-xs text-slate-400 mt-1">
            <template v-if="activeFilter === 'cloud' || activeFilter === 'agy'">
              <span class="flex items-center gap-1.5 text-sky-300 font-medium">
                <Cloud class="w-3.5 h-3.5 text-sky-400" />
                <span>{{ $t('models.cloud_connectors_label') }}</span>
              </span>
              <button
                @click="$emit('openSettings', 'plugins')"
                class="flex items-center gap-1.5 text-slate-300 hover:text-white font-medium bg-[#121522] hover:bg-[#181d2e] px-2.5 py-1 rounded-lg border border-[#1e2338] hover:border-sky-500/40 text-[11px] transition-all cursor-pointer shadow-sm group"
                :title="$t('models.manage_connectors')"
              >
                <Blocks class="w-3 h-3 text-sky-400 group-hover:scale-110 transition-transform" />
                <span>{{ $t('models.manage_connectors') }}</span>
              </button>
            </template>
            <template v-else>
              <span>{{ $t('models.models_dir') }}</span>
              <span class="text-slate-300 font-mono bg-[#121522] px-2 py-0.5 rounded-lg border border-[#1e2338] max-w-sm truncate text-[11px]" :title="contractUserPath(modelsDir)">{{ contractUserPath(modelsDir) }}</span>
              <button
                @click="$emit('selectFolder')"
                class="p-1 rounded-lg bg-[#121522] hover:bg-[#181d2e] border border-[#1e2338] hover:border-indigo-500/40 text-indigo-400 hover:text-indigo-300 transition-all cursor-pointer"
                :title="$t('models.change_dir_tooltip')"
              >
                <FolderOpen class="w-3.5 h-3.5" />
              </button>
            </template>
          </div>
        </div>

        <div class="flex items-center gap-2.5">
          <!-- Hugging Face Download Button (Apenas para modelos locais) -->
          <button
            v-if="activeFilter !== 'cloud' && activeFilter !== 'agy'"
            @click="$emit('openHfModal')"
            class="flex items-center gap-2 px-3.5 py-1.5 rounded-xl bg-gradient-to-r from-amber-500/15 via-orange-500/10 to-amber-500/15 hover:from-amber-500/25 hover:to-orange-500/20 border border-amber-500/30 hover:border-amber-500/50 text-xs font-semibold text-amber-300 hover:text-amber-200 transition-all shadow-sm active:scale-95 cursor-pointer group"
          >
            <Download class="w-3.5 h-3.5 text-amber-400 group-hover:translate-y-[-1px] transition-transform" />
            <span>{{ $t('models.download_from_hf') }}</span>
            <span
              v-if="activeDownloadsCount > 0"
              class="px-1.5 py-0.5 bg-amber-400 text-slate-950 font-bold text-[10px] rounded-full animate-pulse"
            >
              {{ activeDownloadsCount }}
            </span>
          </button>

          <!-- Background Sync Badge -->
          <div
            v-if="isScanning && displayModels.length > 0"
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-xl bg-indigo-500/10 border border-indigo-500/25 text-[11px] font-medium text-indigo-300 animate-pulse select-none"
            :title="$t('models.syncing_bg')"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
            <span>{{ $t('models.updating_bg') }}</span>
          </div>

          <!-- Rescan Models Button -->
          <button
            @click="$emit('rescanModels')"
            :disabled="isScanning"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#121522] hover:bg-[#181d2e] border border-[#1e2338] hover:border-indigo-500/40 text-xs font-medium text-slate-300 hover:text-white transition-all shadow-sm active:scale-95 disabled:opacity-50 cursor-pointer"
          >
            <RefreshCw :class="['w-3.5 h-3.5 text-indigo-400', isScanning ? 'animate-spin' : '']" />
            <span>{{ isScanning ? $t('models.scanning') : $t('models.rescan') }}</span>
          </button>
        </div>
      </div>

      <!-- Filters, Search Bar & Hardware Config Bar -->
      <div class="space-y-3">
        <!-- Row 1: Full-width Search -->
        <div class="relative flex items-center">
          <Search class="w-4 h-4 text-slate-500 absolute left-3 pointer-events-none" />
          <input
            type="text"
            v-model="searchQuery"
            :placeholder="$t('models.search_models')"
            class="w-full pl-9 pr-8 py-2 rounded-xl bg-[#111420]/90 border border-[#1e2338] focus:border-indigo-500/70 focus:ring-2 focus:ring-indigo-500/15 text-xs text-slate-100 placeholder-slate-500 transition-all outline-none"
          />
          <button
            v-if="searchQuery"
            @click="searchQuery = ''"
            class="absolute right-2.5 text-slate-500 hover:text-slate-200 p-0.5 transition-colors"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Row 2: Organized Filter Groups -->
        <div class="flex items-center gap-3 flex-wrap">
          <!-- Group 1: Primary Navigation (Todos, Favoritos, AGY) -->
          <div class="flex items-center gap-1 bg-[#111420]/90 p-1 rounded-xl border border-[#1e2338]">
            <button
              v-for="filter in primaryFilters"
              :key="filter.id"
              @click="activeFilter = filter.id"
              :class="[
                'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer select-none',
                activeFilter === filter.id
                  ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-[#181d2e]'
              ]"
            >
              <component
                :is="filter.icon"
                :class="[
                  'w-3.5 h-3.5',
                  activeFilter === filter.id ? 'text-white' : (filter.iconColor || 'text-slate-400')
                ]"
              />
              <span>{{ filter.label }}</span>
            </button>
          </div>

          <!-- Separator -->
          <div class="w-px h-5 bg-[#1e2338]" />

          <!-- Group 2: Format Filters (MLX, GGUF, Ollama) -->
          <div class="flex items-center gap-1 bg-[#111420]/90 p-1 rounded-xl border border-[#1e2338]">
            <button
              v-for="filter in formatFilters"
              :key="filter.id"
              @click="activeFilter = filter.id"
              :class="[
                'flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer select-none',
                activeFilter === filter.id
                  ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-[#181d2e]'
              ]"
            >
              <component
                :is="filter.icon"
                :class="[
                  'w-3.5 h-3.5',
                  activeFilter === filter.id ? 'text-white' : (filter.iconColor || 'text-slate-400')
                ]"
              />
              <span>{{ filter.label }}</span>
            </button>
          </div>

          <!-- Separator -->
          <div class="w-px h-5 bg-[#1e2338]" />

          <!-- Group 3: Capability Toggles (Icon-only with tooltips) -->
          <div class="flex items-center gap-1">
            <button
              v-for="filter in capabilityFilters"
              :key="filter.id"
              @click="activeFilter = activeFilter === filter.id ? 'all' : filter.id"
              :class="[
                'p-1.5 rounded-lg transition-all cursor-pointer select-none',
                activeFilter === filter.id
                  ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-[#181d2e] bg-[#111420]/90 border border-[#1e2338]'
              ]"
              :title="filter.label"
            >
              <component
                :is="filter.icon"
                :class="[
                  'w-3.5 h-3.5',
                  activeFilter === filter.id ? 'text-white' : (filter.iconColor || 'text-slate-400')
                ]"
              />
            </button>
          </div>
        </div>

        <!-- Bottom Row: Pre-Load Hardware & KV Cache Configuration Bar (Local Models) -->
        <div v-if="activeFilter !== 'cloud' && activeFilter !== 'agy'" class="p-2.5 px-3.5 rounded-2xl bg-gradient-to-r from-[#101424] via-[#0d101c] to-[#101424] border border-[#1d2338] flex items-center justify-between gap-3 text-xs shadow-inner flex-wrap">
          <div class="flex items-center gap-2.5">
            <div class="w-7 h-7 rounded-lg bg-teal-500/10 border border-teal-500/20 flex items-center justify-center text-teal-400 shadow-sm">
              <Zap class="w-3.5 h-3.5" />
            </div>
            <div>
              <span class="font-semibold text-slate-200 text-xs flex items-center gap-1.5">
                <span>{{ $t('models.gpu_loading_params') }}</span>
                <span class="text-[10.5px] font-normal text-teal-400 font-mono">({{ supportsMlx ? 'Metal / VRAM' : 'GPU / VRAM' }})</span>
              </span>
            </div>
          </div>

          <div class="flex items-center gap-2.5 flex-wrap">
            <!-- KV Cache Quantization Selector -->
            <div class="flex items-center gap-2 bg-[#090b12] px-2.5 py-1 rounded-xl border border-[#1e2338]">
              <span class="text-[11px] text-slate-400 font-medium">{{ $t('models.kv_cache_label') }}</span>
              <select
                v-if="params"
                v-model="params.kv_cache_quant"
                class="bg-transparent text-xs font-mono font-semibold text-teal-300 outline-none cursor-pointer pr-1 !bg-none !pr-1"
                :title="$t('models.kv_cache_tooltip')"
              >
                <option value="f16" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_f16') }}</option>
                <option value="q8_0" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_q8_0') }}</option>
                <option value="q4_0" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_q4_0') }}</option>
                <option value="q4_1" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_q4_1') }}</option>
                <option value="q5_0" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_q5_0') }}</option>
                <option value="iq4_nl" class="bg-[#0e111a] text-slate-200">{{ $t('models.kv_cache_iq4_nl') }}</option>
              </select>
            </div>

            <!-- Context Length Selector -->
            <div class="flex items-center gap-2 bg-[#090b12] px-2.5 py-1 rounded-xl border border-[#1e2338]">
              <span class="text-[11px] text-slate-400 font-medium">{{ $t('models.context_label') }}</span>
              <select
                v-if="params"
                v-model.number="params.context_length"
                class="bg-transparent text-xs font-mono font-semibold text-indigo-300 outline-none cursor-pointer pr-1 !bg-none !pr-1"
                :title="$t('models.context_tooltip')"
              >
                <option :value="2048" class="bg-[#0e111a] text-slate-200">2k (2.048 tok)</option>
                <option :value="4096" class="bg-[#0e111a] text-slate-200">4k (4.096 tok)</option>
                <option :value="8000" class="bg-[#0e111a] text-slate-200">8k (8.000 tok)</option>
                <option :value="8192" class="bg-[#0e111a] text-slate-200">8k (8.192 tok)</option>
                <option :value="16384" class="bg-[#0e111a] text-slate-200">16k (16.384 tok)</option>
                <option :value="32768" class="bg-[#0e111a] text-slate-200">32k (32.768 tok)</option>
                <option :value="65536" class="bg-[#0e111a] text-slate-200">64k (65.536 tok)</option>
              </select>
            </div>

            <!-- Flash Attention Toggle -->
            <button
              v-if="params"
              type="button"
              @click="params.flash_attention = params.flash_attention === false ? true : false"
              :class="[
                'flex items-center gap-1.5 px-2.5 py-1 rounded-xl border text-[11px] font-medium transition-all cursor-pointer select-none active:scale-95',
                params.flash_attention !== false
                  ? 'bg-indigo-500/15 text-indigo-300 border-indigo-500/40 shadow-sm'
                  : 'bg-[#090b12] text-slate-400 border-[#1e2338]'
              ]"
              :title="$t('models.flash_attention_tooltip')"
            >
              <span
                :class="[
                  'w-1.5 h-1.5 rounded-full',
                  params.flash_attention !== false ? 'bg-indigo-400 shadow-sm shadow-indigo-400' : 'bg-slate-600'
                ]"
              />
              <span>Flash Attention</span>
            </button>

            <!-- Prompt Cache Toggle -->
            <button
              v-if="params"
              type="button"
              @click="params.enable_prompt_cache = params.enable_prompt_cache === false ? true : false"
              :class="[
                'flex items-center gap-1.5 px-2.5 py-1 rounded-xl border text-[11px] font-medium transition-all cursor-pointer select-none active:scale-95',
                params.enable_prompt_cache !== false
                  ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/40 shadow-sm'
                  : 'bg-[#090b12] text-slate-400 border-[#1e2338]'
              ]"
              :title="$t('models.prompt_cache_tooltip')"
            >
              <span
                :class="[
                  'w-1.5 h-1.5 rounded-full',
                  params.enable_prompt_cache !== false ? 'bg-emerald-400 shadow-sm shadow-emerald-400' : 'bg-slate-600'
                ]"
              />
              <span>{{ $t('models.cache_reuse') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Models List Grid -->
    <div class="flex-1 overflow-y-auto pr-1">
      <div v-if="displayModels.length === 0" class="h-72 flex flex-col items-center justify-center text-center text-slate-400">
        <div v-if="isScanning" class="flex flex-col items-center">
          <RefreshCw class="w-8 h-8 text-indigo-400 animate-spin mb-3" />
          <p class="text-sm font-semibold text-slate-200">{{ $t('models.scanning_models') }}</p>
          <p class="text-xs text-slate-400 mt-1 max-w-sm">
            {{ $t('models.scanning_models_desc') }}
          </p>
        </div>
        <div v-else class="flex flex-col items-center">
          <div class="w-12 h-12 rounded-2xl bg-[#121522] border border-[#1e2338] flex items-center justify-center text-slate-500 mb-3">
            <Box class="w-6 h-6" />
          </div>
          <p class="text-sm font-semibold text-slate-300">{{ $t('models.no_models_found') }}</p>
          <p class="text-xs text-slate-400 mt-1 max-w-sm">
            {{ activeFilter === 'favorites' ? $t('models.no_favorites_desc') : $t('models.no_models_search_desc') }}
          </p>
        </div>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 2xl:grid-cols-3 gap-4 pb-4">
        <div
          v-for="card in displayModels"
          :key="card.key"
          :class="[
            'group relative p-4 rounded-2xl border transition-all duration-200 flex flex-col justify-between overflow-hidden',
            isCardActive(card)
              ? 'bg-[#121628]/95 border-indigo-500/50 shadow-xl shadow-indigo-500/10 ring-1 ring-indigo-500/30'
              : isCardFavorite(card)
              ? 'bg-[#111420]/80 hover:bg-[#141828] border-amber-500/25 hover:border-amber-500/40 shadow-sm'
              : 'bg-[#111420]/70 hover:bg-[#141826] border-[#1e2338]/80 hover:border-slate-700/80 shadow-sm'
          ]"
        >
          <!-- Top active glow line if loaded -->
          <div
            v-if="isCardActive(card)"
            class="absolute top-0 left-0 right-0 h-[2px] bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-400"
          />

          <!-- Top loading progress bar if loading -->
          <div
            v-if="isCardLoading(card)"
            class="absolute top-0 left-0 right-0 h-[2.5px] bg-amber-500/20 overflow-hidden z-10"
          >
            <div
              class="h-full bg-gradient-to-r from-amber-500 via-amber-400 to-emerald-400 transition-all duration-300 ease-out shadow-[0_0_10px_rgba(245,158,11,0.6)]"
              :style="{ width: `${loadingModelProgress}%` }"
            />
          </div>

          <!-- Card Top Content -->
          <div class="space-y-3">
            <div class="flex items-start justify-between gap-2.5">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <!-- Model Avatar Image / Badge -->
                <div
                  class="w-10 h-10 rounded-xl overflow-hidden flex items-center justify-center shrink-0 shadow-md border relative group-hover:scale-105 transition-transform"
                  :class="[
                    getModelLogo(card.model) ? 'bg-white shadow-sm' : 'bg-[#131726]',
                    isAgyModel(card.model)
                      ? 'border-amber-500/30'
                      : card.model.backend === 'MlxLm' || card.model.format === 'Mlx'
                      ? 'border-purple-500/30'
                      : card.model.backend === 'Ollama' || card.model.format === 'Ollama'
                      ? 'border-emerald-500/30'
                      : 'border-cyan-500/30'
                  ]"
                >
                  <img
                    v-if="getModelLogo(card.model)"
                    :src="getModelLogo(card.model) || undefined"
                    :alt="card.model.name"
                    class="w-full h-full object-contain p-1.5"
                    loading="lazy"
                  />
                  <component
                    v-else
                    :is="isAgyModel(card.model) ? Sparkles : (card.model.backend === 'MlxLm' || card.model.format === 'Mlx') ? Cpu : (card.model.backend === 'Ollama' || card.model.format === 'Ollama') ? Terminal : Box"
                    class="w-5 h-5"
                    :class="isAgyModel(card.model) ? 'text-amber-400' : (card.model.backend === 'MlxLm' || card.model.format === 'Mlx') ? 'text-purple-400' : (card.model.backend === 'Ollama' || card.model.format === 'Ollama') ? 'text-emerald-400' : 'text-cyan-400'"
                  />
                </div>

                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1.5 min-w-0">
                    <h3 class="text-sm font-semibold text-slate-100 truncate group-hover:text-amber-300 transition-colors" :title="card.isGroup ? card.baseName : card.model.name">
                      {{ card.isGroup ? card.baseName : card.model.name }}
                    </h3>
                  </div>
                  <div class="flex items-center gap-1.5 mt-0.5 text-[11px] text-slate-400 font-medium truncate">
                    <span class="truncate">{{ card.model.author || 'Local' }}</span>
                    <template v-if="isAgyModel(card.model)">
                      <span class="text-slate-600 shrink-0">•</span>
                      <span class="text-slate-400 font-mono text-[10px] shrink-0">Antigravity</span>
                    </template>
                  </div>
                </div>
              </div>

              <!-- Header Action Icons & Format Badge -->
              <div class="flex items-center gap-1.5 flex-shrink-0">
                <!-- Favorite Star Button -->
                <button
                  @click.stop="toggleCardFavorite(card)"
                  :class="[
                    'p-1.5 rounded-lg border transition-all cursor-pointer active:scale-90',
                    isCardFavorite(card)
                      ? 'bg-amber-500/15 border-amber-500/30 text-amber-400 shadow-sm shadow-amber-500/10'
                      : 'bg-[#141824] hover:bg-[#1b2030] border-[#1e2338] text-slate-400 hover:text-amber-400'
                  ]"
                  :title="isCardFavorite(card) ? $t('models.favorite_remove_tooltip') : $t('models.favorite_add_tooltip')"
                >
                  <Star :class="['w-3.5 h-3.5', isCardFavorite(card) ? 'fill-amber-400 text-amber-400' : '']" />
                </button>

                <!-- Delete Model Button (Apenas modelos locais com arquivo físico) -->
                <button
                  v-if="!isAgyModel(card.model) && !isCloudModel(card.model)"
                  @click.stop="openDeleteConfirm(card.model)"
                  class="p-1.5 rounded-lg bg-[#141824] hover:bg-rose-500/15 border border-[#1e2338] hover:border-rose-500/30 text-slate-400 hover:text-rose-400 transition-all cursor-pointer active:scale-90"
                  :title="$t('models.delete_model_disk_tooltip')"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>

                <!-- Hide / Disable Model Button (Para modelos Cloud / AGY) -->
                <button
                  v-else
                  @click.stop="$emit('hideModel', card.model)"
                  class="p-1.5 rounded-lg bg-[#141824] hover:bg-amber-500/15 border border-[#1e2338] hover:border-amber-500/30 text-slate-400 hover:text-amber-400 transition-all cursor-pointer active:scale-90"
                  :title="$t('models.hide_model_tooltip')"
                >
                  <EyeOff class="w-3.5 h-3.5" />
                </button>

                <!-- Model Engine / Format Badge -->
                <span
                  :class="[
                    'flex items-center gap-1 px-2 py-0.5 rounded-lg text-[10px] font-bold tracking-wider uppercase border shrink-0',
                    isCardActive(card)
                      ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/30'
                      : isAgyModel(card.model)
                      ? 'bg-amber-500/10 text-amber-300 border-amber-500/20'
                      : (card.model.id && card.model.id.startsWith('gemini/'))
                      ? 'bg-blue-500/10 text-blue-300 border-blue-500/20'
                      : (card.model.id && card.model.id.startsWith('groq/'))
                      ? 'bg-amber-500/10 text-amber-300 border-amber-500/20'
                      : (card.model.id && card.model.id.startsWith('openai/'))
                      ? 'bg-teal-500/10 text-teal-300 border-teal-500/20'
                      : (card.model.id && card.model.id.startsWith('openrouter/'))
                      ? 'bg-purple-500/10 text-purple-300 border-purple-500/20'
                      : card.model.backend === 'MlxLm' || card.model.format === 'Mlx'
                      ? 'bg-purple-500/10 text-purple-300 border-purple-500/20'
                      : card.model.backend === 'Ollama' || card.model.format === 'Ollama'
                      ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/20'
                      : 'bg-sky-500/10 text-sky-300 border-sky-500/20'
                  ]"
                >
                  <span v-if="isCardActive(card)" class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                  <Sparkles v-else-if="isAgyModel(card.model)" class="w-2.5 h-2.5 text-amber-400" />
                  <span>{{ isCardActive(card) ? $t('common.active_badge') : getModelFormatLabel(card.model) }}</span>
                </span>
              </div>
            </div>

            <!-- Metadata & Clean Capability Badges -->
            <div class="flex flex-wrap gap-1.5 pt-0.5">
              <!-- Antigravity Badges -->
              <template v-if="isAgyModel(card.model)">
                <!-- Thinking / Reasoning Badge -->
                <span
                  v-if="card.model.supports_thinking"
                  class="px-2 py-0.5 rounded-md bg-purple-500/10 border border-purple-500/25 text-[10.5px] font-medium text-purple-300 flex items-center gap-1"
                  :title="$t('models.thinking_tooltip')"
                >
                  <Brain class="w-3 h-3 text-purple-400" />
                  <span>Thinking</span>
                </span>

                <!-- Multimodal Vision Badge -->
                <span
                  v-if="card.model.supports_vision"
                  class="px-2 py-0.5 rounded-md bg-sky-500/10 border border-sky-500/25 text-[10.5px] font-medium text-sky-300 flex items-center gap-1"
                  :title="$t('models.vision_tooltip')"
                >
                  <Eye class="w-3 h-3 text-sky-400" />
                  <span>{{ $t('models.vision') }}</span>
                </span>

                <!-- Tool Calling Capability Badge -->
                <span
                  v-if="card.model.supports_tools"
                  class="px-2 py-0.5 rounded-md bg-emerald-500/10 border border-emerald-500/25 text-[10.5px] font-medium text-emerald-300 flex items-center gap-1"
                  :title="$t('models.tools_tooltip')"
                >
                  <Wrench class="w-3 h-3 text-emerald-400" />
                  <span>Tools</span>
                </span>
              </template>

              <!-- Local Models Badges -->
              <template v-else>
                <!-- Quantization -->
                <span class="px-2 py-0.5 rounded-md bg-[#151928] border border-[#21283e] text-[10.5px] font-mono text-indigo-300 font-medium">
                  {{ card.model.quantization }}
                </span>

                <!-- Clean Architecture Name -->
                <span class="px-2 py-0.5 rounded-md bg-[#151928] border border-[#21283e] text-[10.5px] text-slate-300 font-medium" :title="card.model.architecture">
                  {{ formatArchitecture(card.model.architecture) }}
                </span>

                <!-- Size in GB (Apenas para modelos locais baixados) -->
                <span
                  v-if="!isCloudModel(card.model) && card.model.size_gb !== undefined && card.model.size_gb > 0"
                  class="px-2 py-0.5 rounded-md bg-[#151928] border border-[#21283e] text-[10.5px] font-mono text-slate-400"
                >
                  {{ Number(card.model.size_gb).toFixed(1) }} GB
                </span>

                <!-- Last Loaded Badge (when not currently active) -->
                <span
                  v-if="lastLoadedId === card.model.id && !isCardActive(card)"
                  class="px-2 py-0.5 rounded-md bg-indigo-500/10 border border-indigo-500/25 text-[10.5px] font-medium text-indigo-300 flex items-center gap-1"
                  :title="$t('models.last_used_tooltip')"
                >
                  <History class="w-3 h-3 text-indigo-400" />
                  <span>{{ $t('models.last_used') }}</span>
                </span>

                <!-- Embedding Specific Badge -->
                <template v-if="isEmbeddingModel(card.model)">
                  <span
                    class="px-2 py-0.5 rounded-md bg-amber-500/10 border border-amber-500/25 text-[10.5px] font-medium text-amber-300 flex items-center gap-1"
                    :title="$t('models.embeddings_tooltip')"
                  >
                    <Layers class="w-3 h-3 text-amber-400" />
                    <span>Embeddings</span>
                  </span>
                </template>

                <!-- Generative LLM Capability Badges -->
                <template v-else>
                  <!-- Thinking / Reasoning Badge -->
                  <span
                    v-if="card.model.supports_thinking"
                    class="px-2 py-0.5 rounded-md bg-purple-500/10 border border-purple-500/25 text-[10.5px] font-medium text-purple-300 flex items-center gap-1"
                    :title="$t('models.thinking_tooltip')"
                  >
                    <Brain class="w-3 h-3 text-purple-400" />
                    <span>{{ $t('models.reasoning') }}</span>
                  </span>

                  <!-- Multimodal Vision Badge -->
                  <span
                    v-if="card.model.supports_vision"
                    class="px-2 py-0.5 rounded-md bg-sky-500/10 border border-sky-500/25 text-[10.5px] font-medium text-sky-300 flex items-center gap-1"
                    :title="$t('models.vision_tooltip')"
                  >
                    <Eye class="w-3 h-3 text-sky-400" />
                    <span>{{ $t('models.vision') }}</span>
                  </span>

                  <!-- Tool Calling Capability Badge -->
                  <span
                    v-if="card.model.supports_tools"
                    class="px-2 py-0.5 rounded-md bg-emerald-500/10 border border-emerald-500/25 text-[10.5px] font-medium text-emerald-300 flex items-center gap-1"
                    :title="$t('models.tools_tooltip')"
                  >
                    <Wrench class="w-3 h-3 text-emerald-400" />
                    <span>Tools</span>
                  </span>
                </template>
              </template>
            </div>

            <!-- Model Description -->
            <p class="text-xs text-slate-400 line-clamp-2 leading-relaxed pt-0.5">
              {{ getModelDescription(card.model) }}
            </p>

            <!-- Seletor de Esforço de Raciocínio (Low / Medium / High) para Modelos AGY com variantes -->
            <div
              v-if="card.isGroup && card.variants.length > 1"
              class="flex items-center justify-between gap-2 p-2 rounded-xl bg-[#141826]/90 border border-[#1e2338] shadow-inner mt-2.5"
            >
              <div class="flex items-center gap-1.5 text-xs text-slate-300 font-medium shrink-0">
                <Brain class="w-3.5 h-3.5 text-purple-400 shrink-0" />
                <span>{{ $t('models.reasoning_label') }}</span>
              </div>

              <select
                :value="getSelectedEffort(card)"
                @click.stop
                @change="onEffortChange(card, ($event.target as HTMLSelectElement).value)"
                class="min-w-0 bg-[#1a2034] hover:bg-[#202740] border border-[#27314d] hover:border-amber-500/50 text-xs font-semibold text-amber-300 rounded-lg px-2.5 py-1 outline-none cursor-pointer transition-all shadow-sm"
              >
                <option v-for="v in card.variants" :key="v.effort" :value="v.effort">
                  {{ v.label }} ({{ v.effort === 'high' ? $t('models.effort_high') : v.effort === 'medium' ? $t('models.effort_medium') : $t('models.effort_low') }})
                </option>
              </select>
            </div>
          </div>

          <!-- Card Bottom Actions -->
          <div class="pt-3 mt-3.5 border-t border-[#1c2236] flex items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0 shrink">
              <span class="text-[11px] font-mono text-slate-400 whitespace-nowrap shrink-0">
                Ctx: {{ formatContextLength(card.model) }}
              </span>

              <!-- Antigravity Quota Tag on Model Card -->
              <span
                v-if="isAgyModel(card.model) && getAgyQuota(card.model)"
                @click.stop="$emit('openAgyModal')"
                class="px-2 py-0.5 rounded-lg font-mono font-bold text-[10px] border cursor-pointer hover:brightness-125 transition-all whitespace-nowrap shrink-0 flex items-center gap-1"
                :class="[
                  (getAgyQuota(card.model)?.percent ?? 0) >= 50
                    ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
                    : (getAgyQuota(card.model)?.percent ?? 0) >= 20
                    ? 'bg-amber-500/15 text-amber-300 border-amber-500/30'
                    : 'bg-rose-500/15 text-rose-300 border-rose-500/30'
                ]"
                :title="getQuotaTooltip(card.model)"
              >
                {{ getAgyQuota(card.model)?.label }}: {{ getAgyQuota(card.model)?.percent }}%
              </span>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <!-- Configure Before Load Button (Apenas modelos locais) -->
              <button
                v-if="!isCardActive(card) && !isAgyModel(card.model)"
                @click="openModelConfigModal(card.model)"
                class="p-2 rounded-xl bg-[#141826] hover:bg-[#1c2236] border border-[#1e2338] text-slate-400 hover:text-indigo-300 transition-all cursor-pointer active:scale-95 shadow-sm"
                :title="$t('models.config_before_load_tooltip')"
              >
                <SlidersHorizontal class="w-3.5 h-3.5" />
              </button>

              <!-- Unload Button -->
              <button
                v-if="isCardActive(card) && !isLoadingModel"
                @click="$emit('unloadModel')"
                class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 border border-rose-500/30 text-xs font-semibold text-rose-300 transition-all active:scale-95 cursor-pointer shadow-sm"
              >
                <Power class="w-3 h-3 text-rose-400" />
                <span>{{ $t('models.unload_model') }}</span>
              </button>

              <!-- Loading Indicator Button -->
              <button
                v-else-if="isCardLoading(card)"
                disabled
                class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-amber-500/20 border border-amber-500/30 text-xs font-semibold text-amber-300 cursor-not-allowed shadow-inner"
              >
                <Loader2 class="w-3 h-3 animate-spin text-amber-400" />
                <span>{{ $t('models.loading_progress', { progress: loadingModelProgress }) }}</span>
              </button>

              <!-- Antigravity Instant Activate Button -->
              <button
                v-else-if="isAgyModel(card.model)"
                :disabled="isLoadingModel"
                @click="$emit('loadModel', card.model)"
                class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-slate-950 font-bold transition-all shadow-md shadow-amber-500/20 active:scale-95 cursor-pointer"
              >
                <Sparkles class="w-3 h-3 fill-slate-950 text-slate-950" />
                <span>{{ $t('models.activate') }}</span>
              </button>

              <!-- Standard Local Model Load Button -->
              <button
                v-else
                :disabled="isLoadingModel"
                @click="$emit('loadModel', card.model)"
                :class="[
                  'flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold transition-all shadow-md',
                  isLoadingModel
                    ? 'bg-[#151928] text-slate-500 cursor-not-allowed border border-[#1e2338]'
                    : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/20 active:scale-95 cursor-pointer'
                ]"
              >
                <Play class="w-3 h-3 fill-white" />
                <span>{{ $t('models.load_model') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Modal: Configure Model Boot Parameters Before Loading -->
    <Teleport to="body">
      <div
        v-if="selectedModelForConfig"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md transition-all select-none"
        @click.self="selectedModelForConfig = null"
      >
        <div class="bg-[#0e111a] border border-[#22283e] rounded-3xl max-w-lg w-full p-6 shadow-2xl space-y-5 text-slate-100 animate-in fade-in zoom-in-95 duration-150">
          <!-- Modal Header -->
          <div class="flex items-center justify-between border-b border-[#1c2236] pb-3.5">
            <div class="flex items-center gap-3">
              <div class="p-2.5 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 shadow-sm">
                <SlidersHorizontal class="w-5 h-5" />
              </div>
              <div>
                <div class="flex items-center gap-2">
                  <h3 class="font-bold text-slate-100 text-sm">{{ $t('models.config_modal_title') }}</h3>
                  <span
                    class="text-[10px] font-mono px-1.5 py-0.2 rounded border font-semibold"
                    :class="[
                      selectedModelForConfig.backend === 'MlxLm' || selectedModelForConfig.format === 'Mlx'
                        ? 'bg-purple-500/15 text-purple-300 border-purple-500/30'
                        : selectedModelForConfig.format === 'Gguf'
                          ? 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30'
                          : 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
                    ]"
                  >
                    {{ getModelFormatLabel(selectedModelForConfig) }}
                  </span>
                  <span v-if="selectedModelForConfig.quantization" class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-slate-800 text-slate-300 border border-slate-700">
                    {{ selectedModelForConfig.quantization }}
                  </span>
                </div>
                <p class="text-xs text-slate-400 mt-0.5 truncate max-w-xs font-semibold">{{ selectedModelForConfig.name }}</p>
              </div>
            </div>

            <button
              @click="selectedModelForConfig = null"
              class="p-2 rounded-xl bg-[#141826] hover:bg-[#1f253a] text-slate-400 hover:text-white transition-all cursor-pointer border border-[#23293f]"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Configuration Fields -->
          <div class="space-y-3.5 text-xs">
            <!-- 1. Quantização de KV Cache -->
            <div class="space-y-2 p-3.5 rounded-2xl bg-[#121626] border border-teal-500/30">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <Database class="w-4 h-4 text-teal-400" />
                  <div>
                    <div class="flex items-center gap-1.5">
                      <span class="font-bold text-slate-100 text-xs">{{ $t('models.kv_cache_quant_title') }}</span>
                      <span
                        class="text-[9px] px-1.5 py-0.2 rounded font-mono"
                        :class="[
                          selectedModelForConfig.format === 'Gguf' || (selectedModelForConfig.local_path && selectedModelForConfig.local_path.toLowerCase().endsWith('.gguf'))
                            ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20'
                            : selectedModelForConfig.backend === 'MlxLm' || selectedModelForConfig.format === 'Mlx'
                              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/20'
                              : 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'
                        ]"
                      >
                        {{ selectedModelForConfig.format === 'Gguf' || (selectedModelForConfig.local_path && selectedModelForConfig.local_path.toLowerCase().endsWith('.gguf')) ? 'GGUF / Metal' : selectedModelForConfig.backend === 'MlxLm' || selectedModelForConfig.format === 'Mlx' ? 'MLX / Metal' : 'Ollama' }}
                      </span>
                    </div>
                    <p class="text-[10.5px] text-teal-300/80">{{ $t('models.kv_cache_quant_desc') }}</p>
                  </div>
                </div>

                <!-- Switch Habilitar/Desabilitar -->
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(isModalKvQuantEnabled)"
                  @click="toggleModalKvQuant"
                  :class="[
                    'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    isModalKvQuantEnabled ? 'bg-teal-600' : 'bg-slate-700'
                  ]"
                  :title="$t('models.kv_cache_toggle_tooltip')"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      isModalKvQuantEnabled ? 'translate-x-4' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>

              <div v-if="isModalKvQuantEnabled" class="space-y-1.5 pt-2 border-t border-teal-500/20">
                <div class="flex justify-between items-center">
                  <label class="text-[11px] text-slate-300 font-medium">{{ $t('models.kv_cache_format_label') }}</label>
                  <span class="text-[10px] text-teal-300 font-mono font-semibold">{{ getKvSavingsLabel(params?.kv_cache_quant) }}</span>
                </div>
                <select
                  v-if="params"
                  v-model="params.kv_cache_quant"
                  class="w-full px-3 py-2 rounded-xl bg-[#0c0f18] border border-[#22283b] focus:border-teal-500 text-xs font-mono text-slate-100 outline-none cursor-pointer"
                >
                  <option value="q8_0" class="bg-[#0e111a]">Q8_0 (8-bit • 50% menos VRAM • Alta Fidelidade)</option>
                  <option value="q4_0" class="bg-[#0e111a]">Q4_0 (4-bit • 75% menos VRAM • Mais leve)</option>
                  <option value="q4_1" class="bg-[#0e111a]">Q4_1 (4-bit otimizado)</option>
                  <option value="q5_0" class="bg-[#0e111a]">Q5_0 (5-bit • Equilíbrio VRAM/Precisão)</option>
                  <option value="iq4_nl" class="bg-[#0e111a]">IQ4_NL (4-bit Não-Linear)</option>
                </select>
              </div>

              <div v-else class="pt-0.5">
                <p class="text-[10px] text-slate-400 italic">
                  {{ $t('models.kv_cache_disabled_hint') }}
                </p>
              </div>
            </div>

            <!-- 2. Janela de Contexto de Carregamento -->
            <div class="space-y-1.5 p-3.5 rounded-2xl bg-[#121626] border border-[#1e2336]">
              <div class="flex items-center justify-between mb-1">
                <label class="font-bold text-slate-100 flex items-center gap-1.5">
                  <Layers class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('models.context_window_label') }}</span>
                </label>
                <span class="text-xs font-mono font-bold text-indigo-300">{{ params?.context_length || 32768 }} tokens</span>
              </div>
              <div class="grid grid-cols-5 gap-1.5 pt-1">
                <button
                  v-for="ctx in [4096, 8192, 16384, 32768, 65536]"
                  :key="ctx"
                  type="button"
                  @click="params && (params.context_length = ctx)"
                  :class="[
                    'py-1.5 rounded-xl font-mono text-[10.5px] font-semibold border transition-all cursor-pointer',
                    params?.context_length === ctx
                      ? 'bg-indigo-600 text-white border-indigo-500 shadow-sm'
                      : 'bg-[#151928] text-slate-400 border-[#22283b] hover:text-white'
                  ]"
                >
                  {{ ctx >= 1024 ? Math.round(ctx / 1024) + 'k' : ctx }}
                </button>
              </div>
            </div>

            <!-- 3. Flash Attention & Prompt Cache Toggles -->
            <div class="grid grid-cols-2 gap-2">
              <button
                type="button"
                @click="params && (params.flash_attention = params.flash_attention === false ? true : false)"
                :class="[
                  'p-3 rounded-2xl border text-left flex flex-col justify-between gap-1 transition-all cursor-pointer',
                  params?.flash_attention !== false
                    ? 'bg-indigo-600/15 border-indigo-500/40 text-indigo-200'
                    : 'bg-[#121522] border-[#1e2336] text-slate-400'
                ]"
              >
                <span class="font-bold text-xs flex items-center gap-1.5">
                  <Cpu class="w-3.5 h-3.5 text-indigo-400" />
                  Flash Attention
                </span>
                <span class="text-[9.5px] text-slate-400">{{ $t('models.flash_attention_desc') }}</span>
                <span class="text-[10px] font-mono font-bold" :class="params?.flash_attention !== false ? 'text-indigo-300' : 'text-slate-500'">
                  {{ params?.flash_attention !== false ? '● ' + $t('common.enabled') : '○ ' + $t('common.disabled') }}
                </span>
              </button>

              <button
                type="button"
                @click="params && (params.enable_prompt_cache = params.enable_prompt_cache === false ? true : false)"
                :class="[
                  'p-3 rounded-2xl border text-left flex flex-col justify-between gap-1 transition-all cursor-pointer',
                  params?.enable_prompt_cache !== false
                    ? 'bg-teal-600/15 border-teal-500/40 text-teal-200'
                    : 'bg-[#121522] border-[#1e2336] text-slate-400'
                ]"
              >
                <span class="font-bold text-xs flex items-center gap-1.5">
                  <Sparkles class="w-3.5 h-3.5 text-teal-400" />
                  Prompt Cache
                </span>
                <span class="text-[9.5px] text-slate-400">{{ $t('models.prompt_cache_desc') }}</span>
                <span class="text-[10px] font-mono font-bold" :class="params?.enable_prompt_cache !== false ? 'text-teal-300' : 'text-slate-500'">
                  {{ params?.enable_prompt_cache !== false ? '● ' + $t('common.enabled') : '○ ' + $t('common.disabled') }}
                </span>
              </button>
            </div>
          </div>

          <!-- Modal Footer -->
          <div class="border-t border-[#1c2236] pt-3 flex items-center justify-between">
            <button
              @click="selectedModelForConfig = null"
              class="px-4 py-2 rounded-xl bg-[#141826] hover:bg-[#1d2338] text-slate-400 hover:text-white text-xs font-semibold transition-all cursor-pointer"
            >
              {{ $t('common.cancel') }}
            </button>

            <button
              @click="loadModelWithCurrentConfig"
              class="px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white text-xs font-bold shadow-lg shadow-indigo-600/30 transition-all cursor-pointer active:scale-95 flex items-center gap-2"
            >
              <Play class="w-3.5 h-3.5 fill-white" />
              <span>{{ $t('models.load_model_now') }}</span>
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <div
        v-if="modelToDelete"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md transition-all select-none"
        @click.self="modelToDelete = null"
      >
        <div class="bg-[#0f121d] border border-rose-500/40 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-4 text-slate-100 animate-in fade-in zoom-in-95 duration-150">
          <!-- Modal Top -->
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-2xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400 shadow-sm flex-shrink-0">
              <Trash2 class="w-5 h-5" />
            </div>
            <div>
              <h3 class="text-base font-bold text-slate-100">{{ $t('models.delete_modal_title') }}</h3>
              <p class="text-xs text-rose-300/80">{{ $t('models.delete_modal_irreversible') }}</p>
            </div>
          </div>

          <!-- Model Details Info Box -->
          <div class="p-3.5 rounded-2xl bg-[#14192b] border border-[#212942] space-y-2 text-xs">
            <div class="flex items-start justify-between gap-2">
              <span class="font-semibold text-slate-200 truncate">{{ modelToDelete.name }}</span>
              <span class="font-mono text-rose-300 font-bold flex-shrink-0">{{ Number(modelToDelete.size_gb).toFixed(1) }} GB</span>
            </div>
            <div class="flex items-center gap-2 text-[11px] text-slate-400">
              <span>{{ $t('models.delete_modal_format') }} <strong class="text-slate-200 font-mono">{{ getModelFormatLabel(modelToDelete) }}</strong></span>
              <span>•</span>
              <span>{{ $t('models.delete_modal_quantization') }} <strong class="text-slate-200 font-mono">{{ modelToDelete.quantization }}</strong></span>
            </div>
            <div v-if="modelToDelete.local_path" class="text-[10.5px] text-slate-500 font-mono truncate flex items-center gap-1 pt-1 border-t border-[#1e253c]">
              <FolderOpen class="w-3 h-3 flex-shrink-0 text-slate-600" />
              <span class="truncate" :title="contractUserPath(modelToDelete.local_path)">{{ contractUserPath(modelToDelete.local_path) }}</span>
            </div>
          </div>

          <!-- Warning Message -->
          <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/25 text-rose-300 text-xs flex items-start gap-2.5 leading-relaxed">
            <AlertTriangle class="w-4 h-4 text-rose-400 flex-shrink-0 mt-0.5" />
            <span>{{ $t('models.delete_modal_warning') }}</span>
          </div>

          <!-- Modal Actions -->
          <div class="pt-2 flex items-center justify-end gap-2.5">
            <button
              @click="modelToDelete = null"
              :disabled="isDeletingModel"
              class="px-4 py-2 rounded-xl bg-[#141826] hover:bg-[#1f263d] text-slate-300 hover:text-white text-xs font-semibold transition-all cursor-pointer disabled:opacity-50"
            >
              {{ $t('common.cancel') }}
            </button>

            <button
              @click="executeDeleteModel"
              :disabled="isDeletingModel"
              class="px-4 py-2 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-bold flex items-center gap-1.5 transition-all shadow-md shadow-rose-600/30 active:scale-95 cursor-pointer disabled:opacity-50"
            >
              <Loader2 v-if="isDeletingModel" class="w-3.5 h-3.5 animate-spin" />
              <Trash2 v-else class="w-3.5 h-3.5" />
              <span>{{ isDeletingModel ? $t('models.deleting') : $t('models.confirm_delete') }}</span>
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Layers,
  RefreshCw,
  Search,
  X,
  Box,
  Play,
  Power,
  Loader2,
  FolderOpen,
  Download,
  Brain,
  Eye,
  FileText,
  Wrench,
  AlertTriangle,
  Zap,
  SlidersHorizontal,
  Sparkles,
  Cpu,
  Database,
  Star,
  Trash2,
  Terminal,
  History,
  Cloud,
  Blocks,
  EyeOff
} from 'lucide-vue-next'
import { addNotification } from '~/utils/notifications'
import { contractUserPath } from '~/utils/pathUtils'
import { resolveModelLogo } from '../../composables/useModelLogo'
import { useAppLocale } from '../../composables/useLocale'
import type { ModelInfo, AppConfig, GenerationParams } from '~/types'

interface Props {
  models?: ModelInfo[]
  allModels?: ModelInfo[]
  activeModelId?: string | null
  loadingModelId?: string | null
  isLoadingModel?: boolean
  loadingModelProgress?: number
  modelsDir?: string
  isScanning?: boolean
  activeDownloadsCount?: number
  agyUsageSummary?: any
  params?: GenerationParams | null
  config?: AppConfig | null
}

const props = withDefaults(defineProps<Props>(), {
  models: () => [],
  allModels: () => [],
  activeModelId: null,
  loadingModelId: null,
  isLoadingModel: false,
  loadingModelProgress: 0,
  modelsDir: '',
  isScanning: false,
  activeDownloadsCount: 0,
  agyUsageSummary: null,
  params: () => ({
    kv_cache_quant: 'q8_0',
    context_length: 8192,
    enable_prompt_cache: true,
    flash_attention: true
  }),
  config: () => ({})
})

const emit = defineEmits<{
  loadModel: [model: ModelInfo]
  unloadModel: []
  rescanModels: []
  openHfModal: []
  selectFolder: []
  openAgyModal: []
  openSettings: [tab?: string]
  hideModel: [model: ModelInfo]
}>()

const { t } = useAppLocale()
const supportsMlx = inject<Ref<boolean> | ComputedRef<boolean>>('supportsMlx', computed(() => true))

const searchQuery = ref('')
const activeFilter = ref('all')
const selectedModelForConfig = ref<ModelInfo | null>(null)

// Resolves localized descriptions for standard model IDs or patterns, falling back to original description
const getModelDescription = (model?: ModelInfo | null): string => {
  if (!model) return t('models.default_description')
  const id = (model.id || '').toLowerCase()

  // 1. Direct model ID matches
  if (id === 'agy/gemini-3.8-flash-medium') return t('models.desc.gemini_3_8_flash_medium')
  if (id === 'agy/gemini-3.8-flash-high') return t('models.desc.gemini_3_8_flash_high')
  if (id === 'agy/claude-sonnet-4-6') return t('models.desc.claude_sonnet_4_6')
  if (id === 'openai/gpt-4o') return t('models.desc.gpt_4o')
  if (id === 'openai/gpt-4o-mini') return t('models.desc.gpt_4o_mini')
  if (id === 'openai/o3-mini') return t('models.desc.o3_mini')
  if (id === 'openrouter/anthropic/claude-3.5-sonnet') return t('models.desc.claude_3_5_sonnet')
  if (id === 'openrouter/deepseek/deepseek-r1') return t('models.desc.deepseek_r1')
  if (id === 'gemini/gemini-2.5-pro') return t('models.desc.gemini_2_5_pro')
  if (id === 'gemini/gemini-2.5-flash') return t('models.desc.gemini_2_5_flash')
  if (id === 'gemini/gemini-2.5-flash-lite') return t('models.desc.gemini_2_5_flash_lite')
  if (id === 'gemini/gemini-1.5-pro') return t('models.desc.gemini_1_5_pro')
  if (id === 'gemini/gemini-1.5-flash') return t('models.desc.gemini_1_5_flash')

  // 2. Pattern matches for dynamically scanned models
  if (id.startsWith('agy/')) {
    if (id.includes('3.8')) return t('models.desc.gemini_3_8')
    if (id.includes('flash')) return t('models.desc.gemini_flash')
    if (id.includes('pro')) return t('models.desc.gemini_pro')
    if (id.includes('claude')) return t('models.desc.claude_coding')
    if (id.includes('gpt')) return t('models.desc.gpt_scale')
    return t('models.desc.agy_default')
  }

  if (id.startsWith('gemini/')) {
    const raw = model.id.replace(/^gemini\//i, '')
    return t('models.desc.gemini_cloud_default', { id: raw })
  }

  if (id.startsWith('openai/')) {
    const raw = model.id.replace(/^openai\//i, '')
    return t('models.desc.openai_cloud_default', { id: raw })
  }

  if (id.startsWith('openrouter/')) {
    return model.description || t('models.desc.openrouter_default')
  }

  if (id.startsWith('ollama/')) {
    return t('models.desc.ollama_default')
  }

  if (model.format === 'Mlx' || model.backend === 'MlxLm') {
    return t('models.desc.mlx_default')
  }

  if (model.format === 'Gguf') {
    return t('models.desc.gguf_default')
  }

  if (id.startsWith('custom/')) {
    return t('models.desc.custom_cloud_default', { url: model.local_path || 'endpoint' })
  }

  return model.description || t('models.default_description')
}

const getModelFormatLabel = (model?: ModelInfo | null): string => {
  if (!model) return 'LOCAL'
  if (model.backend === 'Antigravity' || model.format === 'Agy' || (model.id && model.id.startsWith('agy/'))) return 'AGY'
  if (model.id && model.id.startsWith('gemini/')) return 'GEMINI'
  if (model.id && model.id.startsWith('groq/')) return 'GROQ'
  if (model.id && model.id.startsWith('openai/')) return 'OPENAI'
  if (model.id && model.id.startsWith('openrouter/')) return 'OPENROUTER'
  if (model.id && model.id.startsWith('custom/')) return 'CUSTOM'
  if (model.backend === 'CloudOpenAi' || model.format === 'Cloud') return 'CLOUD'
  if (model.backend === 'MlxLm' || model.format === 'Mlx') return 'MLX'
  if (model.backend === 'Ollama' || model.format === 'Ollama') return 'OLLAMA'
  if (model.format === 'Gguf' || (model.local_path && model.local_path.toLowerCase().endsWith('.gguf'))) return 'GGUF'
  return model.backend || model.format || 'LOCAL'
}

// Clean up messy raw architecture strings
const formatArchitecture = (arch?: string | null): string => {
  if (!arch) return 'LLM'
  const clean = arch
    .replace(/forconditionalgeneration/gi, '')
    .replace(/forcausallm/gi, '')
    .replace(/unified/gi, '')
    .replace(/conditional/gi, '')
    .replace(/model/gi, '')
    .replace(/_/g, ' ')
    .trim()
  
  const lower = clean.toLowerCase()
  if (lower.includes('gemma 4') || lower === 'gemma4') return 'Gemma 4'
  if (lower.includes('gemma 2') || lower === 'gemma2') return 'Gemma 2'
  if (lower.includes('gemma')) return 'Gemma'
  if (lower.includes('qwen 3.5') || lower.includes('qwen3.5') || lower.includes('qwen3 5') || lower === 'qwen35') return 'Qwen 3.5'
  if (lower.includes('qwen 2.5') || lower.includes('qwen2.5') || lower.includes('qwen2 5') || lower === 'qwen25') return 'Qwen 2.5'
  if (lower.includes('qwen 2') || lower === 'qwen2') return 'Qwen 2'
  if (lower.includes('qwen')) return 'Qwen'
  if (lower.includes('llama 3.3') || lower.includes('llama3.3') || lower.includes('llama3 3')) return 'Llama 3.3'
  if (lower.includes('llama 3.2') || lower.includes('llama3.2') || lower.includes('llama3 2')) return 'Llama 3.2'
  if (lower.includes('llama 3.1') || lower.includes('llama3.1') || lower.includes('llama3 1')) return 'Llama 3.1'
  if (lower.includes('llama 3') || lower === 'llama3') return 'Llama 3'
  if (lower.includes('llama')) return 'Llama'
  if (lower.includes('deepseek')) return 'DeepSeek'
  if (lower.includes('mistral')) return 'Mistral'
  if (lower.includes('mixtral')) return 'Mixtral'
  if (lower.includes('phi 4') || lower === 'phi4') return 'Phi-4'
  if (lower.includes('phi 3') || lower === 'phi3') return 'Phi-3'
  if (lower.includes('phi')) return 'Phi'
  if (lower.includes('starcoder')) return 'StarCoder'
  if (lower.includes('command')) return 'Command'
  if (lower.includes('bert') || lower.includes('bge')) return 'Embedding'
  
  if (clean.length > 18) {
    return clean.slice(0, 16) + '...'
  }
  return clean.charAt(0).toUpperCase() + clean.slice(1)
}

// Favorites Management
const FAVORITES_KEY = 'atena_favorite_models'
const LAST_LOADED_KEY = 'atena_last_loaded_model_id'
const LOADED_TIMES_KEY = 'atena_model_last_loaded_times'

const loadFavorites = (): string[] => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const raw = localStorage.getItem(FAVORITES_KEY)
      if (raw) return JSON.parse(raw)
    }
  } catch (e) {
    console.error('Falha ao carregar favoritos:', e)
  }
  return []
}

const loadLoadedTimes = (): Record<string, number> => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const raw = localStorage.getItem(LOADED_TIMES_KEY)
      if (raw) return JSON.parse(raw)
    }
  } catch (e) {
    console.error('Falha ao carregar histórico de modelos:', e)
  }
  return {}
}

const getLastLoadedId = (): string | null => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      return localStorage.getItem(LAST_LOADED_KEY) || localStorage.getItem('atena_active_model_id') || null
    }
  } catch (e) {}
  return null
}

const favoriteModelIds = ref<string[]>(loadFavorites())
const loadedTimes = ref<Record<string, number>>(loadLoadedTimes())
const lastLoadedId = ref<string | null>(getLastLoadedId())

watch(
  () => props.activeModelId,
  (newId) => {
    if (newId) {
      lastLoadedId.value = newId
      const updated = {
        ...loadedTimes.value,
        [newId]: Date.now()
      }
      loadedTimes.value = updated
      try {
        localStorage.setItem(LAST_LOADED_KEY, newId)
        localStorage.setItem(LOADED_TIMES_KEY, JSON.stringify(updated))
      } catch (e) {}
    }
  },
  { immediate: true }
)

const isFavorite = (id?: string | null): boolean => {
  if (!id) return false
  return favoriteModelIds.value.includes(id)
}

const toggleFavorite = (model: ModelInfo) => {
  const id = model.id
  if (isFavorite(id)) {
    favoriteModelIds.value = favoriteModelIds.value.filter((fid) => fid !== id)
    addNotification({
      type: 'info',
      title: t('models.favorite_removed_title'),
      message: t('models.favorite_removed_msg', { name: model.name })
    })
  } else {
    favoriteModelIds.value.push(id)
    addNotification({
      type: 'model_favorited',
      title: t('models.favorite_added_title'),
      message: t('models.favorite_added_msg', { name: model.name })
    })
  }
  try {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(favoriteModelIds.value))
  } catch (e) {
    console.error('Erro ao salvar favoritos:', e)
  }
}

// Delete Model Management
const modelToDelete = ref<ModelInfo | null>(null)
const isDeletingModel = ref(false)

const openDeleteConfirm = (model: ModelInfo) => {
  modelToDelete.value = model
}

const executeDeleteModel = async () => {
  if (!modelToDelete.value) return
  const model = modelToDelete.value
  isDeletingModel.value = true

  try {
    await invoke('delete_model', {
      modelId: model.id,
      localPath: model.local_path || null,
      format: model.format ? String(model.format) : null,
      backend: model.backend ? String(model.backend) : null
    })

    // If it was favorite, clean up
    if (isFavorite(model.id)) {
      favoriteModelIds.value = favoriteModelIds.value.filter((fid) => fid !== model.id)
      localStorage.setItem(FAVORITES_KEY, JSON.stringify(favoriteModelIds.value))
    }

    addNotification({
      type: 'model_deleted',
      title: t('models.model_deleted_title'),
      message: t('models.model_deleted_msg', { name: model.name })
    })

    modelToDelete.value = null
    emit('rescanModels')
  } catch (err) {
    console.error('Erro ao excluir modelo:', err)
    alert(`Falha ao excluir modelo: ${err}`)
  } finally {
    isDeletingModel.value = false
  }
}

const isModalKvQuantEnabled = computed(() => {
  const q = (props.params?.kv_cache_quant || '').toLowerCase()
  return q !== '' && q !== 'f16' && q !== 'fp16' && q !== 'none'
})

const toggleModalKvQuant = () => {
  if (!props.params) return
  if (isModalKvQuantEnabled.value) {
    props.params.kv_cache_quant = 'f16'
  } else {
    props.params.kv_cache_quant = 'q8_0'
  }
}

const openModelConfigModal = (model: ModelInfo) => {
  selectedModelForConfig.value = model
}

const loadModelWithCurrentConfig = () => {
  if (!selectedModelForConfig.value) return
  const modelToLoad = selectedModelForConfig.value
  selectedModelForConfig.value = null
  emit('loadModel', modelToLoad)
}

const getKvSavingsLabel = (quant?: string | null): string => {
  const q = quant || 'f16'
  switch (q.toLowerCase()) {
    case 'q4_0':
    case 'q4_1':
    case 'iq4_nl':
      return '75% Economia VRAM'
    case 'q5_0':
      return '68% Economia VRAM'
    case 'q8_0':
      return '50% Economia VRAM'
    default:
      return '0% Economia (F16)'
  }
}

// --- Filter Groups ---
const primaryFilters = computed(() => [
  { id: 'all', label: t('models.filter_all'), icon: Layers, iconColor: 'text-indigo-400' },
  { id: 'favorites', label: t('models.filter_favorites'), icon: Star, iconColor: 'text-amber-400' },
  { id: 'cloud', label: t('models.filter_cloud'), icon: Cloud, iconColor: 'text-sky-400' },
])

const formatFiltersDef = [
  { id: 'mlx', label: 'MLX', icon: Cpu, iconColor: 'text-purple-400' },
  { id: 'gguf', label: 'GGUF', icon: Box, iconColor: 'text-sky-400' },
  { id: 'ollama', label: 'Ollama', icon: Terminal, iconColor: 'text-emerald-400' },
]

const capabilityFilters = computed(() => [
  { id: 'tools', label: t('models.filter_tools'), icon: Wrench, iconColor: 'text-emerald-400' },
  { id: 'thinking', label: t('models.filter_thinking'), icon: Brain, iconColor: 'text-purple-400' },
  { id: 'vision', label: t('models.filter_vision'), icon: Eye, iconColor: 'text-sky-400' },
])

const formatFilters = computed(() => {
  if (supportsMlx.value) {
    return formatFiltersDef
  }
  return formatFiltersDef.filter(f => f.id !== 'mlx')
})

// Combined for any legacy usage
const filters = computed(() => {
  return [...primaryFilters.value, ...formatFilters.value, ...capabilityFilters.value]
})

const isAgyModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  return m.backend === 'Antigravity' || m.format === 'Agy' || Boolean(m.id && m.id.startsWith('agy/'))
}

const isCloudModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  return (
    isAgyModel(m) ||
    m.backend === 'Cloud' ||
    m.backend === 'CloudOpenAi' ||
    m.format === 'Cloud' ||
    Boolean(m.id && (m.id.startsWith('cloud/') || m.id.startsWith('agy/') || m.id.startsWith('gemini/') || m.id.startsWith('groq/') || m.id.startsWith('openai/') || m.id.startsWith('openrouter/') || m.id.startsWith('custom/')))
  )
}

const dynamicCatalogAvatars = ref<Map<string, string | null>>(new Map())

const allModelsList = computed(() => (props.allModels && props.allModels.length > 0 ? props.allModels : props.models || []))

const fetchCatalogAvatars = async () => {
  const list = allModelsList.value
  if (!list || list.length === 0) return
  const authorsToFetch = [
    ...new Set(
      list
        .filter((m) => m.author && m.author !== 'Local' && m.author !== 'models' && !m.author_avatar_url)
        .map((m) => m.author!)
        .filter((a) => a && !dynamicCatalogAvatars.value.has(a.toLowerCase()))
    )
  ]
  if (authorsToFetch.length === 0) return

  authorsToFetch.forEach(async (author) => {
    try {
      const avatarUrl = await invoke<string>('hf_get_author_avatar', { author })
      dynamicCatalogAvatars.value.set(author.toLowerCase(), avatarUrl)
    } catch {
      dynamicCatalogAvatars.value.set(author.toLowerCase(), null)
    }
  })
}

watch(
  allModelsList,
  () => {
    fetchCatalogAvatars()
  },
  { immediate: true }
)

const getModelLogo = (model: ModelInfo) => {
  return resolveModelLogo(model, dynamicCatalogAvatars.value)
}

const formatContextLength = (model?: ModelInfo | null): string => {
  if (!model) return '32k'
  if (isAgyModel(model)) {
    if (model.context_length && model.context_length >= 1000000) {
      return `${Math.round(model.context_length / 1000000)}M`
    }
    return model.context_length ? `${Math.round(model.context_length / 1000)}k` : '32k'
  }
  if (props.params?.context_length) {
    return `${Math.round(props.params.context_length / 1024)}k`
  }
  return model?.context_length ? `${Math.round(model.context_length / 1024)}k` : '32k'
}

const isEmbeddingModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  const lower = `${m.id || ''} ${m.name || ''} ${m.architecture || ''}`.toLowerCase()
  return lower.includes('embed') || lower.includes('embedding') || lower.includes('bge-') || lower.includes('all-minilm')
}

const isMlxModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  return m.backend === 'MlxLm' || m.format === 'Mlx'
}

const isGgufModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  return m.format === 'Gguf' || Boolean(m.local_path && m.local_path.toLowerCase().endsWith('.gguf'))
}

const isOllamaModel = (m?: ModelInfo | null): boolean => {
  if (!m) return false
  return m.backend === 'Ollama' || m.format === 'Ollama'
}

const filteredModels = computed(() => {
  // Step 1: Strict Category & Search Filtering
  const filtered = (props.models || []).filter((m) => {
    const query = searchQuery.value.trim().toLowerCase()
    const matchesSearch =
      !query ||
      (m.name && m.name.toLowerCase().includes(query)) ||
      (m.architecture && m.architecture.toLowerCase().includes(query)) ||
      (m.author && m.author.toLowerCase().includes(query)) ||
      (m.quantization && m.quantization.toLowerCase().includes(query))

    const matchesFilter =
      activeFilter.value === 'all' ||
      (activeFilter.value === 'favorites' && isFavorite(m.id)) ||
      (activeFilter.value === 'cloud' && isCloudModel(m)) ||
      (activeFilter.value === 'agy' && isAgyModel(m)) ||
      (activeFilter.value === 'mlx' && isMlxModel(m)) ||
      (activeFilter.value === 'gguf' && isGgufModel(m)) ||
      (activeFilter.value === 'ollama' && isOllamaModel(m)) ||
      (activeFilter.value === 'tools' && !!m.supports_tools) ||
      (activeFilter.value === 'thinking' && !!m.supports_thinking) ||
      (activeFilter.value === 'vision' && !!m.supports_vision)

    return matchesSearch && matchesFilter
  })

  // Step 2: Sort within the filtered list
  return filtered.slice().sort((a, b) => {
    // 1. Active model gets top priority
    const aIsActive = props.activeModelId === a.id
    const bIsActive = props.activeModelId === b.id
    if (aIsActive && !bIsActive) return -1
    if (!aIsActive && bIsActive) return 1

    // 2. Favorite status
    const aFav = isFavorite(a.id)
    const bFav = isFavorite(b.id)
    if (aFav && !bFav) return -1
    if (!aFav && bFav) return 1

    // 3. Last loaded timestamps / last loaded ID
    const aTime = loadedTimes.value[a.id] || (lastLoadedId.value === a.id ? 1 : 0)
    const bTime = loadedTimes.value[b.id] || (lastLoadedId.value === b.id ? 1 : 0)
    if (aTime !== bTime) {
      return bTime - aTime
    }

    // 4. Alphabetical fallback
    return (a.name || '').localeCompare(b.name || '', undefined, { sensitivity: 'base', numeric: true })
  })
})

const selectedEffortMap = ref<Record<string, string>>({})

const parseAgyBase = (model?: ModelInfo | null) => {
  if (!model || !isAgyModel(model)) return null
  const id = model.id || ''
  for (const eff of ['high', 'medium', 'low']) {
    if (id.endsWith(`-${eff}`)) {
      const baseId = id.slice(0, -(eff.length + 1))
      const baseName = (model.name || '')
        .replace(/\s*\((High|Medium|Low)\)/i, '')
        .trim()
      return { baseId, baseName, effort: eff }
    }
  }
  return null
}

interface ModelVariant {
  effort: string
  label: string
  model: ModelInfo
}

export interface DisplayCard {
  key: string
  isGroup: boolean
  baseId?: string
  baseName?: string
  variants: ModelVariant[]
  model: ModelInfo
}

const displayModels = computed<DisplayCard[]>(() => {
  const list = filteredModels.value
  const result: { isGroup: boolean; baseId?: string; baseName?: string; variants?: ModelVariant[]; model?: ModelInfo }[] = []
  const groupMap = new Map<string, { isGroup: true; baseId: string; baseName: string; variants: ModelVariant[] }>()

  for (const model of list) {
    const agyInfo = parseAgyBase(model)
    if (agyInfo) {
      if (!groupMap.has(agyInfo.baseId)) {
        const entry = {
          isGroup: true as const,
          baseId: agyInfo.baseId,
          baseName: agyInfo.baseName,
          variants: [] as ModelVariant[]
        }
        groupMap.set(agyInfo.baseId, entry)
        result.push(entry)
      }
      const entry = groupMap.get(agyInfo.baseId)!
      entry.variants.push({
        effort: agyInfo.effort,
        label: agyInfo.effort === 'high' ? 'High' : agyInfo.effort === 'medium' ? 'Medium' : 'Low',
        model
      })
    } else {
      result.push({
        isGroup: false,
        model
      })
    }
  }

  // Ordena as variantes dentro de cada grupo na ordem padrão: High, Medium, Low
  const effortOrder: Record<string, number> = { high: 1, medium: 2, low: 3 }
  for (const entry of groupMap.values()) {
    entry.variants.sort((a, b) => (effortOrder[a.effort] || 99) - (effortOrder[b.effort] || 99))
  }

  return result.map((item) => {
    if (!item.isGroup || !item.model) {
      if (!item.isGroup && item.model) {
        return {
          key: item.model.id,
          isGroup: false,
          baseName: item.model.name,
          variants: [],
          model: item.model
        }
      }
    }

    const variants = item.variants || []
    const activeVar = variants.find((v) => v.model.id === props.activeModelId)
    const userEffort = item.baseId ? selectedEffortMap.value[item.baseId] : undefined
    const chosenVar = activeVar
      || (userEffort ? variants.find((v) => v.effort === userEffort) : null)
      || variants.find((v) => v.effort === 'medium')
      || variants.find((v) => v.effort === 'high')
      || variants[0]

    return {
      key: item.baseId || (chosenVar ? chosenVar.model.id : 'unknown'),
      isGroup: true,
      baseId: item.baseId,
      baseName: item.baseName,
      variants,
      model: (chosenVar ? chosenVar.model : variants[0]?.model || item.model)!
    }
  })
})

const isCardActive = (card: DisplayCard): boolean => {
  if (card.isGroup) {
    return card.variants.some((v) => v.model.id === props.activeModelId)
  }
  return props.activeModelId === card.model.id
}

const isCardLoading = (card: DisplayCard): boolean => {
  if (card.isGroup) {
    return card.variants.some((v) => v.model.id === props.loadingModelId)
  }
  return props.loadingModelId === card.model.id
}

const isCardFavorite = (card: DisplayCard): boolean => {
  if (card.isGroup) {
    return card.variants.some((v) => isFavorite(v.model.id))
  }
  return isFavorite(card.model.id)
}

const toggleCardFavorite = (card: DisplayCard) => {
  toggleFavorite(card.model)
}

const getSelectedEffort = (card: DisplayCard): string => {
  const agyInfo = parseAgyBase(card.model)
  return agyInfo?.effort || 'medium'
}

const onEffortChange = (card: DisplayCard, newEffort: string) => {
  if (!card.baseId) return
  selectedEffortMap.value[card.baseId] = newEffort
  const variant = card.variants.find((v) => v.effort === newEffort)
  if (!variant) return

  const isGroupActive = card.variants.some((v) => v.model.id === props.activeModelId)
  if (isGroupActive) {
    emit('loadModel', variant.model)
  }
}

interface QuotaPriority {
  percent: number
  label: string
  countdown: string
}

const resolveQuotaPriority = (
  percent5h?: number | null,
  countdown5h?: string | null,
  percentWeekly?: number | null,
  countdownWeekly?: string | null
): QuotaPriority | null => {
  const has5h = percent5h !== null && percent5h !== undefined
  const hasWeekly = percentWeekly !== null && percentWeekly !== undefined

  if (has5h && hasWeekly) {
    if (percent5h <= percentWeekly) {
      return { percent: percent5h, label: t('models.quota_5h'), countdown: countdown5h || '' }
    } else {
      return { percent: percentWeekly, label: t('models.quota_weekly'), countdown: countdownWeekly || '' }
    }
  }
  if (hasWeekly) {
    return { percent: percentWeekly, label: t('models.quota_weekly'), countdown: countdownWeekly || '' }
  }
  if (has5h) {
    return { percent: percent5h, label: t('models.quota_5h'), countdown: countdown5h || '' }
  }
  return null
}

const getQuotaTooltip = (model?: ModelInfo | null): string => {
  if (!model) return ''
  const q = getAgyQuota(model)
  if (!q) return ''
  const countdownText = q.countdown ? t('models.quota_renews_in', { time: q.countdown }) : ''
  return t('models.quota_tooltip', {
    label: q.label,
    percent: q.percent,
    countdown: countdownText
  })
}

const getAgyQuota = (model?: ModelInfo | null): QuotaPriority | null => {
  if (!props.agyUsageSummary || !model || !isAgyModel(model)) return null
  const s = props.agyUsageSummary
  const text = `${model.id || ''} ${model.name || ''}`.toLowerCase()
  const isClaudeOrGpt = text.includes('claude') || text.includes('gpt')

  if (isClaudeOrGpt) {
    const q = resolveQuotaPriority(s.claude5hPercent, s.claude5hCountdown || s.lowest5hCountdown, s.claudeWeeklyPercent, s.claudeWeeklyCountdown || s.lowestWeeklyCountdown)
    if (q) return q
  } else {
    const q = resolveQuotaPriority(s.gemini5hPercent, s.gemini5hCountdown || s.lowest5hCountdown, s.geminiWeeklyPercent, s.geminiWeeklyCountdown || s.lowestWeeklyCountdown)
    if (q) return q
  }

  return resolveQuotaPriority(s.fiveHourPercent, s.lowest5hCountdown, s.weeklyPercent, s.lowestWeeklyCountdown)
}

const topAgyQuota = computed<QuotaPriority | null>(() => {
  if (!props.agyUsageSummary || !props.agyUsageSummary.authenticated) return null

  // 1. Se houver modelo ativo que seja da nuvem AGY, reflete a cota do modelo em uso
  if (props.activeModelId && props.models) {
    const activeM = props.models.find((m) => m.id === props.activeModelId)
    if (activeM && isAgyModel(activeM)) {
      const q = getAgyQuota(activeM)
      if (q) return q
    }
  }

  // 2. Se nenhum modelo AGY estiver ativo no momento, prioriza a cota do Gemini
  const s = props.agyUsageSummary
  const geminiQ = resolveQuotaPriority(s.gemini5hPercent, s.gemini5hCountdown, s.geminiWeeklyPercent, s.geminiWeeklyCountdown)
  if (geminiQ) return geminiQ

  const claudeQ = resolveQuotaPriority(s.claude5hPercent, s.claude5hCountdown, s.claudeWeeklyPercent, s.claudeWeeklyCountdown)
  if (claudeQ) return claudeQ

  // 3. Fallbacks gerais
  return resolveQuotaPriority(s.fiveHourPercent, s.lowest5hCountdown, s.weeklyPercent, s.lowestWeeklyCountdown)
})
</script>



