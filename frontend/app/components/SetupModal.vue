<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-6 bg-black/80 backdrop-blur-xl animate-fade-in select-none">
    <div
      class="w-full max-w-2xl bg-[#0d101c] border border-[#232b45] rounded-3xl shadow-2xl shadow-black/90 overflow-hidden flex flex-col max-h-[92vh]"
    >
      <!-- Top Wizard Stepper Header -->
      <div class="p-6 border-b border-[#1c2338] bg-gradient-to-r from-[#121627] via-[#101424] to-[#121627]">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2.5">
            <div class="w-8 h-8 rounded-xl bg-gradient-to-tr from-indigo-600 to-purple-500 flex items-center justify-center shadow-lg shadow-indigo-600/30">
              <Sparkles class="w-4 h-4 text-white" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-sm font-bold text-slate-100 tracking-tight">{{ $t('setup.title') }}</h2>
                <span class="text-[9px] font-bold text-indigo-400 bg-indigo-500/15 border border-indigo-500/30 px-1.5 py-0.5 rounded-full uppercase tracking-wider leading-none">{{ $t('setup.beta_badge') }}</span>
              </div>
              <p class="text-[11px] text-slate-400">{{ $t('setup.subtitle') }}</p>
            </div>
          </div>

          <span class="text-[10px] font-mono text-indigo-400 bg-indigo-500/10 px-2.5 py-1 rounded-full border border-indigo-500/25">
            {{ $t('setup.step_counter', { current: currentStep, total: totalSteps }) }}
          </span>
        </div>

        <!-- Progress Steps Bar -->
        <div class="grid grid-cols-5 gap-2">
          <div
            v-for="s in totalSteps"
            :key="s"
            class="h-1.5 rounded-full transition-all duration-300"
            :class="[
              s <= currentStep
                ? 'bg-gradient-to-r from-indigo-500 to-purple-500'
                : 'bg-[#1b2238]'
            ]"
          />
        </div>
      </div>

      <!-- Step Content Area -->
      <div class="p-6 overflow-y-auto space-y-5 text-xs text-slate-200 flex-1">
        <!-- STEP 1: Persona & Profile Choice -->
        <div v-if="currentStep === 1" class="space-y-4 animate-fade-in">
          <div>
            <h3 class="text-sm font-bold text-slate-100">{{ $t('setup.step1_title') }}</h3>
            <p class="text-[11.5px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('setup.step1_desc') }}
            </p>
          </div>

          <!-- Personas Selection Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
            <div
              v-for="p in selectablePersonas"
              :key="p.id"
              @click="selectedPersonaId = p.id"
              :class="[
                'p-3.5 rounded-2xl border transition-all cursor-pointer flex flex-col justify-between group relative overflow-hidden',
                selectedPersonaId === p.id
                  ? 'bg-[#161c30] border-indigo-500/60 shadow-md shadow-indigo-500/10 ring-1 ring-indigo-500/30'
                  : 'bg-[#111422] border-[#1d243a] hover:border-slate-600 hover:bg-[#131828]'
              ]"
            >
              <div class="flex items-start justify-between gap-2 mb-2">
                <div class="flex items-center gap-2">
                  <div
                    class="w-7 h-7 rounded-xl flex items-center justify-center text-xs"
                    :class="[
                      selectedPersonaId === p.id
                        ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40'
                        : 'bg-[#181f33] text-slate-400 border border-[#232b45]'
                    ]"
                  >
                    <Sparkles v-if="p.iconName === 'Sparkles'" class="w-3.5 h-3.5" />
                    <CheckCircle2 v-else-if="p.iconName === 'CheckCircle2'" class="w-3.5 h-3.5" />
                    <PenTool v-else-if="p.iconName === 'PenTool'" class="w-3.5 h-3.5" />
                    <GraduationCap v-else-if="p.iconName === 'GraduationCap'" class="w-3.5 h-3.5" />
                    <Code2 v-else class="w-3.5 h-3.5" />
                  </div>
                  <div>
                    <h4 class="font-bold text-xs text-slate-100">{{ p.name }}</h4>
                    <span class="text-[10px] text-slate-400">{{ p.tag }}</span>
                  </div>
                </div>

                <div
                  v-if="selectedPersonaId === p.id"
                  class="w-4 h-4 rounded-full bg-indigo-500 text-white flex items-center justify-center"
                >
                  <Check class="w-2.5 h-2.5" />
                </div>
              </div>

              <p class="text-[11px] text-slate-400 leading-snug line-clamp-2">
                {{ p.description }}
              </p>
            </div>
          </div>

          <!-- Edit Persona Toggle / Customization Drawer -->
          <div class="pt-2">
            <button
              type="button"
              @click="isEditingPersona = !isEditingPersona"
              class="flex items-center gap-2 text-indigo-400 hover:text-indigo-300 font-medium text-xs cursor-pointer transition-colors"
            >
              <Edit3 class="w-3.5 h-3.5" />
              <span>{{ isEditingPersona ? $t('setup.edit_persona_close') : $t('setup.edit_persona_open') }}</span>
            </button>

            <div
              v-if="isEditingPersona"
              class="mt-3 p-4 rounded-2xl bg-[#101424] border border-[#20273f] space-y-3 animate-fade-in"
            >
              <div>
                <label class="block text-[10.5px] font-semibold uppercase tracking-wider text-slate-400 mb-1">
                  {{ $t('setup.custom_name_label') }}
                </label>
                <input
                  v-model="customPersonaName"
                  type="text"
                  :placeholder="$t('setup.custom_name_placeholder')"
                  class="w-full px-3 py-2 rounded-xl bg-[#141829] border border-[#232b45] text-xs text-slate-100 outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-[10.5px] font-semibold uppercase tracking-wider text-slate-400 mb-1">
                  {{ $t('setup.custom_prompt_label') }}
                </label>
                <textarea
                  v-model="customPersonaPrompt"
                  rows="3"
                  :placeholder="$t('setup.custom_prompt_placeholder')"
                  class="w-full px-3 py-2 rounded-xl bg-[#141829] border border-[#232b45] text-xs text-slate-100 outline-none focus:border-indigo-500 leading-relaxed resize-none"
                ></textarea>
              </div>
            </div>
          </div>
        </div>

        <!-- STEP 2: Local AI Engines & Hardware Acceleration -->
        <div v-if="currentStep === 2" class="space-y-4 animate-fade-in">
          <div>
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Cpu class="w-4 h-4 text-indigo-400" />
                <span>{{ $t('setup.step2_title') }}</span>
              </h3>
              <span v-if="isAllEnginesReady" class="text-[10.5px] font-bold text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 px-2.5 py-0.5 rounded-full flex items-center gap-1">
                <Check class="w-3 h-3" />
                <span>{{ $t('setup.engines_ready') }}</span>
              </span>
            </div>
            <p class="text-[11.5px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('setup.step2_desc') }}
            </p>
          </div>

          <!-- Engines Status Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <!-- GGUF Engine Card -->
            <div
              class="p-3.5 rounded-2xl border transition-all flex flex-col justify-between"
              :class="[
                isLlamaReady
                  ? 'bg-[#101926] border-emerald-500/40 shadow-sm shadow-emerald-500/5'
                  : isBootstrapping
                    ? 'bg-[#13172c] border-indigo-500/40'
                    : 'bg-[#111422] border-[#222b44]'
              ]"
            >
              <div>
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <div
                      class="w-7 h-7 rounded-xl flex items-center justify-center text-xs"
                      :class="isLlamaReady ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30'"
                    >
                      <Zap class="w-3.5 h-3.5" />
                    </div>
                    <div>
                      <h4 class="font-bold text-xs text-slate-100">Motor GGUF (llama.cpp)</h4>
                      <span class="text-[10px] text-slate-400">Modelos quantizados .gguf</span>
                    </div>
                  </div>

                  <div
                    v-if="isLlamaReady"
                    class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 text-[10px] font-bold flex items-center gap-1 border border-emerald-500/30"
                  >
                    <Check class="w-2.5 h-2.5" />
                    <span>Pronto</span>
                  </div>
                  <div
                    v-else-if="isBootstrapping"
                    class="px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 text-[10px] font-medium flex items-center gap-1 border border-indigo-500/30"
                  >
                    <Loader2 class="w-2.5 h-2.5 animate-spin" />
                    <span>Instalando...</span>
                  </div>
                  <div
                    v-else
                    class="px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-300 text-[10px] font-medium flex items-center gap-1 border border-amber-500/25"
                  >
                    <AlertCircle class="w-2.5 h-2.5" />
                    <span>Pendente</span>
                  </div>
                </div>

                <p class="text-[11px] text-slate-400 leading-snug">
                  Inferência ultra-rápida com offload na GPU Metal, quantização de KV cache e flash attention.
                </p>
              </div>

              <div class="mt-3 pt-2.5 border-t border-[#1c2338] flex items-center justify-between text-[10px] text-slate-400 font-mono">
                <span>Versão GGUF:</span>
                <span class="text-indigo-300 font-medium">{{ runtimeStatus?.llama_version ? runtimeStatus.llama_version.split('(')[0].replace('version:', '').trim() : 'Apple Metal GPU' }}</span>
              </div>
            </div>

            <!-- MLX Engine Card -->
            <div
              v-if="supportsMlx"
              class="p-3.5 rounded-2xl border transition-all flex flex-col justify-between"
              :class="[
                isMlxReady
                  ? 'bg-[#101926] border-emerald-500/40 shadow-sm shadow-emerald-500/5'
                  : isBootstrapping
                    ? 'bg-[#13172c] border-indigo-500/40'
                    : 'bg-[#111422] border-[#222b44]'
              ]"
            >
              <div>
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <div
                      class="w-7 h-7 rounded-xl flex items-center justify-center text-xs"
                      :class="isMlxReady ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'bg-purple-500/20 text-purple-300 border border-purple-500/30'"
                    >
                      <Cpu class="w-3.5 h-3.5" />
                    </div>
                    <div>
                      <h4 class="font-bold text-xs text-slate-100">Aceleração Apple Silicon (MLX)</h4>
                      <span class="text-[10px] text-slate-400">Framework oficial Apple</span>
                    </div>
                  </div>

                  <div
                    v-if="isMlxReady"
                    class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 text-[10px] font-bold flex items-center gap-1 border border-emerald-500/30"
                  >
                    <Check class="w-2.5 h-2.5" />
                    <span>Pronto</span>
                  </div>
                  <div
                    v-else-if="isBootstrapping"
                    class="px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 text-[10px] font-medium flex items-center gap-1 border border-indigo-500/30"
                  >
                    <Loader2 class="w-2.5 h-2.5 animate-spin" />
                    <span>Instalando...</span>
                  </div>
                  <div
                    v-else
                    class="px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-300 text-[10px] font-medium flex items-center gap-1 border border-amber-500/25"
                  >
                    <AlertCircle class="w-2.5 h-2.5" />
                    <span>Pendente</span>
                  </div>
                </div>

                <p class="text-[11px] text-slate-400 leading-snug">
                  Ambiente Python 3.11 isolado em ~/Library gerenciando <span class="font-mono text-slate-300">mlx-lm</span>, <span class="font-mono text-slate-300">mlx-vlm</span> e <span class="font-mono text-slate-300">mlx-whisper</span>.
                </p>
              </div>

              <div class="mt-3 pt-2.5 border-t border-[#1c2338] flex items-center justify-between text-[10px] text-slate-400 font-mono">
                <span>Versão MLX:</span>
                <span class="text-purple-300 font-medium">{{ runtimeStatus?.mlx_version ? `v${runtimeStatus.mlx_version}` : 'Python 3.11 venv' }}</span>
              </div>
            </div>
          </div>

          <!-- Progress Bar & Active Installation Details -->
          <div v-if="isBootstrapping" class="p-4 rounded-2xl bg-[#121629] border border-indigo-500/30 space-y-2.5 animate-fade-in">
            <div class="flex items-center justify-between text-xs font-medium">
              <span class="text-slate-200 truncate flex items-center gap-2">
                <Loader2 class="w-4 h-4 text-indigo-400 animate-spin flex-shrink-0" />
                <span class="truncate">{{ currentMessage || 'Configurando ambiente de IA...' }}</span>
              </span>
              <span class="font-mono text-indigo-400 font-bold ml-2">{{ progressPercent }}%</span>
            </div>

            <div class="w-full h-2 rounded-full bg-[#181f38] overflow-hidden">
              <div
                class="h-full bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-400 rounded-full transition-all duration-300 ease-out"
                :style="{ width: `${progressPercent}%` }"
              ></div>
            </div>
            <p class="text-[10px] text-slate-400">
              O Atena está preparando os pacotes em segundo plano de forma 100% autônoma. Não é necessário executar nenhum comando no terminal.
            </p>
          </div>

          <!-- Error Alert if installation fails -->
          <div v-if="errorMessage" class="p-3.5 rounded-2xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs space-y-2">
            <div class="flex items-center gap-2 font-semibold">
              <AlertCircle class="w-4 h-4 text-rose-400 flex-shrink-0" />
              <span>Ocorreu um erro durante a configuração:</span>
            </div>
            <p class="text-[11px] text-rose-200 leading-relaxed">{{ errorMessage }}</p>
            <button
              type="button"
              @click="startInstallEngines"
              class="px-3 py-1.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer shadow-md"
            >
              <RefreshCw class="w-3.5 h-3.5" />
              <span>Tentar Novamente</span>
            </button>
          </div>

          <!-- Ready Reassurance or Trigger Button -->
          <div v-if="!isBootstrapping && !isAllEnginesReady && !errorMessage" class="p-3.5 rounded-2xl bg-[#13182b] border border-[#232d4b] flex items-center justify-between gap-3">
            <div class="text-xs text-slate-300 space-y-0.5">
              <span class="font-semibold text-slate-100">Pronto para instalar os motores</span>
              <p class="text-[11px] text-slate-400">Instalação 100% autônoma dos binários e bibliotecas de inferência.</p>
            </div>
            <button
              type="button"
              @click="startInstallEngines"
              class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-all shadow-md shadow-indigo-600/30 cursor-pointer active:scale-95 shrink-0"
            >
              <DownloadCloud class="w-3.5 h-3.5" />
              <span>Instalar Agora</span>
            </button>
          </div>

          <div v-if="isAllEnginesReady && !isBootstrapping" class="p-3.5 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 flex items-center gap-2.5 text-xs text-emerald-300 animate-fade-in">
            <CheckCircle2 class="w-4 h-4 text-emerald-400 flex-shrink-0" />
            <span>Todos os motores de IA estão prontos e verificados com aceleração de hardware ativa. Clique em Avançar.</span>
          </div>

          <!-- Bypass Option -->
          <div v-if="!isAllEnginesReady && !isBootstrapping" class="text-center pt-1">
            <button
              type="button"
              @click="setupBypassed = true; currentStep++"
              class="text-[10.5px] text-slate-500 hover:text-slate-400 underline cursor-pointer transition-colors"
            >
              Pular esta etapa por enquanto (usar apenas modelos na nuvem/Antigravity)
            </button>
          </div>
        </div>

        <!-- STEP 3: Cognitive Memory & Continuous Learning (Beta) -->
        <div v-if="currentStep === 3" class="space-y-4 animate-fade-in">
          <div>
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Brain class="w-4 h-4 text-purple-400" />
                <span>{{ $t('setup.step3_title') }}</span>
              </h3>
              <span class="text-[9.5px] font-bold text-purple-300 bg-purple-500/15 border border-purple-500/30 px-2 py-0.5 rounded-full uppercase tracking-wider flex items-center gap-1">
                <Sparkles class="w-3 h-3 text-purple-400" />
                <span>{{ $t('setup.beta_badge') }}</span>
              </span>
            </div>
            <p class="text-[11.5px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('setup.step3_desc') }}
            </p>
          </div>

          <!-- Main Interactive Memory Card with Switch -->
          <div
            @click="memoryEnabled = !memoryEnabled"
            class="p-4 rounded-2xl border transition-all cursor-pointer select-none"
            :class="[
              memoryEnabled
                ? 'bg-gradient-to-r from-[#151a30] via-[#14162b] to-[#16122b] border-indigo-500/50 shadow-lg shadow-indigo-500/10'
                : 'bg-[#111422] border-[#202740] hover:border-slate-600'
            ]"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="flex items-start gap-3">
                <div
                  class="w-10 h-10 rounded-2xl flex items-center justify-center shrink-0 transition-all"
                  :class="[
                    memoryEnabled
                      ? 'bg-gradient-to-tr from-indigo-600/30 to-purple-600/30 border border-indigo-500/40 text-purple-300 shadow-md shadow-indigo-600/20'
                      : 'bg-[#161b2e] border border-[#242c48] text-slate-500'
                  ]"
                >
                  <Brain class="w-5 h-5" />
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <h4 class="font-bold text-xs text-slate-100">{{ $t('setup.memory_card_title') }}</h4>
                    <span
                      class="px-2 py-0.5 rounded-full text-[10px] font-bold flex items-center gap-1 transition-all"
                      :class="[
                        memoryEnabled
                          ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
                          : 'bg-slate-800 text-slate-400 border border-slate-700'
                      ]"
                    >
                      <Check v-if="memoryEnabled" class="w-2.5 h-2.5" />
                      <span>{{ memoryEnabled ? $t('setup.memory_active') : $t('setup.memory_disabled') }}</span>
                    </span>
                  </div>
                  <p class="text-[11px] text-slate-400 leading-relaxed mt-1">
                    {{ $t('setup.memory_card_desc') }}
                  </p>
                </div>
              </div>

              <!-- Switch Toggle -->
              <div
                class="relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out cursor-pointer mt-0.5"
                :class="[
                  memoryEnabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1f263d]'
                ]"
              >
                <span
                  class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                  :class="[
                    memoryEnabled ? 'translate-x-5' : 'translate-x-0'
                  ]"
                />
              </div>
            </div>
          </div>

          <!-- Feature Cards Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
            <div class="p-3 rounded-xl bg-[#111422] border border-[#1e253b] flex flex-col gap-1.5">
              <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-200">
                <ShieldCheck class="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                <span>{{ $t('setup.feature_local_title') }}</span>
              </div>
              <p class="text-[10.5px] text-slate-400 leading-snug">
                {{ $t('setup.feature_local_desc') }}
              </p>
            </div>

            <div class="p-3 rounded-xl bg-[#111422] border border-[#1e253b] flex flex-col gap-1.5">
              <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-200">
                <Zap class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                <span>{{ $t('setup.feature_tokens_title') }}</span>
              </div>
              <p class="text-[10.5px] text-slate-400 leading-snug">
                {{ $t('setup.feature_tokens_desc') }}
              </p>
            </div>

            <div class="p-3 rounded-xl bg-[#111422] border border-[#1e253b] flex flex-col gap-1.5">
              <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-200">
                <Lock class="w-3.5 h-3.5 text-violet-400 shrink-0" />
                <span>{{ $t('setup.feature_private_title') }}</span>
              </div>
              <p class="text-[10.5px] text-slate-400 leading-snug">
                {{ $t('setup.feature_private_desc') }}
              </p>
            </div>
          </div>

          <!-- Beta Notice Pill -->
          <div class="p-3 rounded-xl bg-[#131728] border border-[#202740] flex items-center gap-2 text-[10.5px] text-slate-400">
            <Sparkles class="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span>{{ $t('setup.beta_notice') }}</span>
          </div>
        </div>

        <!-- STEP 4: Model Folder & Local Directories -->
        <div v-if="currentStep === 4" class="space-y-4 animate-fade-in">
          <div>
            <h3 class="text-sm font-bold text-slate-100">{{ $t('setup.step4_title') }}</h3>
            <p class="text-[11.5px] text-slate-400 mt-0.5 leading-relaxed">
              {{ $t('setup.step4_desc') }}
            </p>
          </div>

          <!-- Active / Configured Directories List -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-bold text-slate-300 uppercase tracking-wide">
                Diretórios Selecionados ({{ configuredDirs.length }})
              </span>
              <button
                type="button"
                @click="handleAddCustomFolder"
                class="px-2.5 py-1 rounded-xl bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
              >
                <FolderPlus class="w-3.5 h-3.5" />
                <span>Adicionar Pasta...</span>
              </button>
            </div>

            <!-- List of dirs -->
            <div class="space-y-1.5 max-h-36 overflow-y-auto pr-1">
              <div
                v-for="(dir, idx) in configuredDirs"
                :key="dir"
                class="flex items-center justify-between p-2.5 rounded-xl bg-[#121524] border border-[#202740] text-xs font-mono text-slate-300 group"
              >
                <div class="flex items-center gap-2 truncate flex-1 mr-2">
                  <Folder class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                  <span class="truncate">{{ dir }}</span>
                  <span v-if="idx === 0" class="text-[9.5px] px-1.5 py-0.2 rounded bg-indigo-500/20 text-indigo-300 shrink-0">
                    Principal
                  </span>
                </div>
                <button
                  type="button"
                  @click="removeFolder(idx)"
                  class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-rose-500/20 text-slate-500 hover:text-rose-300 transition-all cursor-pointer"
                  title="Remover pasta"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </div>

          <!-- Suggested Directories Found on the Machine -->
          <div v-if="suggestedDirs.length > 0" class="pt-2 border-t border-[#1c2338] space-y-2">
            <span class="text-[11px] font-bold text-slate-400 uppercase tracking-wide flex items-center gap-1.5">
              <Sparkles class="w-3 h-3 text-amber-400" />
              <span>Pastas Sugeridas Encontradas no Computador</span>
            </span>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <div
                v-for="sug in suggestedDirs"
                :key="sug.path"
                class="p-2.5 rounded-xl bg-[#111422] border border-[#1d243a] flex items-center justify-between text-xs gap-2"
              >
                <div class="truncate">
                  <div class="font-bold text-slate-200 truncate flex items-center gap-1.5">
                    <span>{{ sug.name }}</span>
                  </div>
                  <div class="text-[10px] font-mono text-slate-500 truncate">{{ sug.path }}</div>
                </div>

                <button
                  type="button"
                  @click="addSuggestedFolder(sug.path)"
                  class="px-2 py-1 rounded-lg text-[10.5px] font-semibold flex items-center gap-1 transition-all shrink-0 cursor-pointer"
                  :class="[
                    configuredDirs.includes(sug.path)
                      ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
                      : 'bg-[#181e33] hover:bg-[#222a46] text-slate-300 border border-[#252f4d]'
                  ]"
                >
                  <Check v-if="configuredDirs.includes(sug.path)" class="w-3 h-3" />
                  <Plus v-else class="w-3 h-3" />
                  <span>{{ configuredDirs.includes(sug.path) ? 'Incluída' : 'Importar' }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- STEP 5: Ready & Summary -->
        <div v-if="currentStep === 5" class="space-y-4 animate-fade-in">
          <div class="text-center py-3">
            <div class="w-12 h-12 rounded-2xl bg-gradient-to-br from-emerald-500/20 to-indigo-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400 mx-auto mb-3 shadow-lg shadow-emerald-500/10">
              <CheckCircle2 class="w-6 h-6" />
            </div>
            <h3 class="text-base font-bold text-slate-100">{{ $t('setup.step5_title') }}</h3>
            <p class="text-xs text-slate-400 mt-1">
              {{ $t('setup.step5_desc') }}
            </p>
          </div>

          <!-- Summary Cards -->
          <div class="space-y-2 bg-[#101322] border border-[#1d233a] rounded-2xl p-4">
            <div class="flex items-center justify-between pb-2 border-b border-[#1b2138]">
              <span class="text-slate-400 text-xs">{{ $t('setup.summary_assistant') }}</span>
              <span class="font-bold text-indigo-300 text-xs">{{ activePersonaSummaryName }}</span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-[#1b2138]">
              <span class="text-slate-400 text-xs">{{ $t('setup.summary_engines') }}</span>
              <span class="font-semibold text-emerald-400 text-xs flex items-center gap-1">
                <Check class="w-3 h-3" />
                <span>{{ isAllEnginesReady ? 'GGUF (Metal) + MLX Ativos' : 'Prontos para Uso' }}</span>
              </span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-[#1b2138]">
              <span class="text-slate-400 text-xs">{{ $t('setup.summary_memory') }}</span>
              <span
                class="font-semibold text-xs flex items-center gap-1"
                :class="memoryEnabled ? 'text-emerald-400' : 'text-slate-400'"
              >
                <Check v-if="memoryEnabled" class="w-3 h-3" />
                <span>{{ memoryEnabled ? $t('setup.memory_active') : $t('setup.memory_disabled') }}</span>
              </span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-[#1b2138]">
              <span class="text-slate-400 text-xs">{{ $t('setup.summary_folders') }}</span>
              <span class="font-mono text-slate-200 text-xs">{{ configuredDirs.length }} pasta(s) configurada(s)</span>
            </div>
            <div class="flex items-center justify-between pt-2">
              <span class="text-slate-400 text-xs">{{ $t('setup.summary_connectors') }}</span>
              <span class="text-xs text-indigo-300 font-medium">
                {{ $t('setup.summary_connectors_desc') }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer Controls -->
      <div class="p-4 sm:p-5 border-t border-[#1c2338] bg-[#121627] flex items-center justify-between">
        <button
          v-if="currentStep > 1"
          type="button"
          @click="currentStep--"
          :disabled="isBootstrapping"
          class="px-4 py-2 rounded-xl bg-[#171c2f] hover:bg-[#202740] disabled:opacity-50 text-slate-300 hover:text-white transition-all text-xs font-medium flex items-center gap-1.5 border border-[#222a44] cursor-pointer"
        >
          <ArrowLeft class="w-3.5 h-3.5" />
          <span>{{ $t('setup.btn_back') }}</span>
        </button>
        <div v-else></div>

        <button
          v-if="currentStep < totalSteps"
          type="button"
          @click="handleNextStep"
          :disabled="currentStep === 2 && isBootstrapping"
          class="px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-50 text-white font-semibold text-xs transition-all shadow-md shadow-indigo-600/25 flex items-center gap-1.5 cursor-pointer"
        >
          <Loader2 v-if="currentStep === 2 && isBootstrapping" class="w-3.5 h-3.5 animate-spin" />
          <span>{{ currentStep === 2 && isBootstrapping ? $t('setup.btn_installing') : (currentStep === 2 && !canProceedFromEnginesStep ? $t('setup.btn_install_next') : $t('setup.btn_next')) }}</span>
          <ArrowRight v-if="!(currentStep === 2 && isBootstrapping)" class="w-3.5 h-3.5" />
        </button>

        <button
          v-else
          type="button"
          @click="finishSetup"
          class="px-6 py-2 rounded-xl bg-gradient-to-r from-emerald-600 via-teal-500 to-indigo-600 hover:opacity-95 text-white font-bold text-xs transition-all shadow-lg shadow-emerald-600/25 flex items-center gap-1.5 cursor-pointer active:scale-95"
        >
          <Sparkles class="w-3.5 h-3.5" />
          <span>{{ $t('setup.btn_finish') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, onMounted, type ComputedRef } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import {
  Sparkles,
  CheckCircle2,
  Folder,
  FolderPlus,
  Trash2,
  Cloud,
  Check,
  Plus,
  ArrowRight,
  ArrowLeft,
  Terminal,
  RefreshCw,
  Edit3,
  PenTool,
  GraduationCap,
  Code2,
  Cpu,
  Zap,
  DownloadCloud,
  Loader2,
  AlertCircle,
  Brain,
  ShieldCheck,
  Lock
} from 'lucide-vue-next'
import {
  DEFAULT_PERSONAS,
  setActivePersona,
  saveCustomPersona,
  allPersonas
} from '~/utils/personas'
import { contractUserPath } from '~/utils/pathUtils'
import type { AppConfig, DetectedModelDirectory } from '~/types'

const supportsMlx = inject<ComputedRef<boolean>>('supportsMlx', computed(() => true))

const props = defineProps<{
  config: AppConfig
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'setupComplete'): void
  (e: 'saveConfig', config: AppConfig): void
}>()

const currentStep = ref(1)
const totalSteps = ref(5)

// Step 3: Cognitive Memory (Beta)
const memoryEnabled = ref(props.config.enable_cognitive_memory !== false)

// Step 1: Personas
const selectedPersonaId = ref('default')
const isEditingPersona = ref(false)
const customPersonaName = ref('Atena Studio')
const customPersonaPrompt = ref('')

const selectablePersonas = computed(() => {
  return DEFAULT_PERSONAS
})

const activePersonaSummaryName = computed(() => {
  if (isEditingPersona.value && customPersonaName.value.trim()) {
    return customPersonaName.value.trim()
  }
  const match = selectablePersonas.value.find((p) => p.id === selectedPersonaId.value)
  return match ? match.name : 'Atena Studio'
})

// Step 2: Runtime & AI Engines
const runtimeStatus = ref<any>(null)
const isBootstrapping = ref(false)
const progressPercent = ref(0)
const currentMessage = ref('')
const errorMessage = ref('')
const setupBypassed = ref(false)

const isLlamaReady = computed(() => !!runtimeStatus.value?.llama_server_available)
const isMlxReady = computed(() => !supportsMlx.value || !!runtimeStatus.value?.mlx_ready)
const isAllEnginesReady = computed(() => isLlamaReady.value && isMlxReady.value)

const canProceedFromEnginesStep = computed(() => {
  return isAllEnginesReady.value || setupBypassed.value
})

const fetchRuntimeStatus = async () => {
  try {
    const st = await invoke('get_runtime_status')
    runtimeStatus.value = st
    return st
  } catch (err) {
    console.warn('Falha ao obter status do runtime:', err)
  }
  return null
}

const startInstallEngines = async () => {
  if (isBootstrapping.value) return
  isBootstrapping.value = true
  errorMessage.value = ''
  progressPercent.value = 5
  currentMessage.value = 'Iniciando preparação dos motores de inferência...'

  try {
    const channel = new Channel()
    channel.onmessage = (event: any) => {
      if (typeof event?.progress_percent === 'number') {
        progressPercent.value = event.progress_percent
      }
      if (event?.message) {
        currentMessage.value = event.message
      }
    }

    await invoke('bootstrap_all_runtimes', {
      channel,
      fromGithub: false
    })

    await fetchRuntimeStatus()
    progressPercent.value = 100
    currentMessage.value = 'Todos os motores estão configurados e prontos!'
  } catch (err: any) {
    console.error('Falha na instalação dos motores:', err)
    errorMessage.value = typeof err === 'string' ? err : (err?.message || 'Falha ao instalar motores.')
  } finally {
    isBootstrapping.value = false
  }
}

watch(currentStep, (newStep) => {
  if (newStep === 2) {
    fetchRuntimeStatus().then((st: any) => {
      if (st && (!st.llama_server_available || (supportsMlx.value && !st.mlx_ready))) {
        // Auto start installation seamlessly
        startInstallEngines()
      }
    })
  }
})

// Step 3: Multi-path directories
const configuredDirs = ref<string[]>([])
const suggestedDirs = ref<DetectedModelDirectory[]>([])

const loadDirectories = async () => {
  // Initialize with currently configured dirs or default
  if (props.config.models_directories && Array.isArray(props.config.models_directories) && props.config.models_directories.length > 0) {
    configuredDirs.value = props.config.models_directories.map(contractUserPath)
  } else if (props.config.models_directory) {
    configuredDirs.value = [contractUserPath(props.config.models_directory)]
  }

  // Detect available directories on the system
  try {
    const list = await invoke<any[]>('detect_model_directories')
    if (Array.isArray(list)) {
      const normalizedList = list.map((d) => ({ ...d, path: contractUserPath(d.path) }))
      // Ensure primary atena dir is present
      const atenaDir = normalizedList.find((d) => d.source === 'atena')
      if (atenaDir && configuredDirs.value.length === 0) {
        configuredDirs.value.push(atenaDir.path)
      }

      // Filter suggested dirs that exist and are not yet added
      suggestedDirs.value = normalizedList.filter((d) => d.exists && d.source !== 'atena')
    }
  } catch (err) {
    console.warn('Falha ao detectar diretórios:', err)
  }
}

const addSuggestedFolder = (path: string) => {
  const contracted = contractUserPath(path)
  if (!configuredDirs.value.includes(contracted)) {
    configuredDirs.value.push(contracted)
  }
}

const removeFolder = (index: number) => {
  if (configuredDirs.value.length > 1) {
    configuredDirs.value.splice(index, 1)
  }
}

const handleAddCustomFolder = async () => {
  try {
    const selected = await invoke<string>('select_folder', {
      defaultPath: configuredDirs.value[0] || null
    })
    if (selected) {
      const contracted = contractUserPath(selected)
      if (!configuredDirs.value.includes(contracted)) {
        configuredDirs.value.push(contracted)
      }
    }
  } catch (err) {
    console.error('Erro ao escolher pasta:', err)
  }
}

const handleNextStep = () => {
  if (currentStep.value === 2 && !canProceedFromEnginesStep.value) {
    startInstallEngines()
    return
  }
  if (currentStep.value < totalSteps.value) {
    currentStep.value++
  }
}

const finishSetup = () => {
  // 1. Save chosen persona or customized persona
  if (isEditingPersona.value && customPersonaName.value.trim()) {
    const customId = `custom_user_${Date.now()}`
    saveCustomPersona({
      id: customId,
      name: customPersonaName.value.trim(),
      tag: 'Personalizada',
      iconName: 'Sparkles',
      color: 'indigo',
      description: 'Assistente adaptado com suas instruções personalizadas.',
      system_prompt: customPersonaPrompt.value.trim()
    })
    setActivePersona(customId)
  } else {
    setActivePersona(selectedPersonaId.value)
  }

  // 2. Save configured model directories
  props.config.models_directories = configuredDirs.value.map(contractUserPath)
  if (configuredDirs.value.length > 0) {
    props.config.models_directory = contractUserPath(configuredDirs.value[0])
  }

  // 3. Save cognitive memory setting
  props.config.enable_cognitive_memory = memoryEnabled.value

  // 4. Mark setup as completed in localStorage & save native configuration
  if (typeof window !== 'undefined') {
    localStorage.setItem('atena_config', JSON.stringify(props.config))
    localStorage.setItem('atena_setup_completed', 'true')
  }

  emit('saveConfig', props.config)
  emit('setupComplete')
  emit('close')
}

onMounted(async () => {
  loadDirectories()
  const st: any = await fetchRuntimeStatus()
  if (currentStep.value === 2 && st && (!st.llama_server_available || (supportsMlx.value && !st.mlx_ready))) {
    startInstallEngines()
  }
})
</script>

<style scoped>
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
.animate-fade-in {
  animation: fadeIn 0.15s ease-out forwards;
}
</style>
