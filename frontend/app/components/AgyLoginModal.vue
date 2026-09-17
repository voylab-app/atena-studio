<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-fade-in select-none">
    <div
      class="w-full max-w-xl bg-[#0f121e] border border-[#232a42] rounded-3xl shadow-2xl shadow-black/80 overflow-hidden flex flex-col max-h-[90vh]"
    >
      <!-- Modal Header -->
      <div class="p-5 border-b border-[#1b2135] bg-[#121627] flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-300 shadow-inner">
            <Cloud class="w-5 h-5 text-indigo-400" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-slate-100 tracking-tight">Google Antigravity (AGY)</h3>
              <span
                v-if="!hasInitialCheckDone && isChecking"
                class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-indigo-500/15 text-indigo-300 border border-indigo-500/30 flex items-center gap-1.5"
              >
                <Loader2 class="w-2.5 h-2.5 animate-spin text-indigo-400" />
                Verificando...
              </span>
              <span
                v-else-if="status.authenticated"
                class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 flex items-center gap-1"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                Sessão Ativa
              </span>
              <span
                v-else-if="status.installed"
                class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-amber-500/15 text-amber-300 border border-amber-500/30 flex items-center gap-1"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
                Desconectado
              </span>
              <span
                v-else
                class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-rose-500/15 text-rose-300 border border-rose-500/30 flex items-center gap-1"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-rose-400"></span>
                Não Instalado
              </span>
            </div>
            <p class="text-[11px] text-slate-400 mt-0.5">
              Conexão com a nuvem de inteligência Google DeepMind e modelos Gemini 3.8
            </p>
          </div>
        </div>

        <button
          @click="$emit('close')"
          class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-[#1a2035] transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 overflow-y-auto space-y-5 text-xs text-slate-300">
        <!-- Checking loader banner (somente se já houver dados e estiver sincronizando em segundo plano) -->
        <div
          v-if="isChecking && hasInitialCheckDone"
          class="p-3 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-between gap-3 text-indigo-300 animate-fade-in"
        >
          <div class="flex items-center gap-2.5">
            <Loader2 class="w-3.5 h-3.5 animate-spin text-indigo-400" />
            <span class="text-xs">Sincronizando credenciais e modelos no Antigravity CLI...</span>
          </div>
          <span class="text-[10px] text-indigo-400/80 font-mono">Segundo plano</span>
        </div>

        <!-- Initial Loading State (quando não há cache e a checagem inicial está rodando) -->
        <div v-if="!hasInitialCheckDone && isChecking" class="py-16 flex flex-col items-center justify-center text-center space-y-4 animate-fade-in">
          <div class="w-14 h-14 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <Loader2 class="w-7 h-7 animate-spin text-indigo-400" />
          </div>
          <div class="space-y-1">
            <h4 class="text-sm font-bold text-slate-200">Verificando Antigravity CLI...</h4>
            <p class="text-xs text-slate-400 max-w-sm">
              Detectando instalação, conta Google conectada e modelos disponíveis na nuvem.
            </p>
          </div>
        </div>

        <!-- Authenticated State -->
        <div v-else-if="status.authenticated" class="space-y-4">
          <div class="p-4 rounded-2xl bg-gradient-to-br from-emerald-500/10 via-[#101924] to-[#0e1322] border border-emerald-500/25 space-y-2.5 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <CheckCircle2 class="w-4 h-4 text-emerald-400" />
                <span class="font-bold text-xs text-emerald-200">Sessão Conectada com Sucesso</span>
              </div>
              <span class="text-[10.5px] font-mono text-emerald-400/80 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
                Nuvem Ativa
              </span>
            </div>

            <div v-if="status.active_account" class="flex items-center gap-2 pt-1 text-[11px] text-slate-300">
              <UserCheck class="w-3.5 h-3.5 text-indigo-400" />
              <span>Conta vinculada:</span>
              <span class="font-mono text-indigo-300 font-semibold">{{ status.active_account }}</span>
            </div>

            <p class="text-[11px] text-slate-400 leading-relaxed">
              Você tem acesso aos modelos multimodais de altíssima velocidade do Google Antigravity sem ocupar a memória RAM do seu computador.
            </p>
          </div>

          <!-- Quota & Remaining Limits Section -->
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Gauge class="w-4 h-4 text-indigo-400" />
                <span class="text-[11px] font-bold uppercase tracking-wider text-slate-300">
                  Cotas e Limites de Uso
                </span>
              </div>
              <button
                @click="fetchUsage"
                :disabled="isLoadingUsage"
                title="Atualizar limites agora"
                class="text-[10px] font-medium text-slate-400 hover:text-indigo-300 transition-colors flex items-center gap-1 cursor-pointer disabled:opacity-50"
              >
                <RefreshCw class="w-3 h-3 text-indigo-400" :class="{ 'animate-spin': isLoadingUsage }" />
                <span>{{ isLoadingUsage ? 'Atualizando...' : 'Atualizar' }}</span>
              </button>
            </div>

            <!-- Loading Skeleton for Usage -->
            <div v-if="isLoadingUsage && !usageData" class="p-4 rounded-2xl bg-[#131728] border border-[#1f2640] space-y-3 animate-pulse">
              <div class="h-4 bg-slate-700/40 rounded w-1/3"></div>
              <div class="h-2 bg-slate-700/30 rounded w-full"></div>
              <div class="h-2 bg-slate-700/30 rounded w-4/5"></div>
            </div>

            <!-- Usage Groups -->
            <div v-else-if="usageData && usageData.groups && usageData.groups.length > 0" class="space-y-2.5">
              <div
                v-for="group in usageData.groups"
                :key="group.name"
                class="p-3.5 rounded-2xl bg-[#121627] border border-[#1f2640] space-y-3 shadow-inner"
              >
                <!-- Group Header -->
                <div class="flex items-center justify-between">
                  <div>
                    <h5 class="text-xs font-bold text-slate-200 flex items-center gap-1.5">
                      <span>{{ group.name }}</span>
                    </h5>
                    <p v-if="group.description" class="text-[10px] text-slate-400">
                      {{ group.description }}
                    </p>
                  </div>
                </div>

                <!-- Buckets (Weekly & 5-Hour limits) -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                  <div
                    v-for="bucket in group.buckets"
                    :key="bucket.id"
                    class="p-2.5 rounded-xl bg-[#0e1220] border border-[#1a2035] space-y-1.5 flex flex-col justify-between"
                  >
                    <div class="flex items-start justify-between gap-1">
                      <div class="flex items-center gap-1.5 min-w-0">
                        <Clock v-if="bucket.window === '5h'" class="w-3 h-3 text-amber-400 flex-shrink-0" />
                        <Calendar v-else class="w-3 h-3 text-indigo-400 flex-shrink-0" />
                        <span class="text-[11px] font-semibold text-slate-300 truncate">
                          {{ bucket.window === '5h' ? 'Janela de 5 Horas' : 'Limite Semanal' }}
                        </span>
                      </div>
                      <span
                        class="text-[11px] font-mono font-bold px-1.5 py-0.2 rounded border"
                        :class="getPercentageBadgeClass(bucket.remaining_fraction)"
                      >
                        {{ Math.round(bucket.remaining_fraction * 100) }}% disp.
                      </span>
                    </div>

                    <!-- Progress Bar -->
                    <div class="w-full bg-[#1b2238] h-1.5 rounded-full overflow-hidden">
                      <div
                        class="h-full rounded-full transition-all duration-500"
                        :class="getProgressBarColorClass(bucket.remaining_fraction)"
                        :style="{ width: `${Math.min(Math.max(bucket.remaining_fraction * 100, 0), 100)}%` }"
                      ></div>
                    </div>

                    <!-- Reset Time / Description -->
                    <div class="text-[9.5px] text-slate-400 flex items-center justify-between pt-0.5">
                      <span class="truncate" :title="bucket.description || ''">
                        {{ formatResetCountdown(bucket.reset_time, bucket.description) }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Error Loading Usage -->
            <div v-else-if="usageError" class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-[10.5px] text-amber-300 flex items-center justify-between">
              <span class="truncate">{{ usageError }}</span>
              <button @click="fetchUsage" class="text-indigo-400 hover:underline font-semibold ml-2 flex-shrink-0">Tentar de novo</button>
            </div>
          </div>

          <!-- Models List -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400">
                Modelos Cloud Disponíveis ({{ status.available_models.length }})
              </span>
              <span class="text-[10px] text-slate-500">Prontos para uso</span>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-40 overflow-y-auto pr-1">
              <div
                v-for="m in status.available_models"
                :key="m"
                class="p-2.5 rounded-xl bg-[#131728] border border-[#1f2640] flex items-center justify-between gap-2"
              >
                <div class="flex items-center gap-2 truncate">
                  <Sparkles class="w-3.5 h-3.5 text-amber-400 flex-shrink-0" />
                  <span class="font-mono text-[11px] text-slate-200 truncate">{{ m }}</span>
                </div>
                <span class="text-[9.5px] font-semibold text-indigo-400 bg-indigo-500/10 px-1.5 py-0.5 rounded border border-indigo-500/20 flex-shrink-0">
                  Cloud
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- State: Not Installed -->
        <div v-else-if="!status.installed" class="space-y-4">
          <!-- Banner: CLI Not Installed -->
          <div class="p-4 rounded-2xl bg-[#141828] border border-[#222942] space-y-3">
            <div class="flex items-start gap-3">
              <div class="w-8 h-8 rounded-xl bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400 flex-shrink-0 mt-0.5">
                <Download class="w-4 h-4" />
              </div>
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <h4 class="text-xs font-bold text-slate-200">
                    CLI do Antigravity (`agy`) Não Encontrado
                  </h4>
                  <span class="px-2 py-0.5 rounded-md text-[9.5px] font-semibold bg-indigo-500/15 text-indigo-300 border border-indigo-500/30 uppercase tracking-wider">
                    {{ detectedOs === 'macos' ? 'macOS' : (detectedOs === 'windows' ? 'Windows' : 'Linux') }} Detectado
                  </span>
                </div>
                <p class="text-[11px] text-slate-400 leading-relaxed">
                  O executável <code class="px-1 py-0.5 rounded bg-[#1c2238] text-indigo-300 font-mono text-[10.5px]">agy</code> não está instalado no seu sistema. Instale o CLI oficial do Google Antigravity para desbloquear modelos em nuvem como <strong>Gemini 3.8 Flash</strong> e <strong>Claude Sonnet</strong> de altíssima velocidade e sem consumir a memória RAM do computador.
                </p>
              </div>
            </div>
          </div>

          <!-- OS Selector Tabs -->
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
                <Terminal class="w-3.5 h-3.5 text-indigo-400" />
                <span>Instruções de Instalação por Sistema</span>
              </span>
              <button
                @click="openExternalDocs"
                class="text-[10px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 cursor-pointer transition-colors"
              >
                <span>Documentação Oficial</span>
                <ExternalLink class="w-3 h-3" />
              </button>
            </div>

            <!-- Tabs -->
            <div class="grid grid-cols-3 gap-1.5 p-1 bg-[#0c0e18] rounded-xl border border-[#1a1f33]">
              <button
                type="button"
                @click="selectedOs = 'macos'"
                class="py-1.5 px-2 rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-all cursor-pointer"
                :class="selectedOs === 'macos' ? 'bg-indigo-600/25 border border-indigo-500/40 text-indigo-200 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-[#141828] border border-transparent'"
              >
                <span>macOS</span>
                <span v-if="detectedOs === 'macos'" class="text-[8.5px] font-semibold px-1 py-0.2 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                  Detectado
                </span>
              </button>

              <button
                type="button"
                @click="selectedOs = 'linux'"
                class="py-1.5 px-2 rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-all cursor-pointer"
                :class="selectedOs === 'linux' ? 'bg-indigo-600/25 border border-indigo-500/40 text-indigo-200 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-[#141828] border border-transparent'"
              >
                <span>Linux</span>
                <span v-if="detectedOs === 'linux'" class="text-[8.5px] font-semibold px-1 py-0.2 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                  Detectado
                </span>
              </button>

              <button
                type="button"
                @click="selectedOs = 'windows'"
                class="py-1.5 px-2 rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-all cursor-pointer"
                :class="selectedOs === 'windows' ? 'bg-indigo-600/25 border border-indigo-500/40 text-indigo-200 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-[#141828] border border-transparent'"
              >
                <span>Windows</span>
                <span v-if="detectedOs === 'windows'" class="text-[8.5px] font-semibold px-1 py-0.2 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                  Detectado
                </span>
              </button>
            </div>

            <!-- Tab Content: macOS -->
            <div v-if="selectedOs === 'macos'" class="space-y-3">
              <!-- Method 1: Curl (Recommended) -->
              <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2.5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-[11px] font-semibold text-slate-200">Script Oficial Automático</span>
                    <span class="text-[9.5px] px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-semibold">
                      Recomendado
                    </span>
                  </div>
                  <span class="text-[10px] text-slate-500">Terminal (bash / zsh)</span>
                </div>

                <div class="p-2.5 rounded-xl bg-[#06070d] border border-[#171b2d] font-mono text-[11px] text-emerald-400 select-all overflow-x-auto">
                  <code>{{ COMMANDS.macCurl }}</code>
                </div>

                <div class="flex items-center justify-end gap-2 pt-0.5">
                  <button
                    @click="copyText(COMMANDS.macCurl, 'mac-curl')"
                    class="px-2.5 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Copy v-if="copiedKey !== 'mac-curl'" class="w-3 h-3" />
                    <Check v-else class="w-3 h-3 text-emerald-400" />
                    <span>{{ copiedKey === 'mac-curl' ? 'Copiado!' : 'Copiar' }}</span>
                  </button>
                  <button
                    @click="handleRunCommandInTerminal(COMMANDS.macCurl)"
                    :disabled="isLaunchingCmd"
                    class="px-3 py-1.5 rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 border border-indigo-500/40 text-indigo-200 hover:text-white text-[11px] font-semibold flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Terminal class="w-3 h-3 text-indigo-400" />
                    <span>{{ isLaunchingCmd ? 'Abrindo Terminal...' : 'Abrir Terminal e Instalar' }}</span>
                  </button>
                </div>
              </div>

              <!-- Method 2: Homebrew -->
              <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2.5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-[11px] font-semibold text-slate-200">Via Homebrew</span>
                    <span class="text-[9.5px] px-1.5 py-0.2 rounded bg-indigo-500/15 text-indigo-400 border border-indigo-500/30 font-semibold">
                      Alternativo
                    </span>
                  </div>
                  <span class="text-[10px] text-slate-500">Homebrew Cask</span>
                </div>

                <div class="p-2.5 rounded-xl bg-[#06070d] border border-[#171b2d] font-mono text-[11px] text-emerald-400 select-all overflow-x-auto">
                  <code>{{ COMMANDS.macBrew }}</code>
                </div>

                <div class="flex items-center justify-end gap-2 pt-0.5">
                  <button
                    @click="copyText(COMMANDS.macBrew, 'mac-brew')"
                    class="px-2.5 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Copy v-if="copiedKey !== 'mac-brew'" class="w-3 h-3" />
                    <Check v-else class="w-3 h-3 text-emerald-400" />
                    <span>{{ copiedKey === 'mac-brew' ? 'Copiado!' : 'Copiar' }}</span>
                  </button>
                  <button
                    @click="handleRunCommandInTerminal(COMMANDS.macBrew)"
                    :disabled="isLaunchingCmd"
                    class="px-3 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Terminal class="w-3 h-3 text-indigo-400" />
                    <span>Executar no Terminal</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- Tab Content: Linux -->
            <div v-else-if="selectedOs === 'linux'" class="space-y-3">
              <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2.5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-[11px] font-semibold text-slate-200">Script Oficial via Curl</span>
                    <span class="text-[9.5px] px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-semibold">
                      Recomendado
                    </span>
                  </div>
                  <span class="text-[10px] text-slate-500">Qualquer distro (Debian, Ubuntu, Fedora, Arch)</span>
                </div>

                <div class="p-2.5 rounded-xl bg-[#06070d] border border-[#171b2d] font-mono text-[11px] text-emerald-400 select-all overflow-x-auto">
                  <code>{{ COMMANDS.linuxCurl }}</code>
                </div>

                <div class="flex items-center justify-end gap-2 pt-0.5">
                  <button
                    @click="copyText(COMMANDS.linuxCurl, 'linux-curl')"
                    class="px-2.5 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Copy v-if="copiedKey !== 'linux-curl'" class="w-3 h-3" />
                    <Check v-else class="w-3 h-3 text-emerald-400" />
                    <span>{{ copiedKey === 'linux-curl' ? 'Copiado!' : 'Copiar' }}</span>
                  </button>
                  <button
                    @click="handleRunCommandInTerminal(COMMANDS.linuxCurl)"
                    :disabled="isLaunchingCmd"
                    class="px-3 py-1.5 rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 border border-indigo-500/40 text-indigo-200 hover:text-white text-[11px] font-semibold flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Terminal class="w-3 h-3 text-indigo-400" />
                    <span>{{ isLaunchingCmd ? 'Abrindo Terminal...' : 'Abrir Terminal e Instalar' }}</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- Tab Content: Windows -->
            <div v-else-if="selectedOs === 'windows'" class="space-y-3">
              <!-- PowerShell -->
              <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2.5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-[11px] font-semibold text-slate-200">PowerShell (Administrador)</span>
                    <span class="text-[9.5px] px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-semibold">
                      Recomendado
                    </span>
                  </div>
                  <span class="text-[10px] text-slate-500">PowerShell 5.1+ / 7+</span>
                </div>

                <div class="p-2.5 rounded-xl bg-[#06070d] border border-[#171b2d] font-mono text-[11px] text-emerald-400 select-all overflow-x-auto">
                  <code>{{ COMMANDS.winPs }}</code>
                </div>

                <div class="flex items-center justify-end gap-2 pt-0.5">
                  <button
                    @click="copyText(COMMANDS.winPs, 'win-ps')"
                    class="px-2.5 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Copy v-if="copiedKey !== 'win-ps'" class="w-3 h-3" />
                    <Check v-else class="w-3 h-3 text-emerald-400" />
                    <span>{{ copiedKey === 'win-ps' ? 'Copiado!' : 'Copiar' }}</span>
                  </button>
                  <button
                    @click="handleRunCommandInTerminal(COMMANDS.winPs)"
                    :disabled="isLaunchingCmd"
                    class="px-3 py-1.5 rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 border border-indigo-500/40 text-indigo-200 hover:text-white text-[11px] font-semibold flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Terminal class="w-3 h-3 text-indigo-400" />
                    <span>Executar no Terminal</span>
                  </button>
                </div>
              </div>

              <!-- CMD -->
              <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2.5">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-semibold text-slate-200">Prompt de Comando (CMD)</span>
                  <span class="text-[10px] text-slate-500">Alternativo</span>
                </div>

                <div class="p-2.5 rounded-xl bg-[#06070d] border border-[#171b2d] font-mono text-[11px] text-emerald-400 select-all overflow-x-auto">
                  <code>{{ COMMANDS.winCmd }}</code>
                </div>

                <div class="flex items-center justify-end gap-2 pt-0.5">
                  <button
                    @click="copyText(COMMANDS.winCmd, 'win-cmd')"
                    class="px-2.5 py-1.5 rounded-lg bg-[#161c30] hover:bg-[#1f2742] border border-[#242c4a] text-slate-300 hover:text-white text-[11px] font-medium flex items-center gap-1.5 cursor-pointer transition-colors"
                  >
                    <Copy v-if="copiedKey !== 'win-cmd'" class="w-3 h-3" />
                    <Check v-else class="w-3 h-3 text-emerald-400" />
                    <span>{{ copiedKey === 'win-cmd' ? 'Copiado!' : 'Copiar' }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Post-install advice -->
          <div class="p-3 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-[11px] text-indigo-200/90 flex items-start gap-2.5">
            <Sparkles class="w-4 h-4 text-indigo-400 flex-shrink-0 mt-0.5" />
            <span class="leading-relaxed">
              <strong>Próximo passo:</strong> Assim que a instalação for concluída no terminal, clique em <strong>"Verificar Sessão Novamente"</strong> no rodapé. O Atena detectará o <code>agy</code> imediatamente e abrirá a etapa de login!
            </span>
          </div>
        </div>

        <!-- State: Installed but Disconnected -->
        <div v-else class="space-y-4">
          <div class="p-4 rounded-2xl bg-[#141828] border border-[#222942] space-y-3">
            <div class="flex items-start gap-3">
              <div class="w-8 h-8 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 flex-shrink-0 mt-0.5">
                <AlertCircle class="w-4 h-4" />
              </div>
              <div class="space-y-1">
                <h4 class="text-xs font-bold text-slate-200">
                  Autenticação Necessária no Antigravity
                </h4>
                <p class="text-[11px] text-slate-400 leading-relaxed">
                  O executável <code class="px-1 py-0.2 rounded bg-[#1c2238] text-indigo-300 font-mono text-[10.5px]">agy</code> está instalado! Para usar modelos como Gemini 3.8 Flash e Claude Sonnet via nuvem de alta velocidade, faça o login rápido na sua conta Google através do terminal interativo.
                </p>
              </div>
            </div>

            <!-- Action Button for Login -->
            <div class="pt-2 flex flex-col sm:flex-row gap-2">
              <button
                @click="handleStartLogin"
                :disabled="isLaunchingLogin"
                class="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 bg-gradient-to-r from-indigo-600 via-indigo-500 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white rounded-xl text-xs font-semibold shadow-lg shadow-indigo-600/25 transition-all active:scale-[0.98] cursor-pointer"
              >
                <Terminal class="w-4 h-4" />
                <span>{{ isLaunchingLogin ? 'Abrindo Terminal...' : 'Abrir Terminal para Fazer Login' }}</span>
              </button>
            </div>
          </div>

          <!-- Manual Terminal Instructions -->
          <div class="p-3.5 rounded-2xl bg-[#0c0e18] border border-[#1a1f33] space-y-2">
            <div class="flex items-center justify-between text-[11px]">
              <span class="text-slate-400 font-medium">Ou execute manualmente no seu terminal:</span>
              <button
                @click="copyText('agy', 'login-agy')"
                class="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 cursor-pointer transition-colors"
              >
                <Copy v-if="copiedKey !== 'login-agy'" class="w-3 h-3" />
                <Check v-else class="w-3 h-3 text-emerald-400" />
                <span>{{ copiedKey === 'login-agy' ? 'Copiado!' : 'Copiar' }}</span>
              </button>
            </div>
            <div class="p-2.5 rounded-xl bg-[#080910] border border-[#191e30] font-mono text-[11px] text-emerald-400 flex items-center justify-between">
              <span>agy</span>
            </div>
            <p class="text-[10.5px] text-slate-500 leading-normal">
              Ao executar, siga as instruções na janela do terminal para autorizar via navegador. O Atena detectará automaticamente quando você concluir!
            </p>
          </div>
        </div>

        <!-- Error feedback if any (only when installed and unexpected error occurred) -->
        <div
          v-if="status.error_message && status.installed && !status.authenticated"
          class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-[11px] flex items-start gap-2"
        >
          <AlertTriangle class="w-3.5 h-3.5 text-rose-400 flex-shrink-0 mt-0.5" />
          <span class="leading-relaxed">{{ status.error_message }}</span>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-4 border-t border-[#1b2135] bg-[#121627] flex items-center justify-between">
        <button
          @click="checkStatus"
          :disabled="isChecking"
          class="px-3 py-1.5 rounded-xl bg-[#191f33] hover:bg-[#222a45] text-slate-300 hover:text-white transition-all text-xs font-medium flex items-center gap-1.5 border border-[#232c48] cursor-pointer"
        >
          <RefreshCw class="w-3.5 h-3.5 text-indigo-400" :class="{ 'animate-spin': isChecking }" />
          <span>Verificar Sessão Novamente</span>
        </button>

        <button
          @click="$emit('close')"
          class="px-5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs transition-all shadow-md shadow-indigo-600/20 cursor-pointer"
        >
          Concluir
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  Cloud,
  X,
  CheckCircle2,
  AlertCircle,
  AlertTriangle,
  Loader2,
  Terminal,
  Copy,
  Check,
  RefreshCw,
  Sparkles,
  UserCheck,
  Gauge,
  Clock,
  Calendar,
  Download,
  ExternalLink
} from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'sessionUpdated', session: any): void
}>()

const CACHE_SESSION_KEY = 'atena_agy_session_cache'
const CACHE_USAGE_KEY = 'atena_agy_usage_cache'

const loadCachedStatus = () => {
  try {
    const raw = localStorage.getItem(CACHE_SESSION_KEY)
    if (raw) return JSON.parse(raw)
  } catch (e) {}
  return null
}

const loadCachedUsage = () => {
  try {
    const raw = localStorage.getItem(CACHE_USAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch (e) {}
  return null
}

const cachedStatus = loadCachedStatus()
const cachedUsage = loadCachedUsage()

const hasInitialCheckDone = ref(!!cachedStatus)
const isChecking = ref(!cachedStatus)
const isLaunchingLogin = ref(false)
const isLaunchingCmd = ref(false)
const copiedKey = ref<string | null>(null)

const detectedOs = ref('macos')
const selectedOs = ref('macos')

const COMMANDS = {
  macCurl: 'curl -fsSL https://antigravity.google/cli/install.sh | bash',
  macBrew: 'brew install --cask antigravity-cli',
  linuxCurl: 'curl -fsSL https://antigravity.google/cli/install.sh | bash',
  winPs: 'irm https://antigravity.google/cli/install.ps1 | iex',
  winCmd: 'curl -fsSL https://antigravity.google/cli/install.cmd -o install.cmd && install.cmd && del install.cmd'
}

const status = ref(cachedStatus || {
  installed: false,
  binary_path: null,
  authenticated: false,
  active_account: null,
  available_models: [],
  error_message: null
})

const usageData = ref(cachedUsage || null)
const isLoadingUsage = ref(false)
const usageError = ref<string | null>(null)

let pollingInterval: ReturnType<typeof setInterval> | null = null

const copyText = async (text: string, key = 'default') => {
  try {
    await navigator.clipboard.writeText(text)
    copiedKey.value = key
    setTimeout(() => {
      if (copiedKey.value === key) {
        copiedKey.value = null
      }
    }, 2000)
  } catch (e) {
    console.warn('Erro ao copiar texto:', e)
  }
}

const handleRunCommandInTerminal = async (cmd: string) => {
  isLaunchingCmd.value = true
  try {
    await invoke('launch_terminal_command', { command: cmd })
  } catch (err) {
    console.error('Falha ao abrir o terminal com comando:', err)
  } finally {
    setTimeout(() => {
      isLaunchingCmd.value = false
    }, 1200)
  }
}

const openExternalDocs = async () => {
  try {
    await invoke('open_url', { url: 'https://antigravity.google/docs/cli/reference' })
  } catch (err) {
    console.warn('Falha ao abrir documentação:', err)
  }
}

const fetchUsage = async () => {
  if (!status.value.authenticated) return
  isLoadingUsage.value = true
  usageError.value = null
  try {
    const res = await invoke('get_agy_usage')
    if (res) {
      usageData.value = res
      try {
        localStorage.setItem(CACHE_USAGE_KEY, JSON.stringify(res))
      } catch (e) {}
    }
  } catch (err) {
    console.warn('Falha ao obter cota do Antigravity:', err)
    if (!usageData.value) {
      usageError.value = String(err)
    }
  } finally {
    isLoadingUsage.value = false
  }
}

const getPercentageBadgeClass = (fraction: number) => {
  const percent = fraction * 100
  if (percent >= 50) {
    return 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30'
  }
  if (percent >= 20) {
    return 'bg-amber-500/15 text-amber-300 border-amber-500/30'
  }
  return 'bg-rose-500/15 text-rose-400 border-rose-500/30'
}

const getProgressBarColorClass = (fraction: number) => {
  const percent = fraction * 100
  if (percent >= 50) {
    return 'bg-gradient-to-r from-emerald-500 to-teal-400'
  }
  if (percent >= 20) {
    return 'bg-gradient-to-r from-amber-500 to-yellow-400'
  }
  return 'bg-gradient-to-r from-rose-600 to-rose-400'
}

const formatResetCountdown = (resetTimeStr?: string | null, desc?: string | null) => {
  if (!resetTimeStr) {
    return desc || 'Pronto'
  }
  try {
    const target = new Date(resetTimeStr)
    const now = new Date()
    const diffMs = target.getTime() - now.getTime()
    if (diffMs <= 0) return 'Renovação concluída'

    const totalMinutes = Math.floor(diffMs / 60000)
    const totalHours = Math.floor(totalMinutes / 60)
    const days = Math.floor(totalHours / 24)
    const remHours = totalHours % 24
    const remMinutes = totalMinutes % 60

    if (days > 0) {
      return `Renova em ${days}d ${remHours}h`
    }
    if (totalHours > 0) {
      return `Renova em ${totalHours}h ${remMinutes}m`
    }
    return `Renova em ${totalMinutes}m`
  } catch {
    return desc || 'Renovação em breve'
  }
}

const checkStatus = async () => {
  isChecking.value = true
  try {
    const res = await invoke('check_agy_session') as any
    if (res) {
      const wasAuth = status.value.authenticated
      status.value = res
      hasInitialCheckDone.value = true
      try {
        localStorage.setItem(CACHE_SESSION_KEY, JSON.stringify(res))
      } catch (e) {}
      emit('sessionUpdated', res)
      if (res.authenticated && (!wasAuth || !usageData.value)) {
        fetchUsage()
      }
    }
  } catch (err) {
    status.value = {
      installed: false,
      binary_path: null,
      authenticated: false,
      active_account: null,
      available_models: [],
      error_message: String(err)
    }
    hasInitialCheckDone.value = true
  } finally {
    isChecking.value = false
  }
}

const handleStartLogin = async () => {
  isLaunchingLogin.value = true
  try {
    await invoke('start_agy_login')
  } catch (err) {
    console.error('Falha ao abrir terminal:', err)
  } finally {
    setTimeout(() => {
      isLaunchingLogin.value = false
    }, 1500)
  }
}

onMounted(async () => {
  try {
    const pInfo = await invoke('get_platform_info') as any
    if (pInfo && pInfo.os) {
      detectedOs.value = pInfo.os
      selectedOs.value = pInfo.os
    }
  } catch (e) {
    const ua = navigator.userAgent.toLowerCase()
    if (ua.includes('mac')) {
      detectedOs.value = 'macos'
      selectedOs.value = 'macos'
    } else if (ua.includes('win')) {
      detectedOs.value = 'windows'
      selectedOs.value = 'windows'
    } else {
      detectedOs.value = 'linux'
      selectedOs.value = 'linux'
    }
  }

  checkStatus()
  // Se já estiver autenticado via cache, sincroniza cota em segundo plano
  if (status.value.authenticated) {
    fetchUsage()
  }

  // Poll every 3s to detect login if user is authenticating
  pollingInterval = setInterval(() => {
    if (!status.value.authenticated) {
      checkStatus()
    }
  }, 3000)
})

onUnmounted(() => {
  if (pollingInterval) {
    clearInterval(pollingInterval)
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
