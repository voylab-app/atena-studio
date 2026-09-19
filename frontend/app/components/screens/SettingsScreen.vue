<template>
  <div class="h-full flex bg-[#090a0f] overflow-hidden select-none">
    <!-- Left macOS-style Sidebar inside Settings -->
    <div class="w-64 bg-[#0d0f17] border-r border-[#1c2030] flex flex-col justify-between p-3 flex-shrink-0">
      <div class="space-y-3">
        <!-- Top Search Bar -->
        <div class="relative">
          <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
          <input type="text" v-model="searchQuery" :placeholder="$t('settings.search_placeholder')"
            class="w-full pl-8 pr-3 py-1.5 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 placeholder-slate-400 outline-none focus:border-indigo-500 transition-all font-sans" />
        </div>

        <!-- App Profile / System Banner -->
        <div
          class="p-2.5 rounded-xl bg-gradient-to-r from-[#141828] to-[#121422] border border-[#20273d] flex items-center gap-2.5 shadow-sm">
          <div
            class="w-8 h-8 rounded-xl bg-gradient-to-tr from-indigo-500 via-purple-500 to-pink-500 flex items-center justify-center text-white font-bold text-xs shadow-md">
            A
          </div>
          <div class="truncate">
            <h3 class="text-xs font-bold text-slate-100 truncate">Atena Studio</h3>
            <p class="text-[10px] text-slate-400 truncate">{{ supportsMlx ? 'Apple Silicon • Metal 3' : (hardware.acceleration_backend ? (hardware.acceleration_backend + ' • Local AI') : (platformInfo?.os === 'windows' ? 'Windows • Local AI' : 'Linux • Local AI')) }}</p>
          </div>
        </div>

        <!-- Nav Categories -->
        <div class="space-y-0.5 pt-1">
          <button v-for="item in filteredMenuItems" :key="item.id" @click="currentSection = item.id" :class="[
            'w-full flex items-center gap-2.5 px-2.5 py-2 rounded-xl text-xs font-medium transition-all text-left cursor-pointer group',
            currentSection === item.id
              ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/20 font-semibold'
              : 'text-slate-300 hover:bg-[#141826] hover:text-white'
          ]">
            <div :class="[
              'w-5 h-5 rounded-lg flex items-center justify-center transition-colors',
              currentSection === item.id
                ? 'bg-white/20 text-white'
                : item.iconBg
            ]">
              <component :is="item.icon" class="w-3 h-3" />
            </div>
            <span class="flex-1 truncate">{{ item.label }}</span>
            <ChevronRight :class="[
              'w-3 h-3 transition-transform',
              currentSection === item.id ? 'text-white' : 'text-slate-500 group-hover:text-slate-300'
            ]" />
          </button>
        </div>
      </div>
    </div>

    <!-- Right macOS-style Details Pane -->
    <div class="flex-1 flex flex-col bg-[#090a0f] overflow-hidden">
      <!-- Detail Header -->
      <div class="px-6 py-3 border-b border-[#1c2030] flex items-center justify-between bg-[#0c0e15]">
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1 text-slate-500">
            <button @click="navigateBack"
              class="p-1 rounded-lg hover:bg-[#181c2b] text-slate-400 hover:text-slate-200 transition-colors"
              :title="$t('common.back')">
              <ChevronLeft class="w-4 h-4" />
            </button>
            <button @click="navigateForward"
              class="p-1 rounded-lg hover:bg-[#181c2b] text-slate-400 hover:text-slate-200 transition-colors"
              :title="$t('common.forward')">
              <ChevronRight class="w-4 h-4" />
            </button>
          </div>
          <h2 class="text-sm font-bold text-slate-100 flex items-center gap-2">
            <component :is="activeMenuItem.icon" class="w-4 h-4 text-indigo-400" />
            <span>{{ activeMenuItem.label }}</span>
          </h2>
        </div>

        <div class="flex items-center gap-2">
          <button v-if="currentSection === 'general' || currentSection === 'inference' || currentSection.startsWith('plugin:')" @click="currentSection.startsWith('plugin:') ? saveActivePluginSettings() : saveSettings()" :class="[
            'px-3.5 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95',
            (currentSection.startsWith('plugin:') ? isPluginSaved : isSaved)
              ? 'bg-emerald-600 text-white shadow-emerald-600/30'
              : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/30'
          ]">
            <component :is="(currentSection.startsWith('plugin:') ? isPluginSaved : isSaved) ? Check : Save" class="w-3.5 h-3.5" />
            <span>{{ (currentSection.startsWith('plugin:') ? isPluginSaved : isSaved) ? $t('common.saved') : $t('common.save') }}</span>
          </button>
        </div>
      </div>

      <!-- Detail Body Scrollable -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6 max-w-4xl">
        <!-- 1. GERAL -->
        <div v-if="currentSection === 'general'" class="space-y-5">
          <!-- Atualização do Aplicativo (Estilo LM Studio) -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.updates_channel') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.updates_channel_desc') }}
                </p>
              </div>
              <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                <span>{{ $t('settings.installed') }}</span>
              </span>
            </div>

            <div class="p-4 space-y-3">
              <div class="flex items-center justify-between py-1 border-b border-[#1c2032]/60">
                <div>
                  <span class="text-xs font-semibold text-slate-200">{{ $t('settings.installed') }}</span>
                  <p class="text-[11px] text-slate-400 mt-0.5">Atena Studio para {{ supportsMlx ? 'macOS (Apple Silicon)' : (platformInfo?.os === 'windows' ? 'Windows' : 'Linux') }}</p>
                </div>
                <div class="text-xs font-mono font-bold text-slate-100 bg-[#161a29] px-3 py-1.5 rounded-lg border border-[#22283e]">
                  Atena Studio 0.2.0 (Build 2026.08)
                </div>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="text-xs font-semibold text-slate-200">{{ $t('settings.updates_channel') }}</span>
                  <p class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.updates_channel_desc') }}</p>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    @click="handleCheckUpdates"
                    :disabled="isCheckingUpdates"
                    class="px-3 py-1.5 rounded-xl border border-[#22283e] bg-[#161a29] hover:bg-[#1c2134] text-xs font-medium text-slate-300 hover:text-white transition-all cursor-pointer flex items-center gap-1.5 active:scale-95 shadow-sm disabled:opacity-60"
                  >
                    <RefreshCw :class="['w-3.5 h-3.5', isCheckingUpdates ? 'animate-spin text-indigo-400' : 'text-slate-400']" />
                    <span>{{ isCheckingUpdates ? $t('settings.checking_updates') : (updateCheckFeedback || $t('settings.check_updates')) }}</span>
                  </button>
                  <div class="text-xs text-slate-300 font-medium bg-[#161a29] border border-[#22283e] px-2.5 py-1.5 rounded-xl flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                    <span>{{ $t('settings.stable') }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Card de Integração com Terminal (CLI) -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Terminal class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.cli_title') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.cli_desc') }}
                </p>
              </div>
              <span
                :class="[
                  'text-[10px] font-mono uppercase px-2 py-0.5 rounded-full border flex items-center gap-1',
                  cliStatus.installed
                    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                    : 'bg-slate-500/10 text-slate-400 border-slate-500/20'
                ]"
              >
                <span :class="['w-1.5 h-1.5 rounded-full', cliStatus.installed ? 'bg-emerald-400' : 'bg-slate-500']"></span>
                <span>{{ cliStatus.installed ? $t('settings.installed') : $t('settings.cli_status_not_installed') }}</span>
              </span>
            </div>

            <div class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div class="space-y-1">
                <div class="text-xs text-slate-300 flex items-center gap-2">
                  <span class="font-semibold">{{ cliStatus.installed ? $t('settings.cli_status_installed', { path: cliStatus.path }) : $t('settings.cli_status_not_installed') }}</span>
                </div>
                <p class="text-[11px] font-mono text-slate-400">
                  atena --server --port 7860 --token 1234
                </p>
              </div>

              <button
                type="button"
                @click="handleInstallCli"
                :disabled="isInstallingCli"
                class="px-3.5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all cursor-pointer active:scale-95 flex items-center gap-2 flex-shrink-0 disabled:opacity-60"
              >
                <Terminal class="w-3.5 h-3.5" />
                <span>{{ isInstallingCli ? '...' : (cliStatus.installed ? $t('settings.cli_reinstall_btn') : $t('settings.cli_install_btn')) }}</span>
              </button>
            </div>
          </div>

          <!-- Card de Aparência e Tema Visual -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Sun class="w-3.5 h-3.5 text-amber-400" />
                  <span>{{ $t('settings.appearance') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.appearance_desc') }}
                </p>
              </div>
              <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                {{ activeTheme === 'system' ? $t('settings.auto_theme') : activeTheme === 'light' ? $t('settings.light') : $t('settings.dark') }}
              </span>
            </div>

            <div class="p-4">
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <!-- Opção Escuro -->
                <button
                  type="button"
                  @click="handleSelectTheme('dark')"
                  :class="[
                    'p-3.5 rounded-xl border flex flex-col items-center text-center gap-2 transition-all cursor-pointer select-none active:scale-[0.98]',
                    activeTheme === 'dark'
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 shadow-md ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-400 hover:text-slate-200'
                  ]"
                >
                  <div class="w-8 h-8 rounded-xl bg-[#0e111a] border border-[#1e2439] flex items-center justify-center text-indigo-400">
                    <Moon class="w-4 h-4" />
                  </div>
                  <div>
                    <div class="text-xs font-bold">{{ $t('settings.dark') }}</div>
                    <div class="text-[10px] text-slate-400 mt-0.5">{{ $t('settings.dark_desc') }}</div>
                  </div>
                </button>

                <!-- Opção Claro -->
                <button
                  type="button"
                  @click="handleSelectTheme('light')"
                  :class="[
                    'p-3.5 rounded-xl border flex flex-col items-center text-center gap-2 transition-all cursor-pointer select-none active:scale-[0.98]',
                    activeTheme === 'light'
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 shadow-md ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-400 hover:text-slate-200'
                  ]"
                >
                  <div class="w-8 h-8 rounded-xl bg-[#0e111a] border border-[#1e2439] flex items-center justify-center text-amber-400">
                    <Sun class="w-4 h-4" />
                  </div>
                  <div>
                    <div class="text-xs font-bold">{{ $t('settings.light') }}</div>
                    <div class="text-[10px] text-slate-400 mt-0.5">{{ $t('settings.light_contrast_desc') }}</div>
                  </div>
                </button>

                <!-- Opção Sistema -->
                <button
                  type="button"
                  @click="handleSelectTheme('system')"
                  :class="[
                    'p-3.5 rounded-xl border flex flex-col items-center text-center gap-2 transition-all cursor-pointer select-none active:scale-[0.98]',
                    activeTheme === 'system'
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 shadow-md ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-400 hover:text-slate-200'
                  ]"
                >
                  <div class="w-8 h-8 rounded-xl bg-[#0e111a] border border-[#1e2439] flex items-center justify-center text-teal-400">
                    <Monitor class="w-4 h-4" />
                  </div>
                  <div>
                    <div class="text-xs font-bold">{{ $t('settings.auto_theme') }}</div>
                    <div class="text-[10px] text-slate-400 mt-0.5">{{ $t('settings.auto_theme_desc') }}</div>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- Card de Idioma da Interface (Select Flexível para Múltiplos Idiomas) -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Globe class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.language') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.language_desc') }}
                </p>
              </div>
              <span class="text-[10px] font-mono uppercase px-2.5 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-semibold">
                {{ currentLocaleItem?.name || currentLocale }}
              </span>
            </div>

            <div class="p-4 space-y-3">
              <div class="relative">
                <select
                  :value="currentLocale"
                  @change="handleSelectLocale(($event.target as HTMLSelectElement).value)"
                  class="w-full appearance-none px-4 py-3 pl-11 pr-10 rounded-xl bg-[#141826] border border-[#22283b] hover:border-indigo-500/40 focus:border-indigo-500 text-xs font-semibold text-slate-100 outline-none transition-all cursor-pointer shadow-sm"
                >
                  <option
                    v-for="item in supportedLocales"
                    :key="item.code"
                    :value="item.code"
                    class="bg-[#141826] text-slate-200 py-1"
                  >
                    {{ item.name }} ({{ item.label }}) — {{ $t(item.descriptionKey) }}
                  </option>
                </select>

                <div class="absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none text-base select-none">
                  <span>{{ currentLocaleItem?.flag || '🌐' }}</span>
                </div>

                <div class="absolute right-3.5 top-1/2 -translate-y-1/2 pointer-events-none text-slate-400">
                  <ChevronDown class="w-4 h-4" />
                </div>
              </div>
            </div>
          </div>

          <!-- macOS Card 1: Multi-path Storage -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Folder class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.multi_path') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.multi_path_desc') }}
                </p>
              </div>
              <button
                type="button"
                @click="chooseFolderForMultiPath"
                class="px-3 py-1.5 rounded-xl bg-[#181d2e] hover:bg-[#22283e] border border-[#262f48] hover:border-indigo-500/50 text-xs font-medium text-indigo-300 hover:text-white transition-all flex items-center gap-1.5 flex-shrink-0 cursor-pointer shadow-sm active:scale-95"
              >
                <FolderPlus class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('settings.add_folder_btn') }}</span>
              </button>
            </div>

            <div class="p-4 space-y-3">
              <!-- Active configured directories list -->
              <div class="space-y-2">
                <div
                  v-for="(dir, idx) in activeModelDirectories"
                  :key="idx"
                  class="p-2.5 rounded-xl bg-[#141826] border border-[#22283b] flex items-center justify-between gap-2 group text-xs font-mono"
                >
                  <div class="flex items-center gap-2 truncate flex-1 min-w-0">
                    <Folder class="w-4 h-4 text-indigo-400 flex-shrink-0" />
                    <span class="text-slate-200 truncate" :title="dir">{{ dir }}</span>
                    <span
                      v-if="isDefaultDirectory(dir, idx)"
                      class="px-2 py-0.5 rounded-md text-[9.5px] font-bold uppercase tracking-wider bg-indigo-500/15 text-indigo-300 border border-indigo-500/25 flex-shrink-0"
                    >
                      {{ $t('settings.default_badge') }}
                    </span>
                  </div>

                  <div class="flex items-center gap-1.5">
                    <button
                      v-if="!isDefaultDirectory(dir, idx)"
                      type="button"
                      @click="setDefaultDirectory(dir)"
                      class="px-2 py-0.5 rounded-lg text-[10.5px] font-medium text-slate-400 hover:text-indigo-300 hover:bg-[#1c2236] transition-colors cursor-pointer"
                      :title="$t('settings.make_default_tooltip')"
                    >
                      {{ $t('settings.make_default') }}
                    </button>
                    <button
                      v-if="activeModelDirectories.length > 1"
                      @click="removeModelDirectory(idx)"
                      class="p-1 rounded-lg text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors cursor-pointer"
                      :title="$t('settings.remove_folder')"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Quick add detected external model dirs -->
              <div v-if="suggestedDirsToAdd.length > 0" class="pt-2 border-t border-[#1a1f33]">
                <span class="text-[10.5px] font-semibold text-slate-400 block mb-2">
                  {{ $t('settings.import_detected') }}
                </span>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="sug in suggestedDirsToAdd"
                    :key="sug.path"
                    @click="addModelDirectory(sug.path)"
                    type="button"
                    class="px-3 py-1.5 rounded-xl bg-[#171c2d] hover:bg-[#20273f] text-slate-300 hover:text-indigo-300 border border-[#232c48] text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer shadow-sm"
                  >
                    <Plus class="w-3.5 h-3.5 text-indigo-400" />
                    <span>+ {{ sug.name }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- macOS Card 1.6: Setup Wizard Reopen -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] p-4 flex items-center justify-between shadow-sm">
            <div>
              <span class="text-xs font-semibold text-slate-200 block">{{ $t('settings.setup_wizard_title') }}</span>
              <span class="text-[11px] text-slate-400">{{ $t('settings.setup_wizard_desc') }}</span>
            </div>
            <button
              type="button"
              @click="$emit('openSetup')"
              class="px-3 py-1.5 rounded-xl bg-[#171c2d] hover:bg-[#20273f] text-slate-300 hover:text-white border border-[#232c48] text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer shadow-sm"
            >
              <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
              <span>{{ $t('settings.setup_wizard_btn') }}</span>
            </button>
          </div>

          <!-- macOS Card 2: Runtime options -->
          <div
            class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
            <!-- Auto Load Row -->
            <div class="p-4 flex items-center justify-between">
              <div>
                <span class="text-xs font-semibold text-slate-200 block">{{ $t('settings.auto_load_title') }}</span>
                <span class="text-[11px] text-slate-400">{{ $t('settings.auto_load_desc_long') }}</span>
              </div>
              <button type="button" role="switch" :aria-checked="Boolean(config.auto_load_last_model)" @click="config.auto_load_last_model = !config.auto_load_last_model" :class="[
                'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                config.auto_load_last_model ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
              ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  config.auto_load_last_model ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>

            <!-- GPU Offload Row -->
            <div class="p-4 flex items-center justify-between">
              <div>
                <span class="text-xs font-semibold text-slate-200 block">{{ supportsMlx ? $t('settings.gpu_accel_title') : $t('settings.gpu_offload_title') }}</span>
                <span class="text-[11px] text-slate-400">{{ supportsMlx ? $t('settings.gpu_accel_desc') : $t('settings.gpu_offload_desc') }}</span>
              </div>
              <button type="button" role="switch" :aria-checked="Boolean(config.gpu_offload)" @click="config.gpu_offload = !config.gpu_offload" :class="[
                'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                config.gpu_offload ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
              ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  config.gpu_offload ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>

            <!-- CPU Threads Row -->
            <div class="p-4 flex items-center justify-between">
              <div>
                <span class="text-xs font-semibold text-slate-200 block">{{ $t('settings.cpu_threads') }}</span>
                <span class="text-[11px] text-slate-400">{{ $t('settings.cpu_threads_desc') }}</span>
              </div>
              <div class="flex items-center gap-1 bg-[#141826] p-1 rounded-xl border border-[#22283b]">
                <button @click="config.thread_count = Math.max(1, (config.thread_count || 8) - 1); saveSettings()"
                  class="w-7 h-7 rounded-lg bg-[#1a1f30] hover:bg-[#22283d] text-slate-300 hover:text-white flex items-center justify-center transition-colors">
                  <Minus class="w-3 h-3" />
                </button>
                <span class="w-10 text-center font-mono font-bold text-xs text-indigo-300">
                  {{ config.thread_count || 8 }}
                </span>
                <button @click="config.thread_count = Math.min(32, (config.thread_count || 8) + 1); saveSettings()"
                  class="w-7 h-7 rounded-lg bg-[#1a1f30] hover:bg-[#22283d] text-slate-300 hover:text-white flex items-center justify-center transition-colors">
                  <Plus class="w-3 h-3" />
                </button>
              </div>
            </div>

            <!-- Show Efficiency Metrics Row -->
            <div class="p-4 flex items-center justify-between">
              <div>
                <span class="text-xs font-semibold text-slate-200 block flex items-center gap-1.5">
                  <Zap class="w-3.5 h-3.5 text-teal-400" />
                  <span>{{ $t('settings.show_efficiency_title') }}</span>
                </span>
                <span class="text-[11px] text-slate-400">{{ $t('settings.show_efficiency_desc') }}</span>
              </div>
              <button type="button" role="switch" :aria-checked="Boolean(config.show_efficiency_metrics !== false)" @click="config.show_efficiency_metrics = config.show_efficiency_metrics === false ? true : false; saveSettings()" :class="[
                'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                config.show_efficiency_metrics !== false ? 'bg-teal-600 shadow-sm shadow-teal-600/30' : 'bg-[#1e2436]'
              ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  config.show_efficiency_metrics !== false ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>
          </div>


          <!-- macOS Card 3: Timezone & Temporal Context Settings -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
            <!-- Header Section -->
            <div class="p-4 bg-[#141826]/40">
              <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                <Clock class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('settings.temporal_context_title') }}</span>
              </h4>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {{ $t('settings.temporal_context_desc') }}
              </p>
            </div>

            <!-- Option 1: Inject Time in User Messages (Disabled by default for token & cache savings) -->
            <div class="p-4 flex items-center justify-between">
              <div class="pr-4">
                <div class="flex items-center gap-2">
                  <h5 class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
                    <Clock class="w-3.5 h-3.5 text-amber-400" />
                    <span>{{ $t('settings.inject_message_time_title') }}</span>
                  </h5>
                  <span class="px-1.5 py-0.5 rounded text-[9.5px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    {{ $t('settings.inject_message_time_badge') }}
                  </span>
                </div>
                <p class="text-[11px] text-slate-400 mt-1 leading-relaxed">
                  {{ $t('settings.inject_message_time_desc') }}
                </p>
              </div>
              <button type="button" role="switch" :aria-checked="Boolean(config.inject_message_time)" @click="config.inject_message_time = !config.inject_message_time; saveSettings()" :class="[
                'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                config.inject_message_time ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
              ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  config.inject_message_time ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>

            <!-- Option 2: Inject Date into System Prompt (Disabled by default) -->
            <div class="p-4 flex items-center justify-between">
              <div class="pr-4">
                <div class="flex items-center gap-2">
                  <h5 class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
                    <Calendar class="w-3.5 h-3.5 text-indigo-400" />
                    <span>{{ $t('settings.inject_current_date_title') }}</span>
                  </h5>
                </div>
                <p class="text-[11px] text-slate-400 mt-1 leading-relaxed">
                  {{ $t('settings.inject_current_date_desc') }}
                </p>
              </div>
              <button type="button" role="switch" :aria-checked="Boolean(config.inject_current_date)" @click="config.inject_current_date = !config.inject_current_date; saveSettings()" :class="[
                'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                config.inject_current_date ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
              ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  config.inject_current_date ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>

            <!-- Timezone & Real-time Settings Panel -->
            <div class="p-4 space-y-3.5 bg-[#0d0f17]/40">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                <!-- Timezone Selector -->
                <div class="space-y-1.5">
                  <div class="flex items-center justify-between">
                    <label class="text-[11px] font-semibold text-slate-300 flex items-center gap-1.5">
                      <Globe class="w-3 h-3 text-indigo-400" />
                      <span>{{ $t('settings.timezone_label') }}</span>
                    </label>
                    <button type="button" @click="detectSystemTimezone"
                      class="text-[10.5px] text-indigo-400 hover:text-indigo-300 transition-colors flex items-center gap-1 font-medium cursor-pointer">
                      <RefreshCw class="w-2.5 h-2.5" />
                      <span>{{ $t('settings.detect_system') }}</span>
                    </button>
                  </div>
                  <select v-model="config.timezone" @change="saveSettings"
                    class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] focus:border-indigo-500 text-xs text-slate-100 outline-none cursor-pointer font-sans transition-all">
                    <optgroup :label="$t('settings.tz_group_brazil')">
                      <option value="America/Sao_Paulo">Brasília / São Paulo (UTC-3)</option>
                      <option value="America/Manaus">Manaus / Amazonas (UTC-4)</option>
                      <option value="America/Belem">Belém / Pará (UTC-3)</option>
                      <option value="America/Fortaleza">Fortaleza / Ceará (UTC-3)</option>
                      <option value="America/Recife">Recife / Pernambuco (UTC-3)</option>
                      <option value="America/Cuiaba">Cuiabá / Mato Grosso (UTC-4)</option>
                      <option value="America/Porto_Velho">Porto Velho / Rondônia (UTC-4)</option>
                      <option value="America/Rio_Branco">Rio Branco / Acre (UTC-5)</option>
                      <option value="America/Noronha">Fernando de Noronha (UTC-2)</option>
                    </optgroup>
                    <optgroup :label="$t('settings.tz_group_americas')">
                      <option value="America/New_York">Nova York / Leste dos EUA (UTC-5)</option>
                      <option value="America/Chicago">Chicago / Centro dos EUA (UTC-6)</option>
                      <option value="America/Denver">Denver / Montanhas dos EUA (UTC-7)</option>
                      <option value="America/Los_Angeles">Los Angeles / Pacífico dos EUA (UTC-8)</option>
                      <option value="America/Argentina/Buenos_Aires">Buenos Aires / Argentina (UTC-3)</option>
                      <option value="America/Santiago">Santiago / Chile (UTC-4)</option>
                      <option value="America/Bogota">Bogotá / Colômbia (UTC-5)</option>
                      <option value="America/Mexico_City">Cidade do México (UTC-6)</option>
                    </optgroup>
                    <optgroup :label="$t('settings.tz_group_europe')">
                      <option value="Europe/Lisbon">Lisboa / Portugal (UTC+0 / WET)</option>
                      <option value="Europe/London">Londres / Reino Unido (UTC+0 / BST)</option>
                      <option value="Europe/Madrid">Madri / Espanha (UTC+1 / CEST)</option>
                      <option value="Europe/Paris">Paris / França (UTC+1 / CEST)</option>
                      <option value="Europe/Berlin">Berlim / Alemanha (UTC+1 / CEST)</option>
                      <option value="Europe/Rome">Roma / Itália (UTC+1 / CEST)</option>
                    </optgroup>
                    <optgroup :label="$t('settings.tz_group_asia_oceania')">
                      <option value="UTC">UTC (Tempo Universal Coordenado)</option>
                      <option value="Asia/Tokyo">Tóquio / Japão (UTC+9)</option>
                      <option value="Asia/Shanghai">Xangai / China (UTC+8)</option>
                      <option value="Asia/Singapore">Singapura (UTC+8)</option>
                      <option value="Asia/Dubai">Dubai / EAU (UTC+4)</option>
                      <option value="Australia/Sydney">Sydney / Austrália (UTC+10)</option>
                    </optgroup>
                    <optgroup v-if="isCustomTimezone" :label="$t('settings.tz_group_custom')">
                      <option :value="config.timezone">{{ config.timezone }}</option>
                    </optgroup>
                  </select>
                </div>

                <!-- Live Preview of Date/Time in selected Timezone -->
                <div class="space-y-1.5">
                  <label class="text-[11px] font-semibold text-slate-300 flex items-center justify-between">
                    <span class="flex items-center gap-1.5">
                      <Calendar class="w-3 h-3 text-emerald-400" />
                      <span>{{ $t('settings.current_date_time') }}</span>
                    </span>
                    <span class="text-[10px] text-emerald-400/80 font-mono flex items-center gap-1">
                      <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                      {{ $t('common.realtime') }}
                    </span>
                  </label>
                  <div class="px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs font-mono text-indigo-300 flex items-center justify-between">
                    <span class="truncate">{{ liveDateTimePreview }}</span>
                    <span class="text-[10px] text-slate-400 font-mono ml-2 flex-shrink-0">{{ timezoneOffsetStr }}</span>
                  </div>
                </div>
              </div>

              <!-- Custom IANA Input -->
              <div class="flex items-center gap-2 pt-0.5">
                <span class="text-xs text-slate-400 flex-shrink-0">{{ $t('settings.other_tz_iana') }}</span>
                <input type="text" v-model="customTzInput" @change="applyCustomTz" :placeholder="$t('settings.other_tz_placeholder')"
                  class="flex-1 px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] focus:border-indigo-500 text-xs font-sans text-slate-100 outline-none transition-all" />
                <button type="button" @click="applyCustomTz"
                  class="px-3.5 py-2 rounded-xl bg-[#1a1f30] hover:bg-indigo-600 text-xs font-medium text-slate-200 hover:text-white transition-colors cursor-pointer flex-shrink-0">
                  {{ $t('common.apply') }}
                </button>
              </div>

              <!-- Preview of the injected Prompt (if enabled) -->
              <div v-if="config.inject_current_date" class="p-2.5 rounded-xl bg-[#131726] border border-[#1e253c] text-[11px] space-y-1.5">
                <div class="text-slate-300 font-semibold flex items-center gap-1.5">
                  <Sparkles class="w-3 h-3 text-indigo-400" />
                  <span>{{ $t('settings.injected_prompt_preview') }}</span>
                </div>
                <pre class="font-mono text-[10px] text-slate-300 bg-[#0a0c13] p-2.5 rounded-lg border border-[#1a1e2d] whitespace-pre-wrap select-text leading-relaxed">{{ samplePromptPreview }}</pre>
              </div>
            </div>
          </div>

          <!-- Card: Audio & Voice Transcription (Whisper) -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Mic class="w-3.5 h-3.5 text-purple-400" />
                  <span>{{ $t('settings.whisper_title') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ supportsMlx ? 'Motor local acelerado na GPU Metal Apple Silicon com fallback universal.' : 'Motor local acelerado via DirectML, CUDA, Python ou WebAssembly.' }}
                </p>
              </div>
            </div>

            <div class="p-4 space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                <!-- Whisper Model Selector -->
                <div class="space-y-1.5">
                  <label class="text-[11px] font-semibold text-slate-300 flex items-center justify-between">
                    <span>{{ $t('settings.whisper_model_label') }}</span>
                    <span class="text-[10px] font-mono" :class="supportsMlx ? 'text-purple-400' : 'text-cyan-400'">
                      {{ supportsMlx ? 'Apple Silicon Metal' : (platformInfo.os === 'windows' ? 'Windows DirectML / CPU' : 'Local Whisper Engine') }}
                    </span>
                  </label>
                  <select
                    v-model="config.whisper_model"
                    @change="saveSettings"
                    class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] focus:border-indigo-500 text-xs text-slate-100 outline-none cursor-pointer font-sans"
                  >
                    <option value="mlx-community/whisper-small-mlx">{{ $t('settings.whisper_small_opt') }}</option>
                    <option value="mlx-community/whisper-base-mlx">{{ $t('settings.whisper_base_opt') }}</option>
                    <option value="mlx-community/whisper-large-v3-turbo">{{ $t('settings.whisper_large_opt') }}</option>
                    <option value="mlx-community/whisper-tiny">{{ $t('settings.whisper_tiny_opt') }}</option>
                  </select>
                  <p class="text-[10.5px] text-slate-400">
                    {{ $t('settings.whisper_model_desc') }}
                  </p>
                </div>

                <!-- Whisper Language Selector -->
                <div class="space-y-1.5">
                  <label class="text-[11px] font-semibold text-slate-300 flex items-center justify-between">
                    <span>{{ $t('settings.whisper_lang_label') }}</span>
                    <span class="text-[10px] text-indigo-400 font-mono">{{ $t('settings.whisper_lang_tip') }}</span>
                  </label>
                  <select
                    v-model="config.whisper_language"
                    @change="saveSettings"
                    class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] focus:border-indigo-500 text-xs text-slate-100 outline-none cursor-pointer font-sans"
                  >
                    <option value="pt">{{ $t('settings.whisper_lang_pt') }}</option>
                    <option value="auto">{{ $t('settings.whisper_lang_auto') }}</option>
                    <option value="en">{{ $t('settings.whisper_lang_en') }}</option>
                    <option value="es">{{ $t('settings.whisper_lang_es') }}</option>
                    <option value="zh">{{ $t('settings.whisper_lang_zh') }}</option>
                    <option value="ru">{{ $t('settings.whisper_lang_ru') }}</option>
                  </select>
                  <p class="text-[10.5px] text-slate-400">
                    {{ $t('settings.whisper_lang_desc') }}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <!-- Card: Archived Chats Management -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Archive class="w-3.5 h-3.5 text-amber-400" />
                  <span>{{ $t('settings.archived_chats_title') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.archived_chats_desc') }}
                </p>
              </div>
              <button
                type="button"
                @click="$emit('openArchivedModal')"
                class="px-3.5 py-1.5 rounded-xl bg-[#181d2e] hover:bg-[#22283e] border border-[#262f48] hover:border-amber-500/50 text-xs font-medium text-amber-300 hover:text-white transition-all flex items-center gap-1.5 flex-shrink-0 cursor-pointer shadow-sm active:scale-95"
              >
                <Archive class="w-3.5 h-3.5 text-amber-400" />
                <span>{{ $t('settings.manage_archived_chats', { count: archivedCount }) }}</span>
              </button>
            </div>
          </div>
        </div>

        <!-- PROVEDORES DE NUVEM -->
        <div v-else-if="currentSection === 'cloud_providers'">
          <CloudProvidersSettings
            :config="config"
            :agy-session-status="agySessionStatus"
            @save-config="saveSettings"
            @open-agy-login="$emit('openAgyLogin')"
            @refresh-models="$emit('refreshModels')"
          />
        </div>

        <!-- CANAIS E MENSAGENS REMOTAS (TELEGRAM, DISCORD, TRAY) -->
        <div v-else-if="currentSection === 'gateways'">
          <GatewaysSettings
            :config="config"
            @save-config="saveSettings"
          />
        </div>

        <!-- AUTOMAÇÃO E ROTINAS PROATIVAS (CRON) -->
        <div v-else-if="currentSection === 'automation'">
          <AutomationSettings />
        </div>

        <!-- 1.5 PLUGINS & EXTENSÕES -->
        <div v-else-if="currentSection === 'plugins'" class="space-y-5">
          <!-- Top Banner & Actions -->
          <div class="flex items-center justify-between gap-4">
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Blocks class="w-4 h-4 text-indigo-400 flex-shrink-0" />
                <span class="truncate">{{ $t('settings.plugins_title') }}</span>
              </h3>
              <p class="text-xs text-slate-400 mt-0.5 leading-relaxed">
                {{ $t('settings.plugins_desc') }}
              </p>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
              <button
                @click="openExternalUrl('https://github.com/voylab-app/atena-studio/blob/main/PLUGINS.md')"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all cursor-pointer"
                :title="$t('plugins.doc_tooltip')"
              >
                <BookOpen class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('plugins.doc_btn') }}</span>
              </button>
              <button
                @click="openPluginsFolder"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all cursor-pointer"
                :title="$t('settings.open_folder_tooltip')"
              >
                <FolderOpen class="w-3.5 h-3.5 text-amber-400" />
                <span>{{ $t('settings.open_folder') }}</span>
              </button>
              <button
                @click="fetchPlugins"
                :disabled="isLoadingPlugins"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all cursor-pointer disabled:opacity-60"
              >
                <RefreshCw :class="['w-3.5 h-3.5 text-indigo-400', isLoadingPlugins ? 'animate-spin' : '']" />
                <span>{{ $t('settings.reload') }}</span>
              </button>
            </div>
          </div>

          <!-- Loading State -->
          <div v-if="isLoadingPlugins" class="p-8 text-center text-xs text-slate-400 flex flex-col items-center justify-center gap-2">
            <Loader2 class="w-5 h-5 text-indigo-400 animate-spin" />
            <span>{{ $t('settings.loading_plugins') }}</span>
          </div>

          <!-- Empty State (Nenhum Plugin Instalado) -->
          <div v-else-if="plugins.length === 0" class="p-8 text-center rounded-2xl bg-[#111422] border border-[#1d2338] text-slate-400 space-y-3">
            <div class="w-12 h-12 rounded-2xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto border border-indigo-500/20">
              <Blocks class="w-6 h-6" />
            </div>
            <div>
              <h4 class="text-sm font-bold text-slate-200">{{ $t('settings.no_plugins') }}</h4>
              <p class="text-xs text-slate-400 max-w-md mx-auto mt-1 leading-relaxed">
                {{ $t('settings.no_plugins_desc') }}
              </p>
            </div>
            <div class="flex items-center justify-center gap-3 pt-1">
              <button
                @click="openPluginsFolder"
                class="px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer flex items-center gap-1.5"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>{{ $t('settings.open_plugins_folder') }}</span>
              </button>
              <button
                @click="openExternalUrl('https://github.com/voylab-app/atena-studio/blob/main/PLUGINS.md')"
                class="px-3.5 py-1.5 rounded-xl bg-[#171c2c] hover:bg-[#20273d] border border-[#262f48] hover:border-indigo-500/40 text-xs font-medium text-slate-200 transition-all cursor-pointer flex items-center gap-1.5"
                :title="$t('plugins.doc_tooltip')"
              >
                <BookOpen class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('plugins.doc_btn') }}</span>
              </button>
            </div>
          </div>

          <!-- Plugins List -->
          <div v-else class="space-y-3">
            <div
              v-for="plugin in plugins"
              :key="plugin.id"
              :class="[
                'p-4 rounded-2xl border transition-all flex flex-col gap-3',
                plugin.enabled
                  ? 'bg-[#111422] border-[#222b48] shadow-sm'
                  : 'bg-[#0e101c]/70 border-[#1a1f33] opacity-75'
              ]"
            >
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
                <div class="flex items-center gap-2.5 flex-shrink-0">
                  <button
                    v-if="plugin.enabled && ((plugin.settings_schema && plugin.settings_schema.length > 0) || (plugin.views && plugin.views.some(v => v.location === 'settings')))"
                    @click="currentSection = `plugin:${plugin.id}`"
                    class="px-2.5 py-1 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 text-xs font-medium flex items-center gap-1 transition-colors cursor-pointer"
                    :title="$t('settings.configure_plugin_tooltip')"
                  >
                    <Sliders class="w-3 h-3" />
                    <span>{{ $t('settings.configure_btn') }}</span>
                  </button>
                  <button
                    type="button"
                    role="switch"
                    :aria-checked="plugin.enabled"
                    @click="handleTogglePlugin(plugin)"
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

              <!-- Footer do Plugin com Hooks e Views -->
              <div class="pt-2 border-t border-[#1a2034] flex items-center justify-between flex-wrap gap-2 text-[11px] text-slate-400">
                <div class="flex items-center gap-1.5 flex-wrap">
                  <span class="text-slate-500 text-[10.5px]">{{ $t('plugins.features') }}</span>
                  <span
                    v-for="hook in plugin.hooks"
                    :key="hook"
                    class="px-1.5 py-0.5 rounded bg-[#151828] text-indigo-300/80 border border-[#20253d] font-mono text-[9.5px]"
                  >
                    hook:{{ hook }}
                  </span>
                  <span
                    v-for="view in plugin.views"
                    :key="view.id"
                    class="px-1.5 py-0.5 rounded bg-[#151828] text-amber-300/80 border border-[#20253d] text-[9.5px] flex items-center gap-1"
                  >
                    <LayoutGrid class="w-2.5 h-2.5" />
                    {{ $t('plugins.tab_prefix') }}:{{ view.title }}
                  </span>
                  <span
                    v-if="plugin.settings_schema && plugin.settings_schema.length > 0"
                    class="px-1.5 py-0.5 rounded bg-[#151828] text-emerald-300/80 border border-[#20253d] text-[9.5px] flex items-center gap-1"
                  >
                    <Settings2 class="w-2.5 h-2.5" />
                    {{ $t('plugins.settings_badge') }}
                  </span>
                </div>

                <div class="flex items-center gap-2 text-[10.5px] text-slate-500">
                  <span>{{ $t('plugins.author_label') }} <strong class="text-slate-400">{{ plugin.author }}</strong></span>
                  <span v-if="plugin.id === 'atena-plugin-memory'" class="text-purple-400 font-medium">
                    {{ $t('plugins.memory_ram_save') }}
                  </span>
                </div>
              </div>
            </div>
          </div>


          <!-- Dev Guide Box in Settings -->
          <div class="p-4 rounded-2xl bg-[#111422] border border-[#1e253d] flex items-start gap-3">
            <div class="w-8 h-8 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center flex-shrink-0 mt-0.5">
              <Code2 class="w-4 h-4" />
            </div>
            <div class="flex-1">
              <h5 class="text-xs font-bold text-slate-200">{{ $t('plugins.dev_guide_title') }}</h5>
              <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
                {{ $t('plugins.dev_guide_desc') }}
              </p>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
              <button
                @click="openPluginsFolder"
                class="px-3 py-1.5 rounded-xl bg-[#161a2c] hover:bg-[#20273f] border border-[#252c48] text-xs font-medium text-slate-200 hover:text-white transition-all cursor-pointer"
              >
                {{ $t('plugins.open_folder') }}
              </button>
              <button
                @click="openExternalUrl('https://github.com/voylab-app/atena-studio/blob/main/PLUGINS.md')"
                class="px-3 py-1.5 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 border border-indigo-500/30 text-xs font-medium text-indigo-300 hover:text-white transition-all cursor-pointer flex items-center gap-1.5"
                :title="$t('plugins.doc_tooltip')"
              >
                <BookOpen class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('plugins.doc_btn') }}</span>
              </button>
            </div>
          </div>
        </div>

        <!-- 1.6 PAINEL DE CONFIGURAÇÕES DE PLUGIN DINÂMICO -->
        <div v-else-if="currentSection.startsWith('plugin:') && selectedPlugin" class="space-y-5">
          <!-- Top Banner do Plugin -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] p-5 shadow-sm">
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-start gap-3.5">
                <div class="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/25 flex items-center justify-center text-indigo-400 shrink-0">
                  <component :is="getPluginIcon(selectedPlugin.icon)" class="w-5 h-5" />
                </div>
                <div>
                  <div class="flex items-center gap-2 flex-wrap">
                    <h3 class="text-sm font-bold text-slate-100">{{ selectedPlugin.name }}</h3>
                    <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 font-mono">
                      v{{ selectedPlugin.version }}
                    </span>
                    <span v-if="selectedPlugin.is_builtin" class="px-2 py-0.5 rounded text-[10px] font-semibold bg-purple-500/10 text-purple-300 border border-purple-500/20">
                      Nativo
                    </span>
                    <span v-else class="px-2 py-0.5 rounded text-[10px] font-semibold bg-emerald-500/10 text-emerald-300 border border-emerald-500/20">
                      Comunidade
                    </span>
                  </div>
                  <p class="text-xs text-slate-400 mt-1 leading-relaxed">
                    {{ selectedPlugin.description }}
                  </p>
                  <div class="text-[11px] text-slate-500 mt-1 flex items-center gap-3">
                    <span>Autor: <strong class="text-slate-400 font-medium">{{ selectedPlugin.author }}</strong></span>
                    <span v-if="selectedPlugin.directory" class="text-slate-500 font-mono text-[10px] truncate max-w-xs">
                      {{ selectedPlugin.directory }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Quick Toggle Plugin State -->
              <div class="flex items-center gap-2 shrink-0">
                <span class="text-xs text-slate-400">{{ selectedPlugin.enabled ? 'Ativo' : 'Inativo' }}</span>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="selectedPlugin.enabled"
                  @click="togglePlugin(selectedPlugin.id, !selectedPlugin.enabled)"
                  :class="[
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    selectedPlugin.enabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      selectedPlugin.enabled ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>
            </div>
          </div>

          <!-- Painel Especial do Conector Antigravity Nuvem -->
          <div
            v-if="selectedPlugin.id === 'atena-plugin-agy'"
            class="rounded-2xl bg-[#111420] border border-[#1e2336] p-5 space-y-4 shadow-sm"
          >
            <div class="flex items-start justify-between gap-4 pb-3 border-b border-[#1c2236] flex-wrap">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Cloud class="w-4 h-4 text-sky-400" />
                  <span>Conexão & Cotas do Google Antigravity</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
                  Gerencie o login da conta Google, visualize as cotas semanais e configure o comportamento da nuvem.
                </p>
              </div>

              <button
                type="button"
                @click="$emit('openAgyLogin')"
                class="px-3.5 py-1.5 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white font-semibold text-xs transition-all shadow-md shadow-indigo-600/25 flex items-center gap-1.5 cursor-pointer active:scale-95"
              >
                <Sparkles class="w-3.5 h-3.5 text-amber-300" />
                <span>Gerenciar Conexão / Cotas</span>
              </button>
            </div>

            <!-- Informações do Conector -->
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 text-xs">
              <div class="p-3 rounded-xl bg-[#141828] border border-[#202640] space-y-1">
                <span class="text-[10px] uppercase font-bold text-slate-400 tracking-wider">Protocolo Nativo</span>
                <div class="font-mono text-indigo-300 text-[11px] font-semibold flex items-center gap-1">
                  <span>Antigravity CLI (Stdio IPC)</span>
                </div>
              </div>

              <div class="p-3 rounded-xl bg-[#141828] border border-[#202640] space-y-1">
                <span class="text-[10px] uppercase font-bold text-slate-400 tracking-wider">{{ $t('settings.module_separate') }}</span>
                <div class="font-mono text-emerald-300 text-[11px] font-semibold truncate" title="plugins/atena-plugin-agy">
                  plugins/atena-plugin-agy
                </div>
              </div>

              <div class="p-3 rounded-xl bg-[#141828] border border-[#202640] space-y-1">
                <span class="text-[10px] uppercase font-bold text-slate-400 tracking-wider">{{ $t('settings.translation_i18n') }}</span>
                <div class="text-slate-200 text-[11px] font-mono">
                  pt-BR / en-US
                </div>
              </div>
            </div>
          </div>

          <!-- Campos de Configuração do Schema -->
          <div
            v-if="selectedPlugin.settings_schema && selectedPlugin.settings_schema.length > 0"
            class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]"
          >
            <div class="p-4 bg-[#141826]/40 flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <Sliders class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.config_parameters') }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.config_parameters_desc') }}
                </p>
              </div>
            </div>

            <!-- Loop pelos campos do schema -->
            <div
              v-for="field in selectedPlugin.settings_schema"
              :key="field.id"
              class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3"
            >
              <div class="pr-4 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold text-slate-200 block">{{ field.label }}</span>
                  <span class="text-[10px] font-mono text-slate-500">({{ field.id }})</span>
                </div>
                <p v-if="field.description" class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
                  {{ field.description }}
                </p>
              </div>

              <!-- Input: Boolean Switch -->
              <div v-if="field.type === 'boolean'" class="flex-shrink-0">
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(pluginFormValues[field.id])"
                  @click="togglePluginBooleanField(field.id)"
                  :class="[
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    pluginFormValues[field.id] ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      pluginFormValues[field.id] ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>

              <!-- Input: Number (stepper) -->
              <div v-else-if="field.type === 'number'" class="flex items-center gap-2 flex-shrink-0">
                <div class="flex items-center bg-[#151928] border border-[#232a44] rounded-xl overflow-hidden shadow-inner">
                  <button
                    type="button"
                    @click="stepNumberField(field, -1)"
                    class="px-2.5 py-1.5 text-slate-400 hover:text-white hover:bg-[#1f263e] transition-colors"
                  >
                    <Minus class="w-3.5 h-3.5" />
                  </button>
                  <input
                    type="number"
                    v-model.number="pluginFormValues[field.id]"
                    :min="field.min"
                    :max="field.max"
                    :step="field.step || 1"
                    @change="onPluginFieldChange(field.id)"
                    class="w-16 bg-transparent text-center text-xs font-semibold text-indigo-300 outline-none"
                  />
                  <button
                    type="button"
                    @click="stepNumberField(field, 1)"
                    class="px-2.5 py-1.5 text-slate-400 hover:text-white hover:bg-[#1f263e] transition-colors"
                  >
                    <Plus class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              <!-- Input: Select Dropdown -->
              <div v-else-if="field.type === 'select'" class="flex-shrink-0 w-full sm:w-48">
                <select
                  v-model="pluginFormValues[field.id]"
                  @change="onPluginFieldChange(field.id)"
                  class="w-full bg-[#151928] border border-[#232a44] rounded-xl px-3 py-1.5 text-xs text-slate-200 focus:border-indigo-500/50 outline-none"
                >
                  <option
                    v-for="opt in (field.options || [])"
                    :key="opt.value"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
              </div>

              <!-- Input: Text / Password -->
              <div v-else class="flex-shrink-0 w-full sm:w-64">
                <input
                  :type="field.type === 'password' ? 'password' : 'text'"
                  v-model="pluginFormValues[field.id]"
                  :placeholder="field.placeholder || ''"
                  @change="onPluginFieldChange(field.id)"
                  class="w-full bg-[#151928] border border-[#232a44] rounded-xl px-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:border-indigo-500/50 outline-none"
                />
              </div>
            </div>
          </div>

          <!-- Caso o plugin não tenha settings_schema declarado -->
          <div v-else class="rounded-2xl bg-[#111420] border border-[#1e2336] p-8 text-center">
            <Sliders class="w-8 h-8 text-slate-600 mx-auto mb-2" />
            <p class="text-xs text-slate-400">{{ $t('settings.plugin_no_inputs') }}</p>
          </div>

          <!-- Rodapé de Documentação, Tradução e Repositório do Plugin -->
          <div
            v-if="selectedPlugin.readme || selectedPlugin.doc || selectedPlugin.repository"
            class="p-4 rounded-2xl bg-[#0e111d] border border-[#1b2033] flex items-center justify-between text-xs text-slate-400 flex-wrap gap-3"
          >
            <div class="flex items-center gap-2">
              <FileText class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
              <span>{{ $t('settings.documentation') }}</span>
              <span v-if="selectedPlugin.doc" class="font-mono text-[10.5px] text-slate-300 bg-[#161a2c] px-2 py-0.5 rounded border border-[#212740]">
                {{ selectedPlugin.doc }}
              </span>
              <span v-if="selectedPlugin.locales" class="font-mono text-[10.5px] text-emerald-300 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
                i18n: {{ selectedPlugin.locales }}
              </span>
            </div>

            <div class="flex items-center gap-3 text-[11px]">
              <span v-if="selectedPlugin.license" class="text-slate-500 font-mono">
                {{ $t('settings.license_label') }}: {{ selectedPlugin.license }}
              </span>
              <a
                v-if="selectedPlugin.repository"
                :href="selectedPlugin.repository"
                target="_blank"
                class="text-indigo-400 hover:text-indigo-300 flex items-center gap-1 hover:underline font-medium"
              >
                <span>{{ $t('settings.official_repo') }}</span>
                <ExternalLink class="w-3 h-3" />
              </a>
            </div>
          </div>
        </div>

        <!-- 2. FERRAMENTAS MCP -->
        <div v-else-if="currentSection === 'mcp'" class="space-y-5">
          <!-- Top Banner & Actions -->
          <div class="flex items-center justify-between gap-4">
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Wrench class="w-4 h-4 text-purple-400 flex-shrink-0" />
                <span class="truncate">{{ $t('settings.mcp_servers_title') }}</span>
              </h3>
              <p class="text-xs text-slate-400 mt-0.5 leading-relaxed">
                {{ $t('settings.mcp_servers_banner_desc') }}
              </p>
            </div>

            <div class="flex items-center gap-2 flex-shrink-0">
              <button @click="refreshMcpTools" :disabled="isRefreshingMcp"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#141824] hover:bg-[#1b2030] border border-[#22283b] text-xs font-medium text-slate-200 transition-all active:scale-95 disabled:opacity-60 whitespace-nowrap cursor-pointer">
                <RefreshCw :class="['w-3.5 h-3.5 text-indigo-400', isRefreshingMcp ? 'animate-spin' : '']" />
                <span>{{ isRefreshingMcp ? $t('settings.mcp_rescanning') : $t('settings.mcp_rescan') }}</span>
              </button>

              <button @click="openAddMcpModal"
                class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white transition-all shadow-md shadow-indigo-600/25 active:scale-95 cursor-pointer whitespace-nowrap">
                <Plus class="w-3.5 h-3.5" />
                <span>{{ $t('settings.mcp_new_server') }}</span>
              </button>
            </div>
          </div>

          <!-- Quick Templates -->
          <div class="space-y-2">
            <h4 class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">{{ $t('settings.mcp_quick_templates') }}</h4>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div v-for="tmpl in quickMcpTemplates" :key="tmpl.id"
                class="p-3.5 rounded-2xl bg-[#111420] border border-[#1e2336] hover:border-indigo-500/40 transition-all flex flex-col justify-between group">
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <span
                      class="text-xs font-bold text-slate-100 group-hover:text-indigo-300 transition-colors flex items-center gap-1.5">
                      <component :is="tmpl.icon" class="w-3.5 h-3.5 text-indigo-400" />
                      {{ tmpl.name }}
                    </span>
                    <span
                      class="text-[9px] font-mono uppercase px-1.5 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                      {{ tmpl.transport }}
                    </span>
                  </div>
                  <p class="text-[11px] text-slate-400 line-clamp-2 leading-relaxed">
                    {{ tmpl.description }}
                  </p>
                </div>
                <button @click="applyMcpTemplate(tmpl)"
                  class="mt-3 w-full py-1.5 rounded-xl bg-[#171c2b] hover:bg-indigo-600 hover:text-white text-slate-300 text-xs font-medium border border-[#232a3f] transition-all flex items-center justify-center gap-1.5 cursor-pointer">
                  <Plus class="w-3 h-3" />
                  <span>{{ $t('settings.mcp_install_template') }}</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Configured MCP Servers -->
          <div class="space-y-3">
            <div class="flex items-center justify-between flex-wrap gap-2">
              <h4 class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">
                {{ $t('settings.mcp_installed_servers', { count: mcpServers.length }) }}
              </h4>
              <div class="flex items-center gap-2">
                <input ref="mcpFileInput" type="file" accept=".json" class="hidden" @change="handleImportMcpFile" />
                <button
                  @click="triggerImportMcp"
                  class="px-2.5 py-1 rounded-xl bg-[#171c2b] hover:bg-[#22293e] border border-[#232a3f] text-slate-300 hover:text-white text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer shadow-xs"
                  :title="$t('settings.mcp_import_tooltip')"
                >
                  <Upload class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.mcp_import_btn') }}</span>
                </button>
                <button
                  @click="exportMcpServers"
                  :disabled="mcpServers.length === 0"
                  class="px-2.5 py-1 rounded-xl bg-[#171c2b] hover:bg-[#22293e] border border-[#232a3f] text-slate-300 hover:text-white text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shadow-xs"
                  :title="$t('settings.mcp_export_tooltip')"
                >
                  <Download class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.mcp_export_btn') }}</span>
                </button>
              </div>
            </div>

            <div v-if="mcpServers.length === 0"
              class="p-8 text-center rounded-2xl bg-[#111420] border border-[#1e2336] text-slate-400 text-xs">
              {{ $t('settings.mcp_no_servers_configured') }}
            </div>

            <div v-else class="space-y-3">
              <div v-for="server in mcpServers" :key="server.id"
                class="p-4 rounded-2xl bg-[#111420] border border-[#1e2336] space-y-3 transition-all">
                <!-- Server Header -->
                <div class="flex items-start justify-between gap-4">
                  <div class="space-y-1 min-w-0 flex-1">
                    <div class="flex items-center gap-2 flex-wrap">
                      <h4 class="text-sm font-bold text-slate-100 shrink-0" :title="server.id === 'atena_native' ? $t('memory.server_atena_native_tools') : server.name">
                        {{ server.id === 'atena_native' ? $t('memory.server_atena_native_tools') : server.name }}
                      </h4>
                      <span
                        :class="[
                          'text-[10px] font-mono px-2 py-0.5 rounded-md border',
                          server.transport === 'builtin'
                            ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/30 font-semibold'
                            : 'bg-[#181d2e] text-indigo-300 border-[#262e45]'
                        ]">
                        {{ server.transport === 'builtin' ? $t('settings.mcp_builtin_transport') : server.transport }}
                      </span>
                      <span :class="[
                        'text-[10px] font-semibold px-2 py-0.5 rounded-md border',
                        server.enabled
                          ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                          : 'bg-slate-500/10 text-slate-400 border-slate-500/20'
                      ]">
                        {{ server.enabled ? $t('settings.mcp_status_active_caps') : $t('settings.mcp_status_inactive_caps') }}
                      </span>
                      <!-- Permission Mode Toggle Badge -->
                      <button
                        @click="toggleMcpPermission(server)"
                        :class="[
                          'text-[10px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1 transition-all cursor-pointer shadow-sm',
                          server.permission_mode === 'auto'
                            ? 'bg-cyan-500/15 text-cyan-300 border-cyan-500/30 hover:bg-cyan-500/25'
                            : 'bg-amber-500/15 text-amber-300 border-amber-500/30 hover:bg-amber-500/25'
                        ]"
                        :title="server.permission_mode === 'auto' ? $t('settings.mcp_mode_auto_tooltip') : $t('settings.mcp_mode_ask_tooltip')"
                      >
                        <component :is="server.permission_mode === 'auto' ? Zap : ShieldCheck" class="w-3 h-3" />
                        <span>{{ server.permission_mode === 'auto' ? $t('settings.mcp_mode_auto') : $t('settings.mcp_mode_ask') }}</span>
                      </button>
                    </div>
                    <p class="text-xs text-slate-400 truncate font-mono">
                      {{ server.transport === 'stdio' ? `${server.command} ${(server.args || []).join(' ')}` :
                        (server.transport === 'builtin' ? (server.description || $t('settings.mcp_native_server_desc')) : server.url) }}
                    </p>
                    <div v-if="server.headers && Object.keys(server.headers).length > 0"
                      class="flex flex-wrap gap-1 pt-0.5">
                      <span v-for="(val, key) in server.headers" :key="key"
                        class="text-[9.5px] font-mono px-1.5 py-0.5 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
                        {{ key }}: {{ String(key).toLowerCase().includes('auth') || String(key).toLowerCase().includes('key') ?
                          '••••••••' : val }}
                      </span>
                    </div>
                  </div>

                  <!-- Controls -->
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <button @click="testExistingMcpServer(server)"
                      :disabled="inspectingMcpId === server.id"
                      class="p-1.5 rounded-xl bg-[#141824] hover:bg-[#1b2030] border border-[#22283b] text-indigo-400 hover:text-indigo-300 transition-all cursor-pointer disabled:opacity-50"
                      :title="$t('settings.mcp_test_server_tooltip')">
                      <RefreshCw :class="['w-3.5 h-3.5', inspectingMcpId === server.id ? 'animate-spin' : '']" />
                    </button>

                    <button @click="toggleMcpServer(server)" :class="[
                      'px-3 py-1.5 rounded-xl text-xs font-semibold border transition-all cursor-pointer shadow-sm',
                      server.enabled
                        ? 'bg-emerald-500/15 border-emerald-500/30 text-emerald-300 hover:bg-emerald-500/25'
                        : 'bg-[#181d2e] border-[#262e45] text-slate-400 hover:text-slate-200'
                    ]">
                      {{ server.enabled ? $t('settings.mcp_disable_server') : $t('settings.mcp_enable_server') }}
                    </button>

                    <button v-if="server.transport !== 'builtin' && server.id !== 'atena_native'" @click="editMcpServer(server)"
                      class="p-1.5 rounded-xl bg-[#141824] hover:bg-[#1b2030] border border-[#22283b] text-slate-400 hover:text-slate-200 transition-all cursor-pointer"
                      :title="$t('settings.mcp_edit_server_tooltip')">
                      <Settings class="w-3.5 h-3.5" />
                    </button>

                    <button v-if="server.transport !== 'builtin' && server.id !== 'atena_native'" @click="deleteMcpServer(server.id)"
                      class="p-1.5 rounded-xl bg-[#141824] hover:bg-rose-500/15 border border-[#22283b] hover:border-rose-500/30 text-slate-400 hover:text-rose-300 transition-all cursor-pointer"
                      :title="$t('settings.mcp_delete_server_tooltip')">
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                <!-- Error Banner for Failed Server -->
                <div v-if="mcpServerErrors[server.id] && server.enabled" class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/25 text-rose-300 text-xs space-y-1.5">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-1.5 font-semibold text-rose-300">
                      <AlertTriangle class="w-3.5 h-3.5 text-rose-400 flex-shrink-0" />
                      <span>{{ $t('settings.mcp_server_comm_failed') }}</span>
                    </div>
                    <button
                      @click="testExistingMcpServer(server)"
                      :disabled="inspectingMcpId === server.id"
                      class="px-2 py-0.5 rounded-md bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/30 text-[10.5px] font-medium transition-all cursor-pointer flex items-center gap-1"
                    >
                      <RefreshCw :class="['w-3 h-3', inspectingMcpId === server.id ? 'animate-spin' : '']" />
                      <span>{{ $t('common.retry') }}</span>
                    </button>
                  </div>
                  <pre class="p-2 rounded-lg bg-black/40 border border-rose-500/20 text-[10.5px] text-rose-200 font-mono overflow-x-auto whitespace-pre-wrap leading-relaxed select-text">{{ mcpServerErrors[server.id] }}</pre>
                </div>

                <!-- Discovered Tools -->
                <div class="pt-2 border-t border-[#1c2133]">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wide">
                      {{ $t('settings.mcp_tools_count', { active: getActiveMcpServerToolsCount(server), total: (mcpServerTools[server.id] || []).length }) }}
                    </span>
                    <div class="flex items-center gap-2">
                      <button
                        v-if="(mcpServerTools[server.id] || []).length > 0"
                        @click="translateServerToolLabels(server)"
                        :disabled="isTranslatingTools[server.id] || isAnyTranslating"
                        class="px-2 py-0.5 rounded-lg bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/30 text-indigo-300 text-[10.5px] font-medium transition-all flex items-center gap-1 cursor-pointer disabled:opacity-50"
                        :title="$t('settings.mcp_translate_ai_tooltip')"
                      >
                        <Sparkles :class="['w-3 h-3 text-indigo-400', isTranslatingTools[server.id] ? 'animate-spin' : '']" />
                        <span>{{ translatingProgress[server.id] ? $t('settings.mcp_translating_progress', { current: translatingProgress[server.id]?.current, total: translatingProgress[server.id]?.total }) : $t('settings.mcp_translate_ai') }}</span>
                      </button>
                      <span v-if="inspectingMcpId === server.id" class="text-[10.5px] text-indigo-400 animate-pulse">
                        {{ $t('settings.mcp_fetching_tools') }}
                      </span>
                    </div>
                  </div>

                  <div v-if="!mcpServerTools[server.id] || (mcpServerTools[server.id]?.length ?? 0) === 0"
                    class="text-xs text-slate-400 italic py-1">
                    {{ inspectingMcpId === server.id ? $t('settings.mcp_loading_tools') : (mcpServerErrors[server.id] ? $t('settings.mcp_error_loading_tools') : $t('settings.mcp_empty_tools_or_disabled')) }}
                  </div>

                  <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-2">
                    <div
                      v-for="tool in mcpServerTools[server.id]"
                      :key="tool.name"
                      :class="[
                        'p-3 rounded-xl border flex flex-col justify-between gap-2.5 transition-all shadow-xs',
                        isMcpToolEnabled(server, tool.name)
                          ? 'bg-[#141826] border-[#22283b] hover:border-[#2f3750]'
                          : 'bg-[#0f121d]/80 border-[#1c2235] opacity-70 hover:opacity-95'
                      ]"
                    >
                      <div class="space-y-1.5">
                        <!-- Top Row: Full Width Title, Status Toggle & Terminal identifier -->
                        <div class="min-w-0">
                          <div class="group/toollabel flex items-start justify-between gap-2">
                            <h4
                              @click="openEditToolModal(server, tool)"
                              class="text-xs font-bold text-slate-100 hover:text-indigo-300 transition-colors cursor-pointer truncate flex items-center gap-1.5 flex-1 min-w-0"
                              :title="server.tool_labels?.[tool.name] || tool.label || tool.name"
                            >
                              <span class="truncate">{{ server.tool_labels?.[tool.name] || tool.label || tool.name }}</span>
                              <Pencil class="w-2.5 h-2.5 opacity-0 group-hover/toollabel:opacity-100 text-indigo-400 transition-opacity flex-shrink-0" />
                            </h4>

                            <!-- Tool Active/Inactive Badge Toggle -->
                            <button
                              @click.stop="toggleMcpTool(server, tool.name)"
                              :class="[
                                'text-[9.5px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1.5 transition-all cursor-pointer flex-shrink-0 shadow-2xs',
                                isMcpToolEnabled(server, tool.name)
                                  ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30 hover:bg-emerald-500/25'
                                  : 'bg-[#181d2e] text-slate-400 border-[#262e45] hover:bg-[#20273c] hover:text-slate-200'
                              ]"
                              :title="isMcpToolEnabled(server, tool.name) ? $t('modals.mcp_tool.disable_tool') : $t('modals.mcp_tool.enable_tool')"
                            >
                              <span class="w-1.5 h-1.5 rounded-full flex-shrink-0" :class="isMcpToolEnabled(server, tool.name) ? 'bg-emerald-400' : 'bg-slate-500'"></span>
                              <span>{{ isMcpToolEnabled(server, tool.name) ? $t('modals.mcp_tool.status_active') : $t('modals.mcp_tool.status_inactive') }}</span>
                            </button>
                          </div>
                          <div class="font-mono text-[10px] text-indigo-400/80 flex items-center gap-1 mt-0.5 truncate" :title="tool.name">
                            <Terminal class="w-2.5 h-2.5 flex-shrink-0" />
                            <span class="truncate">{{ tool.name }}</span>
                          </div>
                        </div>

                        <!-- Middle: Tool Description -->
                        <p class="text-[11px] text-slate-400 line-clamp-2 leading-relaxed">
                          {{ tool.description || $t('settings.mcp_no_desc') }}
                        </p>
                      </div>

                      <!-- Bottom Row: Actions Bar -->
                      <div class="pt-2 border-t border-white/[0.04] flex items-center justify-between gap-2 flex-wrap mt-auto">
                        <!-- Per-Tool Permission Toggle Badge -->
                        <button
                          @click="toggleToolPermission(server, tool.name)"
                          :class="[
                            'text-[9.5px] font-semibold px-2 py-0.5 rounded-md border flex items-center gap-1 transition-all cursor-pointer shadow-2xs',
                            getToolPermission(server, tool.name) === 'auto'
                              ? 'bg-cyan-500/15 text-cyan-300 border-cyan-500/30 hover:bg-cyan-500/25'
                              : 'bg-amber-500/15 text-amber-300 border-amber-500/30 hover:bg-amber-500/25'
                          ]"
                          :title="getToolPermission(server, tool.name) === 'auto' ? $t('settings.mcp_mode_auto_tooltip') : $t('settings.mcp_mode_ask_tooltip')"
                        >
                          <component :is="getToolPermission(server, tool.name) === 'auto' ? Zap : ShieldCheck" class="w-2.5 h-2.5" />
                          <span>{{ getToolPermission(server, tool.name) === 'auto' ? $t('settings.mcp_mode_direct') : $t('settings.mcp_mode_ask') }}</span>
                        </button>

                        <div class="flex items-center gap-1.5 flex-shrink-0">
                          <button
                            @click="openEditToolModal(server, tool)"
                            class="text-[10px] px-2.5 py-0.5 rounded-md bg-white/5 hover:bg-white/10 text-slate-300 hover:text-white border border-white/10 transition-all cursor-pointer flex items-center gap-1"
                            :title="$t('settings.mcp_edit_tool_tooltip')"
                          >
                            <Pencil class="w-2.5 h-2.5 text-indigo-400" />
                            <span>{{ $t('settings.mcp_edit_btn') }}</span>
                          </button>

                          <button
                            @click="openTestToolModal(server, tool)"
                            class="text-[10px] px-2.5 py-0.5 rounded-md bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-300 border border-indigo-500/20 transition-all cursor-pointer"
                          >
                            {{ $t('settings.mcp_test_btn') }}
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 3. SERVIDORES LOCAIS (MLX / OLLAMA) -->
        <div v-else-if="currentSection === 'servers'" class="space-y-5">
          <div class="flex items-center justify-between gap-4">
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Zap class="w-4 h-4 text-amber-400 flex-shrink-0" />
                <span class="truncate">{{ $t('settings.local_inference_servers') }}</span>
              </h3>
              <p class="text-xs text-slate-400 mt-0.5">{{ $t('settings.local_inference_servers_desc') }}</p>
            </div>

            <div class="flex items-center gap-2 flex-shrink-0">
              <button @click="$emit('checkServices')"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#141824] hover:bg-[#1b2030] border border-[#22283b] text-xs font-medium text-slate-200 transition-all active:scale-95 whitespace-nowrap cursor-pointer">
                <RefreshCw class="w-3.5 h-3.5 text-indigo-400" />
                <span>{{ $t('common.check') }}</span>
              </button>

              <button @click="$emit('stopAllServers')"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 border border-rose-500/30 text-xs font-medium text-rose-300 transition-all active:scale-95 whitespace-nowrap cursor-pointer">
                <Square class="w-3.5 h-3.5 text-rose-400" />
                <span>{{ $t('common.stop_all') }}</span>
              </button>
            </div>
          </div>

          <!-- Zero-Config Sidecars & Native Runtime Status Card -->
          <div class="p-4 rounded-2xl bg-gradient-to-r from-[#121624] to-[#0f1320] border border-[#20273c] space-y-3.5 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <div class="w-8 h-8 rounded-xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
                  <ShieldCheck class="w-4 h-4" />
                </div>
                <div>
                  <h4 class="text-xs font-bold text-slate-100 flex items-center gap-2">
                    <span>Zero-Config Runtimes & Sidecars</span>
                    <span class="text-[9.5px] px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-mono font-medium">{{ $t('settings.auto_embedded') }}</span>
                  </h4>
                  <p class="text-[10.5px] text-slate-400">{{ $t('settings.binaries_desc') }}</p>
                </div>
              </div>

              <button 
                @click="showRuntimeModal = true" 
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 border border-indigo-500/30 text-xs font-semibold text-indigo-300 transition-all active:scale-95 cursor-pointer"
              >
                <DownloadCloud class="w-3.5 h-3.5" />
                <span>{{ (runtimeStatus?.mlx_ready && runtimeStatus?.llama_server_available && runtimeStatus?.ffmpeg_available) ? $t('settings.repair_engines') : $t('settings.configure_engines') }}</span>
              </button>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5 pt-1">
              <!-- llama-server -->
              <div class="p-2.5 rounded-xl bg-[#0b0e17] border border-[#1b2133] space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-bold text-slate-200">llama-server</span>
                  <span :class="['w-2 h-2 rounded-full', runtimeStatus?.llama_server_available ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-rose-400']"></span>
                </div>
                <p class="text-[10px] text-slate-400 truncate">{{ runtimeStatus?.llama_server_available ? $t('settings.status_ready_metal') : $t('settings.status_not_found') }}</p>
              </div>

              <!-- ffmpeg -->
              <div class="p-2.5 rounded-xl bg-[#0b0e17] border border-[#1b2133] space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-bold text-slate-200">ffmpeg</span>
                  <span :class="['w-2 h-2 rounded-full', runtimeStatus?.ffmpeg_available ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-rose-400']"></span>
                </div>
                <p class="text-[10px] text-slate-400 truncate">{{ runtimeStatus?.ffmpeg_available ? $t('settings.status_ready_audio') : $t('settings.status_not_found') }}</p>
              </div>

              <!-- uv -->
              <div class="p-2.5 rounded-xl bg-[#0b0e17] border border-[#1b2133] space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-bold text-slate-200">uv engine</span>
                  <span :class="['w-2 h-2 rounded-full', runtimeStatus?.uv_available ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-rose-400']"></span>
                </div>
                <p class="text-[10px] text-slate-400 truncate">{{ runtimeStatus?.uv_available ? $t('settings.status_ready_sidecar') : $t('settings.status_not_found') }}</p>
              </div>

              <!-- MLX Python Venv (Only on macOS) -->
              <div v-if="supportsMlx" class="p-2.5 rounded-xl bg-[#0b0e17] border border-[#1b2133] space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-bold text-slate-200">MLX Runtime</span>
                  <span :class="['w-2 h-2 rounded-full', runtimeStatus?.mlx_ready ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-amber-400 animate-pulse']"></span>
                </div>
                <p class="text-[10px] text-slate-400 truncate">
                  {{ runtimeStatus?.mlx_ready ? (runtimeStatus?.mlx_version ? `${$t('settings.status_ready_metal')} (v${runtimeStatus.mlx_version})` : $t('settings.status_ready_metal')) : $t('settings.status_setup_pending') }}
                </p>
              </div>
            </div>
          </div>

          <!-- Server Cards Grid -->
          <div :class="['grid gap-4', supportsMlx ? 'grid-cols-1 md:grid-cols-2' : 'grid-cols-1']">
            <!-- MLX Card -->
            <div v-if="supportsMlx" class="p-4 rounded-2xl bg-[#111420] border border-[#1e2336] space-y-3 shadow-sm">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <div
                    class="w-9 h-9 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400">
                    <Zap class="w-4 h-4" />
                  </div>
                  <div>
                    <div class="flex items-center gap-2">
                      <h4 class="text-xs font-bold text-slate-100">MLX-LM Server</h4>
                      <span v-if="runtimeStatus?.mlx_version" class="text-[9.5px] font-mono font-medium px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20">
                        v{{ runtimeStatus.mlx_version }}
                      </span>
                    </div>
                    <p class="text-[10.5px] font-mono text-slate-400">http://{{ config.mlx_host }}:{{ config.mlx_port }}
                    </p>
                  </div>
                </div>

                <span :class="[
                  'text-[10px] font-bold uppercase px-2 py-0.5 rounded-md border flex items-center gap-1',
                  serviceHealth.mlx_online
                    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                    : 'bg-rose-500/10 text-rose-400 border-rose-500/30'
                ]">
                  <span
                    :class="['w-1.5 h-1.5 rounded-full', serviceHealth.mlx_online ? 'bg-emerald-400 animate-pulse' : 'bg-rose-400']"></span>
                  <span>{{ serviceHealth.mlx_online ? $t('common.online') : $t('common.offline') }}</span>
                </span>
              </div>

              <div class="flex items-center justify-between pt-1">
                <span class="text-[11px] text-slate-400">Apple Silicon Metal 3 Engine {{ runtimeStatus?.mlx_version ? `• mlx-lm v${runtimeStatus.mlx_version}` : '' }}</span>
                <button @click="$emit('startMlx')" :disabled="serviceHealth.mlx_online" :class="[
                  'px-3 py-1 rounded-xl text-xs font-semibold transition-all',
                  serviceHealth.mlx_online
                    ? 'bg-[#181d2e] text-slate-500 cursor-not-allowed border border-[#23283b]'
                    : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/20 active:scale-95 cursor-pointer'
                ]">
                  {{ serviceHealth.mlx_online ? $t('common.active') : $t('settings.start_mlx') }}
                </button>
              </div>
            </div>

            <!-- Ollama Card -->
            <div class="p-4 rounded-2xl bg-[#111420] border border-[#1e2336] space-y-3 shadow-sm">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <div
                    class="w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                    <Server class="w-4 h-4" />
                  </div>
                  <div>
                    <h4 class="text-xs font-bold text-slate-100">Ollama Service</h4>
                    <p class="text-[10.5px] font-mono text-slate-400">http://{{ config.ollama_host }}:{{
                      config.ollama_port }}</p>
                  </div>
                </div>

                <span :class="[
                  'text-[10px] font-bold uppercase px-2 py-0.5 rounded-md border flex items-center gap-1',
                  serviceHealth.ollama_online
                    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                    : 'bg-rose-500/10 text-rose-400 border-rose-500/30'
                ]">
                  <span
                    :class="['w-1.5 h-1.5 rounded-full', serviceHealth.ollama_online ? 'bg-emerald-400 animate-pulse' : 'bg-rose-400']"></span>
                  <span>{{ serviceHealth.ollama_online ? $t('common.online') : $t('common.offline') }}</span>
                </span>
              </div>

              <div class="flex items-center justify-between pt-1">
                <span class="text-[11px] text-slate-400">GGUF Runtime Engine</span>
                <button @click="$emit('startOllama')" :disabled="serviceHealth.ollama_online" :class="[
                  'px-3 py-1 rounded-xl text-xs font-semibold transition-all',
                  serviceHealth.ollama_online
                    ? 'bg-[#181d2e] text-slate-500 cursor-not-allowed border border-[#23283b]'
                    : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/20 active:scale-95 cursor-pointer'
                ]">
                  {{ serviceHealth.ollama_online ? $t('common.active') : $t('settings.start_ollama') }}
                </button>
              </div>
            </div>
          </div>

          <!-- Developer Logs Console (LM Studio Style) -->
          <DeveloperLogsConsole
            :logs="logs"
            :developerLogs="developerLogs"
            :isGenerating="isGenerating"
            @refreshLogs="$emit('refreshLogs')"
            @refreshDeveloperLogs="$emit('refreshDeveloperLogs')"
            @clearLogs="$emit('clearLogs')"
            @clearDeveloperLogs="$emit('clearDeveloperLogs')"
          />
        </div>


        <!-- 5. INFERÊNCIA & PENSAMENTO -->
        <div v-else-if="currentSection === 'inference'" class="space-y-5">
          <!-- Model Loading Guardrails Card -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm">
            <div class="p-4 border-b border-[#1e2336] flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide flex items-center gap-1.5">
                  <ShieldCheck class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.guardrails_title') || 'Model Loading Guardrails' }}</span>
                </h4>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.guardrails_desc') || 'Define precauções contra sobrecarga do sistema, limitando o uso de memória RAM/VRAM para evitar lentidão e travamentos do computador.' }}
                </p>
              </div>
              <span class="text-[10px] font-mono uppercase px-2.5 py-1 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full" :class="(props.config.guardrail_mode || 'relaxed') === 'off' ? 'bg-amber-400' : 'bg-emerald-400'"></span>
                <span>{{ currentGuardrailLimit }} GB Máx</span>
              </span>
            </div>

            <!-- Card Sub-header: Proteções de Carregamento de Modelo -->
            <div class="p-4 space-y-3">
              <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-200">
                <span>{{ $t('settings.guardrails_card_title') || 'Proteções de Carregamento de Modelo' }}</span>
                <span class="text-slate-400 hover:text-slate-200 transition-colors cursor-help" :title="$t('settings.guardrails_desc')">
                  <Info class="w-3.5 h-3.5" />
                </span>
              </div>

              <!-- Radio Options List -->
              <div class="space-y-2">
                <!-- 1. DESLIGADO (Não Recomendado) -->
                <label
                  @click="props.config.guardrail_mode = 'off'; saveSettings()"
                  :class="[
                    'flex items-start gap-3 p-3 rounded-xl border transition-all cursor-pointer select-none',
                    (props.config.guardrail_mode === 'off')
                      ? 'bg-amber-500/10 border-amber-500/40 text-slate-100 ring-1 ring-amber-500/20'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-300'
                  ]"
                >
                  <div class="pt-0.5">
                    <div :class="[
                      'w-4 h-4 rounded-full border flex items-center justify-center transition-all',
                      props.config.guardrail_mode === 'off' ? 'border-amber-400 bg-amber-500/20' : 'border-slate-500 bg-transparent'
                    ]">
                      <div v-if="props.config.guardrail_mode === 'off'" class="w-2 h-2 rounded-full bg-amber-400"></div>
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="text-xs font-bold text-slate-200">{{ $t('settings.guardrails_off') || 'DESLIGADO (Não Recomendado)' }}</div>
                    <div class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.guardrails_off_desc') || 'Sem precauções contra sobrecarga do sistema' }}</div>
                  </div>
                </label>

                <!-- 2. Relaxado -->
                <label
                  @click="props.config.guardrail_mode = 'relaxed'; saveSettings()"
                  :class="[
                    'flex items-start gap-3 p-3 rounded-xl border transition-all cursor-pointer select-none',
                    ((props.config.guardrail_mode || 'relaxed') === 'relaxed')
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-300'
                  ]"
                >
                  <div class="pt-0.5">
                    <div :class="[
                      'w-4 h-4 rounded-full border flex items-center justify-center transition-all',
                      (props.config.guardrail_mode || 'relaxed') === 'relaxed' ? 'border-indigo-400 bg-indigo-500/20' : 'border-slate-500 bg-transparent'
                    ]">
                      <div v-if="(props.config.guardrail_mode || 'relaxed') === 'relaxed'" class="w-2 h-2 rounded-full bg-indigo-400"></div>
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-bold text-slate-200">{{ $t('settings.guardrails_relaxed') || 'Relaxado' }}</span>
                      <span class="text-[10px] font-mono text-indigo-400">~{{ getGuardrailCeilingGb('relaxed') }} GB</span>
                    </div>
                    <div class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.guardrails_relaxed_desc') || 'Precauções leves contra sobrecarga do sistema' }}</div>
                  </div>
                </label>

                <!-- 3. Balanceado -->
                <label
                  @click="props.config.guardrail_mode = 'balanced'; saveSettings()"
                  :class="[
                    'flex items-start gap-3 p-3 rounded-xl border transition-all cursor-pointer select-none',
                    (props.config.guardrail_mode === 'balanced')
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-300'
                  ]"
                >
                  <div class="pt-0.5">
                    <div :class="[
                      'w-4 h-4 rounded-full border flex items-center justify-center transition-all',
                      props.config.guardrail_mode === 'balanced' ? 'border-indigo-400 bg-indigo-500/20' : 'border-slate-500 bg-transparent'
                    ]">
                      <div v-if="props.config.guardrail_mode === 'balanced'" class="w-2 h-2 rounded-full bg-indigo-400"></div>
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-bold text-slate-200">{{ $t('settings.guardrails_balanced') || 'Balanceado' }}</span>
                      <span class="text-[10px] font-mono text-emerald-400 font-semibold">~{{ getGuardrailCeilingGb('balanced') }} GB • Recomendado</span>
                    </div>
                    <div class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.guardrails_balanced_desc') || 'Precauções moderadas contra sobrecarga do sistema' }}</div>
                  </div>
                </label>

                <!-- 4. Rigoroso -->
                <label
                  @click="props.config.guardrail_mode = 'strict'; saveSettings()"
                  :class="[
                    'flex items-start gap-3 p-3 rounded-xl border transition-all cursor-pointer select-none',
                    (props.config.guardrail_mode === 'strict')
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-300'
                  ]"
                >
                  <div class="pt-0.5">
                    <div :class="[
                      'w-4 h-4 rounded-full border flex items-center justify-center transition-all',
                      props.config.guardrail_mode === 'strict' ? 'border-indigo-400 bg-indigo-500/20' : 'border-slate-500 bg-transparent'
                    ]">
                      <div v-if="props.config.guardrail_mode === 'strict'" class="w-2 h-2 rounded-full bg-indigo-400"></div>
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-bold text-slate-200">{{ $t('settings.guardrails_strict') || 'Rigoroso' }}</span>
                      <span class="text-[10px] font-mono text-indigo-400">~{{ getGuardrailCeilingGb('strict') }} GB</span>
                    </div>
                    <div class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.guardrails_strict_desc') || 'Precauções fortes contra sobrecarga do sistema' }}</div>
                  </div>
                </label>

                <!-- 5. Personalizado -->
                <label
                  @click="props.config.guardrail_mode = 'custom'; saveSettings()"
                  :class="[
                    'flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3 rounded-xl border transition-all cursor-pointer select-none',
                    (props.config.guardrail_mode === 'custom')
                      ? 'bg-indigo-600/15 border-indigo-500 text-slate-100 ring-1 ring-indigo-500/30'
                      : 'bg-[#141826] hover:bg-[#181d2e] border-[#22283b] text-slate-300'
                  ]"
                >
                  <div class="flex items-start gap-3 flex-1">
                    <div class="pt-0.5">
                      <div :class="[
                        'w-4 h-4 rounded-full border flex items-center justify-center transition-all',
                        props.config.guardrail_mode === 'custom' ? 'border-indigo-400 bg-indigo-500/20' : 'border-slate-500 bg-transparent'
                      ]">
                        <div v-if="props.config.guardrail_mode === 'custom'" class="w-2 h-2 rounded-full bg-indigo-400"></div>
                      </div>
                    </div>
                    <div>
                      <div class="text-xs font-bold text-slate-200">{{ $t('settings.guardrails_custom') || 'Personalizado' }}</div>
                      <div class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.guardrails_custom_desc') || 'Defina seu próprio limite para o tamanho máximo de modelo que pode ser carregado' }}</div>
                    </div>
                  </div>

                  <!-- Custom Limit Input (Exact representation from Screenshot 2) -->
                  <div v-if="props.config.guardrail_mode === 'custom'" class="flex items-center gap-2 pl-7 sm:pl-0" @click.stop>
                    <div class="flex items-center gap-1">
                      <span class="text-xs text-slate-300 font-medium">{{ $t('settings.guardrails_memory_limit') || 'Limite de Memória:' }}</span>
                      <span class="text-slate-400 cursor-help" :title="$t('settings.guardrails_memory_limit_desc')">
                        <Info class="w-3.5 h-3.5" />
                      </span>
                    </div>
                    <div class="flex items-center gap-1.5 bg-[#0e111a] border border-[#22283b] px-2 py-1 rounded-lg focus-within:border-indigo-500 transition-colors">
                      <input
                        type="number"
                        v-model.number="props.config.guardrail_custom_limit_gb"
                        @change="saveSettings"
                        @blur="saveSettings"
                        min="1"
                        :max="totalRamGb"
                        step="0.5"
                        class="w-12 bg-transparent text-right text-xs font-mono font-bold text-slate-100 outline-none"
                      />
                      <span class="text-xs font-bold text-slate-400 font-sans">GB</span>
                    </div>
                  </div>
                </label>
              </div>

              <!-- Helper footer explaining large prompts and stability -->
              <div class="pt-2 border-t border-[#1e2336]/60 flex items-start gap-2 text-[11px] text-slate-400">
                <ShieldCheck class="w-3.5 h-3.5 text-indigo-400 mt-0.5 flex-shrink-0" />
                <span>
                  As proteções previnem o congelamento do sistema ao carregar modelos pesados ou processar prompts longos, ajustando com segurança a janela de contexto para caber na memória RAM/VRAM disponível.
                </span>
              </div>
            </div>
          </div>

          <!-- Native Cognitive Associative Memory Card -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
            <div class="p-4 flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-9 h-9 rounded-xl bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shrink-0">
                  <Brain class="w-4 h-4" />
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <h4 class="text-xs font-bold text-slate-100">{{ $t('settings.associative_memory_title') }}</h4>
                    <span class="px-1.5 py-0.5 rounded text-[9.5px] font-bold bg-indigo-500/15 text-indigo-300 border border-indigo-500/30">
                      {{ $t('settings.native_badge') }}
                    </span>
                  </div>
                  <p class="text-[11px] text-slate-400 mt-0.5">
                    {{ $t('settings.associative_memory_learn_desc') }}
                  </p>
                </div>
              </div>

              <!-- Switch Toggle -->
              <button
                type="button"
                role="switch"
                :aria-checked="Boolean(props.config.enable_cognitive_memory === true)"
                @click="props.config.enable_cognitive_memory = !Boolean(props.config.enable_cognitive_memory); saveSettings()"
                :class="[
                  'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ml-4',
                  props.config.enable_cognitive_memory === true ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                ]"
              >
                <span
                  :class="[
                    'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    props.config.enable_cognitive_memory === true ? 'translate-x-5' : 'translate-x-0'
                  ]"
                />
              </button>
            </div>

            <!-- Granular Memory Subsystems -->
            <div v-if="props.config.enable_cognitive_memory === true" class="p-4 space-y-3 bg-[#0c0f1a]/60">
              <!-- Rede de Fatos (Declarative) -->
              <div class="flex items-center justify-between p-3 rounded-xl bg-[#141828] border border-[#1e253b]">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-lg bg-indigo-500/15 border border-indigo-500/25 flex items-center justify-center text-indigo-400 shrink-0">
                    <Network class="w-4 h-4" />
                  </div>
                  <div>
                    <h5 class="text-xs font-semibold text-slate-100">{{ $t('settings.facts_memory_title') }}</h5>
                    <p class="text-[10.5px] text-slate-400 mt-0.5">
                      {{ $t('settings.facts_memory_desc') }}
                    </p>
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(props.config.enable_facts_memory !== false)"
                  @click="props.config.enable_facts_memory = props.config.enable_facts_memory === false ? true : false; saveSettings()"
                  :class="[
                    'relative inline-flex h-5 w-10 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ml-4',
                    props.config.enable_facts_memory !== false ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      props.config.enable_facts_memory !== false ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>

              <!-- Habilidades (Procedural) -->
              <div class="flex items-center justify-between p-3 rounded-xl bg-[#141828] border border-[#1e253b]">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-lg bg-teal-500/15 border border-teal-500/25 flex items-center justify-center text-teal-400 shrink-0">
                    <Wrench class="w-4 h-4" />
                  </div>
                  <div>
                    <h5 class="text-xs font-semibold text-slate-100">{{ $t('settings.skills_memory_title') }}</h5>
                    <p class="text-[10.5px] text-slate-400 mt-0.5">
                      {{ $t('settings.skills_memory_desc') }}
                    </p>
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(props.config.enable_skills_memory !== false)"
                  @click="props.config.enable_skills_memory = props.config.enable_skills_memory === false ? true : false; saveSettings()"
                  :class="[
                    'relative inline-flex h-5 w-10 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ml-4',
                    props.config.enable_skills_memory !== false ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      props.config.enable_skills_memory !== false ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>

              <!-- Diário de Bordo (Episodic) -->
              <div class="flex items-center justify-between p-3 rounded-xl bg-[#141828] border border-[#1e253b]">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-lg bg-amber-500/15 border border-amber-500/25 flex items-center justify-center text-amber-400 shrink-0">
                    <BookOpen class="w-4 h-4" />
                  </div>
                  <div>
                    <h5 class="text-xs font-semibold text-slate-100">{{ $t('settings.episodic_memory_title') }}</h5>
                    <p class="text-[10.5px] text-slate-400 mt-0.5">
                      {{ $t('settings.episodic_memory_desc') }}
                    </p>
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(props.config.enable_episodic_memory !== false)"
                  @click="props.config.enable_episodic_memory = props.config.enable_episodic_memory === false ? true : false; saveSettings()"
                  :class="[
                    'relative inline-flex h-5 w-10 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ml-4',
                    props.config.enable_episodic_memory !== false ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span
                    :class="[
                      'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      props.config.enable_episodic_memory !== false ? 'translate-x-5' : 'translate-x-0'
                    ]"
                  />
                </button>
              </div>
            </div>

            <!-- Context Info / Recommendation -->
            <div class="p-3 bg-[#141826]/60 flex items-center gap-2 text-[10.5px] text-slate-400">
              <Sparkles class="w-3.5 h-3.5 text-amber-400 shrink-0" />
              <span>
                {{ $t('settings.associative_memory_desc') }}
              </span>
            </div>
          </div>

          <!-- Thinking Mode Card -->
          <div
            class="rounded-2xl bg-gradient-to-b from-[#181426] to-[#120f1e] border border-[#3b2b5c]/80 p-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <div
                  class="w-9 h-9 rounded-xl bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-300">
                  <BrainCircuit class="w-4 h-4" />
                </div>
                <div>
                  <h4 class="text-xs font-bold text-slate-100">{{ $t('settings.thinking_mode') }}</h4>
                  <p class="text-[11px] text-purple-300/80">{{ $t('settings.thinking_tokens_desc') }}</p>
                </div>
              </div>

              <button type="button" role="switch" :aria-checked="Boolean(params.enable_thinking !== false)" @click="params.enable_thinking = params.enable_thinking === false ? true : false"
                :class="[
                  'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                  params.enable_thinking !== false ? 'bg-purple-600 shadow-sm shadow-purple-600/30' : 'bg-[#1e2436]'
                ]">
                <span :class="[
                  'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                  params.enable_thinking !== false ? 'translate-x-5' : 'translate-x-0'
                ]" />
              </button>
            </div>
          </div>

          <!-- KV Cache Quantization Card -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] p-4 space-y-3.5 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <div class="w-9 h-9 rounded-xl bg-teal-500/20 border border-teal-500/30 flex items-center justify-center text-teal-300">
                  <Zap class="w-4 h-4" />
                </div>
                <div>
                  <h4 class="text-xs font-bold text-slate-100">{{ $t('settings.kv_cache_metal_title') }}</h4>
                  <p class="text-[11px] text-teal-300/80">{{ $t('settings.kv_cache_metal_desc') }}</p>
                </div>
              </div>
            </div>

            <div class="space-y-1.5 pt-1">
              <label class="text-[11px] font-semibold text-slate-300">{{ $t('settings.kv_cache_precision_label') }}</label>
              <select
                v-model="params.kv_cache_quant"
                class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] focus:border-teal-500 text-xs text-slate-100 font-mono outline-none"
              >
                <option value="f16">{{ $t('settings.kv_cache_f16') }}</option>
                <option value="q8_0">{{ $t('settings.kv_cache_q8') }}</option>
                <option value="q4_0">{{ $t('settings.kv_cache_q4') }}</option>
                <option value="q4_1">{{ $t('settings.kv_cache_q4_1') }}</option>
                <option value="q5_0">{{ $t('settings.kv_cache_q5') }}</option>
                <option value="iq4_nl">{{ $t('settings.kv_cache_iq4_nl') }}</option>
              </select>
            </div>

            <div class="pt-2 border-t border-[#1e2336] space-y-2">
              <div class="flex items-center justify-between">
                <div>
                  <span class="text-xs font-semibold text-slate-200 block">{{ $t('settings.prompt_cache_reuse') }}</span>
                  <span class="text-[11px] text-slate-400">{{ $t('settings.prompt_cache_reuse_desc') }}</span>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(params.enable_prompt_cache !== false)"
                  @click="params.enable_prompt_cache = params.enable_prompt_cache === false ? true : false"
                  :class="[
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    params.enable_prompt_cache !== false ? 'bg-teal-600 shadow-sm shadow-teal-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span :class="[
                    'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    params.enable_prompt_cache !== false ? 'translate-x-5' : 'translate-x-0'
                  ]" />
                </button>
              </div>

              <div class="flex items-center justify-between pt-2 border-t border-[#1e2336]">
                <div>
                  <span class="text-xs font-semibold text-slate-200 block">{{ $t('settings.flash_attention_metal') }}</span>
                  <span class="text-[11px] text-slate-400">{{ $t('settings.flash_attention_metal_desc') }}</span>
                </div>
                <button
                  type="button"
                  role="switch"
                  :aria-checked="Boolean(params.flash_attention !== false)"
                  @click="params.flash_attention = params.flash_attention === false ? true : false"
                  :class="[
                    'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
                    params.flash_attention !== false ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
                  ]"
                >
                  <span :class="[
                    'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    params.flash_attention !== false ? 'translate-x-5' : 'translate-x-0'
                  ]" />
                </button>
              </div>
            </div>
          </div>

          <!-- System Prompt Card -->
          <div class="rounded-2xl bg-[#111420] border border-[#1e2336] p-4 space-y-2 shadow-sm">
            <h4 class="text-xs font-bold text-slate-200 uppercase tracking-wide">{{ $t('settings.default_system_prompt') }}</h4>
            <textarea v-model="params.system_prompt" rows="3"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-xs text-slate-100 font-sans outline-none focus:border-indigo-500 resize-none"></textarea>
          </div>
        </div>

        <!-- 5. HARDWARE & METAL -->
        <div v-else-if="currentSection === 'hardware'" class="space-y-5">
          <div class="p-5 rounded-2xl bg-[#111420] border border-[#1e2336] space-y-4">
            <div class="flex items-center gap-3">
              <div
                class="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
                <Cpu class="w-5 h-5" />
              </div>
              <div>
                <h4 class="text-sm font-bold text-slate-100">{{ hardware.chip_name || (supportsMlx ? 'Apple Silicon' : (hardware.device_name || 'CPU')) }}</h4>
                <p class="text-xs text-slate-400">
                  {{ supportsMlx ? `${hardware.gpu_cores || 16} GPU Cores • ${hardware.metal_version || 'Metal 3'}` : (hardware.acceleration_backend && hardware.acceleration_backend !== 'CPU' ? `${hardware.acceleration_backend} Acceleration` : 'CPU Execution') }}
                </p>
              </div>
            </div>

            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 pt-2">
              <div class="p-3 rounded-xl bg-[#141826] border border-[#1e2336]">
                <span class="text-[10px] text-slate-400 uppercase tracking-wider block">{{ $t('settings.ram_total') }}</span>
                <span class="text-base font-bold font-mono text-indigo-300">{{ Number(hardware.total_ram_gb || 16).toFixed(1) }} GB</span>
              </div>
              <div class="p-3 rounded-xl bg-[#141826] border border-[#1e2336]">
                <span class="text-[10px] text-teal-400 uppercase tracking-wider block">{{ $t('settings.ram_ai') }}</span>
                <span class="text-base font-bold font-mono text-teal-300">{{ (hardware.ai_ram_gb || hardware.used_vram_gb || 0).toFixed(1) }} GB</span>
              </div>
              <div class="p-3 rounded-xl bg-[#141826] border border-[#1e2336]">
                <span class="text-[10px] text-slate-400 uppercase tracking-wider block">{{ $t('settings.ram_system') }}</span>
                <span class="text-base font-bold font-mono text-purple-300">{{ (hardware.system_other_ram_gb || 0).toFixed(1) }} GB</span>
              </div>
              <div class="p-3 rounded-xl bg-[#141826] border border-[#1e2336]">
                <span class="text-[10px] text-emerald-400 uppercase tracking-wider block">{{ $t('settings.ram_free') }}</span>
                <span class="text-base font-bold font-mono text-emerald-300">{{ (hardware.free_ram_gb || 0).toFixed(1) }} GB</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Enhanced Add / Edit MCP Server Modal -->
    <div v-if="isMcpModalOpen"
      class="fixed inset-0 bg-black/70 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div
        class="w-full max-w-xl bg-[#0f121d] border border-[#22283b] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        <!-- Modal Header -->
        <div class="px-5 py-4 border-b border-[#22283b] flex items-center justify-between bg-[#131724]">
          <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
            <Wrench class="w-4 h-4 text-indigo-400" />
            <span>{{ isEditingMcp ? $t('settings.mcp_edit_server') : $t('settings.mcp_new_server') }}</span>
          </h3>
          <button @click="isMcpModalOpen = false" class="text-slate-400 hover:text-slate-200">
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Modal Body Scrollable -->
        <div class="p-5 space-y-4 text-xs overflow-y-auto flex-1">
          <!-- Server Name -->
          <div class="space-y-1">
            <label class="font-semibold text-slate-300">{{ $t('settings.mcp_server_name') }}</label>
            <input type="text" v-model="mcpModalForm.name" :placeholder="$t('settings.mcp_server_name_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 outline-none focus:border-indigo-500 font-sans" />
          </div>

          <!-- Transport -->
          <div class="space-y-1">
            <label class="font-semibold text-slate-300">{{ $t('settings.mcp_transport_type') }}</label>
            <div class="grid grid-cols-2 gap-2">
              <button type="button" @click="mcpModalForm.transport = 'stdio'" :class="[
                'py-2 px-3 rounded-xl border text-xs font-semibold transition-all cursor-pointer',
                mcpModalForm.transport === 'stdio'
                  ? 'bg-indigo-600 border-indigo-500 text-white shadow-sm'
                  : 'bg-[#151926] border-[#22283b] text-slate-400 hover:text-slate-200'
              ]">
                {{ $t('settings.mcp_transport_stdio') }}
              </button>
              <button type="button" @click="mcpModalForm.transport = 'sse'" :class="[
                'py-2 px-3 rounded-xl border text-xs font-semibold transition-all cursor-pointer',
                mcpModalForm.transport === 'sse' || mcpModalForm.transport === 'http'
                  ? 'bg-indigo-600 border-indigo-500 text-white shadow-sm'
                  : 'bg-[#151926] border-[#22283b] text-slate-400 hover:text-slate-200'
              ]">
                {{ $t('settings.mcp_transport_sse') }}
              </button>
            </div>
          </div>

          <!-- Stdio Specific Settings -->
          <template v-if="mcpModalForm.transport === 'stdio'">
            <div class="space-y-1">
              <label class="font-semibold text-slate-300">{{ $t('settings.mcp_command_label') }}</label>
              <input type="text" v-model="mcpModalForm.command" :placeholder="$t('settings.mcp_command_placeholder')"
                class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
            </div>

            <!-- Arguments (args) List -->
            <div class="space-y-1.5">
              <div class="flex items-center justify-between">
                <label class="font-semibold text-slate-300">{{ $t('settings.mcp_args_label') }}</label>
                <button type="button" @click="addMcpArg"
                  class="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-medium cursor-pointer">
                  <Plus class="w-3 h-3" />
                  <span>{{ $t('settings.mcp_add_arg') }}</span>
                </button>
              </div>
              <div v-if="mcpModalForm.argsList.length === 0" class="text-[11px] text-slate-400 italic">
                {{ $t('settings.mcp_no_args') }}
              </div>
              <div v-for="(arg, idx) in mcpModalForm.argsList" :key="idx" class="flex items-center gap-2">
                <input type="text" v-model="mcpModalForm.argsList[idx]"
                  :placeholder="$t('settings.mcp_arg_placeholder')"
                  class="flex-1 px-3 py-1.5 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
                <button type="button" @click="removeMcpArg(idx)" class="p-1.5 text-slate-400 hover:text-rose-400">
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </template>

          <!-- HTTP/SSE Specific Settings -->
          <template v-else>
            <div class="space-y-1">
              <label class="font-semibold text-slate-300">{{ $t('settings.mcp_url_label') }}</label>
              <input type="text" v-model="mcpModalForm.url" placeholder="https://example.com/mcp"
                class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
            </div>

            <!-- Custom Headers (headers) Key-Value List -->
            <div class="space-y-1.5 pt-1">
              <div class="flex items-center justify-between">
                <label class="font-semibold text-slate-300">{{ $t('settings.mcp_headers_label') }}</label>
                <button type="button" @click="addMcpHeader"
                  class="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-medium cursor-pointer">
                  <Plus class="w-3 h-3" />
                  <span>{{ $t('settings.mcp_add_header') }}</span>
                </button>
              </div>
              <div v-if="mcpModalForm.headersList.length === 0" class="text-[11px] text-slate-400 italic">
                {{ $t('settings.mcp_no_headers') }}
              </div>
              <div v-for="(h, idx) in mcpModalForm.headersList" :key="idx" class="flex items-center gap-2">
                <input type="text" v-model="h.key" :placeholder="$t('settings.mcp_header_key_placeholder')"
                  class="w-1/3 px-3 py-1.5 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
                <input type="text" v-model="h.value" :placeholder="$t('settings.mcp_header_val_placeholder')"
                  class="flex-1 px-3 py-1.5 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
                <button type="button" @click="removeMcpHeader(idx)" class="p-1.5 text-slate-400 hover:text-rose-400">
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </template>

          <!-- Environment Variables (env) for both stdio & http -->
          <div class="space-y-1.5 pt-1 border-t border-[#1e2336]">
            <div class="flex items-center justify-between">
              <label class="font-semibold text-slate-300">{{ $t('settings.mcp_env_label') }}</label>
              <button type="button" @click="addMcpEnv"
                class="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-medium cursor-pointer">
                <Plus class="w-3 h-3" />
                <span>{{ $t('settings.mcp_add_env') }}</span>
              </button>
            </div>
            <div v-if="mcpModalForm.envList.length === 0" class="text-[11px] text-slate-400 italic">
              {{ $t('settings.mcp_no_env') }}
            </div>
            <div v-for="(envItem, idx) in mcpModalForm.envList" :key="idx" class="flex items-center gap-2">
              <input type="text" v-model="envItem.key" placeholder="VAR_NAME"
                class="w-1/3 px-3 py-1.5 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500 uppercase" />
              <input type="text" v-model="envItem.value" :placeholder="$t('settings.mcp_val_placeholder')"
                class="flex-1 px-3 py-1.5 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500" />
              <button type="button" @click="removeMcpEnv(idx)" class="p-1.5 text-slate-400 hover:text-rose-400">
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          <!-- Permission Mode Selector -->
          <div class="space-y-1 pt-1 border-t border-[#1e2336]">
            <label class="font-semibold text-slate-300 flex items-center gap-1.5">
              <ShieldCheck class="w-3.5 h-3.5 text-indigo-400" />
              <span>{{ $t('settings.mcp_permission_control') }}</span>
            </label>
            <div class="grid grid-cols-2 gap-2 pt-1">
              <button
                type="button"
                @click="mcpModalForm.permission_mode = 'ask'"
                :class="[
                  'p-2.5 rounded-xl border text-left flex flex-col gap-1 transition-all cursor-pointer',
                  mcpModalForm.permission_mode !== 'auto'
                    ? 'bg-indigo-600/20 border-indigo-500 text-indigo-200 ring-1 ring-indigo-500/40 shadow-sm'
                    : 'bg-[#151926] border-[#22283b] text-slate-400 hover:text-slate-200'
                ]"
              >
                <span class="font-bold text-xs flex items-center gap-1.5 text-indigo-300">
                  <ShieldCheck class="w-3.5 h-3.5" />
                  {{ $t('settings.mcp_ask_permission') }}
                </span>
                <span class="text-[10px] text-slate-400 leading-tight">
                  {{ $t('settings.mcp_ask_permission_desc') }}
                </span>
              </button>

              <button
                type="button"
                @click="mcpModalForm.permission_mode = 'auto'"
                :class="[
                  'p-2.5 rounded-xl border text-left flex flex-col gap-1 transition-all cursor-pointer',
                  mcpModalForm.permission_mode === 'auto'
                    ? 'bg-cyan-600/20 border-cyan-500 text-cyan-200 ring-1 ring-cyan-500/40 shadow-sm'
                    : 'bg-[#151926] border-[#22283b] text-slate-400 hover:text-slate-200'
                ]"
              >
                <span class="font-bold text-xs flex items-center gap-1.5 text-cyan-300">
                  <Zap class="w-3.5 h-3.5" />
                  {{ $t('settings.mcp_always_allow') }}
                </span>
                <span class="text-[10px] text-slate-400 leading-tight">
                  {{ $t('settings.mcp_always_allow_desc') }}
                </span>
              </button>
            </div>
          </div>

          <!-- Description -->
          <div class="space-y-1 pt-1 border-t border-[#1e2336]">
            <label class="font-semibold text-slate-300">{{ $t('settings.mcp_description_label') }}</label>
            <input type="text" v-model="mcpModalForm.description" :placeholder="$t('settings.mcp_description_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 outline-none focus:border-indigo-500" />
          </div>

          <!-- Pre-Save Live MCP Server Test Section -->
          <div class="pt-3 border-t border-[#1e2336] space-y-2.5">
            <div class="flex items-center justify-between gap-3">
              <div>
                <label class="font-semibold text-slate-200 flex items-center gap-1.5">
                  <Zap class="w-3.5 h-3.5 text-indigo-400" />
                  <span>{{ $t('settings.mcp_test_before_save') }}</span>
                </label>
                <p class="text-[11px] text-slate-400 mt-0.5">
                  {{ $t('settings.mcp_test_desc') }}
                </p>
              </div>
              <button
                type="button"
                @click="testMcpModalServer"
                :disabled="isTestingMcp || (!mcpModalForm.command && !mcpModalForm.url)"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-indigo-500/15 hover:bg-indigo-500/25 border border-indigo-500/30 text-xs font-semibold text-indigo-300 transition-all cursor-pointer disabled:opacity-40 active:scale-95 flex-shrink-0 shadow-sm"
              >
                <component :is="isTestingMcp ? Loader2 : Play" :class="['w-3.5 h-3.5 text-indigo-400', isTestingMcp ? 'animate-spin' : '']" />
                <span>{{ isTestingMcp ? $t('settings.mcp_testing') : $t('settings.mcp_test_conn') }}</span>
              </button>
            </div>

            <!-- Testing In Progress -->
            <div v-if="isTestingMcp" class="p-3 rounded-xl bg-indigo-500/10 border border-indigo-500/25 text-indigo-300 flex items-center gap-2.5 text-xs animate-pulse">
              <Loader2 class="w-4 h-4 animate-spin text-indigo-400 flex-shrink-0" />
              <span>{{ $t('settings.mcp_testing_hint') }}</span>
            </div>

            <!-- Test Success Banner -->
            <div v-else-if="mcpTestResult && mcpTestResult.success" class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 space-y-2 text-xs">
              <div class="flex items-center gap-2 font-bold text-emerald-300">
                <CheckCircle2 class="w-4 h-4 text-emerald-400 flex-shrink-0" />
                <span>{{ $t('settings.mcp_test_success', { count: mcpTestResult.count }) }}</span>
              </div>
              <div v-if="mcpTestResult.tools && mcpTestResult.tools.length > 0" class="flex flex-wrap gap-1.5 pt-1">
                <div
                  v-for="t in mcpTestResult.tools"
                  :key="t.name"
                  class="px-2 py-1 rounded-lg bg-emerald-500/15 text-emerald-200 border border-emerald-500/25 font-mono text-[11px] flex items-center gap-1.5"
                  :title="t.description || ''"
                >
                  <Terminal class="w-3 h-3 text-emerald-400 flex-shrink-0" />
                  <span class="font-bold">{{ t.name }}</span>
                  <span v-if="t.description" class="text-[10px] text-emerald-300/70 font-sans truncate max-w-[200px]">({{ t.description }})</span>
                </div>
              </div>
              <div v-else class="text-[11px] text-slate-400 italic">
                {{ $t('settings.mcp_test_empty') }}
              </div>
            </div>

            <!-- Test Error Banner -->
            <div v-else-if="mcpTestResult && !mcpTestResult.success" class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 space-y-2 text-xs">
              <div class="flex items-center gap-2 font-bold text-rose-300">
                <AlertTriangle class="w-4 h-4 text-rose-400 flex-shrink-0" />
                <span>{{ $t('settings.mcp_test_fail') }}</span>
              </div>
              <pre class="p-2.5 rounded-lg bg-black/50 border border-rose-500/25 text-[11px] text-rose-200 font-mono overflow-x-auto whitespace-pre-wrap leading-relaxed select-text">{{ mcpTestResult.error }}</pre>
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="p-4 border-t border-[#22283b] bg-[#131724] flex items-center justify-between gap-2">
          <div>
            <button
              type="button"
              @click="testMcpModalServer"
              :disabled="isTestingMcp || (!mcpModalForm.command && !mcpModalForm.url)"
              class="px-3.5 py-1.5 rounded-xl bg-[#171c2b] hover:bg-[#20273d] text-indigo-300 text-xs font-semibold border border-indigo-500/30 transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-40"
            >
              <component :is="isTestingMcp ? Loader2 : Play" :class="['w-3.5 h-3.5 text-indigo-400', isTestingMcp ? 'animate-spin' : '']" />
              <span>{{ isTestingMcp ? $t('settings.mcp_testing') : $t('settings.mcp_test_mcp') }}</span>
            </button>
          </div>
          <div class="flex items-center gap-2">
            <button @click="isMcpModalOpen = false"
              class="px-3.5 py-1.5 rounded-xl bg-[#171c2b] text-slate-300 text-xs font-medium hover:bg-[#1f253a] cursor-pointer">
              {{ $t('common.cancel') }}
            </button>
            <button @click="saveMcpModal"
              class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 cursor-pointer">
              {{ $t('settings.mcp_save_server') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Live Test Tool Modal -->
    <div v-if="testToolModal.isOpen"
      class="fixed inset-0 bg-black/70 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div
        class="w-full max-w-lg bg-[#0f121d] border border-[#22283b] rounded-2xl shadow-2xl overflow-hidden flex flex-col">
        <div class="px-5 py-4 border-b border-[#22283b] flex items-center justify-between bg-[#131724]">
          <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
            <Terminal class="w-4 h-4 text-indigo-400" />
            <span>{{ $t('settings.mcp_test_tool_title', { name: testToolModal.tool?.name }) }}</span>
          </h3>
          <button @click="testToolModal.isOpen = false" class="text-slate-400 hover:text-slate-200">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="p-5 space-y-3 text-xs">
          <p class="text-slate-400">{{ testToolModal.tool?.description }}</p>
          <div class="space-y-1">
            <label class="font-semibold text-slate-300">{{ $t('settings.mcp_json_args') }}</label>
            <textarea v-model="testToolModal.argsJson" rows="4"
              class="w-full px-3 py-2 rounded-xl bg-[#151926] border border-[#22283b] text-slate-100 font-mono outline-none focus:border-indigo-500"></textarea>
          </div>

          <div v-if="testToolModal.result" class="space-y-1">
            <label class="font-semibold text-emerald-400">{{ $t('settings.mcp_tool_result') }}</label>
            <pre
              class="p-3 rounded-xl bg-[#131724] border border-[#22283b] font-mono text-[11px] text-slate-200 max-h-48 overflow-y-auto whitespace-pre-wrap">
          {{ testToolModal.result }}</pre>
          </div>
        </div>

        <div class="p-4 border-t border-[#22283b] bg-[#131724] flex items-center justify-between">
          <button @click="testToolModal.isOpen = false"
            class="px-3.5 py-1.5 rounded-xl bg-[#171c2b] text-slate-300 text-xs">
            {{ $t('common.close') }}
          </button>
          <button @click="runMcpTestTool" :disabled="testToolModal.isRunning"
            class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md flex items-center gap-1.5 disabled:opacity-60">
            <Loader2 v-if="testToolModal.isRunning" class="w-3.5 h-3.5 animate-spin" />
            <span>{{ testToolModal.isRunning ? $t('settings.mcp_running_tool') : $t('settings.mcp_run_tool') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Modal de Configuração do Runtime MLX -->
    <RuntimeSetupModal v-model="showRuntimeModal" @ready="fetchRuntimeStatus" />

    <!-- Modal de Edição de Rótulos e Parâmetros da Ferramenta MCP -->
    <EditMcpToolModal
      :is-open="isEditToolModalOpen"
      :server="selectedServerToEdit"
      :tool="selectedToolToEdit"
      :config="props.config"
      @close="isEditToolModalOpen = false"
      @saved="handleToolDetailsSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, inject, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { saveTextFile } from '~/utils/exportMarkdown'
import { addNotification } from '~/utils/notifications'
import type { AppConfig, GenerationParams, HardwareInfo, McpServerConfig, McpToolDefinition, DetectedModelDirectory, ServerRequestLog, DeveloperLogEntry } from '~/types'
import DeveloperLogsConsole from '../settings/DeveloperLogsConsole.vue'

const platformInfo = inject<Ref<{ os: string; supports_mlx: boolean }>>('platformInfo', ref({ os: 'macos', supports_mlx: true }))
const supportsMlx = inject<Ref<boolean> | ComputedRef<boolean>>('supportsMlx', computed(() => true))
import {
  Settings,
  Folder,
  FolderOpen,
  Cpu,
  Mic,
  FolderPlus,
  Cloud,
  Zap,
  Wrench,
  BrainCircuit,
  Globe,
  Info,
  Search,
  ChevronRight,
  ChevronLeft,
  Save,
  Check,
  Minus,
  Plus,
  Trash2,
  RefreshCw,
  Terminal,
  Square,
  Server,
  FolderTree,
  Brain,
  X,
  Loader2,
  ShieldCheck,
  ShieldAlert,
  Clock,
  Calendar,
  Sparkles,
  DownloadCloud,
  Play,
  AlertTriangle,
  CheckCircle2,
  Database,
  Sun,
  Moon,
  Monitor,
  Blocks,
  Code2,
  Sliders,
  FileText,
  ExternalLink,
  BookOpen,
  Download,
  Upload,
  Archive,
  Network,
  Radio
} from 'lucide-vue-next'
import RuntimeSetupModal from '../RuntimeSetupModal.vue'
import EditMcpToolModal from '../EditMcpToolModal.vue'
import CloudProvidersSettings from '../settings/CloudProvidersSettings.vue'
import GatewaysSettings from '../settings/GatewaysSettings.vue'
import AutomationSettings from '../settings/AutomationSettings.vue'
import { formatLiveDateTime, getTemporalContextPrompt, getDefaultTimezone } from '~/utils/dateContext'
import { contractUserPath } from '~/utils/pathUtils'
import { useAppLocale } from '../../composables/useLocale'
import { usePlugins } from '../../composables/usePlugins'

const { locale, currentLocale, setLocale, supportedLocales, t } = useAppLocale()
const currentLocaleItem = computed(() => supportedLocales.find((l) => l.code === currentLocale.value) || supportedLocales[0])
const {
  plugins,
  isLoadingPlugins,
  pluginsFolder,
  fetchPlugins,
  togglePlugin,
  openPluginsFolder,
  isMemoryPluginActive,
  isAgyPluginActive,
  activePluginSettingsViews,
  savePluginSettings,
  updatePluginSetting
} = usePlugins()

const openExternalUrl = async (url: string) => {
  if (!url) return
  try {
    await invoke('open_url', { url })
  } catch (err) {
    console.error('Failed to open URL via Tauri, falling back to window.open:', err)
    window.open(url, '_blank')
  }
}

interface Props {
  config?: AppConfig
  params?: GenerationParams
  serviceHealth?: any
  logs?: ServerRequestLog[]
  developerLogs?: DeveloperLogEntry[]
  isGenerating?: boolean
  hardware?: HardwareInfo
  currentTheme?: string
  initialSection?: string | null
  sessions?: any[]
}

const props = withDefaults(defineProps<Props>(), {
  config: () => ({
    models_directory: '~/.atena/models',
    models_directories: ['~/.atena/models'],
    auto_load_last_model: true,
    gpu_offload: true,
    thread_count: 8,
    ollama_host: '127.0.0.1',
    ollama_port: 11434,
    mlx_host: '127.0.0.1',
    mlx_port: 8080,
    timezone: 'America/Sao_Paulo',
    inject_current_date: false,
    inject_message_time: false,
    whisper_model: 'mlx-community/whisper-small-mlx',
    whisper_language: 'pt',
    enable_cognitive_memory: false,
    enable_facts_memory: true,
    enable_skills_memory: true,
    enable_episodic_memory: true,
    theme: 'dark'
  }),
  params: () => ({
    system_prompt: '',
    enable_thinking: true,
    thinking_budget: 2048
  }),
  serviceHealth: () => ({ mlx_online: false, ollama_online: false }),
  logs: () => [],
  developerLogs: () => [],
  isGenerating: false,
  hardware: () => ({
    chip_name: 'Apple Silicon',
    gpu_cores: 8,
    metal_version: '3.0',
    total_vram_gb: 16,
    used_vram_gb: 0,
    system_ram_gb: 16,
    used_system_ram_gb: 0
  }),
  currentTheme: 'dark',
  initialSection: null
})

const emit = defineEmits<{
  saveConfig: [config?: any]
  checkServices: []
  startMlx: []
  startOllama: []
  stopAllServers: []
  clearLogs: []
  refreshLogs: []
  clearDeveloperLogs: []
  refreshDeveloperLogs: []
  refreshMcp: []
  openAgyLogin: []
  openSetup: []
  setTheme: [themeKey: string]
  setLanguage: [localeCode: string]
  refreshModels: []
  openArchivedModal: []
}>()

const handleTogglePlugin = (plugin: any) => {
  togglePlugin(plugin.id, !plugin.enabled)
}

const archivedCount = computed(() => {
  return (props.sessions || []).filter((s: any) => Boolean(s.archived)).length
})

// Theme selection logic
const activeTheme = computed(() => props.currentTheme || props.config?.theme || 'dark')

const handleSelectTheme = (themeKey: string) => {
  if (props.config) {
    props.config.theme = themeKey
  }
  emit('setTheme', themeKey)
  saveSettings()
}

const handleSelectLocale = (localeCode: string) => {
  setLocale(localeCode)
  if (props.config) {
    props.config.language = localeCode
  }
  emit('setLanguage', localeCode)
  saveSettings()
}

// Multi-path and AGY States
const agySessionStatus = ref({
  installed: false,
  authenticated: false,
  active_account: null
})

const checkAgySession = async () => {
  try {
    const res = await invoke<any>('check_agy_session')
    if (res) agySessionStatus.value = res
  } catch (e) {
    console.warn('Erro ao checar sessão AGY:', e)
  }
}

const detectedSystemDirs = ref<DetectedModelDirectory[]>([])
const loadDetectedDirs = async () => {
  try {
    const list = await invoke<any[]>('detect_model_directories')
    if (Array.isArray(list)) {
      detectedSystemDirs.value = list.map((d) => ({ ...d, path: contractUserPath(d.path) }))
    }
  } catch (e) {
    console.warn('Erro ao detectar diretórios do sistema:', e)
  }
}

const activeModelDirectories = computed(() => {
  const dirs = props.config.models_directories && Array.isArray(props.config.models_directories) && props.config.models_directories.length > 0
    ? props.config.models_directories
    : props.config.models_directory
      ? [props.config.models_directory]
      : ['~/.atena/models']
  return dirs.map(contractUserPath)
})

const isDefaultDirectory = (dir: string, idx: number): boolean => {
  if (props.config.models_directory) {
    return contractUserPath(props.config.models_directory) === contractUserPath(dir)
  }
  return idx === 0
}

const setDefaultDirectory = (dir: string) => {
  const contracted = contractUserPath(dir)
  props.config.models_directory = contracted
  if (props.config.models_directories && Array.isArray(props.config.models_directories)) {
    const idx = props.config.models_directories.findIndex((d) => contractUserPath(d) === contracted)
    if (idx >= 0) {
      props.config.models_directories.splice(idx, 1)
      props.config.models_directories.unshift(contracted)
    } else {
      props.config.models_directories.unshift(contracted)
    }
  }
  saveSettings()
}

const suggestedDirsToAdd = computed<DetectedModelDirectory[]>(() => {
  return detectedSystemDirs.value
    .map((d) => ({ ...d, path: contractUserPath(d.path) }))
    .filter(
      (d) => d.exists && d.source !== 'atena' && !activeModelDirectories.value.some((active) => contractUserPath(active) === d.path)
    )
})

const addModelDirectory = (path: string) => {
  const contracted = contractUserPath(path)
  if (!props.config.models_directories || !Array.isArray(props.config.models_directories)) {
    props.config.models_directories = props.config.models_directory ? [contractUserPath(props.config.models_directory)] : []
  }
  props.config.models_directories = props.config.models_directories.map(contractUserPath)
  if (!props.config.models_directories.includes(contracted)) {
    props.config.models_directories.push(contracted)
    if (!props.config.models_directory || props.config.models_directories.length === 1) {
      props.config.models_directory = contracted
    }
    saveSettings()
  }
}

const removeModelDirectory = (index: number) => {
  if (props.config.models_directories && props.config.models_directories.length > 1) {
    const [removed] = props.config.models_directories.splice(index, 1)
    if (contractUserPath(props.config.models_directory) === contractUserPath(removed)) {
      props.config.models_directory = contractUserPath(props.config.models_directories[0])
    }
    saveSettings()
  }
}

const chooseFolderForMultiPath = async () => {
  try {
    const selected = await invoke<string>('select_folder', {
      defaultPath: props.config.models_directory || null
    })
    if (selected) {
      addModelDirectory(contractUserPath(selected))
    }
  } catch (err) {
    console.error('Erro ao selecionar pasta:', err)
  }
}

// macOS Navigation
const currentSection = ref(props.initialSection || 'general')
const navigationHistory = ref([props.initialSection || 'general'])
const historyIndex = ref(0)
const searchQuery = ref('')
const isSaved = ref(false)

// HTTP request logs auto-poll and formatting helpers
let logPollTimer: ReturnType<typeof setInterval> | null = null

watch(
  currentSection,
  (sec) => {
    if (sec === 'servers') {
      emit('refreshLogs')
      if (!logPollTimer) {
        logPollTimer = setInterval(() => {
          emit('refreshLogs')
        }, 2500)
      }
    } else if (logPollTimer) {
      clearInterval(logPollTimer)
      logPollTimer = null
    }
  },
  { immediate: true }
)

onUnmounted(() => {
  if (logPollTimer) {
    clearInterval(logPollTimer)
    logPollTimer = null
  }
})

const formatLogTime = (iso: string) => {
  if (!iso) return ''
  try {
    const d = new Date(iso)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch (_) {
    return ''
  }
}

const getMethodClass = (method: string) => {
  switch ((method || '').toUpperCase()) {
    case 'GET':
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
    case 'POST':
      return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20'
    case 'DELETE':
      return 'bg-rose-500/10 text-rose-400 border-rose-500/20'
    case 'PUT':
    case 'PATCH':
      return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
    default:
      return 'bg-slate-500/10 text-slate-400 border-slate-500/20'
  }
}

const getStatusClass = (status: number) => {
  if (status >= 200 && status < 300) return 'text-emerald-400'
  if (status >= 300 && status < 400) return 'text-sky-400'
  if (status >= 400 && status < 500) return 'text-amber-400'
  return 'text-rose-400'
}

watch(
  () => props.initialSection,
  (sec) => {
    if (sec) {
      currentSection.value = sec
    }
  },
  { immediate: true }
)

// Helper para resolver ícone do plugin dinamicamente
const getPluginIcon = (iconName?: string | null) => {
  switch (iconName?.toLowerCase()) {
    case 'brain': return Brain
    case 'sparkles': return Sparkles
    case 'code':
    case 'code2': return Code2
    case 'terminal': return Terminal
    case 'server': return Server
    case 'database': return Database
    case 'wrench': return Wrench
    case 'sliders': return Sliders
    case 'clock': return Clock
    case 'globe': return Globe
    case 'cpu': return Cpu
    case 'blocks':
    default:
      return Blocks
  }
}

const isCheckingUpdates = ref(false)
const updateCheckFeedback = ref('')

const handleCheckUpdates = () => {
  if (isCheckingUpdates.value) return
  isCheckingUpdates.value = true
  updateCheckFeedback.value = ''
  setTimeout(() => {
    isCheckingUpdates.value = false
    updateCheckFeedback.value = 'Versão mais recente'
    setTimeout(() => {
      updateCheckFeedback.value = ''
    }, 4000)
  }, 1000)
}

// CLI terminal installation state & logic
const cliStatus = ref<{ installed: boolean; path: string | null }>({ installed: false, path: null })
const isInstallingCli = ref(false)

const fetchCliStatus = async () => {
  try {
    const res = await invoke<any>('check_cli_installed')
    if (res) {
      cliStatus.value = res
    }
  } catch {
    // Handled silently
  }
}

const handleInstallCli = async () => {
  if (isInstallingCli.value) return
  isInstallingCli.value = true
  try {
    const installedPath = await invoke<string>('install_cli_command')
    await fetchCliStatus()
    addNotification({
      type: 'success',
      title: t('settings.cli_title'),
      message: t('settings.cli_install_success')
    })
  } catch (err: any) {
    addNotification({
      type: 'error',
      title: t('settings.cli_title'),
      message: t('settings.cli_install_error', { error: err?.message || String(err) })
    })
  } finally {
    isInstallingCli.value = false
  }
}

const menuItems = computed(() => {
  const base = [
    { id: 'general', label: t('settings.general'), icon: Settings, iconBg: 'bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300' },
    { id: 'cloud_providers', label: t('settings.cloud_providers'), icon: Cloud, iconBg: 'bg-sky-100 text-sky-600 dark:bg-sky-600/30 dark:text-sky-300' },
    { id: 'gateways', label: t('settings.gateways.nav_title'), icon: Radio, iconBg: 'bg-indigo-100 text-indigo-600 dark:bg-indigo-600/30 dark:text-indigo-300' },
    { id: 'automation', label: t('memory.automation_nav'), icon: Clock, iconBg: 'bg-emerald-100 text-emerald-600 dark:bg-emerald-600/30 dark:text-emerald-300' },
    { id: 'plugins', label: t('settings.plugins'), icon: Blocks, iconBg: 'bg-indigo-100 text-indigo-600 dark:bg-indigo-600/30 dark:text-indigo-300' },
    { id: 'mcp', label: t('settings.mcp'), icon: Wrench, iconBg: 'bg-purple-100 text-purple-600 dark:bg-purple-600/30 dark:text-purple-300' },
    { id: 'servers', label: t('settings.servers'), icon: Zap, iconBg: 'bg-amber-100 text-amber-700 dark:bg-amber-600/30 dark:text-amber-300' },
    { id: 'inference', label: t('settings.inference'), icon: BrainCircuit, iconBg: 'bg-pink-100 text-pink-600 dark:bg-pink-600/30 dark:text-pink-300' },
    { id: 'hardware', label: supportsMlx.value ? t('settings.hardware_metal') : t('settings.hardware_accel'), icon: Cpu, iconBg: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-600/30 dark:text-emerald-300' }
  ]

  // Adiciona itens de menu dinâmicos para plugins ativos com configurações
  for (const plugin of activePluginSettingsViews.value) {
    base.push({
      id: `plugin:${plugin.id}`,
      label: plugin.name,
      icon: getPluginIcon(plugin.icon),
      iconBg: plugin.id === 'atena-plugin-memory' ? 'bg-purple-100 text-purple-600 dark:bg-purple-600/30 dark:text-purple-300' : 'bg-indigo-100 text-indigo-600 dark:bg-indigo-600/30 dark:text-indigo-300',
      isPlugin: true,
      pluginId: plugin.id
    } as any)
  }

  return base
})

// Gerenciamento de Estado do Formulário do Plugin Selecionado
const selectedPlugin = computed(() => {
  if (!currentSection.value.startsWith('plugin:')) return null
  const pluginId = currentSection.value.replace('plugin:', '')
  return plugins.value.find((p) => p.id === pluginId) || null
})

const pluginFormValues = ref<Record<string, any>>({})
const isPluginSaved = ref(false)

watch(
  selectedPlugin,
  (newPlugin) => {
    if (newPlugin) {
      const initial: Record<string, any> = {}
      if (newPlugin.settings_schema) {
        for (const field of newPlugin.settings_schema) {
          initial[field.id] = newPlugin.settings_values?.[field.id] ?? field.default ?? (field.type === 'boolean' ? false : '')
        }
      }
      if (newPlugin.id === 'atena-plugin-memory') {
        initial['enable_cognitive_memory'] = props.config.enable_cognitive_memory !== false
        initial['enable_facts_memory'] = props.config.enable_facts_memory !== false
        initial['enable_skills_memory'] = props.config.enable_skills_memory !== false
        initial['enable_episodic_memory'] = props.config.enable_episodic_memory !== false
      }
      pluginFormValues.value = initial
    }
  },
  { immediate: true }
)

// Se o plugin ativo selecionado for desativado, redireciona suavemente para 'general'
watch(
  activePluginSettingsViews,
  (activeList) => {
    if (currentSection.value.startsWith('plugin:')) {
      const pId = currentSection.value.replace('plugin:', '')
      if (!activeList.some((p) => p.id === pId)) {
        currentSection.value = 'general'
      }
    }
  }
)

const togglePluginBooleanField = (fieldId: string) => {
  pluginFormValues.value[fieldId] = !pluginFormValues.value[fieldId]
  onPluginFieldChange(fieldId)
}

const stepNumberField = (field: any, delta: number) => {
  const current = Number(pluginFormValues.value[field.id] ?? field.default ?? 0)
  let next = current + delta * (field.step || 1)
  if (field.min !== undefined && next < field.min) next = field.min
  if (field.max !== undefined && next > field.max) next = field.max
  pluginFormValues.value[field.id] = next
  onPluginFieldChange(field.id)
}

const onPluginFieldChange = async (fieldId: string) => {
  if (!selectedPlugin.value) return
  const pluginId = selectedPlugin.value.id
  const val = pluginFormValues.value[fieldId]

  if (pluginId === 'atena-plugin-memory') {
    if (fieldId === 'enable_cognitive_memory') {
      props.config.enable_cognitive_memory = val
      saveSettings()
    } else if (fieldId === 'enable_facts_memory') {
      props.config.enable_facts_memory = val
      saveSettings()
    } else if (fieldId === 'enable_skills_memory') {
      props.config.enable_skills_memory = val
      saveSettings()
    } else if (fieldId === 'enable_episodic_memory') {
      props.config.enable_episodic_memory = val
      saveSettings()
    }
  }

  await updatePluginSetting(pluginId, fieldId, val)
}

const saveActivePluginSettings = async () => {
  if (!selectedPlugin.value) return
  const pluginId = selectedPlugin.value.id
  await savePluginSettings(pluginId, pluginFormValues.value)

  if (pluginId === 'atena-plugin-memory') {
    props.config.enable_cognitive_memory = pluginFormValues.value['enable_cognitive_memory']
    props.config.enable_facts_memory = pluginFormValues.value['enable_facts_memory']
    props.config.enable_skills_memory = pluginFormValues.value['enable_skills_memory']
    props.config.enable_episodic_memory = pluginFormValues.value['enable_episodic_memory']
    saveSettings()
  }

  isPluginSaved.value = true
  setTimeout(() => {
    isPluginSaved.value = false
  }, 2000)
}

const activeMenuItem = computed(() => {
  return (menuItems.value.find((m) => m.id === currentSection.value) || menuItems.value[0])!
})

const totalRamGb = computed(() => {
  return props.hardware?.total_ram_gb || 16
})

const getGuardrailCeilingGb = (mode: string, customLimit?: number | null) => {
  const ram = totalRamGb.value
  switch (mode) {
    case 'off':
      return ram
    case 'relaxed':
      return Math.round(ram * 0.88 * 10) / 10
    case 'balanced':
      return Math.round(ram * 0.75 * 10) / 10
    case 'strict':
      return Math.round(ram * 0.60 * 10) / 10
    case 'custom':
      return Number(customLimit) || 4.0
    default:
      return Math.round(ram * 0.75 * 10) / 10
  }
}

const currentGuardrailLimit = computed(() => {
  return getGuardrailCeilingGb(props.config.guardrail_mode || 'relaxed', props.config.guardrail_custom_limit_gb)
})

const filteredMenuItems = computed(() => {
  if (!searchQuery.value.trim()) return menuItems.value
  const q = searchQuery.value.toLowerCase()
  return menuItems.value.filter((m) => {
    if (m.label.toLowerCase().includes(q)) return true
    if (m.id === 'inference') {
      const keywords = ['guardrail', 'guardrails', 'proteção', 'proteções', 'memória', 'limite', 'travar', 'congelar', 'contexto', 'ram']
      return keywords.some(k => k.includes(q) || q.includes(k))
    }
    return false
  })
})

const navigateBack = () => {
  if (historyIndex.value > 0) {
    historyIndex.value--
    currentSection.value = navigationHistory.value[historyIndex.value] || 'general'
  }
}

const navigateForward = () => {
  if (historyIndex.value < navigationHistory.value.length - 1) {
    historyIndex.value++
    currentSection.value = navigationHistory.value[historyIndex.value] || 'general'
  }
}

const saveSettings = () => {
  emit('saveConfig', props.config)
  isSaved.value = true
  setTimeout(() => {
    isSaved.value = false
  }, 2000)
}

const chooseFolder = async () => {
  try {
    const selected = await invoke<string>('select_folder', {
      defaultPath: props.config.models_directory
    })
    if (selected) {
      props.config.models_directory = contractUserPath(selected)
      saveSettings()
    }
  } catch (err) {
    console.error('Erro ao selecionar pasta:', err)
  }
}


// Zero-Config Runtime State
const runtimeStatus = ref<any>(null)
const showRuntimeModal = ref(false)

const fetchRuntimeStatus = async () => {
  try {
    runtimeStatus.value = await invoke('get_runtime_status')
  } catch (err) {
    console.error('Falha ao obter status do runtime:', err)
  }
}


// MCP State & Methods
const mcpServers = ref<McpServerConfig[]>([])
const mcpServerTools = ref<Record<string, McpToolDefinition[]>>({})
const mcpServerErrors = ref<Record<string, string>>({})
const isRefreshingMcp = ref(false)
const inspectingMcpId = ref<string | null>(null)

const isMcpModalOpen = ref(false)
const isEditingMcp = ref(false)
const isTestingMcp = ref(false)
const mcpTestResult = ref<any>(null)

const mcpModalForm = ref<{
  id: string
  name: string
  transport: string
  command: string
  argsList: string[]
  headersList: { key: string; value: string }[]
  envList: { key: string; value: string }[]
  url: string
  description: string
  permission_mode: string
  enabled: boolean
}>({
  id: '',
  name: '',
  transport: 'stdio',
  command: '',
  argsList: [],
  headersList: [],
  envList: [],
  url: '',
  description: '',
  permission_mode: 'ask',
  enabled: true
})

const testToolModal = ref<{
  isOpen: boolean
  server: any
  tool: any
  argsJson: string
  isRunning: boolean
  result: string | null
}>({
  isOpen: false,
  server: null,
  tool: null,
  argsJson: '{}',
  isRunning: false,
  result: null
})

const quickMcpTemplates = computed(() => [
  {
    id: 'filesystem',
    name: t('settings.mcp_template_fs_name'),
    transport: 'stdio',
    icon: FolderTree,
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-filesystem', '.'],
    description: t('settings.mcp_template_fs_desc')
  },
  {
    id: 'fetch',
    name: t('settings.mcp_template_fetch_name'),
    transport: 'stdio',
    icon: Globe,
    command: 'uvx',
    args: ['duckduckgo-mcp-server'],
    description: t('settings.mcp_template_fetch_desc')
  },
  {
    id: 'sqlite',
    name: t('settings.mcp_template_sqlite_name'),
    transport: 'stdio',
    icon: Database,
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-sqlite', '--db-path', './data.db'],
    description: t('settings.mcp_template_sqlite_desc')
  }
])

const loadMcpServers = async () => {
  try {
    const list = await invoke<McpServerConfig[]>('get_mcp_servers')
    if (list) {
      mcpServers.value = list
      await refreshMcpTools()
    }
  } catch (err) {
    console.error('Failed to load MCP servers:', err)
  }
}

const refreshMcpTools = async () => {
  if (isRefreshingMcp.value) return
  isRefreshingMcp.value = true
  try {
    for (const server of mcpServers.value) {
      if (server.enabled) {
        inspectingMcpId.value = server.id
        try {
          const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server })
          mcpServerTools.value[server.id] = tools || []
          delete mcpServerErrors.value[server.id]
        } catch (e) {
          console.warn(`Failed to inspect ${server.name}:`, e)
          mcpServerErrors.value[server.id] = String(e)
          mcpServerTools.value[server.id] = []
        }
      } else {
        mcpServerTools.value[server.id] = []
        delete mcpServerErrors.value[server.id]
      }
    }
    emit('refreshMcp')
  } finally {
    isRefreshingMcp.value = false
    inspectingMcpId.value = null
  }
}

const testExistingMcpServer = async (server: any) => {
  inspectingMcpId.value = server.id
  try {
    const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server })
    mcpServerTools.value[server.id] = tools || []
    delete mcpServerErrors.value[server.id]
    emit('refreshMcp')
  } catch (err) {
    mcpServerErrors.value[server.id] = String(err)
    mcpServerTools.value[server.id] = []
  } finally {
    inspectingMcpId.value = null
  }
}

const toggleMcpServer = async (server: any) => {
  server.enabled = !server.enabled
  await saveMcpServersList()
  if (server.enabled) {
    try {
      const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server })
      mcpServerTools.value[server.id] = tools || []
      delete mcpServerErrors.value[server.id]
    } catch (e) {
      mcpServerErrors.value[server.id] = String(e)
      mcpServerTools.value[server.id] = []
    }
  } else {
    mcpServerTools.value[server.id] = []
    delete mcpServerErrors.value[server.id]
  }
  emit('refreshMcp')
}

