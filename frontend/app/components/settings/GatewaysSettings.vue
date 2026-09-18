<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between gap-4">
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
          <Radio class="w-4 h-4 text-indigo-400 flex-shrink-0" />
          <span class="truncate">{{ $t('settings.gateways.title') }}</span>
        </h3>
        <p class="text-xs text-slate-400 mt-0.5">
          {{ $t('settings.gateways.subtitle') }}
        </p>
      </div>

      <div class="flex items-center gap-2 flex-shrink-0">
        <button
          type="button"
          @click="saveGateways"
          :disabled="isSaving"
          :class="[
            'px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95',
            isSaved
              ? 'bg-emerald-600 text-white shadow-emerald-600/30'
              : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/30 disabled:opacity-50'
          ]"
        >
          <component :is="isSaved ? Check : Save" class="w-3.5 h-3.5" />
          <span>{{ isSaved ? $t('common.saved') : $t('common.save') }}</span>
        </button>
      </div>
    </div>

    <!-- Status Strip -->
    <div class="flex items-center justify-between px-3.5 py-2.5 rounded-2xl bg-[#0e101a] border border-[#1b2033] text-xs text-slate-400 flex-wrap gap-2 shadow-sm">
      <div class="flex items-center gap-2.5 flex-wrap">
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full" :class="localConfig.close_to_tray ? 'bg-emerald-400 animate-pulse' : 'bg-slate-600'" />
          <span class="text-slate-200 font-medium">{{ $t('settings.gateways.tray_status') }}:</span>
          <span :class="localConfig.close_to_tray ? 'text-emerald-400 font-semibold' : 'text-slate-500'">
            {{ localConfig.close_to_tray ? $t('settings.gateways.tray_active') : $t('common.disabled') }}
          </span>
        </div>
        <span class="text-slate-600">•</span>
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full" :class="getGatewayBadgeClass('telegram')" />
          <span class="text-slate-200 font-medium">Telegram:</span>
          <span :class="getGatewayTextClass('telegram')">
            {{ getGatewayStatusText('telegram') }}
          </span>
        </div>
        <span class="text-slate-600">•</span>
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full" :class="getGatewayBadgeClass('discord')" />
          <span class="text-slate-200 font-medium">Discord:</span>
          <span :class="getGatewayTextClass('discord')">
            {{ getGatewayStatusText('discord') }}
          </span>
        </div>
      </div>

      <button
        type="button"
        @click="refreshStatuses"
        class="text-[11px] text-slate-400 hover:text-indigo-300 flex items-center gap-1 cursor-pointer transition-colors"
      >
        <RefreshCw class="w-3 h-3" :class="{ 'animate-spin': isRefreshing }" />
        <span>{{ $t('common.refresh') }}</span>
      </button>
    </div>

    <!-- 1. Background Execution & System Tray Card -->
    <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
      <div class="p-4 bg-[#141828]/50 flex items-center gap-2.5">
        <div class="w-7 h-7 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
          <Monitor class="w-4 h-4" />
        </div>
        <div>
          <h4 class="text-xs font-bold text-slate-200">{{ $t('settings.gateways.system_tray_card_title') }}</h4>
          <p class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.gateways.system_tray_card_desc') }}</p>
        </div>
      </div>

      <!-- Close to Tray toggle -->
      <div class="p-4 flex items-center justify-between">
        <div class="pr-4">
          <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.close_to_tray_title') }}</span>
          <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
            {{ $t('settings.gateways.close_to_tray_desc') }}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          :aria-checked="localConfig.close_to_tray"
          @click="localConfig.close_to_tray = !localConfig.close_to_tray"
          :class="[
            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
            localConfig.close_to_tray ? 'bg-indigo-600' : 'bg-slate-700'
          ]"
        >
          <span
            :class="[
              'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out',
              localConfig.close_to_tray ? 'translate-x-5' : 'translate-x-0'
            ]"
          />
        </button>
      </div>

      <!-- Run in Background toggle -->
      <div class="p-4 flex items-center justify-between">
        <div class="pr-4">
          <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.run_in_background_title') }}</span>
          <p class="text-[11px] text-slate-400 mt-0.5 leading-relaxed">
            {{ $t('settings.gateways.run_in_background_desc') }}
          </p>
        </div>
        <button
          type="button"
          role="switch"
          :aria-checked="localConfig.run_in_background"
          @click="localConfig.run_in_background = !localConfig.run_in_background"
          :class="[
            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
            localConfig.run_in_background ? 'bg-indigo-600' : 'bg-slate-700'
          ]"
        >
          <span
            :class="[
              'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out',
              localConfig.run_in_background ? 'translate-x-5' : 'translate-x-0'
            ]"
          />
        </button>
      </div>
    </div>

    <!-- 2. Telegram Bot Gateway Card -->
    <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
      <div class="p-4 bg-[#141828]/50 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-lg bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
            <Send class="w-4 h-4" />
          </div>
          <div>
            <h4 class="text-xs font-bold text-slate-200 flex items-center gap-2">
              <span>{{ $t('settings.gateways.telegram_card_title') }}</span>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-medium" :class="getGatewayBadgeBg('telegram')">
                {{ getGatewayStatusText('telegram') }}
              </span>
            </h4>
            <p class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.gateways.telegram_card_desc') }}</p>
          </div>
        </div>

        <!-- Toggle Enable Telegram -->
        <button
          type="button"
          role="switch"
          :aria-checked="localConfig.gateways.telegram.enabled"
          @click="localConfig.gateways.telegram.enabled = !localConfig.gateways.telegram.enabled"
          :class="[
            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
            localConfig.gateways.telegram.enabled ? 'bg-sky-600' : 'bg-slate-700'
          ]"
        >
          <span
            :class="[
              'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out',
              localConfig.gateways.telegram.enabled ? 'translate-x-5' : 'translate-x-0'
            ]"
          />
        </button>
      </div>

      <div v-if="localConfig.gateways.telegram.enabled" class="p-4 space-y-4">
        <!-- Bot Token Input -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <label class="text-xs font-semibold text-slate-300">
              {{ $t('settings.gateways.telegram_token_label') }}
            </label>
            <button
              type="button"
              @click="showTelegramTutorial = !showTelegramTutorial"
              class="text-[11px] text-sky-400 hover:text-sky-300 flex items-center gap-1 cursor-pointer transition-colors"
            >
              <HelpCircle class="w-3.5 h-3.5" />
              <span>{{ $t('settings.gateways.how_to_get_token') }}</span>
              <component :is="showTelegramTutorial ? ChevronUp : ChevronDown" class="w-3 h-3" />
            </button>
          </div>

          <!-- Telegram Tutorial Box -->
          <div
            v-if="showTelegramTutorial"
            class="mb-3 p-3.5 rounded-xl bg-[#0b0d18] border border-sky-500/30 text-xs text-slate-300 space-y-2.5 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <span class="font-bold text-sky-300 text-xs flex items-center gap-1.5">
                <Send class="w-3.5 h-3.5" />
                <span>{{ $t('settings.gateways.how_to_get_token') }}</span>
              </span>
              <button
                type="button"
                @click="openLink('https://t.me/BotFather')"
                class="text-[11px] text-sky-400 hover:underline flex items-center gap-1 cursor-pointer font-medium"
              >
                <span>{{ $t('settings.gateways.open_botfather') }}</span>
                <ExternalLink class="w-3 h-3" />
              </button>
            </div>
            <ol class="space-y-1.5 text-[11px] text-slate-400 leading-relaxed list-none">
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-sky-500/10 text-sky-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">1</span>
                <span>{{ $t('settings.gateways.telegram_tutorial_step1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-sky-500/10 text-sky-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">2</span>
                <span>{{ $t('settings.gateways.telegram_tutorial_step2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-sky-500/10 text-sky-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">3</span>
                <span>{{ $t('settings.gateways.telegram_tutorial_step3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-sky-500/10 text-sky-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">4</span>
                <span>{{ $t('settings.gateways.telegram_tutorial_step4') }}</span>
              </li>
            </ol>
          </div>

          <div class="flex items-center gap-2">
            <div class="relative flex-1">
              <input
                :type="showTelegramToken ? 'text' : 'password'"
                v-model="localConfig.gateways.telegram.bot_token"
                :placeholder="$t('settings.gateways.telegram_token_placeholder')"
                class="w-full px-3 py-2 pr-10 rounded-xl bg-[#0b0d16] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-sky-500 font-mono transition-colors"
              />
              <button
                type="button"
                @click="showTelegramToken = !showTelegramToken"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showTelegramToken ? EyeOff : Eye" class="w-3.5 h-3.5" />
              </button>
            </div>

            <button
              type="button"
              @click="testTelegram"
              :disabled="isTestingTelegram || !localConfig.gateways.telegram.bot_token.trim()"
              class="px-3.5 py-2 rounded-xl text-xs font-semibold bg-[#181c2e] hover:bg-[#20263f] text-sky-400 border border-[#27304e] flex items-center gap-1.5 transition-all cursor-pointer disabled:opacity-40"
            >
              <component :is="isTestingTelegram ? Loader2 : CheckCircle2" class="w-3.5 h-3.5" :class="{ 'animate-spin': isTestingTelegram }" />
              <span>{{ $t('settings.gateways.test_connection') }}</span>
            </button>
          </div>
          <p v-if="telegramTestFeedback" :class="['text-[11px] mt-1.5 font-medium', telegramTestSuccess ? 'text-emerald-400' : 'text-rose-400']">
            {{ telegramTestFeedback }}
          </p>
        </div>

        <!-- Allowed Users (Whitelist Guardrail) -->
        <div>
          <label class="text-xs font-semibold text-slate-300 block mb-1">
            {{ $t('settings.gateways.allowed_users_label') }}
          </label>
          <p class="text-[11px] text-slate-400 mb-2">
            {{ $t('settings.gateways.allowed_users_desc') }}
          </p>
          <input
            type="text"
            :value="telegramUsersInput"
            @input="updateTelegramUsers(($event.target as HTMLInputElement).value)"
            :placeholder="$t('settings.gateways.allowed_users_placeholder')"
            class="w-full px-3 py-2 rounded-xl bg-[#0b0d16] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-sky-500 font-mono transition-colors"
          />
        </div>

        <!-- Sliding Window Context Control -->
        <div class="p-3.5 rounded-xl bg-[#0b0d16] border border-[#1d2235] space-y-2">
          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.sliding_window_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.sliding_window_desc') }}</span>
            </div>
            <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#141828] border border-[#22283b]">
              <span class="text-xs font-mono font-bold text-sky-400">{{ localConfig.gateways.telegram.sliding_window || 4 }}</span>
              <span class="text-[10px] text-slate-500">{{ $t('settings.gateways.sliding_window_unit') }}</span>
            </div>
          </div>
          <div class="flex items-center gap-3 pt-1">
            <span class="text-[10px] font-mono text-slate-500">2</span>
            <input
              type="range"
              min="2"
              max="20"
              step="2"
              v-model.number="localConfig.gateways.telegram.sliding_window"
              class="flex-1 h-1.5 bg-[#1a1f33] rounded-lg appearance-none cursor-pointer accent-sky-500"
            />
            <span class="text-[10px] font-mono text-slate-500">20</span>
          </div>
        </div>

        <!-- Cognitive Memory & Tools Options -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
          <label class="p-3 rounded-xl bg-[#0b0d16] border border-[#1d2235] flex items-center gap-3 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="localConfig.gateways.telegram.enable_memory"
              class="rounded border-[#262c42] bg-[#141828] text-indigo-500 focus:ring-indigo-500/20"
            />
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.enable_memory_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.enable_memory_desc') }}</span>
            </div>
          </label>

          <label class="p-3 rounded-xl bg-[#0b0d16] border border-[#1d2235] flex items-center gap-3 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="localConfig.gateways.telegram.enable_tools"
              class="rounded border-[#262c42] bg-[#141828] text-indigo-500 focus:ring-indigo-500/20"
            />
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.enable_tools_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.enable_tools_desc') }}</span>
            </div>
          </label>
        </div>
      </div>
    </div>

    <!-- 3. Discord Bot Gateway Card -->
    <div class="rounded-2xl bg-[#111420] border border-[#1e2336] overflow-hidden shadow-sm divide-y divide-[#1e2336]">
      <div class="p-4 bg-[#141828]/50 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <MessageSquareShare class="w-4 h-4" />
          </div>
          <div>
            <h4 class="text-xs font-bold text-slate-200 flex items-center gap-2">
              <span>{{ $t('settings.gateways.discord_card_title') }}</span>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-medium" :class="getGatewayBadgeBg('discord')">
                {{ getGatewayStatusText('discord') }}
              </span>
            </h4>
            <p class="text-[11px] text-slate-400 mt-0.5">{{ $t('settings.gateways.discord_card_desc') }}</p>
          </div>
        </div>

        <!-- Toggle Enable Discord -->
        <button
          type="button"
          role="switch"
          :aria-checked="localConfig.gateways.discord.enabled"
          @click="localConfig.gateways.discord.enabled = !localConfig.gateways.discord.enabled"
          :class="[
            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
            localConfig.gateways.discord.enabled ? 'bg-indigo-600' : 'bg-slate-700'
          ]"
        >
          <span
            :class="[
              'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out',
              localConfig.gateways.discord.enabled ? 'translate-x-5' : 'translate-x-0'
            ]"
          />
        </button>
      </div>

      <div v-if="localConfig.gateways.discord.enabled" class="p-4 space-y-4">
        <!-- Discord Bot Token Input -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <label class="text-xs font-semibold text-slate-300">
              {{ $t('settings.gateways.discord_token_label') }}
            </label>
            <button
              type="button"
              @click="showDiscordTutorial = !showDiscordTutorial"
              class="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 cursor-pointer transition-colors"
            >
              <HelpCircle class="w-3.5 h-3.5" />
              <span>{{ $t('settings.gateways.how_to_get_token') }}</span>
              <component :is="showDiscordTutorial ? ChevronUp : ChevronDown" class="w-3 h-3" />
            </button>
          </div>

          <!-- Discord Tutorial Box -->
          <div
            v-if="showDiscordTutorial"
            class="mb-3 p-3.5 rounded-xl bg-[#0b0d18] border border-indigo-500/30 text-xs text-slate-300 space-y-2.5 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <span class="font-bold text-indigo-300 text-xs flex items-center gap-1.5">
                <MessageSquareShare class="w-3.5 h-3.5" />
                <span>{{ $t('settings.gateways.how_to_get_token') }}</span>
              </span>
              <button
                type="button"
                @click="openLink('https://discord.com/developers/applications')"
                class="text-[11px] text-indigo-400 hover:underline flex items-center gap-1 cursor-pointer font-medium"
              >
                <span>{{ $t('settings.gateways.open_discord_portal') }}</span>
                <ExternalLink class="w-3 h-3" />
              </button>
            </div>
            <ol class="space-y-1.5 text-[11px] text-slate-400 leading-relaxed list-none">
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">1</span>
                <span>{{ $t('settings.gateways.discord_tutorial_step1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">2</span>
                <span>{{ $t('settings.gateways.discord_tutorial_step2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">3</span>
                <span>{{ $t('settings.gateways.discord_tutorial_step3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">4</span>
                <span class="font-medium text-amber-300/90">{{ $t('settings.gateways.discord_tutorial_step4') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="w-4 h-4 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center font-bold text-[10px] shrink-0 mt-0.5">5</span>
                <span>{{ $t('settings.gateways.discord_tutorial_step5') }}</span>
              </li>
            </ol>
          </div>

          <div class="flex items-center gap-2">
            <div class="relative flex-1">
              <input
                :type="showDiscordToken ? 'text' : 'password'"
                v-model="localConfig.gateways.discord.bot_token"
                :placeholder="$t('settings.gateways.discord_token_placeholder')"
                class="w-full px-3 py-2 pr-10 rounded-xl bg-[#0b0d16] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 font-mono transition-colors"
              />
              <button
                type="button"
                @click="showDiscordToken = !showDiscordToken"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 cursor-pointer"
              >
                <component :is="showDiscordToken ? EyeOff : Eye" class="w-3.5 h-3.5" />
              </button>
            </div>

            <button
              type="button"
              @click="testDiscord"
              :disabled="isTestingDiscord || !localConfig.gateways.discord.bot_token.trim()"
              class="px-3.5 py-2 rounded-xl text-xs font-semibold bg-[#181c2e] hover:bg-[#20263f] text-indigo-400 border border-[#27304e] flex items-center gap-1.5 transition-all cursor-pointer disabled:opacity-40"
            >
              <component :is="isTestingDiscord ? Loader2 : CheckCircle2" class="w-3.5 h-3.5" :class="{ 'animate-spin': isTestingDiscord }" />
              <span>{{ $t('settings.gateways.test_connection') }}</span>
            </button>
          </div>
          <p v-if="discordTestFeedback" :class="['text-[11px] mt-1.5 font-medium', discordTestSuccess ? 'text-emerald-400' : 'text-rose-400']">
            {{ discordTestFeedback }}
          </p>
        </div>

        <!-- Allowed Channels & Users -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div>
            <label class="text-xs font-semibold text-slate-300 block mb-1">
              {{ $t('settings.gateways.discord_channels_label') }}
            </label>
            <input
              type="text"
              :value="discordChannelsInput"
              @input="updateDiscordChannels(($event.target as HTMLInputElement).value)"
              :placeholder="$t('settings.gateways.discord_channels_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#0b0d16] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 font-mono transition-colors"
            />
          </div>

          <div>
            <label class="text-xs font-semibold text-slate-300 block mb-1">
              {{ $t('settings.gateways.discord_users_label') }}
            </label>
            <input
              type="text"
              :value="discordUsersInput"
              @input="updateDiscordUsers(($event.target as HTMLInputElement).value)"
              :placeholder="$t('settings.gateways.discord_users_placeholder')"
              class="w-full px-3 py-2 rounded-xl bg-[#0b0d16] border border-[#22283b] text-xs text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 font-mono transition-colors"
            />
          </div>
        </div>

        <!-- Sliding Window Context Control -->
        <div class="p-3.5 rounded-xl bg-[#0b0d16] border border-[#1d2235] space-y-2">
          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.sliding_window_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.sliding_window_desc') }}</span>
            </div>
            <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#141828] border border-[#22283b]">
              <span class="text-xs font-mono font-bold text-indigo-400">{{ localConfig.gateways.discord.sliding_window || 4 }}</span>
              <span class="text-[10px] text-slate-500">{{ $t('settings.gateways.sliding_window_unit') }}</span>
            </div>
          </div>
          <div class="flex items-center gap-3 pt-1">
            <span class="text-[10px] font-mono text-slate-500">2</span>
            <input
              type="range"
              min="2"
              max="20"
              step="2"
              v-model.number="localConfig.gateways.discord.sliding_window"
              class="flex-1 h-1.5 bg-[#1a1f33] rounded-lg appearance-none cursor-pointer accent-indigo-500"
            />
            <span class="text-[10px] font-mono text-slate-500">20</span>
          </div>
        </div>

        <!-- Cognitive Memory & Tools Options -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
          <label class="p-3 rounded-xl bg-[#0b0d16] border border-[#1d2235] flex items-center gap-3 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="localConfig.gateways.discord.enable_memory"
              class="rounded border-[#262c42] bg-[#141828] text-indigo-500 focus:ring-indigo-500/20"
            />
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.enable_memory_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.enable_memory_desc') }}</span>
            </div>
          </label>

          <label class="p-3 rounded-xl bg-[#0b0d16] border border-[#1d2235] flex items-center gap-3 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="localConfig.gateways.discord.enable_tools"
              class="rounded border-[#262c42] bg-[#141828] text-indigo-500 focus:ring-indigo-500/20"
            />
            <div>
              <span class="text-xs font-bold text-slate-200 block">{{ $t('settings.gateways.enable_tools_title') }}</span>
              <span class="text-[10px] text-slate-400">{{ $t('settings.gateways.enable_tools_desc') }}</span>
            </div>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  Radio,
  Save,
  Check,
  RefreshCw,
  Monitor,
  Send,
  MessageSquareShare,
  Eye,
  EyeOff,
  CheckCircle2,
  Loader2,
  HelpCircle,
  ExternalLink,
  ChevronDown,
  ChevronUp
} from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

interface GatewayReport {
  id: string
  name: string
  enabled: boolean
  status: string
  error?: string | null
}

const props = defineProps<{
  config: any
}>()

const emit = defineEmits<{
  (e: 'saveConfig'): void
}>()

const localConfig = ref(JSON.parse(JSON.stringify(props.config || {})))

// Ensure defaults
if (localConfig.value.run_in_background === undefined) localConfig.value.run_in_background = true
if (localConfig.value.close_to_tray === undefined) localConfig.value.close_to_tray = true
if (!localConfig.value.gateways) {
  localConfig.value.gateways = {
    telegram: { enabled: false, bot_token: '', allowed_user_ids: [], enable_memory: true, enable_tools: false, sliding_window: 4 },
    discord: { enabled: false, bot_token: '', allowed_user_ids: [], allowed_channel_ids: [], enable_memory: true, enable_tools: false, sliding_window: 4 }
  }
}
if (!localConfig.value.gateways.telegram.sliding_window) {
  localConfig.value.gateways.telegram.sliding_window = 4
}
if (!localConfig.value.gateways.discord.sliding_window) {
  localConfig.value.gateways.discord.sliding_window = 4
}

const isSaved = ref(false)
const isSaving = ref(false)
const isRefreshing = ref(false)

const showTelegramTutorial = ref(false)
const showTelegramToken = ref(false)
const isTestingTelegram = ref(false)
const telegramTestFeedback = ref('')
const telegramTestSuccess = ref(false)

const showDiscordTutorial = ref(false)
const showDiscordToken = ref(false)
const isTestingDiscord = ref(false)
const discordTestFeedback = ref('')
const discordTestSuccess = ref(false)

const gatewayStatuses = ref<GatewayReport[]>([])

async function openLink(url: string) {
  try {
    await invoke('open_url', { url })
  } catch (err) {
    if (typeof window !== 'undefined') {
      window.open(url, '_blank')
    }
  }
}

const telegramUsersInput = computed(() => {
  return (localConfig.value.gateways?.telegram?.allowed_user_ids || []).join(', ')
})

function updateTelegramUsers(val: string) {
  const ids = val
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)
    .map(s => parseInt(s, 10))
    .filter(n => !isNaN(n))
  localConfig.value.gateways.telegram.allowed_user_ids = ids
}

const discordUsersInput = computed(() => {
  return (localConfig.value.gateways?.discord?.allowed_user_ids || []).join(', ')
})

function updateDiscordUsers(val: string) {
  const ids = val
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)
  localConfig.value.gateways.discord.allowed_user_ids = ids
}

const discordChannelsInput = computed(() => {
  return (localConfig.value.gateways?.discord?.allowed_channel_ids || []).join(', ')
})

function updateDiscordChannels(val: string) {
  const ids = val
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)
  localConfig.value.gateways.discord.allowed_channel_ids = ids
}

async function refreshStatuses() {
  isRefreshing.value = true
  try {
    const res = await invoke<GatewayReport[]>('get_gateways_status')
    if (res) {
      gatewayStatuses.value = res
    }
  } catch (e) {
    console.error('Failed to get gateways status:', e)
  } finally {
    isRefreshing.value = false
  }
}

function getGatewayStatus(id: string): string {
  const gw = gatewayStatuses.value.find(g => g.id === id)
  if (!gw) return 'offline'
  return gw.status
}

function getGatewayBadgeClass(id: string) {
  const st = getGatewayStatus(id)
  if (st === 'connected') return 'bg-emerald-400 animate-pulse'
  if (st === 'connecting') return 'bg-amber-400 animate-pulse'
  if (st === 'error') return 'bg-rose-400'
  return 'bg-slate-600'
}

function getGatewayBadgeBg(id: string) {
  const st = getGatewayStatus(id)
  if (st === 'connected') return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
  if (st === 'connecting') return 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
  if (st === 'error') return 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
  return 'bg-slate-800 text-slate-400 border border-slate-700'
}

function getGatewayTextClass(id: string) {
  const st = getGatewayStatus(id)
  if (st === 'connected') return 'text-emerald-400 font-semibold'
  if (st === 'connecting') return 'text-amber-400 font-semibold'
  if (st === 'error') return 'text-rose-400 font-semibold'
  return 'text-slate-500'
}

function getGatewayStatusText(id: string): string {
  const st = getGatewayStatus(id)
  if (st === 'connected') return 'Connected'
  if (st === 'connecting') return 'Connecting...'
  if (st === 'error') return 'Error'
  return 'Offline'
}

async function testTelegram() {
  isTestingTelegram.value = true
  telegramTestFeedback.value = ''
  try {
    const res = await invoke<string>('test_telegram_connection', {
      token: localConfig.value.gateways.telegram.bot_token
    })
    telegramTestSuccess.value = true
    telegramTestFeedback.value = `Verified: ${res}`
  } catch (e: any) {
    telegramTestSuccess.value = false
    telegramTestFeedback.value = `Error: ${e?.message || e}`
  } finally {
    isTestingTelegram.value = false
  }
}

async function testDiscord() {
  isTestingDiscord.value = true
  discordTestFeedback.value = ''
  try {
    const res = await invoke<string>('test_discord_connection', {
      token: localConfig.value.gateways.discord.bot_token
    })
    discordTestSuccess.value = true
    discordTestFeedback.value = `Verified: ${res}`
  } catch (e: any) {
    discordTestSuccess.value = false
    discordTestFeedback.value = `Error: ${e?.message || e}`
  } finally {
    isTestingDiscord.value = false
  }
}

async function saveGateways() {
  isSaving.value = true
  try {
    await invoke('save_app_config', { config: localConfig.value })
    if (props.config) {
      Object.assign(props.config, localConfig.value)
    }
    emit('saveConfig')
    isSaved.value = true
    setTimeout(() => {
      isSaved.value = false
    }, 2500)
    await refreshStatuses()
  } catch (e) {
    console.error('Failed to save gateway config:', e)
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  refreshStatuses()
})
</script>
