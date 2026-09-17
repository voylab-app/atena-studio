<template>
  <div class="space-y-6">
    <!-- Header e Introdução -->
    <div class="flex items-center justify-between gap-4">
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
          <Cloud class="w-4 h-4 text-sky-400 flex-shrink-0" />
          <span class="truncate">{{ $t('cloud.title') }}</span>
        </h3>
        <p class="text-xs text-slate-400 mt-0.5">
          {{ $t('cloud.subtitle') }}
        </p>
      </div>

      <div class="flex items-center gap-2 flex-shrink-0">
        <button
          type="button"
          @click="saveProviders"
          :class="[
            'px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95',
            isSaved ? 'bg-emerald-600 text-white shadow-emerald-600/30' : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/30'
          ]"
        >
          <component :is="isSaved ? Check : Save" class="w-3.5 h-3.5" />
          <span>{{ isSaved ? $t('cloud.saved') : $t('cloud.save_changes') }}</span>
        </button>
      </div>
    </div>

    <!-- Summary Stats Strip -->
    <div class="flex items-center justify-between px-3.5 py-2.5 rounded-2xl bg-[#0e101a] border border-[#1b2033] text-xs text-slate-400 flex-wrap gap-2 shadow-sm">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
        <span class="text-slate-200 font-semibold">
          {{ $t('cloud.summary_providers_active', { active: totalActiveProviders, total: providerTabs.length }) }}
        </span>
        <span class="text-slate-600">•</span>
        <span class="text-indigo-300 font-mono font-medium">
          {{ $t('cloud.summary_models_visible', { count: totalVisibleModels }) }}
        </span>
      </div>
      <div class="text-[11px] text-slate-400">
        {{ $t('cloud.models_visibility_desc') }}
      </div>
    </div>

    <!-- Provider Tabs Selector (Clean, Responsive, No Squashing/Truncating) -->
    <div class="flex flex-wrap items-center gap-2 p-2 bg-[#0b0d16] border border-[#1a1f33] rounded-2xl select-none">
      <button
        v-for="tab in providerTabs"
        :key="tab.id"
        type="button"
        @click="activeTab = tab.id"
        :class="[
          'px-3.5 py-2.5 rounded-xl text-xs font-semibold flex items-center gap-2.5 transition-all cursor-pointer border select-none shrink-0',
          activeTab === tab.id
            ? 'bg-gradient-to-r from-indigo-600 to-indigo-500 text-white shadow-md shadow-indigo-600/30 border-indigo-400/40 ring-1 ring-indigo-400/30 font-bold'
            : 'bg-[#111422] hover:bg-[#161a2d] text-slate-300 hover:text-white border-[#1c2236]'
        ]"
      >
        <component
          :is="tab.icon"
          class="w-3.5 h-3.5 shrink-0"
          :class="activeTab === tab.id ? 'text-white' : tab.iconColor"
        />
        <span class="whitespace-nowrap font-medium text-xs">{{ tab.label }}</span>
        <div class="flex items-center gap-1.5 pl-1 shrink-0">
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="isProviderEnabled(tab.id) ? 'bg-emerald-400 shadow-sm shadow-emerald-400/60' : 'bg-slate-600'"
            :title="isProviderEnabled(tab.id) ? $t('cloud.provider_active') : $t('cloud.provider_disabled')"
          />
          <span
            class="px-1.5 py-0.5 rounded-md text-[10px] font-mono font-bold"
            :class="activeTab === tab.id ? 'bg-white/20 text-white' : 'bg-[#1b2034] text-slate-400'"
          >
            {{ getProviderActiveModelsCount(tab.id) }}
          </span>
        </div>
      </button>
    </div>

    <!-- Painel 1: Google Antigravity (AGY) -->
    <div v-if="activeTab === 'antigravity'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <!-- Toggle Ativação -->
        <div class="p-4 flex items-center justify-between">
          <div class="pr-4">
            <span class="text-xs font-bold text-slate-200 block">{{ $t('cloud.agy.title') }}</span>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('cloud.agy.desc') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="providersConfig.antigravity_enabled"
            @click="toggleProvider('antigravity')"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              providersConfig.antigravity_enabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                providersConfig.antigravity_enabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Status da Sessão & Ações -->
        <div class="p-4 space-y-3">
          <div class="flex items-center justify-between gap-3 flex-wrap">
            <div class="flex items-center gap-2">
              <span
                class="px-2.5 py-1 rounded-full text-[11px] font-semibold flex items-center gap-1.5"
                :class="[
                  agySessionStatus?.authenticated
                    ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30'
                    : 'bg-amber-500/15 text-amber-300 border border-amber-500/30'
                ]"
              >
                <span class="w-1.5 h-1.5 rounded-full" :class="agySessionStatus?.authenticated ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'" />
                <span>{{ agySessionStatus?.authenticated ? $t('cloud.agy.authenticated') : $t('cloud.agy.not_connected') }}</span>
              </span>
              <span v-if="agySessionStatus?.active_account" class="text-xs text-slate-300 font-mono">
                ({{ agySessionStatus.active_account }})
              </span>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="$emit('openAgyLogin')"
                class="px-3 py-1.5 rounded-xl bg-[#181d2e] hover:bg-[#20273d] text-slate-200 border border-[#262f48] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95"
              >
                <Terminal class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('cloud.agy.auth_terminal') }}</span>
              </button>
              <button
                type="button"
                @click="testConnection('antigravity')"
                :disabled="isTestingConnection"
                class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
              >
                <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isTestingConnection }" />
                <span>{{ $t('cloud.agy.verify_session') }}</span>
              </button>
            </div>
          </div>

          <!-- Feedback de Teste -->
          <div v-if="testFeedback.provider === 'antigravity'" class="p-3 rounded-xl text-xs flex items-center gap-2" :class="testFeedback.success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="testFeedback.success ? CheckCircle2 : AlertTriangle" class="w-4 h-4 shrink-0" />
            <span>{{ testFeedback.message }}</span>
          </div>
        </div>

        <!-- Opções Avançadas AGY -->
        <div class="p-4 space-y-3">
          <label class="text-xs font-semibold text-slate-200 block">{{ $t('cloud.thinking_budget_label') }}</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="effort in ['low', 'medium', 'high']"
              :key="effort"
              type="button"
              @click="providersConfig.antigravity_default_effort = effort; saveProviders()"
              :class="[
                'p-2.5 rounded-xl border text-xs font-semibold capitalize transition-all cursor-pointer text-center',
                providersConfig.antigravity_default_effort === effort
                  ? 'bg-indigo-600/20 border-indigo-500 text-indigo-200'
                  : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-400'
              ]"
            >
              {{ effort === 'low' ? $t('cloud.effort_low') : effort === 'medium' ? $t('cloud.effort_medium') : $t('cloud.effort_high') }}
            </button>
          </div>
        </div>

        <!-- Modelos AGY e Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-amber-400" />
                <span>{{ $t('cloud.agy.models_title') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-amber-300 border border-[#232a42]">
                  {{ $t('cloud.models_count', { count: getProviderModelsList('antigravity').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.models_visibility_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-1.5">
              <button
                type="button"
                @click="setAllModelsState('antigravity', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.enable_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('antigravity', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.disable_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('antigravity').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['antigravity']"
              :placeholder="$t('cloud.search_models_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500"
            />
          </div>

          <div class="space-y-1.5 pt-1">
            <div
              v-for="m in getFilteredProviderModels('antigravity')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> {{ $t('cloud.tools') }}</span>
                </div>
              </div>

              <button
                type="button"
                role="switch"
                :aria-checked="isModelEnabled(m.id)"
                @click="toggleModelEnabled(m.id)"
                class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                :class="isModelEnabled(m.id) ? 'bg-amber-600' : 'bg-[#22283a]'"
                :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu') : $t('cloud.hidden_in_menu')"
              >
                <span
                  class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                  :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 2: Google Gemini Oficial -->
    <div v-else-if="activeTab === 'gemini'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <!-- Toggle Ativação -->
        <div class="p-4 flex items-center justify-between">
          <div class="pr-4">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-slate-200">{{ $t('cloud.gemini.title') }}</span>
              <span class="px-1.5 py-0.5 rounded text-[9.5px] font-bold bg-blue-500/15 text-blue-300 border border-blue-500/30">
                Google AI Studio
              </span>
            </div>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('cloud.gemini.desc') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="providersConfig.gemini_enabled"
            @click="toggleProvider('gemini')"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              providersConfig.gemini_enabled ? 'bg-blue-600 shadow-sm shadow-blue-600/30' : 'bg-[#1e2436]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                providersConfig.gemini_enabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Configuração de Chave & Modelo -->
        <div class="p-4 space-y-4">
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.gemini.api_key') }}</label>
              <a
                href="https://aistudio.google.com/app/apikey"
                @click.prevent="openExternalUrl('https://aistudio.google.com/app/apikey')"
                class="text-[11px] text-blue-400 hover:text-blue-300 underline font-medium cursor-pointer"
              >
                {{ $t('cloud.gemini.get_key') }}
              </a>
            </div>
            <div class="relative">
              <input
                :type="showGeminiKey ? 'text' : 'password'"
                v-model="providersConfig.gemini_api_key"
                @change="saveProviders"
                placeholder="AIzaSy..."
                class="w-full pl-3 pr-10 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 font-mono outline-none focus:border-blue-500"
              />
              <button
                type="button"
                @click="showGeminiKey = !showGeminiKey"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showGeminiKey ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.suggested_default_model') }}</label>
            <select
              v-model="providersConfig.gemini_selected_model"
              @change="saveProviders"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-blue-500"
            >
              <option value="">{{ $t('cloud.select_or_default', { default: 'gemini-2.5-flash' }) }}</option>
              <option
                v-for="m in getProviderModelsList('gemini')"
                :key="m.id"
                :value="extractRawModelId(m.id)"
              >
                {{ extractRawModelId(m.id) }} ({{ m.name }})
              </option>
            </select>
          </div>

          <!-- Testar Conexão -->
          <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
            <div v-if="testFeedback.provider === 'gemini'" class="flex items-center gap-2 text-xs">
              <component
                :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                class="w-4 h-4"
                :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
              />
              <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                {{ testFeedback.message }}
              </span>
            </div>
            <div v-else class="text-[11px] text-slate-400">
              {{ $t('cloud.gemini.test_helper') }}
            </div>

            <button
              type="button"
              @click="testConnection('gemini')"
              :disabled="isTestingConnection || !providersConfig.gemini_api_key?.trim()"
              class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
            >
              <RefreshCw class="w-3.5 h-3.5 text-blue-400" :class="{ 'animate-spin': isTestingConnection }" />
              <span>{{ $t('cloud.test_connection') }}</span>
            </button>
          </div>
        </div>

        <!-- Seção de Modelos da API & Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-blue-400" />
                <span>{{ $t('cloud.available_api_models') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-blue-300 border border-[#232a42]">
                  {{ $t('cloud.models_count_badge', { count: getProviderModelsList('gemini').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.api_models_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="fetchModelsFromApi('gemini')"
                :disabled="isFetchingModels['gemini'] || !providersConfig.gemini_api_key?.trim()"
                class="px-3 py-1.5 rounded-xl bg-blue-600/20 hover:bg-blue-600/30 text-blue-300 border border-blue-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels['gemini'] }" />
                <span>{{ $t('cloud.fetch_models_api') }}</span>
              </button>

              <button
                type="button"
                @click="setAllModelsState('gemini', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.activate_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('gemini', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.deactivate_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('gemini').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['gemini']"
              :placeholder="$t('cloud.search_models_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-blue-500"
            />
          </div>

          <!-- Feedback da busca de modelos -->
          <div v-if="fetchFeedback['gemini']" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback['gemini'].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="fetchFeedback['gemini'].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
            <span>{{ fetchFeedback['gemini'].message }}</span>
          </div>

          <!-- Lista de Modelos com Switches -->
          <div class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            <div
              v-for="m in getFilteredProviderModels('gemini')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate" :title="m.name">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                  <span v-if="providersConfig.gemini_selected_model === extractRawModelId(m.id)" class="text-[9.5px] font-bold text-amber-300 bg-amber-500/15 border border-amber-500/30 px-1.5 py-0.2 rounded flex items-center gap-1">
                    <Star class="w-2.5 h-2.5 fill-amber-400 text-amber-400" /> {{ $t('cloud.badge_default') }}
                  </span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                </div>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  @click="setDefaultModel('gemini', m.id)"
                  class="p-1 rounded-lg hover:bg-[#1f253d] text-slate-400 hover:text-amber-300 transition-colors cursor-pointer"
                  :title="providersConfig.gemini_selected_model === extractRawModelId(m.id) ? $t('cloud.current_default_tooltip') : $t('cloud.set_as_default_tooltip')"
                >
                  <Star class="w-3.5 h-3.5" :class="providersConfig.gemini_selected_model === extractRawModelId(m.id) ? 'fill-amber-400 text-amber-400' : 'text-slate-500'" />
                </button>

                <button
                  type="button"
                  role="switch"
                  :aria-checked="isModelEnabled(m.id)"
                  @click="toggleModelEnabled(m.id)"
                  class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="isModelEnabled(m.id) ? 'bg-blue-600' : 'bg-[#22283a]'"
                  :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                    :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 3: Groq Cloud Oficial (LPU) -->
    <div v-else-if="activeTab === 'groq'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <!-- Toggle Ativação -->
        <div class="p-4 flex items-center justify-between">
          <div class="pr-4">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-slate-200">{{ $t('cloud.groq.title') }}</span>
              <span class="px-1.5 py-0.5 rounded text-[9.5px] font-bold bg-orange-500/15 text-orange-300 border border-orange-500/30">
                Groq LPU Engine
              </span>
            </div>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('cloud.groq.desc') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="providersConfig.groq_enabled"
            @click="toggleProvider('groq')"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              providersConfig.groq_enabled ? 'bg-orange-600 shadow-sm shadow-orange-600/30' : 'bg-[#1e2436]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                providersConfig.groq_enabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Configuração de Chave & Modelo -->
        <div class="p-4 space-y-4">
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.groq.api_key') }}</label>
              <a
                href="https://console.groq.com/keys"
                @click.prevent="openExternalUrl('https://console.groq.com/keys')"
                class="text-[11px] text-orange-400 hover:text-orange-300 underline font-medium cursor-pointer"
              >
                {{ $t('cloud.groq.get_key') }}
              </a>
            </div>
            <div class="relative">
              <input
                :type="showGroqKey ? 'text' : 'password'"
                v-model="providersConfig.groq_api_key"
                @change="saveProviders"
                placeholder="gsk_..."
                class="w-full pl-3 pr-10 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 font-mono outline-none focus:border-orange-500"
              />
              <button
                type="button"
                @click="showGroqKey = !showGroqKey"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showGroqKey ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.suggested_default_model') }}</label>
            <select
              v-model="providersConfig.groq_selected_model"
              @change="saveProviders"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-orange-500"
            >
              <option value="">{{ $t('cloud.select_or_default', { model: 'llama-3.3-70b-versatile' }) }}</option>
              <option
                v-for="m in getProviderModelsList('groq')"
                :key="m.id"
                :value="extractRawModelId(m.id)"
              >
                {{ extractRawModelId(m.id) }} ({{ m.name }})
              </option>
            </select>
          </div>

          <!-- Ação Testar Conexão -->
          <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
            <div v-if="testFeedback.provider === 'groq'" class="flex items-center gap-2 text-xs">
              <component
                :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                class="w-4 h-4"
                :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
              />
              <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                {{ testFeedback.message }}
              </span>
            </div>
            <div v-else class="text-[11px] text-slate-400">
              {{ $t('cloud.groq.test_helper') }}
            </div>

            <button
              type="button"
              @click="testConnection('groq')"
              :disabled="isTestingConnection || !providersConfig.groq_api_key?.trim()"
              class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
            >
              <RefreshCw class="w-3.5 h-3.5 text-orange-400" :class="{ 'animate-spin': isTestingConnection }" />
              <span>{{ $t('cloud.test_connection') }}</span>
            </button>
          </div>
        </div>

        <!-- Seção de Modelos Groq & Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-orange-400" />
                <span>{{ $t('cloud.groq.models_title') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-orange-300 border border-[#232a42]">
                  {{ $t('cloud.models_count_badge', { count: getProviderModelsList('groq').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.api_models_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="fetchModelsFromApi('groq')"
                :disabled="isFetchingModels['groq'] || !providersConfig.groq_api_key?.trim()"
                class="px-3 py-1.5 rounded-xl bg-orange-600/20 hover:bg-orange-600/30 text-orange-300 border border-orange-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels['groq'] }" />
                <span>{{ $t('cloud.fetch_models_api') }}</span>
              </button>

              <button
                type="button"
                @click="setAllModelsState('groq', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.activate_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('groq', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.deactivate_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('groq').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['groq']"
              :placeholder="$t('cloud.search_models_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-orange-500"
            />
          </div>

          <!-- Feedback da busca de modelos -->
          <div v-if="fetchFeedback['groq']" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback['groq'].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="fetchFeedback['groq'].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
            <span>{{ fetchFeedback['groq'].message }}</span>
          </div>

          <!-- Lista de Modelos com Switches -->
          <div class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            <div
              v-for="m in getFilteredProviderModels('groq')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                  <span
                    v-if="providersConfig.groq_selected_model === extractRawModelId(m.id)"
                    class="text-[9.5px] font-semibold text-orange-300 bg-orange-500/15 border border-orange-500/30 px-1.5 py-0.2 rounded"
                  >
                    {{ $t('cloud.badge_default') }}
                  </span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                </div>
              </div>

              <div class="flex items-center gap-2 flex-shrink-0">
                <button
                  type="button"
                  @click="setDefaultModel('groq', m.id)"
                  class="p-1 rounded-lg transition-colors cursor-pointer"
                  :class="providersConfig.groq_selected_model === extractRawModelId(m.id) ? 'text-orange-400' : 'text-slate-600 hover:text-slate-400'"
                  :title="providersConfig.groq_selected_model === extractRawModelId(m.id) ? $t('cloud.current_default_tooltip') : $t('cloud.set_as_default_tooltip')"
                >
                  <Star class="w-3.5 h-3.5" :class="{ 'fill-orange-400': providersConfig.groq_selected_model === extractRawModelId(m.id) }" />
                </button>

                <button
                  type="button"
                  role="switch"
                  :aria-checked="isModelEnabled(m.id)"
                  @click="toggleModelEnabled(m.id)"
                  class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="isModelEnabled(m.id) ? 'bg-orange-600' : 'bg-[#22283a]'"
                  :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                    :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 4: OpenAI Oficial -->
    <div v-else-if="activeTab === 'openai'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <!-- Toggle Ativação -->
        <div class="p-4 flex items-center justify-between">
          <div class="pr-4">
            <span class="text-xs font-bold text-slate-200 block">{{ $t('cloud.openai.title') }}</span>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('cloud.openai.desc') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="providersConfig.openai_enabled"
            @click="toggleProvider('openai')"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              providersConfig.openai_enabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                providersConfig.openai_enabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Campos de Chave & Base URL -->
        <div class="p-4 space-y-4">
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.openai.api_key') }}</label>
              <a
                href="https://platform.openai.com/api-keys"
                @click.prevent="openExternalUrl('https://platform.openai.com/api-keys')"
                class="text-[11px] text-indigo-400 hover:text-indigo-300 underline font-medium cursor-pointer"
              >
                {{ $t('cloud.openai.get_key') }}
              </a>
            </div>
            <div class="relative">
              <input
                :type="showOpenAiKey ? 'text' : 'password'"
                v-model="providersConfig.openai_api_key"
                @change="saveProviders"
                placeholder="sk-proj-..."
                class="w-full pl-3 pr-10 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 font-mono outline-none focus:border-indigo-500"
              />
              <button
                type="button"
                @click="showOpenAiKey = !showOpenAiKey"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showOpenAiKey ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.openai.base_url') }}</label>
            <input
              type="text"
              v-model="providersConfig.openai_base_url"
              @change="saveProviders"
              placeholder="https://api.openai.com/v1"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.suggested_default_model') }}</label>
            <select
              v-model="providersConfig.openai_selected_model"
              @change="saveProviders"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
            >
              <option value="">{{ $t('cloud.select_or_default', { default: 'gpt-4o' }) }}</option>
              <option
                v-for="m in getProviderModelsList('openai')"
                :key="m.id"
                :value="extractRawModelId(m.id)"
              >
                {{ extractRawModelId(m.id) }} ({{ m.name }})
              </option>
            </select>
          </div>

          <!-- Ação Testar Conexão -->
          <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
            <div v-if="testFeedback.provider === 'openai'" class="flex items-center gap-2 text-xs">
              <component
                :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                class="w-4 h-4"
                :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
              />
              <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                {{ testFeedback.message }}
              </span>
            </div>
            <div v-else class="text-[11px] text-slate-400">
              {{ $t('cloud.openai_test_helper') }}
            </div>

            <button
              type="button"
              @click="testConnection('openai')"
              :disabled="isTestingConnection || !providersConfig.openai_api_key?.trim()"
              class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
            >
              <RefreshCw class="w-3.5 h-3.5 text-indigo-400" :class="{ 'animate-spin': isTestingConnection }" />
              <span>{{ $t('cloud.test_connection') }}</span>
            </button>
          </div>
        </div>

        <!-- Seção de Modelos OpenAI & Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('cloud.available_api_models') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-indigo-300 border border-[#232a42]">
                  {{ $t('cloud.models_count_badge', { count: getProviderModelsList('openai').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.api_models_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="fetchModelsFromApi('openai')"
                :disabled="isFetchingModels['openai'] || !providersConfig.openai_api_key?.trim()"
                class="px-3 py-1.5 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels['openai'] }" />
                <span>{{ $t('cloud.fetch_models_api') }}</span>
              </button>

              <button
                type="button"
                @click="setAllModelsState('openai', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.activate_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('openai', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.deactivate_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('openai').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['openai']"
              :placeholder="$t('cloud.search_models_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500"
            />
          </div>

          <!-- Feedback da busca de modelos -->
          <div v-if="fetchFeedback['openai']" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback['openai'].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="fetchFeedback['openai'].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
            <span>{{ fetchFeedback['openai'].message }}</span>
          </div>

          <!-- Lista de Modelos com Switches -->
          <div class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            <div
              v-for="m in getFilteredProviderModels('openai')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate" :title="m.name">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                  <span v-if="providersConfig.openai_selected_model === extractRawModelId(m.id)" class="text-[9.5px] font-bold text-amber-300 bg-amber-500/15 border border-amber-500/30 px-1.5 py-0.2 rounded flex items-center gap-1">
                    <Star class="w-2.5 h-2.5 fill-amber-400 text-amber-400" /> {{ $t('cloud.badge_default') }}
                  </span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                </div>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  @click="setDefaultModel('openai', m.id)"
                  class="p-1 rounded-lg hover:bg-[#1f253d] text-slate-400 hover:text-amber-300 transition-colors cursor-pointer"
                  :title="providersConfig.openai_selected_model === extractRawModelId(m.id) ? $t('cloud.current_default_tooltip') : $t('cloud.set_as_default_tooltip')"
                >
                  <Star class="w-3.5 h-3.5" :class="providersConfig.openai_selected_model === extractRawModelId(m.id) ? 'fill-amber-400 text-amber-400' : 'text-slate-500'" />
                </button>

                <button
                  type="button"
                  role="switch"
                  :aria-checked="isModelEnabled(m.id)"
                  @click="toggleModelEnabled(m.id)"
                  class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="isModelEnabled(m.id) ? 'bg-indigo-600' : 'bg-[#22283a]'"
                  :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                    :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 4: OpenRouter -->
    <div v-else-if="activeTab === 'openrouter'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <!-- Toggle Ativação -->
        <div class="p-4 flex items-center justify-between">
          <div class="pr-4">
            <span class="text-xs font-bold text-slate-200 block">{{ $t('cloud.openrouter.title') }}</span>
            <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('cloud.openrouter.desc') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="providersConfig.openrouter_enabled"
            @click="toggleProvider('openrouter')"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              providersConfig.openrouter_enabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
            ]"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                providersConfig.openrouter_enabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Campos de Chave -->
        <div class="p-4 space-y-4">
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.openrouter.api_key') }}</label>
              <a
                href="https://openrouter.ai/keys"
                @click.prevent="openExternalUrl('https://openrouter.ai/keys')"
                class="text-[11px] text-indigo-400 hover:text-indigo-300 underline font-medium cursor-pointer"
              >
                {{ $t('cloud.openrouter.get_key') }}
              </a>
            </div>
            <div class="relative">
              <input
                :type="showOpenRouterKey ? 'text' : 'password'"
                v-model="providersConfig.openrouter_api_key"
                @change="saveProviders"
                placeholder="sk-or-v1-..."
                class="w-full pl-3 pr-10 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 font-mono outline-none focus:border-indigo-500"
              />
              <button
                type="button"
                @click="showOpenRouterKey = !showOpenRouterKey"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showOpenRouterKey ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.suggested_default_model') }}</label>
            <select
              v-model="providersConfig.openrouter_selected_model"
              @change="saveProviders"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
            >
              <option value="">{{ $t('cloud.select_or_default', { default: 'anthropic/claude-3.5-sonnet' }) }}</option>
              <option
                v-for="m in getProviderModelsList('openrouter')"
                :key="m.id"
                :value="extractRawModelId(m.id)"
              >
                {{ extractRawModelId(m.id) }} ({{ m.name }})
              </option>
            </select>
          </div>

          <!-- Ação Testar Conexão -->
          <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
            <div v-if="testFeedback.provider === 'openrouter'" class="flex items-center gap-2 text-xs">
              <component
                :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                class="w-4 h-4"
                :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
              />
              <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                {{ testFeedback.message }}
              </span>
            </div>
            <div v-else class="text-[11px] text-slate-400">
              {{ $t('cloud.openrouter_test_helper') }}
            </div>

            <button
              type="button"
              @click="testConnection('openrouter')"
              :disabled="isTestingConnection || !providersConfig.openrouter_api_key?.trim()"
              class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
            >
              <RefreshCw class="w-3.5 h-3.5 text-indigo-400" :class="{ 'animate-spin': isTestingConnection }" />
              <span>{{ $t('cloud.test_connection') }}</span>
            </button>
          </div>
        </div>

        <!-- Seção de Modelos OpenRouter & Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-purple-400" />
                <span>{{ $t('cloud.available_api_models') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-purple-300 border border-[#232a42]">
                  {{ $t('cloud.models_count_badge', { count: getProviderModelsList('openrouter').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.api_models_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="fetchModelsFromApi('openrouter')"
                :disabled="isFetchingModels['openrouter']"
                class="px-3 py-1.5 rounded-xl bg-purple-600/20 hover:bg-purple-600/30 text-purple-300 border border-purple-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels['openrouter'] }" />
                <span>{{ $t('cloud.fetch_models_api') }}</span>
              </button>

              <button
                type="button"
                @click="setAllModelsState('openrouter', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.activate_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('openrouter', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.deactivate_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('openrouter').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['openrouter']"
              :placeholder="$t('cloud.search_models_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-purple-500"
            />
          </div>

          <!-- Feedback da busca de modelos -->
          <div v-if="fetchFeedback['openrouter']" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback['openrouter'].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="fetchFeedback['openrouter'].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
            <span>{{ fetchFeedback['openrouter'].message }}</span>
          </div>

          <!-- Lista de Modelos com Switches -->
          <div class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            <div
              v-for="m in getFilteredProviderModels('openrouter')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate" :title="m.name">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                  <span v-if="providersConfig.openrouter_selected_model === extractRawModelId(m.id)" class="text-[9.5px] font-bold text-amber-300 bg-amber-500/15 border border-amber-500/30 px-1.5 py-0.2 rounded flex items-center gap-1">
                    <Star class="w-2.5 h-2.5 fill-amber-400 text-amber-400" /> {{ $t('cloud.badge_default') }}
                  </span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                </div>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  @click="setDefaultModel('openrouter', m.id)"
                  class="p-1 rounded-lg hover:bg-[#1f253d] text-slate-400 hover:text-amber-300 transition-colors cursor-pointer"
                  :title="providersConfig.openrouter_selected_model === extractRawModelId(m.id) ? $t('cloud.current_default_tooltip') : $t('cloud.set_as_default_tooltip')"
                >
                  <Star class="w-3.5 h-3.5" :class="providersConfig.openrouter_selected_model === extractRawModelId(m.id) ? 'fill-amber-400 text-amber-400' : 'text-slate-500'" />
                </button>

                <button
                  type="button"
                  role="switch"
                  :aria-checked="isModelEnabled(m.id)"
                  @click="toggleModelEnabled(m.id)"
                  class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="isModelEnabled(m.id) ? 'bg-purple-600' : 'bg-[#22283a]'"
                  :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
                >
                  <span
                    class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                    :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                  />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 5: Ollama Local/Remoto -->
    <div v-else-if="activeTab === 'ollama'" class="space-y-4">
      <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
        <div class="p-4">
          <span class="text-xs font-bold text-slate-200 block">{{ $t('cloud.ollama.title') }}</span>
          <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
            {{ $t('cloud.ollama.desc') }}
          </p>
        </div>

        <div class="p-4 space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div class="sm:col-span-2 space-y-1.5">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.ollama.host_url') }}</label>
              <input
                type="text"
                v-model="props.config.ollama_host"
                @change="saveProviders"
                placeholder="127.0.0.1"
                class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
              />
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-slate-200">Porta</label>
              <input
                type="number"
                v-model.number="props.config.ollama_port"
                @change="saveProviders"
                placeholder="11434"
                class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <!-- Ação Testar Conexão -->
          <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
            <div v-if="testFeedback.provider === 'ollama'" class="flex items-center gap-2 text-xs">
              <component
                :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                class="w-4 h-4"
                :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
              />
              <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                {{ testFeedback.message }}
              </span>
            </div>
            <div v-else class="text-[11px] text-slate-400">
              {{ $t('cloud.ollama_test_helper') }}
            </div>

            <button
              type="button"
              @click="testConnection('ollama')"
              :disabled="isTestingConnection"
              class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
            >
              <RefreshCw class="w-3.5 h-3.5 text-sky-400" :class="{ 'animate-spin': isTestingConnection }" />
              <span>{{ $t('cloud.test_connection') }}</span>
            </button>
          </div>
        </div>

        <!-- Seção de Modelos Ollama & Visibilidade -->
        <div class="p-4 space-y-3 bg-[#0d0f19]/70">
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <div>
              <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                <Layers class="w-3.5 h-3.5 text-sky-400" />
                <span>{{ $t('cloud.ollama.models_title') }}</span>
                <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-sky-300 border border-[#232a42]">
                  {{ $t('cloud.models_count_badge', { count: getProviderModelsList('ollama').length }) }}
                </span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('cloud.api_models_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="fetchModelsFromApi('ollama')"
                :disabled="isFetchingModels['ollama']"
                class="px-3 py-1.5 rounded-xl bg-sky-600/20 hover:bg-sky-600/30 text-sky-300 border border-sky-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels['ollama'] }" />
                <span>{{ $t('cloud.fetch_from_ollama') }}</span>
              </button>

              <button
                type="button"
                @click="setAllModelsState('ollama', true)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.activate_all') }}
              </button>
              <button
                type="button"
                @click="setAllModelsState('ollama', false)"
                class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
              >
                {{ $t('cloud.deactivate_all') }}
              </button>
            </div>
          </div>

          <!-- Campo de Busca se houver mais de 3 modelos -->
          <div v-if="getProviderModelsList('ollama').length > 3" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              type="text"
              v-model="modelSearchQueries['ollama']"
              :placeholder="$t('cloud.filter_ollama_placeholder')"
              class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-sky-500"
            />
          </div>

          <!-- Feedback da busca de modelos -->
          <div v-if="fetchFeedback['ollama']" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback['ollama'].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
            <component :is="fetchFeedback['ollama'].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
            <span>{{ fetchFeedback['ollama'].message }}</span>
          </div>

          <!-- Lista de Modelos com Switches -->
          <div v-if="getFilteredProviderModels('ollama').length > 0" class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            <div
              v-for="m in getFilteredProviderModels('ollama')"
              :key="m.id"
              class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
              :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-xs font-semibold text-slate-200 truncate" :title="m.name">{{ m.name }}</span>
                  <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                </div>
                <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                  <span v-if="m.size_gb" class="font-mono">{{ Number(m.size_gb).toFixed(1) }} GB</span>
                  <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                  <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                  <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                </div>
              </div>

              <button
                type="button"
                role="switch"
                :aria-checked="isModelEnabled(m.id)"
                @click="toggleModelEnabled(m.id)"
                class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                :class="isModelEnabled(m.id) ? 'bg-sky-600' : 'bg-[#22283a]'"
                :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
              >
                <span
                  class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                  :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                />
              </button>
            </div>
          </div>
          <div v-else class="text-xs text-slate-500 italic p-3 text-center">
            Clique em "Buscar do Ollama" para carregar os modelos instalados no seu servidor.
          </div>
        </div>
      </div>
    </div>

    <!-- Painel 6: Custom OpenAI (MÚLTIPLOS PROVEDORES) -->
    <div v-else-if="activeTab === 'custom'" class="space-y-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h4 class="text-xs font-bold text-slate-200">{{ $t('cloud.custom_endpoints_title') }}</h4>
          <p class="text-[11px] text-slate-400 mt-0.5">
            {{ $t('cloud.custom_endpoints_desc') }}
          </p>
        </div>

        <button
          type="button"
          @click="addNewCustomProvider"
          class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all flex items-center gap-1.5 cursor-pointer active:scale-95"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>{{ $t('cloud.custom.add_provider') }}</span>
        </button>
      </div>

      <!-- Lista Vazia de Provedores Custom -->
      <div
        v-if="!providersConfig.custom_providers || providersConfig.custom_providers.length === 0"
        class="p-8 text-center rounded-2xl bg-[#111420] border border-[#1e2336] text-slate-400 space-y-3"
      >
        <div class="w-10 h-10 rounded-xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto border border-indigo-500/20">
          <Cloud class="w-5 h-5" />
        </div>
        <div>
          <h5 class="text-xs font-bold text-slate-200">{{ $t('cloud.custom_endpoints_empty') }}</h5>
          <p class="text-[11px] text-slate-400 max-w-md mx-auto mt-1">
            {{ $t('cloud.custom.desc') }}
          </p>
        </div>
        <button
          type="button"
          @click="addNewCustomProvider"
          class="px-3.5 py-1.5 rounded-xl bg-[#161a2b] hover:bg-[#1e243a] text-indigo-300 border border-indigo-500/30 text-xs font-semibold transition-all inline-flex items-center gap-1.5 cursor-pointer"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>{{ $t('cloud.create_first_endpoint') }}</span>
        </button>
      </div>

      <!-- Lista de Provedores Customizados -->
      <div v-else class="space-y-4">
        <div
          v-for="(prov, idx) in providersConfig.custom_providers"
          :key="prov.id || idx"
          class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]"
        >
          <!-- Card Header com Nome, Toggle e Botão de Excluir -->
          <div class="p-4 flex items-center justify-between gap-3 bg-[#131726]/40">
            <div class="flex items-center gap-2.5 flex-1 min-w-0">
              <span class="w-2 h-2 rounded-full" :class="prov.enabled ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-slate-600'" />
              <input
                type="text"
                v-model="prov.name"
                @change="saveProviders"
                :placeholder="$t('cloud.custom.display_name')"
                class="text-xs font-bold text-slate-100 bg-transparent border-b border-transparent hover:border-slate-600 focus:border-indigo-500 outline-none px-1 py-0.5 transition-colors"
              />
              <span class="text-[10px] text-slate-500 font-mono">#{{ idx + 1 }}</span>
            </div>

            <div class="flex items-center gap-3 flex-shrink-0">
              <!-- Switch de Habilitação -->
              <button
                type="button"
                role="switch"
                :aria-checked="prov.enabled"
                @click="prov.enabled = !prov.enabled; saveProviders()"
                :class="[
                  'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                  prov.enabled ? 'bg-indigo-600' : 'bg-[#1e2436]'
                ]"
              >
                <span
                  :class="[
                    'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    prov.enabled ? 'translate-x-4' : 'translate-x-0'
                  ]"
                />
              </button>

              <!-- Botão Excluir -->
              <button
                type="button"
                @click="removeCustomProvider(idx)"
                class="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer"
                :title="$t('cloud.custom.remove_provider')"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Campos de Configuração do Provedor -->
          <div class="p-4 space-y-3">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.custom.base_url') }}</label>
                <input
                  type="text"
                  v-model="prov.base_url"
                  @change="saveProviders"
                  placeholder="http://localhost:1234/v1 ou https://api.groq.com/openai/v1"
                  class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
                />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.custom.model_name') }}</label>
                <input
                  type="text"
                  v-model="prov.selected_model"
                  @change="saveProviders"
                  placeholder="ex: llama-3.3-70b-versatile ou default"
                  class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-mono outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-slate-200">{{ $t('cloud.custom.api_key') }}</label>
              <div class="relative">
                <input
                  :type="prov._showKey ? 'text' : 'password'"
                  v-model="prov.api_key"
                  @change="saveProviders"
                  :placeholder="$t('cloud.custom_no_auth_placeholder')"
                  class="w-full pl-3 pr-10 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 font-mono outline-none focus:border-indigo-500"
                />
                <button
                  type="button"
                  @click="prov._showKey = !prov._showKey"
                  class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
                >
                  <component :is="prov._showKey ? EyeOff : Eye" class="w-4 h-4" />
                </button>
              </div>
            </div>

            <!-- Ação Testar Conexão Individual -->
            <div class="pt-2 flex items-center justify-between gap-3 border-t border-[#1c2236] flex-wrap">
              <div v-if="testFeedback.provider === prov.id" class="flex items-center gap-2 text-xs">
                <component
                  :is="testFeedback.success ? CheckCircle2 : AlertTriangle"
                  class="w-4 h-4"
                  :class="testFeedback.success ? 'text-emerald-400' : 'text-rose-400'"
                />
                <span :class="testFeedback.success ? 'text-emerald-300' : 'text-rose-300'">
                  {{ testFeedback.message }}
                </span>
              </div>
              <div v-else class="text-[11px] text-slate-400">
                Testa a resposta de rota <code class="text-indigo-300 font-mono text-[10px]">/models</code> neste endpoint.
              </div>

              <button
                type="button"
                @click="testCustomEndpoint(prov)"
                :disabled="isTestingConnection || !prov.base_url?.trim()"
                class="px-3.5 py-1.5 rounded-xl bg-[#151928] hover:bg-[#1f263e] text-slate-200 border border-[#242c44] text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer active:scale-95 disabled:opacity-50"
              >
                <RefreshCw class="w-3.5 h-3.5 text-indigo-400" :class="{ 'animate-spin': isTestingConnection && testFeedback.provider === prov.id }" />
                <span>{{ $t('cloud.test_connection') }}</span>
              </button>
            </div>
          </div>

          <!-- Seção de Modelos da API Custom e Visibilidade -->
          <div class="p-4 space-y-3 bg-[#0d0f19]/70">
            <div class="flex items-center justify-between gap-2 flex-wrap">
              <div>
                <h4 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                  <Layers class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('cloud.available_endpoint_models') }}</span>
                  <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[#181d2e] text-indigo-300 border border-[#232a42]">
                    {{ $t('cloud.models_count_badge', { count: getCustomProviderModels(prov).length }) }}
                  </span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('cloud.endpoint_models_desc') }}
                </p>
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  @click="fetchModelsFromApi(prov.id, prov)"
                  :disabled="isFetchingModels[prov.id] || !prov.base_url?.trim()"
                  class="px-3 py-1.5 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                >
                  <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isFetchingModels[prov.id] }" />
                  <span>{{ $t('cloud.fetch_models_api') }}</span>
                </button>

                <button
                  type="button"
                  @click="setAllCustomModelsState(prov, true)"
                  class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-300 border border-[#242b44] cursor-pointer"
                >
                  {{ $t('cloud.activate_all') }}
                </button>
                <button
                  type="button"
                  @click="setAllCustomModelsState(prov, false)"
                  class="px-2.5 py-1 rounded-lg bg-[#181d2e] hover:bg-[#222940] text-[11px] text-slate-400 hover:text-slate-200 border border-[#242b44] cursor-pointer"
                >
                  {{ $t('cloud.deactivate_all') }}
                </button>
              </div>
            </div>

            <!-- Feedback da busca de modelos -->
            <div v-if="fetchFeedback[prov.id]" class="p-2.5 rounded-xl text-xs flex items-center gap-2" :class="fetchFeedback[prov.id].success ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'">
              <component :is="fetchFeedback[prov.id].success ? CheckCircle2 : AlertTriangle" class="w-3.5 h-3.5 shrink-0" />
              <span>{{ fetchFeedback[prov.id].message }}</span>
            </div>

            <!-- Lista de Modelos Retornados -->
            <div v-if="getCustomProviderModels(prov).length > 0" class="space-y-1.5 max-h-72 overflow-y-auto pr-1">
              <div
                v-for="m in getCustomProviderModels(prov)"
                :key="m.id"
                class="p-2.5 rounded-xl border flex items-center justify-between gap-3 transition-all"
                :class="isModelEnabled(m.id) ? 'bg-[#141826] border-[#22283b]' : 'bg-[#101320]/60 border-[#1a1e2d] opacity-50'"
              >
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-xs font-semibold text-slate-200 truncate" :title="m.name">{{ m.name }}</span>
                    <span class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">{{ m.id }}</span>
                    <span v-if="prov.selected_model === extractRawModelId(m.id)" class="text-[9.5px] font-bold text-amber-300 bg-amber-500/15 border border-amber-500/30 px-1.5 py-0.2 rounded flex items-center gap-1">
                      <Star class="w-2.5 h-2.5 fill-amber-400 text-amber-400" /> {{ $t('cloud.badge_default') }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2.5 mt-1 text-[10.5px] text-slate-400">
                    <span v-if="m.context_length" class="font-mono">{{ formatContext(m.context_length) }}</span>
                    <span v-if="m.supports_thinking" class="text-purple-300 flex items-center gap-1"><Brain class="w-2.5 h-2.5" /> {{ $t('cloud.badge_reasoning') }}</span>
                    <span v-if="m.supports_vision" class="text-sky-300 flex items-center gap-1"><Eye class="w-2.5 h-2.5" /> {{ $t('cloud.badge_vision') }}</span>
                    <span v-if="m.supports_tools" class="text-emerald-300 flex items-center gap-1"><Wrench class="w-2.5 h-2.5" /> Tools</span>
                  </div>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                  <button
                    type="button"
                    @click="setCustomDefaultModel(prov, m.id)"
                    class="p-1 rounded-lg hover:bg-[#1f253d] text-slate-400 hover:text-amber-300 transition-colors cursor-pointer"
                    :title="prov.selected_model === extractRawModelId(m.id) ? $t('cloud.current_default_tooltip') : $t('cloud.set_as_default_tooltip')"
                  >
                    <Star class="w-3.5 h-3.5" :class="prov.selected_model === extractRawModelId(m.id) ? 'fill-amber-400 text-amber-400' : 'text-slate-500'" />
                  </button>

                  <button
                    type="button"
                    role="switch"
                    :aria-checked="isModelEnabled(m.id)"
                    @click="toggleModelEnabled(m.id)"
                    class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                    :class="isModelEnabled(m.id) ? 'bg-indigo-600' : 'bg-[#22283a]'"
                    :title="isModelEnabled(m.id) ? $t('cloud.visible_in_menu_tooltip') : $t('cloud.hidden_in_menu_tooltip')"
                  >
                    <span
                      class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                      :class="isModelEnabled(m.id) ? 'translate-x-4' : 'translate-x-0'"
                    />
                  </button>
                </div>
              </div>
            </div>
            <div v-else class="text-xs text-slate-500 italic p-3 text-center">
              Informe a URL do endpoint acima e clique em "Buscar Modelos da API" para sincronizar a lista.
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import {
  Cloud,
  Sparkles,
  RefreshCw,
  Terminal,
  Check,
  Save,
  CheckCircle2,
  AlertTriangle,
  Eye,
  EyeOff,
  Server,
  Zap,
  Globe,
  Plus,
  Trash2,
  Star,
  Layers,
  Search,
  Brain,
  Wrench
} from 'lucide-vue-next'
import type { AppConfig } from '~/types'

const { t } = useI18n()

const props = withDefaults(
  defineProps<{
    config: AppConfig
    agySessionStatus?: {
      installed?: boolean
      authenticated?: boolean
      active_account?: any
      [key: string]: any
    }
  }>(),
  {
    agySessionStatus: () => ({
      installed: false,
      authenticated: false,
      active_account: null
    })
  }
)

const emit = defineEmits<{
  (e: 'saveConfig', config: AppConfig): void
  (e: 'openAgyLogin'): void
  (e: 'refreshModels'): void
}>()

const activeTab = ref('antigravity')
const isSaved = ref(false)
const isTestingConnection = ref(false)
const showOpenAiKey = ref(false)
const showGeminiKey = ref(false)
const showOpenRouterKey = ref(false)
const showGroqKey = ref(false)

const isFetchingModels = reactive<Record<string, boolean>>({})
const fetchFeedback = reactive<Record<string, any>>({})
const modelSearchQueries = reactive<Record<string, string>>({})

const testFeedback = reactive<{
  provider: any
  success: boolean
  message: string
}>({
  provider: null,
  success: false,
  message: ''
})

const providersConfig = reactive<Record<string, any>>({
  openai_enabled: false,
  openai_api_key: '',
  openai_base_url: '',
  openai_selected_model: '',
  gemini_enabled: false,
  gemini_api_key: '',
  gemini_selected_model: '',
  openrouter_enabled: false,
  openrouter_api_key: '',
  openrouter_selected_model: '',
  groq_enabled: false,
  groq_api_key: '',
  groq_selected_model: '',
  antigravity_enabled: true,
  antigravity_default_effort: 'medium',
  custom_providers: [] as any[],
  disabled_models: [] as string[],
  cached_models: {} as Record<string, any>
})

const providerTabs = computed(() => [
  { id: 'antigravity', label: t('cloud.tab_antigravity'), icon: Sparkles, iconColor: 'text-amber-400' },
  { id: 'gemini', label: t('cloud.tab_gemini'), icon: Sparkles, iconColor: 'text-blue-400' },
  { id: 'groq', label: t('cloud.tab_groq'), icon: Zap, iconColor: 'text-orange-400' },
  { id: 'openai', label: t('cloud.tab_openai'), icon: Globe, iconColor: 'text-emerald-400' },
  { id: 'openrouter', label: t('cloud.tab_openrouter'), icon: Zap, iconColor: 'text-purple-400' },
  { id: 'ollama', label: t('cloud.tab_ollama'), icon: Server, iconColor: 'text-sky-400' },
  { id: 'custom', label: t('cloud.tab_custom'), icon: Cloud, iconColor: 'text-indigo-400' }
])

const DEFAULT_MODELS = {
  antigravity: [
    { id: 'agy/gemini-3.8-flash', name: 'Gemini 3.8 Flash', context_length: 1048576, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'agy/gemini-3.8-pro', name: 'Gemini 3.8 Pro', context_length: 2097152, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'agy/claude-3.7-sonnet', name: 'Claude 3.7 Sonnet', context_length: 200000, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'agy/claude-3.5-sonnet', name: 'Claude 3.5 Sonnet', context_length: 200000, supports_thinking: true, supports_vision: true, supports_tools: true }
  ],
  gemini: [
    { id: 'gemini/gemini-2.5-pro', name: 'Gemini 2.5 Pro', context_length: 2097152, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'gemini/gemini-2.5-flash', name: 'Gemini 2.5 Flash', context_length: 1048576, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'gemini/gemini-2.5-flash-lite', name: 'Gemini 2.5 Flash-Lite', context_length: 1048576, supports_thinking: false, supports_vision: true, supports_tools: true },
    { id: 'gemini/gemini-1.5-pro', name: 'Gemini 1.5 Pro', context_length: 2097152, supports_thinking: false, supports_vision: true, supports_tools: true },
    { id: 'gemini/gemini-1.5-flash', name: 'Gemini 1.5 Flash', context_length: 1048576, supports_thinking: false, supports_vision: true, supports_tools: true }
  ],
  openai: [
    { id: 'openai/gpt-4o', name: 'GPT-4o (Omni)', context_length: 128000, supports_thinking: false, supports_vision: true, supports_tools: true },
    { id: 'openai/gpt-4o-mini', name: 'GPT-4o Mini', context_length: 128000, supports_thinking: false, supports_vision: true, supports_tools: true },
    { id: 'openai/o3-mini', name: 'OpenAI o3-mini', context_length: 200000, supports_thinking: true, supports_vision: false, supports_tools: true },
    { id: 'openai/o1', name: 'OpenAI o1', context_length: 200000, supports_thinking: true, supports_vision: true, supports_tools: true }
  ],
  openrouter: [
    { id: 'openrouter/anthropic/claude-3.5-sonnet', name: 'Claude 3.5 Sonnet', context_length: 200000, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'openrouter/deepseek/deepseek-r1', name: 'DeepSeek R1', context_length: 64000, supports_thinking: true, supports_vision: false, supports_tools: true },
    { id: 'openrouter/google/gemini-2.5-flash', name: 'Gemini 2.5 Flash', context_length: 1048576, supports_thinking: true, supports_vision: true, supports_tools: true },
    { id: 'openrouter/meta-llama/llama-3.3-70b-instruct', name: 'Llama 3.3 70B Instruct', context_length: 128000, supports_thinking: false, supports_vision: false, supports_tools: true }
  ],
  groq: [
    { id: 'groq/llama-3.3-70b-versatile', name: 'Meta Llama 3.3 70B Versatile', context_length: 128000, supports_thinking: false, supports_vision: false, supports_tools: true },
    { id: 'groq/llama-3.1-8b-instant', name: 'Meta Llama 3.1 8B Instant', context_length: 128000, supports_thinking: false, supports_vision: false, supports_tools: true },
    { id: 'groq/deepseek-r1-distill-llama-70b', name: 'DeepSeek R1 Distill Llama 70B', context_length: 128000, supports_thinking: true, supports_vision: false, supports_tools: true },
    { id: 'groq/mixtral-8x7b-32768', name: 'Mixtral 8x7B 32k', context_length: 32768, supports_thinking: false, supports_vision: false, supports_tools: true },
    { id: 'groq/gemma2-9b-it', name: 'Google Gemma 2 9B IT', context_length: 8192, supports_thinking: false, supports_vision: false, supports_tools: false },
    { id: 'groq/llama-3.2-11b-vision-preview', name: 'Meta Llama 3.2 11B Vision', context_length: 128000, supports_thinking: false, supports_vision: true, supports_tools: true }
  ],
  ollama: []
}

const openExternalUrl = async (url: string) => {
  try {
    await invoke('open_url', { url })
  } catch (err) {
    console.warn('Falha ao abrir URL via Tauri open_url, fallback para window.open:', err)
    if (typeof window !== 'undefined') {
      window.open(url, '_blank')
    }
  }
}

const formatContext = (tokens?: number | null) => {
  if (!tokens) return ''
  if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(tokens % 1000000 === 0 ? 0 : 1)}M ctx`
  if (tokens >= 1000) return `${Math.round(tokens / 1000)}k ctx`
  return `${tokens} ctx`
}

const extractRawModelId = (id: string) => {
  if (!id) return ''
  const parts = id.split('/')
  return parts.length > 1 ? parts.slice(1).join('/') : id
}

const isProviderEnabled = (providerId: string) => {
  switch (providerId) {
    case 'antigravity':
      return providersConfig.antigravity_enabled
    case 'gemini':
      return providersConfig.gemini_enabled
    case 'groq':
      return providersConfig.groq_enabled
    case 'openai':
      return providersConfig.openai_enabled
    case 'openrouter':
      return providersConfig.openrouter_enabled
    case 'ollama':
      return true
    case 'custom':
      return (providersConfig.custom_providers || []).some((p: any) => p.enabled)
    default:
      return false
  }
}

const getProviderActiveModelsCount = (providerId: string) => {
  if (providerId === 'custom') {
    let count = 0
    for (const prov of providersConfig.custom_providers || []) {
      const models = getCustomProviderModels(prov)
      count += models.filter((m: any) => isModelEnabled(m.id)).length
    }
    return count
  }
  const models = getProviderModelsList(providerId)
  return models.filter((m: any) => isModelEnabled(m.id)).length
}

const totalActiveProviders = computed(() => {
  return providerTabs.value.filter((tab: any) => isProviderEnabled(tab.id)).length
})

const totalVisibleModels = computed(() => {
  let count = 0
  for (const tab of providerTabs.value) {
    if (isProviderEnabled(tab.id)) {
      count += getProviderActiveModelsCount(tab.id)
    }
  }
  return count
})

const toggleProvider = (providerId: string) => {
  switch (providerId) {
    case 'antigravity':
      providersConfig.antigravity_enabled = !providersConfig.antigravity_enabled
      break
    case 'gemini':
      providersConfig.gemini_enabled = !providersConfig.gemini_enabled
      break
    case 'groq':
      providersConfig.groq_enabled = !providersConfig.groq_enabled
      break
    case 'openai':
      providersConfig.openai_enabled = !providersConfig.openai_enabled
      break
    case 'openrouter':
      providersConfig.openrouter_enabled = !providersConfig.openrouter_enabled
      break
  }
  saveProviders()
}

const checkModelIdDisabled = (modelId?: string | null, disabledList?: string[] | null) => {
  if (!modelId || !Array.isArray(disabledList) || disabledList.length === 0) return false
  const cleanId = String(modelId).trim()
  const rawId = cleanId.includes('/') ? cleanId.split('/').slice(1).join('/') : cleanId

  return disabledList.some((d: string) => {
    if (!d) return false
    const cleanD = String(d).trim()
    const rawD = cleanD.includes('/') ? cleanD.split('/').slice(1).join('/') : cleanD
    if (cleanId === cleanD || rawId === rawD) return true
    if ((cleanId.startsWith('agy/') || cleanId.startsWith('gemini/')) && (cleanD.startsWith('agy/') || cleanD.startsWith('gemini/'))) {
      const baseId = cleanId.replace(/-(high|medium|low)$/i, '')
      const baseD = cleanD.replace(/-(high|medium|low)$/i, '')
      if (baseId === baseD) return true
    }
    return false
  })
}

const isModelEnabled = (modelId: string) => {
  if (!Array.isArray(providersConfig.disabled_models)) {
    providersConfig.disabled_models = []
  }
  return !checkModelIdDisabled(modelId, providersConfig.disabled_models)
}

const toggleModelEnabled = (modelId: string) => {
  if (!Array.isArray(providersConfig.disabled_models)) {
    providersConfig.disabled_models = []
  }
  const cleanId = String(modelId).trim()
  const rawId = cleanId.includes('/') ? cleanId.split('/').slice(1).join('/') : cleanId
  const enabled = isModelEnabled(modelId)

  if (enabled) {
    providersConfig.disabled_models.push(modelId)
  } else {
    providersConfig.disabled_models = providersConfig.disabled_models.filter((d: string) => {
      const cleanD = String(d).trim()
      const rawD = cleanD.includes('/') ? cleanD.split('/').slice(1).join('/') : cleanD
      if (cleanId === cleanD || rawId === rawD) return false
      if ((cleanId.startsWith('agy/') || cleanId.startsWith('gemini/')) && (cleanD.startsWith('agy/') || cleanD.startsWith('gemini/'))) {
        const baseId = cleanId.replace(/-(high|medium|low)$/i, '')
        const baseD = cleanD.replace(/-(high|medium|low)$/i, '')
        if (baseId === baseD) return false
      }
      return true
    })
  }
  saveProviders()
}

const setAllModelsState = (providerKey: string, enable: boolean) => {
  if (!Array.isArray(providersConfig.disabled_models)) {
    providersConfig.disabled_models = []
  }
  const models = getProviderModelsList(providerKey)
  for (const m of models) {
    const cleanId = String(m.id).trim()
    const rawId = cleanId.includes('/') ? cleanId.split('/').slice(1).join('/') : cleanId
    if (enable) {
      providersConfig.disabled_models = providersConfig.disabled_models.filter((d: string) => {
        const cleanD = String(d).trim()
        const rawD = cleanD.includes('/') ? cleanD.split('/').slice(1).join('/') : cleanD
        if (cleanId === cleanD || rawId === rawD) return false
        if ((cleanId.startsWith('agy/') || cleanId.startsWith('gemini/')) && (cleanD.startsWith('agy/') || cleanD.startsWith('gemini/'))) {
          const baseId = cleanId.replace(/-(high|medium|low)$/i, '')
          const baseD = cleanD.replace(/-(high|medium|low)$/i, '')
          if (baseId === baseD) return false
        }
        return true
      })
    } else {
      if (isModelEnabled(m.id)) {
        providersConfig.disabled_models.push(m.id)
      }
    }
  }
  saveProviders()
}

const setAllCustomModelsState = (prov: any, enable: boolean) => {
  if (!Array.isArray(providersConfig.disabled_models)) {
    providersConfig.disabled_models = []
  }
  const models = getCustomProviderModels(prov)
  for (const m of models) {
    const cleanId = String(m.id).trim()
    const rawId = cleanId.includes('/') ? cleanId.split('/').slice(1).join('/') : cleanId
    if (enable) {
      providersConfig.disabled_models = providersConfig.disabled_models.filter((d: string) => {
        const cleanD = String(d).trim()
        const rawD = cleanD.includes('/') ? cleanD.split('/').slice(1).join('/') : cleanD
        return cleanId !== cleanD && rawId !== rawD
      })
    } else {
      if (isModelEnabled(m.id)) {
        providersConfig.disabled_models.push(m.id)
      }
    }
  }
  saveProviders()
}

const setDefaultModel = (providerKey: string, modelId: string) => {
  const rawId = extractRawModelId(modelId)
  if (providerKey === 'gemini') {
    providersConfig.gemini_selected_model = rawId
  } else if (providerKey === 'groq') {
    providersConfig.groq_selected_model = rawId
  } else if (providerKey === 'openai') {
    providersConfig.openai_selected_model = rawId
  } else if (providerKey === 'openrouter') {
    providersConfig.openrouter_selected_model = rawId
  }
  saveProviders()
}

const setCustomDefaultModel = (prov: any, modelId: string) => {
  prov.selected_model = extractRawModelId(modelId)
  saveProviders()
}

const getProviderModelsList = (providerKey: string): any[] => {
  if (
    providersConfig.cached_models &&
    Array.isArray(providersConfig.cached_models[providerKey]) &&
    providersConfig.cached_models[providerKey].length > 0
  ) {
    return providersConfig.cached_models[providerKey]
  }
  return (DEFAULT_MODELS as any)[providerKey] || []
}

const getFilteredProviderModels = (providerKey: string): any[] => {
  const list = getProviderModelsList(providerKey)
  const q = (modelSearchQueries[providerKey] || '').trim().toLowerCase()
  let filtered = list
  if (q) {
    filtered = list.filter(
      (m: any) =>
        (m.name && m.name.toLowerCase().includes(q)) ||
        (m.id && m.id.toLowerCase().includes(q))
    )
  }
  return filtered.slice().sort((a: any, b: any) => (a.name || '').localeCompare(b.name || '', undefined, { sensitivity: 'base', numeric: true }))
}

const getCustomProviderModels = (prov: any): any[] => {
  if (!prov) return []
  if (
    providersConfig.cached_models &&
    Array.isArray(providersConfig.cached_models[prov.id]) &&
    providersConfig.cached_models[prov.id].length > 0
  ) {
    return providersConfig.cached_models[prov.id]
  }
  const modelId = prov.selected_model || 'default'
  return [
    {
      id: `custom/${prov.id}/${modelId}`,
      name: `${prov.name || 'Custom'} (${modelId})`,
      context_length: 65536,
      supports_thinking: false,
      supports_vision: true,
      supports_tools: true
    }
  ]
}

const fetchModelsFromApi = async (providerKey: string, customProv: any = null) => {
  isFetchingModels[providerKey] = true
  fetchFeedback[providerKey] = null

  try {
    let provType = providerKey
    let apiKey = ''
    let baseUrl: string | null = null

    if (providerKey === 'gemini') {
      apiKey = providersConfig.gemini_api_key
    } else if (providerKey === 'groq') {
      provType = 'groq'
      apiKey = providersConfig.groq_api_key
      baseUrl = 'https://api.groq.com/openai/v1'
    } else if (providerKey === 'openai') {
      apiKey = providersConfig.openai_api_key
      baseUrl = providersConfig.openai_base_url || 'https://api.openai.com/v1'
    } else if (providerKey === 'openrouter') {
      apiKey = providersConfig.openrouter_api_key
      baseUrl = 'https://openrouter.ai/api/v1'
    } else if (providerKey === 'ollama') {
      baseUrl = `http://${props.config.ollama_host || '127.0.0.1'}:${props.config.ollama_port || 11434}`
    } else if (providerKey === 'antigravity') {
      provType = 'antigravity'
    } else if (customProv) {
      provType = 'custom'
      apiKey = customProv.api_key || ''
      baseUrl = customProv.base_url || ''
    }

    const res: any = await invoke('fetch_provider_models', {
      provider: provType,
      apiKey: apiKey || null,
      baseUrl: baseUrl || null
    })

    if (Array.isArray(res) && res.length > 0) {
      if (!providersConfig.cached_models) {
        providersConfig.cached_models = {}
      }
      providersConfig.cached_models[providerKey] = res
      fetchFeedback[providerKey] = {
        success: true,
        message: t('cloud.models_synced_success', { count: res.length })
      }
      saveProviders()
    } else {
      fetchFeedback[providerKey] = {
        success: false,
        message: t('cloud.no_models_returned')
      }
    }
  } catch (err: any) {
    fetchFeedback[providerKey] = {
      success: false,
      message: String(err)
    }
  } finally {
    isFetchingModels[providerKey] = false
  }
}

const addNewCustomProvider = () => {
  if (!Array.isArray(providersConfig.custom_providers)) {
    providersConfig.custom_providers = []
  }
  const newId = 'custom_' + Date.now().toString(36)
  providersConfig.custom_providers.push({
    id: newId,
    name: t('cloud.new_provider_default_name'),
    base_url: 'http://localhost:1234/v1',
    api_key: '',
    enabled: true,
    selected_model: 'default',
    _showKey: false
  })
  saveProviders()
}

const removeCustomProvider = (index: number) => {
  if (Array.isArray(providersConfig.custom_providers)) {
    const removed = providersConfig.custom_providers.splice(index, 1)[0]
    if (removed && providersConfig.cached_models && providersConfig.cached_models[removed.id]) {
      delete providersConfig.cached_models[removed.id]
    }
    saveProviders()
  }
}

onMounted(async () => {
  try {
    const loaded: any = await invoke('get_cloud_providers_config')
    if (loaded) {
      Object.assign(providersConfig, loaded)
      if (!Array.isArray(providersConfig.custom_providers)) {
        providersConfig.custom_providers = []
      }
      if (!Array.isArray(providersConfig.disabled_models)) {
        providersConfig.disabled_models = []
      }
      if (!providersConfig.cached_models || typeof providersConfig.cached_models !== 'object') {
        providersConfig.cached_models = {}
      }
      // Migrate legacy single custom provider to custom_providers list
      if (loaded.custom_base_url && providersConfig.custom_providers.length === 0) {
        providersConfig.custom_providers.push({
          id: 'custom_default',
          name: loaded.custom_name || 'Custom OpenAI',
          base_url: loaded.custom_base_url,
          api_key: loaded.custom_api_key || '',
          enabled: loaded.custom_enabled ?? true,
          selected_model: loaded.custom_selected_model || 'default',
          _showKey: false
        })
      }
    }
  } catch (err) {
    console.warn('Error loading from backend, falling back to localStorage:', err)
    if (typeof window !== 'undefined') {
      const local = localStorage.getItem('atena_cloud_providers')
      if (local) {
        try {
          Object.assign(providersConfig, JSON.parse(local))
        } catch (_) {}
      }
    }
  }
})

const saveProviders = async () => {
  try {
    if (!Array.isArray(providersConfig.custom_providers)) {
      providersConfig.custom_providers = []
    }
    if (!Array.isArray(providersConfig.disabled_models)) {
      providersConfig.disabled_models = []
    }
    if (!providersConfig.cached_models || typeof providersConfig.cached_models !== 'object') {
      providersConfig.cached_models = {}
    }

    const cleanPayload = JSON.parse(JSON.stringify(providersConfig))
    if (Array.isArray(cleanPayload.custom_providers)) {
      cleanPayload.custom_providers.forEach((p: any) => {
        delete p._showKey
      })
    }

    // 1. Persist directly to Rust backend
    await invoke('save_cloud_providers_config', {
      cloudProviders: cleanPayload
    })

    // 2. Persist in localStorage for fallback and instant hydration
    if (typeof window !== 'undefined') {
      localStorage.setItem('atena_cloud_providers', JSON.stringify(cleanPayload))
    }

    // 3. Update parent props and emit events
    if (props.config) {
      props.config.cloud_providers = cleanPayload
      emit('saveConfig', props.config)
    }

    emit('refreshModels')

    isSaved.value = true
    setTimeout(() => {
      isSaved.value = false
    }, 2500)
  } catch (err) {
    console.error('Failed to save cloud providers configuration:', err)
  }
}

const testCustomEndpoint = async (prov: any) => {
  if (!prov || !prov.base_url) return
  isTestingConnection.value = true
  testFeedback.provider = prov.id
  testFeedback.message = t('cloud.testing_connectivity')

  try {
    const res: any = await invoke('test_cloud_provider_connection', {
      provider: 'custom',
      apiKey: prov.api_key || '',
      baseUrl: prov.base_url
    })
    testFeedback.success = true
    testFeedback.message = (typeof res === 'string' ? res : '') || t('cloud.conn_success')
  } catch (err) {
    testFeedback.success = false
    testFeedback.message = String(err)
  } finally {
    isTestingConnection.value = false
  }
}

const testConnection = async (provider: string) => {
  isTestingConnection.value = true
  testFeedback.provider = provider as any
  testFeedback.message = t('cloud.testing_connectivity')

  try {
    let apiKey = ''
    let baseUrl: string | null = null

    if (provider === 'gemini') {
      apiKey = providersConfig.gemini_api_key
    } else if (provider === 'groq') {
      apiKey = providersConfig.groq_api_key
      baseUrl = 'https://api.groq.com/openai/v1'
    } else if (provider === 'openai') {
      apiKey = providersConfig.openai_api_key
      baseUrl = providersConfig.openai_base_url || 'https://api.openai.com/v1'
    } else if (provider === 'openrouter') {
      apiKey = providersConfig.openrouter_api_key
      baseUrl = 'https://openrouter.ai/api/v1'
    } else if (provider === 'ollama') {
      baseUrl = `http://${props.config.ollama_host || '127.0.0.1'}:${props.config.ollama_port || 11434}`
    }

    const res: any = await invoke('test_cloud_provider_connection', {
      provider,
      apiKey,
      baseUrl
    })

    testFeedback.success = true
    testFeedback.message = (typeof res === 'string' ? res : '') || t('cloud.conn_success')
  } catch (err) {
    testFeedback.success = false
    testFeedback.message = String(err)
  } finally {
    isTestingConnection.value = false
  }
}
</script>