const toggleMcpPermission = async (server: any) => {
  server.permission_mode = server.permission_mode === 'auto' ? 'ask' : 'auto'
  await saveMcpServersList()
}

const getToolPermission = (server: any, toolName: string): string => {
  if (server.tool_permissions && server.tool_permissions[toolName]) {
    return server.tool_permissions[toolName]
  }
  return server.permission_mode || 'ask'
}

const toggleToolPermission = async (server: any, toolName: string) => {
  if (!server.tool_permissions) {
    server.tool_permissions = {}
  }
  const current = getToolPermission(server, toolName)
  server.tool_permissions[toolName] = current === 'auto' ? 'ask' : 'auto'
  await saveMcpServersList()
}

const saveMcpServersList = async () => {
  try {
    await invoke('save_mcp_servers', { servers: mcpServers.value })
    emit('refreshMcp')
  } catch (err) {
    console.error('Failed to save servers:', err)
  }
}

const mcpFileInput = ref<HTMLInputElement | null>(null)
const isTranslatingTools = ref<Record<string, boolean>>({})
const isAnyTranslating = computed(() => Object.values(isTranslatingTools.value).some(Boolean))
const translatingProgress = ref<Record<string, { current: number; total: number }>>({})
const editingTool = ref<any>(null)

const isEditToolModalOpen = ref(false)
const selectedToolToEdit = ref<any>(null)
const selectedServerToEdit = ref<any>(null)

const openEditToolModal = (server: any, tool: any) => {
  selectedServerToEdit.value = server
  selectedToolToEdit.value = tool
  isEditToolModalOpen.value = true
}

const handleToolDetailsSaved = async ({ serverId, toolName, label, fieldLabels, enabled }: any) => {
  const server = mcpServers.value.find((s) => s.id === serverId)
  if (server) {
    if (!server.tool_labels) server.tool_labels = {}
    if (!server.tool_field_labels) server.tool_field_labels = {}
    if (!server.disabled_tools) server.disabled_tools = []
    if (label) {
      server.tool_labels[toolName] = label
    } else {
      delete server.tool_labels[toolName]
    }
    if (fieldLabels && Object.keys(fieldLabels).length > 0) {
      server.tool_field_labels[toolName] = fieldLabels
    } else {
      delete server.tool_field_labels[toolName]
    }
    if (typeof enabled === 'boolean') {
      if (enabled) {
        server.disabled_tools = server.disabled_tools.filter((x: string) => x !== toolName)
      } else if (!server.disabled_tools.includes(toolName)) {
        server.disabled_tools.push(toolName)
      }
    }
    await saveMcpServersList()
    const tools = mcpServerTools.value[serverId] || []
    for (const t of tools) {
      if (t.name === toolName) {
        t.label = label || null
        t.field_labels = fieldLabels || {}
      }
    }
  }
  await refreshMcpTools()
}

const isMcpToolEnabled = (server: any, toolName: string): boolean => {
  if (!server) return true
  if (!Array.isArray(server.disabled_tools)) return true
  return !server.disabled_tools.includes(toolName)
}

const toggleMcpTool = async (server: any, toolName: string) => {
  if (!server) return
  if (!server.disabled_tools) {
    server.disabled_tools = []
  }
  if (server.disabled_tools.includes(toolName)) {
    server.disabled_tools = server.disabled_tools.filter((x: string) => x !== toolName)
  } else {
    server.disabled_tools.push(toolName)
  }
  await saveMcpServersList()
}

const getActiveMcpServerToolsCount = (server: any): number => {
  const tools = mcpServerTools.value[server.id] || []
  if (!server.enabled || tools.length === 0) return 0
  const disabled = new Set(server.disabled_tools || [])
  return tools.filter((t) => !disabled.has(t.name)).length
}

const startEditLabel = (serverId: string, toolName: string, currentLabel: string) => {
  const server = mcpServers.value.find((s) => s.id === serverId)
  const tools = mcpServerTools.value[serverId] || []
  const tool = tools.find((t) => t.name === toolName) || { name: toolName, label: currentLabel }
  if (server) {
    openEditToolModal(server, tool)
  }
}

const saveToolLabelEdit = async (server: any) => {
  if (!editingTool.value) return
  if (!server.tool_labels) server.tool_labels = {}
  const cleanLabel = editingTool.value.label.trim()
  if (cleanLabel) {
    server.tool_labels[editingTool.value.toolName] = cleanLabel
  } else {
    delete server.tool_labels[editingTool.value.toolName]
  }
  const toolName = editingTool.value.toolName
  editingTool.value = null
  await saveMcpServersList()
  const tools = mcpServerTools.value[server.id] || []
  for (const t of tools) {
    if (t.name === toolName) {
      t.label = cleanLabel || null
    }
  }
}

const exportMcpServers = async () => {
  const content = JSON.stringify(mcpServers.value, null, 2)
  await saveTextFile({
    content,
    filename: `atena_mcp_tools_${new Date().toISOString().slice(0, 10)}.json`,
    title: t('settings.mcp_export_dialog_title'),
    filters: [
      { name: 'JSON (*.json)', extensions: ['json'] },
      { name: t('settings.mcp_all_files'), extensions: ['*'] }
    ]
  })
}

const triggerImportMcp = () => {
  if (mcpFileInput.value) mcpFileInput.value.click()
}

const handleImportMcpFile = async (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  try {
    const text = await file.text()
    const imported = JSON.parse(text)
    if (Array.isArray(imported)) {
      for (const s of imported) {
        if (!s.id || !s.name) continue
        const idx = mcpServers.value.findIndex((x) => x.id === s.id)
        if (idx !== -1) {
          mcpServers.value[idx] = { ...mcpServers.value[idx], ...s }
        } else {
          mcpServers.value.push(s)
        }
      }
      await saveMcpServersList()
      await refreshMcpTools()
    }
  } catch (err) {
    alert(t('settings.mcp_import_json_error'))
  } finally {
    target.value = ''
  }
}

const translateServerToolLabels = async (server: any) => {
  if (isTranslatingTools.value[server.id] || isAnyTranslating.value) return
  const tools = mcpServerTools.value[server.id] || []
  if (tools.length === 0) return

  isTranslatingTools.value[server.id] = true
  translatingProgress.value[server.id] = { current: 1, total: tools.length }

  try {
    if (!server.tool_labels) server.tool_labels = {}
    if (!server.tool_field_labels) server.tool_field_labels = {}

    for (let i = 0; i < tools.length; i++) {
      const tool = tools[i]
      if (!tool) continue
      translatingProgress.value[server.id] = { current: i + 1, total: tools.length }

      try {
        const result = await invoke<any>('translate_mcp_tool_labels', {
          tools: [tool],
          mlxHost: props.config?.mlx_host || '127.0.0.1',
          mlxPort: props.config?.mlx_port || 8080,
          ollamaHost: props.config?.ollama_host || '127.0.0.1',
          ollamaPort: props.config?.ollama_port || 11434,
          targetLocale: currentLocale.value
        })

        if (result && typeof result === 'object') {
          if (result.tool_labels && result.tool_labels[tool.name]) {
            server.tool_labels[tool.name] = result.tool_labels[tool.name]
            tool.label = result.tool_labels[tool.name]
          }
          if (result.tool_field_labels && result.tool_field_labels[tool.name]) {
            server.tool_field_labels[tool.name] = result.tool_field_labels[tool.name]
            tool.field_labels = result.tool_field_labels[tool.name]
          }
        }
      } catch (toolErr) {
        console.warn(`Failed to translate tool ${tool.name}:`, toolErr)
      }

      await saveMcpServersList()
      await new Promise((r) => setTimeout(r, 60))
    }
  } catch (err) {
    console.error('Failed to translate labels:', err)
  } finally {
    isTranslatingTools.value[server.id] = false
    delete translatingProgress.value[server.id]
  }
}

const applyMcpTemplate = async (tmpl: any) => {
  const existing = mcpServers.value.find((s) => s.id === tmpl.id)
  let targetServer = existing
  if (existing) {
    existing.enabled = true
  } else {
    targetServer = {
      id: tmpl.id,
      name: tmpl.name,
      transport: tmpl.transport,
      command: tmpl.command,
      args: tmpl.args,
      headers: null,
      env: null,
      enabled: true,
      description: tmpl.description,
      permission_mode: 'ask',
      tool_permissions: {},
      tool_labels: {},
      tool_field_labels: {}
    }
    mcpServers.value.push(targetServer)
  }
  await saveMcpServersList()
  await refreshMcpTools()
}

const openAddMcpModal = () => {
  isEditingMcp.value = false
  isTestingMcp.value = false
  mcpTestResult.value = null
  mcpModalForm.value = {
    id: `mcp-${Date.now()}`,
    name: '',
    transport: 'stdio',
    command: 'npx',
    argsList: [],
    headersList: [],
    envList: [],
    url: '',
    description: '',
    permission_mode: 'ask',
    enabled: true
  }
  isMcpModalOpen.value = true
}

const editMcpServer = (server: any) => {
  isEditingMcp.value = true
  isTestingMcp.value = false
  mcpTestResult.value = null

  // Convert headers map to array
  const headersList: { key: string; value: string }[] = []
  if (server.headers) {
    for (const [key, value] of Object.entries(server.headers)) {
      headersList.push({ key, value: String(value) })
    }
  }

  // Convert env map to array
  const envList: { key: string; value: string }[] = []
  if (server.env) {
    for (const [key, value] of Object.entries(server.env)) {
      envList.push({ key, value: String(value) })
    }
  }

  mcpModalForm.value = {
    id: server.id,
    name: server.name,
    transport: server.transport || 'stdio',
    command: server.command || '',
    argsList: server.args ? [...server.args] : [],
    headersList,
    envList,
    url: server.url || '',
    description: server.description || '',
    permission_mode: server.permission_mode || 'ask',
    enabled: server.enabled
  }
  isMcpModalOpen.value = true
}

const addMcpArg = () => {
  mcpModalForm.value.argsList.push('')
}

const removeMcpArg = (idx: number) => {
  mcpModalForm.value.argsList.splice(idx, 1)
}

const addMcpHeader = () => {
  mcpModalForm.value.headersList.push({ key: '', value: '' })
}

const removeMcpHeader = (idx: number) => {
  mcpModalForm.value.headersList.splice(idx, 1)
}

const addMcpEnv = () => {
  mcpModalForm.value.envList.push({ key: '', value: '' })
}

const removeMcpEnv = (idx: number) => {
  mcpModalForm.value.envList.splice(idx, 1)
}

const testMcpModalServer = async () => {
  if (isTestingMcp.value) return
  isTestingMcp.value = true
  mcpTestResult.value = null

  // Format headers map
  let headersMap: Record<string, string> | null = null
  if (mcpModalForm.value.headersList.length > 0) {
    const valid = mcpModalForm.value.headersList.filter((h) => h.key.trim() && h.value.trim())
    if (valid.length > 0) {
      headersMap = {}
      for (const h of valid) {
        headersMap[h.key.trim()] = h.value.trim()
      }
    }
  }

  // Format env map
  let envMap: Record<string, string> | null = null
  if (mcpModalForm.value.envList.length > 0) {
    const valid = mcpModalForm.value.envList.filter((e) => e.key.trim() && e.value.trim())
    if (valid.length > 0) {
      envMap = {}
      for (const e of valid) {
        envMap[e.key.trim()] = e.value.trim()
      }
    }
  }

  const cleanArgs = mcpModalForm.value.argsList.map((a) => a.trim()).filter(Boolean)

  const tempServer = {
    id: mcpModalForm.value.id || `mcp-${Date.now()}`,
    name: mcpModalForm.value.name.trim() || t('settings.mcp_default_server_name'),
    transport: mcpModalForm.value.transport,
    command: mcpModalForm.value.command ? mcpModalForm.value.command.trim() : null,
    args: cleanArgs.length ? cleanArgs : null,
    headers: headersMap,
    env: envMap,
    url: mcpModalForm.value.url ? mcpModalForm.value.url.trim() : null,
    description: mcpModalForm.value.description || null,
    permission_mode: mcpModalForm.value.permission_mode || 'ask',
    enabled: true,
    tool_permissions: {}
  }

  try {
    const tools = await invoke<McpToolDefinition[]>('inspect_server_tools', { server: tempServer })
    mcpTestResult.value = {
      success: true,
      count: tools ? tools.length : 0,
      tools: tools || []
    }
  } catch (err) {
    mcpTestResult.value = {
      success: false,
      error: String(err)
    }
  } finally {
    isTestingMcp.value = false
  }
}

const saveMcpModal = async () => {
  if (!mcpModalForm.value.name.trim()) return

  // Format headers map
  let headersMap: Record<string, string> | null = null
  if (mcpModalForm.value.headersList.length > 0) {
    const valid = mcpModalForm.value.headersList.filter((h) => h.key.trim() && h.value.trim())
    if (valid.length > 0) {
      headersMap = {}
      for (const h of valid) {
        headersMap[h.key.trim()] = h.value.trim()
      }
    }
  }

  // Format env map
  let envMap: Record<string, string> | null = null
  if (mcpModalForm.value.envList.length > 0) {
    const valid = mcpModalForm.value.envList.filter((e) => e.key.trim() && e.value.trim())
    if (valid.length > 0) {
      envMap = {}
      for (const e of valid) {
        envMap[e.key.trim()] = e.value.trim()
      }
    }
  }

  const cleanArgs = mcpModalForm.value.argsList.map((a) => a.trim()).filter(Boolean)

  const existingServer = isEditingMcp.value ? mcpServers.value.find((s) => s.id === mcpModalForm.value.id) : null
  const newServer: any = {
    id: mcpModalForm.value.id,
    name: mcpModalForm.value.name.trim(),
    transport: mcpModalForm.value.transport,
    command: mcpModalForm.value.command ? mcpModalForm.value.command.trim() : null,
    args: cleanArgs.length ? cleanArgs : null,
    headers: headersMap,
    env: envMap,
    url: mcpModalForm.value.url ? mcpModalForm.value.url.trim() : null,
    description: mcpModalForm.value.description || null,
    permission_mode: mcpModalForm.value.permission_mode || 'ask',
    enabled: mcpModalForm.value.enabled,
    tool_permissions: existingServer?.tool_permissions || {},
    tool_labels: existingServer?.tool_labels || {},
    tool_field_labels: existingServer?.tool_field_labels || {}
  }

  if (isEditingMcp.value) {
    const idx = mcpServers.value.findIndex((s) => s.id === newServer.id)
    if (idx !== -1) mcpServers.value[idx] = newServer
  } else {
    mcpServers.value.push(newServer)
  }

  isMcpModalOpen.value = false
  await saveMcpServersList()
  await refreshMcpTools()
}

const deleteMcpServer = async (id: string) => {
  const target = mcpServers.value.find((s) => s.id === id)
  if (target?.transport === 'builtin' || id === 'atena_native') {
    return
  }
  mcpServers.value = mcpServers.value.filter((s) => s.id !== id)
  delete mcpServerTools.value[id]
  delete mcpServerErrors.value[id]
  await saveMcpServersList()
}

const openTestToolModal = (server: any, tool: any) => {
  testToolModal.value = {
    isOpen: true,
    server,
    tool,
    argsJson: '{}',
    isRunning: false,
    result: null
  }
}

const runMcpTestTool = async () => {
  testToolModal.value.isRunning = true
  testToolModal.value.result = null
  try {
    const args = JSON.parse(testToolModal.value.argsJson)
    const res = await invoke('call_mcp_tool', {
      serverId: testToolModal.value.server.id,
      toolName: testToolModal.value.tool.name,
      arguments: args
    })
    testToolModal.value.result = JSON.stringify(res, null, 2)
  } catch (err) {
    testToolModal.value.result = `${t('common.error')}: ${err}`
  } finally {
    testToolModal.value.isRunning = false
  }
}

// Timezone and Live Date/Time
const nowTick = ref(new Date())
let dateTickTimer: any = null
const customTzInput = ref('')

const knownTimezones = [
  'America/Sao_Paulo',
  'America/Manaus',
  'America/Belem',
  'America/Fortaleza',
  'America/Recife',
  'America/Cuiaba',
  'America/Porto_Velho',
  'America/Rio_Branco',
  'America/Noronha',
  'America/New_York',
  'America/Chicago',
  'America/Denver',
  'America/Los_Angeles',
  'America/Argentina/Buenos_Aires',
  'America/Santiago',
  'America/Bogota',
  'America/Mexico_City',
  'Europe/Lisbon',
  'Europe/London',
  'Europe/Madrid',
  'Europe/Paris',
  'Europe/Berlin',
  'Europe/Rome',
  'UTC',
  'Asia/Tokyo',
  'Asia/Shanghai',
  'Asia/Singapore',
  'Asia/Dubai',
  'Australia/Sydney'
]

const isCustomTimezone = computed(() => {
  const tz = props.config.timezone
  return tz && !knownTimezones.includes(tz)
})

const liveDateTimePreview = computed(() => {
  const _ = nowTick.value
  return formatLiveDateTime(new Date(), props.config.timezone || 'America/Sao_Paulo')
})

const timezoneOffsetStr = computed(() => {
  const _ = nowTick.value
  try {
    const tz = props.config.timezone || 'America/Sao_Paulo'
    const formatter = new Intl.DateTimeFormat('pt-BR', {
      timeZone: tz,
      timeZoneName: 'shortOffset'
    })
    const parts = formatter.formatToParts(new Date())
    const tzPart = parts.find((p) => p.type === 'timeZoneName')
    return tzPart ? tzPart.value : ''
  } catch (_) {
    return ''
  }
})

const samplePromptPreview = computed(() => {
  const _ = nowTick.value
  return getTemporalContextPrompt(props.config.timezone || 'America/Sao_Paulo').trim()
})

const detectSystemTimezone = () => {
  props.config.timezone = getDefaultTimezone()
}

const applyCustomTz = () => {
  const trimmed = customTzInput.value.trim()
  if (trimmed) {
    try {
      Intl.DateTimeFormat(undefined, { timeZone: trimmed })
      props.config.timezone = trimmed
      customTzInput.value = ''
    } catch (e) {
      alert(t('settings.invalid_tz_alert', { tz: trimmed }))
    }
  }
}

onMounted(() => {
  loadMcpServers()
  fetchRuntimeStatus()
  checkAgySession()
  loadDetectedDirs()
  fetchCliStatus()
  if (props.config.models_directories && Array.isArray(props.config.models_directories)) {
    props.config.models_directories = props.config.models_directories.map(contractUserPath)
  }
  if (props.config.models_directory) {
    props.config.models_directory = contractUserPath(props.config.models_directory)
  }
  if (props.config.models_directories && Array.isArray(props.config.models_directories) && props.config.models_directories.length > 0) {
    if (!props.config.models_directory || !props.config.models_directories.includes(props.config.models_directory)) {
      props.config.models_directory = props.config.models_directories[0]
      saveSettings()
    }
  }
  if (!props.config.timezone) {
    props.config.timezone = getDefaultTimezone()
  }
  if (!props.config.whisper_model) {
    props.config.whisper_model = 'mlx-community/whisper-small-mlx'
  }
  if (!props.config.whisper_language) {
    props.config.whisper_language = 'pt'
  }
  dateTickTimer = setInterval(() => {
    nowTick.value = new Date()
  }, 1000)
})

watch(
  () => props.config,
  (cfg) => {
    if (cfg) {
      if (!cfg.whisper_model) {
        cfg.whisper_model = 'mlx-community/whisper-small-mlx'
      }
      if (!cfg.whisper_language) {
        cfg.whisper_language = 'pt'
      }
    }
  },
  { immediate: true, deep: true }
)

onUnmounted(() => {
  if (dateTickTimer) clearInterval(dateTickTimer)
})
</script>
