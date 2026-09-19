<template>
  <div :class="['w-full py-2 px-1 sm:px-2 flex flex-col chat-message-enter', isUser ? 'items-end' : 'items-start']">
    <!-- Bubble Meta Header -->
    <div :class="['flex items-center gap-2 mb-1.5 text-xs text-slate-400 select-none flex-wrap', isUser ? 'flex-row-reverse' : 'flex-row']">
      <div class="flex items-center gap-1.5">
        <span
          :class="[
            'w-2 h-2 rounded-full',
            isUser ? 'bg-indigo-400' : 'bg-purple-400'
          ]"
        ></span>
        <span :class="['font-semibold text-xs', isUser ? 'text-indigo-300' : 'text-purple-300']">
          {{ isUser ? $t('chat.you') : 'Atena Studio' }}
        </span>
      </div>

      <span class="text-[10px] font-mono text-slate-500">{{ formattedTime }}</span>

      <!-- Metrics for Assistant (oculto quando é erro de modelo não selecionado) -->
      <template v-if="!isUser && showEfficiencyMetrics !== false && !isModelMissingError">
        <span
          v-if="message.metrics && message.metrics.cache_efficiency_pct !== undefined && message.metrics.cache_efficiency_pct > 0"
          class="text-teal-300 font-mono text-[10.5px] px-1.5 py-0.5 rounded bg-teal-500/15 border border-teal-500/25 flex items-center gap-1 cursor-help"
          :title="$t('chat.kv_cache_tooltip', { cached: message.metrics.cached_tokens, prefill: message.metrics.prefill_tokens, prompt: message.metrics.prompt_tokens })"
        >
          <Zap class="w-2.5 h-2.5 text-teal-400" />
          <span>{{ message.metrics.cache_efficiency_pct.toFixed(0) }}% Cache</span>
        </span>
        <span v-if="message.generation_speed_tps || (message.metrics && message.metrics.generation_speed_tps)" class="text-emerald-400 font-mono text-[10.5px] px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20">
          {{ Number((message.metrics && message.metrics.generation_speed_tps) || message.generation_speed_tps).toFixed(1) }} t/s
        </span>
        <span
          v-if="message.tokens_count || (message.metrics && message.metrics.completion_tokens)"
          :class="[
            'font-mono text-[10.5px]',
            isTokenLimitReached
              ? 'text-amber-400 font-semibold px-1.5 py-0.5 rounded bg-amber-500/15 border border-amber-500/30'
              : 'text-slate-400'
          ]"
          :title="isTokenLimitReached ? $t('chat.token_limit_reached') : ''"
        >
          <span v-if="!isTokenLimitReached">• </span>
          <span>{{ (message.metrics && message.metrics.completion_tokens) || message.tokens_count }} tok</span>
          <span v-if="isTokenLimitReached" class="font-sans ml-1 text-[9.5px] uppercase font-bold tracking-wider text-amber-300">{{ $t('chat.limit') }}</span>
        </span>
      </template>

      <!-- Text-to-Speech (TTS) Action for Assistant (apenas para respostas válidas) -->
      <button
        v-if="!isUser && message.content && !message.is_streaming && !isModelMissingError"
        @click="handleTtsClick"
        class="px-2 py-0.5 rounded-lg text-[10.5px] transition-all flex items-center gap-1 border cursor-pointer shadow-sm active:scale-95"
        :class="[
          isThisSpeaking
            ? 'bg-purple-600/25 text-purple-300 border-purple-500/50 hover:bg-purple-600/35 shadow-purple-500/10'
            : isThisPaused
            ? 'bg-amber-600/25 text-amber-300 border-amber-500/50 hover:bg-amber-600/35'
            : 'bg-[#141826] hover:bg-[#1c2236] text-slate-400 hover:text-slate-200 border-[#22283b]'
        ]"
        :title="isThisSpeaking ? $t('chat.tts_pause_title') : (isThisPaused ? $t('chat.tts_resume_title') : $t('chat.tts_listen_title'))"
      >
        <Volume2 v-if="!isThisSpeaking && !isThisPaused" class="w-3 h-3 text-purple-400" />
        <VolumeX v-else-if="isThisPaused" class="w-3 h-3 text-amber-400" />
        <div v-else class="flex items-center gap-0.5">
          <span class="w-1 h-2.5 bg-purple-400 rounded-full animate-bounce"></span>
          <span class="w-1 h-3.5 bg-purple-300 rounded-full animate-bounce [animation-delay:0.15s]"></span>
          <span class="w-1 h-2 bg-purple-400 rounded-full animate-bounce [animation-delay:0.3s]"></span>
        </div>
        <span>{{ isThisSpeaking ? $t('chat.tts_listening') : (isThisPaused ? $t('chat.tts_paused') : $t('chat.tts_listen')) }}</span>
      </button>
      
      <!-- Copy Action -->
      <button
        @click="copyText(message.content)"
        class="px-2 py-0.5 rounded-lg bg-[#141826] hover:bg-[#1c2236] text-[10.5px] text-slate-400 hover:text-slate-200 transition-all flex items-center gap-1 border border-[#22283b] cursor-pointer shadow-sm active:scale-95"
        :title="$t('chat.copy_message')"
      >
        <component :is="copied ? Check : Copy" class="w-3 h-3" />
        <span>{{ copied ? $t('chat.code_copied') : $t('common.copy') }}</span>
      </button>

      <!-- Delete Action -->
      <button
        @click="handleDeleteClick"
        :class="[
          'px-2 py-0.5 rounded-lg text-[10.5px] transition-all flex items-center gap-1 border cursor-pointer shadow-sm active:scale-95',
          confirmingDelete
            ? 'bg-rose-600 text-white border-rose-500 hover:bg-rose-700 animate-pulse'
            : 'bg-[#141826] hover:bg-rose-500/15 text-slate-400 hover:text-rose-300 border-[#22283b] hover:border-rose-500/30'
        ]"
        :title="confirmingDelete ? $t('chat.confirm_delete_message') : $t('chat.delete_message')"
      >
        <Trash2 class="w-3 h-3" />
        <span>{{ confirmingDelete ? $t('chat.delete_confirm') : $t('common.delete') }}</span>
      </button>
    </div>

    <!-- Bubble Card Container -->
    <div
      :class="[
        'w-auto rounded-2xl p-3.5 sm:p-4.5 transition-all text-sm leading-relaxed select-text cursor-text',
        isUser
          ? 'max-w-[88%] sm:max-w-[80%] md:max-w-[75%] bg-indigo-600 bg-gradient-to-br from-indigo-600 to-indigo-700 text-white rounded-tr-sm shadow-lg shadow-indigo-600/10 border border-indigo-500/30'
          : 'w-full max-w-full bg-[#111420] border border-[#1c2030] text-slate-100 rounded-tl-sm shadow-sm'
      ]"
    >
      <!-- User Attached Images Gallery -->
      <div v-if="isUser && message.images && message.images.length > 0" class="flex flex-wrap gap-2 mb-3">
        <div
          v-for="(img, idx) in message.images"
          :key="idx"
          class="relative group rounded-xl overflow-hidden border border-white/20 shadow-md max-w-[240px] max-h-[180px]"
        >
          <img
            :src="img"
            :alt="$t('chat.attached_image')"
            class="w-full h-full object-cover cursor-pointer hover:scale-105 transition-transform"
          />
        </div>
      </div>

      <!-- User Attached Documents & Audio Cards (Rich UI) -->
      <div v-if="isUser && message.attachments && message.attachments.length > 0" class="space-y-2 mb-3">
        <div
          v-for="(att, idx) in message.attachments.filter(a => !a.is_image)"
          :key="att.id || idx"
          class="rounded-xl bg-black/30 border border-white/15 overflow-hidden text-xs text-slate-200 backdrop-blur-sm shadow-sm"
        >
          <!-- Attachment Summary Row -->
          <div class="p-2.5 flex items-center justify-between gap-3">
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- XML Badge -->
              <div
                v-if="att.kind === 'xml' || (att.name && att.name.toLowerCase().endsWith('.xml'))"
                class="attachment-badge w-8 h-7 rounded-lg bg-amber-500/25 border border-amber-400/40 flex items-center justify-center text-amber-200 font-bold text-[9px] flex-shrink-0 tracking-wider shadow-inner"
              >
                XML
              </div>
              <!-- JSON Badge -->
              <div
                v-else-if="att.kind === 'json' || (att.name && att.name.toLowerCase().endsWith('.json'))"
                class="attachment-badge w-8 h-7 rounded-lg bg-emerald-500/25 border border-emerald-400/40 flex items-center justify-center text-emerald-200 font-bold text-[9px] flex-shrink-0 tracking-wider shadow-inner"
              >
                JSON
              </div>
              <!-- CSV Badge -->
              <div
                v-else-if="att.kind === 'csv' || (att.name && (att.name.toLowerCase().endsWith('.csv') || att.name.toLowerCase().endsWith('.tsv')))"
                class="attachment-badge w-8 h-7 rounded-lg bg-teal-500/25 border border-teal-400/40 flex items-center justify-center text-teal-200 font-bold text-[9px] flex-shrink-0 tracking-wider shadow-inner"
              >
                CSV
              </div>
              <!-- PDF Badge -->
              <div
                v-else-if="att.kind === 'pdf' || (att.name && att.name.toLowerCase().endsWith('.pdf'))"
                class="attachment-badge w-7 h-7 rounded-lg bg-rose-500/25 border border-rose-400/40 flex items-center justify-center text-rose-200 font-bold text-[9.5px] flex-shrink-0 tracking-wider shadow-inner"
              >
                PDF
              </div>
              <!-- Audio Badge -->
              <div
                v-else-if="att.kind === 'audio' || isAudioExt(att.name)"
                class="attachment-badge w-7 h-7 rounded-lg bg-purple-500/25 border border-purple-400/40 flex items-center justify-center text-purple-200 flex-shrink-0 shadow-inner"
              >
                <Mic class="w-3.5 h-3.5" />
              </div>
              <!-- Code / Text Badge -->
              <div
                v-else
                class="attachment-badge w-7 h-7 rounded-lg bg-indigo-500/25 border border-indigo-400/40 flex items-center justify-center text-indigo-200 flex-shrink-0 shadow-inner"
              >
                <FileCode2 class="w-3.5 h-3.5" />
              </div>

              <div class="min-w-0">
                <p class="font-semibold text-xs text-white truncate leading-tight">{{ att.name }}</p>
                <div class="flex items-center gap-2 text-[10px] text-indigo-200/80 font-mono mt-0.5">
                  <span v-if="att.size_bytes">{{ formatBytes(att.size_bytes) }}</span>
                  <span v-if="att.pages">• {{ att.pages }} {{ $t('chat.pages') }}</span>
                  <span v-if="att.duration">• {{ formatDuration(att.duration) }}</span>
                </div>
              </div>
            </div>

            <!-- View content / transcript toggle button -->
            <button
              v-if="att.text"
              @click="toggleAttachmentContent(idx)"
              class="px-2.5 py-1 rounded-lg bg-white/10 hover:bg-white/20 text-[10.5px] text-indigo-100 flex items-center gap-1 font-medium transition-all cursor-pointer flex-shrink-0 active:scale-95"
            >
              <span>{{ expandedAttachments[idx] ? $t('chat.hide') : (att.kind === 'audio' ? $t('chat.view_transcription') : $t('chat.view_content')) }}</span>
              <component :is="expandedAttachments[idx] ? ChevronUp : ChevronDown" class="w-3 h-3" />
            </button>
          </div>

          <!-- Collapsible Extracted Content / Transcript Box -->
          <div
            v-if="expandedAttachments[idx] && att.text"
            class="px-3 pb-3 pt-1 border-t border-white/10 bg-black/25"
          >
            <p class="text-[10px] font-semibold text-indigo-300 uppercase tracking-wider mb-1.5">
              {{ att.kind === 'audio' ? $t('chat.audio_transcription_title') : (att.kind === 'xml' ? $t('chat.xml_content_title') : (att.kind === 'json' ? $t('chat.json_content_title') : $t('chat.document_content_title'))) }}
            </p>
            <pre class="attachment-content-pre p-2.5 rounded-lg bg-[#0b0e18]/95 border border-white/10 font-mono text-[11px] text-slate-200 max-h-48 overflow-y-auto whitespace-pre-wrap select-text leading-relaxed shadow-inner">{{ att.text }}</pre>
          </div>
        </div>
      </div>

      <!-- Thinking / Reasoning Block -->
      <div v-if="!isUser && message.thinking" class="mb-3">
        <!-- Toggle / Header Button -->
        <button
          @click="isThinkingExpanded = !isThinkingExpanded"
          class="flex items-center gap-2 py-1 px-3 rounded-xl bg-[#181326] hover:bg-[#201934] border border-[#3b2b5c] text-xs font-semibold text-purple-300 transition-all mb-1.5 select-none cursor-pointer shadow-inner"
        >
          <BrainCircuit class="w-3.5 h-3.5 text-purple-400" :class="{ 'animate-pulse': isActivelyThinking }" />
          <span>{{ isActivelyThinking ? $t('chat.thinking_realtime') : $t('chat.internal_reasoning') }}</span>
          <component :is="isThinkingExpanded ? ChevronDown : ChevronRight" class="w-3.5 h-3.5 text-purple-400" />
        </button>

        <!-- Live Streaming Preview (Auto-scrolling to bottom) -->
        <div
          v-if="isActivelyThinking && !isThinkingExpanded"
          ref="liveThinkingRef"
          class="p-3 bg-[#140f21] rounded-xl border border-[#3b2b5c] text-xs text-purple-300/90 font-mono select-text overflow-y-auto max-h-36 leading-relaxed whitespace-pre-wrap break-words scroll-smooth"
        >
          {{ message.thinking }}
        </div>

        <!-- Full Expanded Reasoning Content -->
        <div
          v-if="isThinkingExpanded"
          class="p-3.5 bg-[#140f21] rounded-xl border border-[#3b2b5c] text-xs text-purple-200/90 prose prose-invert max-w-none select-text overflow-x-auto max-h-96 overflow-y-auto font-sans leading-relaxed"
          @click="handleMessageBodyClick"
          v-html="renderMarkdown(message.thinking)"
        />
      </div>

      <!-- Main Message Content (Preamble / Explanation) -->
      <div
        v-if="renderedContent"
        class="prose prose-invert max-w-none select-text cursor-text leading-relaxed font-sans"
        :class="[
          isUser ? 'text-white' : 'text-slate-100',
          (!isUser && message.tool_calls && message.tool_calls.length > 0) ? 'mb-3' : ''
        ]"
        @click="handleMessageBodyClick"
        v-html="renderedContent"
      />

      <!-- Fallback when model only generated background memory tags -->
      <div
        v-else-if="!isUser && !message.is_streaming && (!message.tool_calls || message.tool_calls.length === 0) && hasMemoryTagsOnly"
        class="text-xs text-indigo-300/80 italic flex items-center gap-1.5 py-1 select-none"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
        <span>{{ $t('chat.memory_recorded_ack') }}</span>
      </div>

      <!-- Fallback when model finished thinking with no output -->
      <div
        v-else-if="!isUser && !message.is_streaming && (!message.tool_calls || message.tool_calls.length === 0) && message.thinking && !cleanContentText"
        class="text-xs text-slate-400/80 italic flex items-center gap-1.5 py-1 select-none"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-slate-500"></span>
        <span>{{ $t('chat.empty_response_after_thinking') }}</span>
      </div>

      <!-- Active Streaming Indicator (only when text is already rendered and still actively generating) -->
      <div
        v-if="!isUser && message.is_streaming && cleanContentText && (!message.tool_calls || message.tool_calls.length === 0)"
        class="flex items-center gap-2 mt-2 select-none text-xs text-indigo-400 dark:text-indigo-300 font-mono"
      >
        <span class="w-2 h-2 rounded-full bg-indigo-500 animate-ping"></span>
        <span>{{ $t('chat.generating_response') }}</span>
      </div>

      <!-- MCP Tool Call Interactive Cards (Grouped & Collapsible) -->
      <div v-if="!isUser && message.tool_calls && message.tool_calls.length > 0" class="mb-3 space-y-2">
        <!-- Group Header for multiple tools -->
        <div
          v-if="message.tool_calls.length > 1"
          class="sticky -top-[18px] z-20 p-2.5 rounded-xl bg-slate-100 dark:bg-[#141828] border border-slate-200 dark:border-[#23293f] text-xs select-none space-y-2 shadow-md transition-all"
        >
          <!-- Top Row: Stats & Global Expand/Collapse -->
          <div class="flex items-center justify-between gap-2 flex-wrap">
            <!-- Left: Title & Status Badges (clickable accordion when finished/no pending) -->
            <div
              class="flex items-center gap-2 flex-wrap"
              :class="pendingToolsCount === 0 ? 'cursor-pointer group' : ''"
              @click="pendingToolsCount === 0 && (isToolsListCollapsed = !isToolsListCollapsed)"
              :title="pendingToolsCount === 0 ? $t('chat.toggle_tools_list_title') : ''"
            >
              <button
                v-if="pendingToolsCount === 0"
                type="button"
                class="p-0.5 rounded-md hover:bg-slate-200 dark:hover:bg-white/10 text-slate-500 dark:text-slate-400 transition-colors cursor-pointer"
              >
                <component :is="isToolsListCollapsed ? ChevronRight : ChevronDown" class="w-4 h-4 text-indigo-500 dark:text-indigo-400 transition-transform" />
              </button>
              <Layers class="w-3.5 h-3.5 text-indigo-500 dark:text-indigo-400" />
              <span
                class="font-bold text-slate-800 dark:text-slate-100 text-[11.5px] transition-colors"
                :class="pendingToolsCount === 0 ? 'group-hover:text-indigo-500 dark:group-hover:text-indigo-300' : ''"
              >
                {{ $t('chat.tool_calls_header', { count: message.tool_calls.length }) }}
              </span>
              <span class="text-[10px] px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-500/20 font-mono font-medium">
                {{ $t('chat.tools_executed', { completed: completedToolsCount, total: message.tool_calls.length }) }}
              </span>
              <span v-if="pendingToolsCount > 0" class="text-[10px] px-2 py-0.5 rounded-full bg-amber-100 dark:bg-amber-500/15 text-amber-800 dark:text-amber-300 border border-amber-300 dark:border-amber-500/30 font-mono font-semibold animate-pulse">
                {{ $t('chat.tools_pending_count', { count: pendingToolsCount }) }}
              </span>
              <span v-if="rejectedToolsCount > 0" class="text-[10px] px-2 py-0.5 rounded-full bg-rose-50 dark:bg-rose-500/10 text-rose-700 dark:text-rose-300 border border-rose-200 dark:border-rose-500/20 font-mono">
                {{ $t('chat.tools_rejected_count', { count: rejectedToolsCount }) }}
              </span>
            </div>

            <!-- Right: Action & Navigation Buttons -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <!-- Sleek Segmented Quick Jump Control (visível quando a lista está aberta e tem 4 ou mais ferramentas) -->
              <div v-if="!isToolsListCollapsed && message.tool_calls.length >= 4" class="flex items-center rounded-lg bg-slate-200/60 dark:bg-white/5 p-0.5 border border-slate-300/60 dark:border-white/10 text-slate-500 dark:text-slate-400">
                <button
                  type="button"
                  @click.stop="scrollToFirstTool"
                  class="flex items-center gap-1 px-2 py-0.5 rounded-md hover:bg-white dark:hover:bg-white/10 hover:text-indigo-600 dark:hover:text-indigo-300 text-[10.5px] font-medium transition-all cursor-pointer active:scale-95"
                  :title="$t('chat.scroll_first_tool')"
                >
                  <ArrowUp class="w-3 h-3 text-indigo-500 dark:text-indigo-400" />
                  <span>{{ $t('chat.first_tool') }}</span>
                </button>
                <span class="w-[1px] h-3 bg-slate-300/80 dark:bg-white/10 mx-0.5"></span>
                <button
                  type="button"
                  @click.stop="scrollToLastTool"
                  class="flex items-center gap-1 px-2 py-0.5 rounded-md hover:bg-white dark:hover:bg-white/10 hover:text-indigo-600 dark:hover:text-indigo-300 text-[10.5px] font-medium transition-all cursor-pointer active:scale-95"
                  :title="$t('chat.scroll_last_tool')"
                >
                  <ArrowDown class="w-3 h-3 text-indigo-500 dark:text-indigo-400" />
                  <span>{{ $t('chat.last') }}</span>
                </button>
              </div>

              <!-- Recolher Detalhes & Recolher Lista: visíveis apenas quando não há pendências (após aceitar ou recusar) -->
              <template v-if="pendingToolsCount === 0">
                <!-- Expand/Collapse Details Button (visível quando a lista está aberta) -->
                <button
                  v-if="!isToolsListCollapsed"
                  type="button"
                  @click.stop="toggleAllTools"
                  class="text-[10.5px] text-indigo-600 dark:text-indigo-300 hover:text-indigo-800 dark:hover:text-indigo-100 flex items-center gap-1 font-medium px-2 py-1 rounded-lg hover:bg-indigo-50 dark:hover:bg-indigo-500/10 transition-colors cursor-pointer"
                  :title="$t('chat.toggle_all_tools')"
                >
                  <span>{{ areAllToolsExpanded ? $t('chat.collapse_details') : $t('chat.expand_details') }}</span>
                  <component :is="areAllToolsExpanded ? ChevronUp : ChevronDown" class="w-3 h-3 text-indigo-500 dark:text-indigo-400" />
                </button>

                <!-- Main Toggle: Recolher / Expandir Toda a Lista -->
                <button
                  type="button"
                  @click.stop="isToolsListCollapsed = !isToolsListCollapsed"
                  class="text-[10.5px] font-semibold px-2.5 py-1 rounded-lg transition-all flex items-center gap-1 cursor-pointer border shadow-2xs active:scale-95"
                  :class="[
                    isToolsListCollapsed
                      ? 'bg-indigo-600 text-white border-indigo-500 hover:bg-indigo-500 shadow-indigo-500/20'
                      : 'bg-slate-200/80 dark:bg-white/10 text-slate-700 dark:text-slate-200 border-slate-300 dark:border-white/15 hover:bg-slate-300 dark:hover:bg-white/15'
                  ]"
                  :title="isToolsListCollapsed ? $t('chat.show_tools_list_title') : $t('chat.hide_tools_list_title')"
                >
                  <component :is="isToolsListCollapsed ? ChevronDown : ChevronUp" class="w-3 h-3" />
                  <span>{{ isToolsListCollapsed ? $t('chat.view_tools_count', { count: message.tool_calls.length }) : $t('chat.collapse_tools_list') }}</span>
                </button>
              </template>
            </div>
          </div>

          <!-- Middle Row: Search Filter (if 5 or more tools and not collapsed) -->
          <div v-if="!isToolsListCollapsed && message.tool_calls.length >= 5" class="relative">
            <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
            <input
              v-model="toolSearchQuery"
              type="text"
              :placeholder="$t('chat.search_tools_placeholder')"
              class="w-full pl-8 pr-7 py-1 text-[11px] rounded-lg bg-white dark:bg-[#0b0e18] border border-slate-200 dark:border-[#1e253a] text-slate-900 dark:text-slate-200 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-none focus:border-indigo-500/50"
            />
            <button
              v-if="toolSearchQuery"
              @click="toolSearchQuery = ''"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            >
              <X class="w-3 h-3" />
            </button>
          </div>

          <!-- Bottom Row: Multi-selection & Batch Action Buttons -->
          <div v-if="!isToolsListCollapsed && pendingToolsCount > 0" class="flex items-center justify-between gap-2 pt-1.5 border-t border-slate-200 dark:border-white/[0.06] flex-wrap">
            <!-- Select All Checkbox Toggle -->
            <div class="flex items-center gap-2">
              <button
                @click="toggleSelectAllPending"
                class="flex items-center gap-1.5 text-[11px] text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white px-2 py-1 rounded-lg bg-white dark:bg-white/5 hover:bg-slate-100 dark:hover:bg-white/10 border border-slate-200 dark:border-white/10 transition-all cursor-pointer select-none shadow-2xs"
              >
                <component
                  :is="areAllPendingToolsSelected ? CheckSquare : (selectedPendingToolsCount > 0 ? CheckSquare : Square)"
                  class="w-3.5 h-3.5"
                  :class="selectedPendingToolsCount > 0 ? 'text-indigo-600 dark:text-indigo-400' : 'text-slate-400 dark:text-slate-500'"
                />
                <span>{{ areAllPendingToolsSelected ? $t('chat.unselect_all') : $t('chat.select_all', { count: pendingToolsCount }) }}</span>
              </button>

              <span v-if="selectedPendingToolsCount > 0" class="text-[10.5px] text-indigo-600 dark:text-indigo-300 font-mono font-medium">
                {{ $t('chat.selected_count', { count: selectedPendingToolsCount }) }}
              </span>
            </div>

            <!-- Batch Execution & Rejection Actions -->
            <div class="flex items-center gap-2">
              <!-- When some tools are individually selected -->
              <template v-if="selectedPendingToolsCount > 0">
                <button
                  :disabled="message.is_streaming"
                  @click.stop="openRejectSelectedModal"
                  class="px-2.5 py-1 rounded-lg bg-rose-50 hover:bg-rose-100 dark:bg-rose-500/15 dark:hover:bg-rose-500/25 border border-rose-200/90 dark:border-rose-500/30 text-rose-700 dark:text-rose-200 text-[10.5px] font-semibold transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed shadow-2xs"
                  :title="$t('chat.reject_selected_title', { key: deleteKeyLabel })"
                >
                  <X class="w-3 h-3 text-rose-600 dark:text-rose-400" />
                  <span>{{ $t('chat.reject_selected', { count: selectedPendingToolsCount }) }}</span>
                  <kbd class="ml-1 px-1.5 py-0.5 rounded bg-rose-200/70 dark:bg-rose-500/25 border border-rose-300/60 dark:border-rose-500/30 text-[9px] font-mono font-medium text-rose-800 dark:text-rose-200 leading-none">{{ deleteKeyLabel }}</kbd>
                </button>
                <button
                  :disabled="message.is_streaming"
                  @click.stop="handleApproveSelected"
                  class="px-2.5 py-1 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-[10.5px] font-semibold shadow-sm shadow-emerald-600/30 transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed"
                  :title="$t('chat.approve_selected_title', { key: enterKeyLabel })"
                >
                  <Check class="w-3 h-3" />
                  <span>{{ $t('chat.approve_selected', { count: selectedPendingToolsCount }) }}</span>
                  <kbd class="ml-1.5 px-1.5 py-0.5 rounded bg-black/25 dark:bg-emerald-700/60 border border-white/20 dark:border-emerald-500/30 text-[9px] font-mono font-medium text-white leading-none">{{ enterKeyLabel }}</kbd>
                </button>
              </template>

              <!-- Default All Tools Actions (when no selection active) -->
              <template v-else>
                <button
                  :disabled="message.is_streaming"
                  @click.stop="openRejectAllModal"
                  class="px-2.5 py-1 rounded-lg bg-rose-50 hover:bg-rose-100 dark:bg-rose-500/15 dark:hover:bg-rose-500/25 border border-rose-200/90 dark:border-rose-500/30 text-rose-700 dark:text-rose-200 text-[10.5px] font-semibold transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed shadow-2xs"
                  :title="$t('chat.reject_all_title', { count: pendingToolsCount, key: deleteKeyLabel })"
                >
                  <X class="w-3 h-3 text-rose-600 dark:text-rose-400" />
                  <span>{{ $t('chat.reject_all', { count: pendingToolsCount }) }}</span>
                  <kbd class="ml-1 px-1.5 py-0.5 rounded bg-rose-200/70 dark:bg-rose-500/25 border border-rose-300/60 dark:border-rose-500/30 text-[9px] font-mono font-medium text-rose-800 dark:text-rose-200 leading-none">⇧ {{ deleteKeyLabel }}</kbd>
                </button>
                <button
                  :disabled="message.is_streaming"
                  @click.stop="$emit('approveAllTools', { message })"
                  class="px-2.5 py-1 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-[10.5px] font-semibold shadow-sm shadow-emerald-600/30 transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed"
                  :title="message.is_streaming ? $t('chat.waiting_ai') : $t('chat.approve_all_title', { count: pendingToolsCount, key: enterKeyLabel })"
                >
                  <Loader2 v-if="message.is_streaming" class="w-3 h-3 animate-spin" />
                  <Check v-else class="w-3 h-3" />
                  <span>{{ message.is_streaming ? $t('chat.waiting_ai') : $t('chat.approve_all', { count: pendingToolsCount }) }}</span>
                  <kbd v-if="!message.is_streaming" class="ml-1.5 px-1.5 py-0.5 rounded bg-black/25 dark:bg-emerald-700/60 border border-white/20 dark:border-emerald-500/30 text-[9px] font-mono font-medium text-white leading-none">⇧ {{ enterKeyLabel }}</kbd>
                </button>
              </template>
            </div>
          </div>
        </div>

        <!-- Tool Cards List -->
        <div
          v-if="!isToolsListCollapsed"
          :class="[
            message.tool_calls.length > 1
              ? 'space-y-1.5 p-2 rounded-2xl bg-slate-50/80 dark:bg-[#0d101a]/70 border border-slate-200 dark:border-[#1b2033]'
              : 'space-y-2'
          ]"
        >
          <div
            v-for="(tc, tcIdx) in filteredToolCalls"
            :key="getToolKey(tc, tcIdx)"
            :id="getToolCardId(tcIdx)"
            :class="[
              'rounded-xl border transition-all text-xs font-sans shadow-xs overflow-hidden scroll-mt-28',
              isToolSelected(tc, tcIdx) ? 'ring-2 ring-indigo-500 border-indigo-500 dark:border-indigo-400/80' : '',
              tc.status === 'pending_approval'
                ? 'bg-amber-50/50 dark:bg-gradient-to-b dark:from-[#18192a] dark:to-[#121424] border-amber-300 dark:border-amber-500/40 ring-1 ring-amber-400/30 dark:ring-amber-500/20'
                : tc.status === 'streaming'
                ? 'bg-purple-50/40 dark:bg-gradient-to-b dark:from-[#19152b] dark:to-[#121020] border-purple-300 dark:border-purple-500/40 ring-1 ring-purple-400/30 dark:ring-purple-500/20'
                : tc.status === 'executing'
                ? 'bg-indigo-50/40 dark:bg-gradient-to-b dark:from-[#161b2e] dark:to-[#111626] border-indigo-300 dark:border-indigo-500/40 ring-1 ring-indigo-400/30 dark:ring-indigo-500/20'
                : tc.status === 'rejected'
                ? 'bg-rose-50/30 dark:bg-[#151620] border-rose-200 dark:border-rose-500/30'
                : tc.status === 'error'
                ? 'bg-rose-50/40 dark:bg-[#181116] border-rose-300 dark:border-rose-500/40 ring-1 ring-rose-400/30 dark:ring-rose-500/20'
                : 'bg-emerald-50/20 dark:bg-[#121626] border-emerald-300 dark:border-emerald-500/30 hover:border-emerald-400 dark:hover:border-emerald-500/50'
            ]"
          >
            <!-- Clickable Card Header / Summary Row (Always Visible) -->
            <div
              @click="toggleToolExpand(tc, tcIdx)"
              class="p-2.5 flex items-center justify-between gap-2.5 cursor-pointer hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors select-none"
            >
              <div class="flex items-center gap-2 min-w-0 flex-wrap">
                <!-- Multi-selection Checkbox (when multiple tools in message) -->
                <button
                  v-if="message.tool_calls.length > 1 && (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error')"
                  @click.stop="toggleToolSelection(tc, tcIdx)"
                  class="p-0.5 text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 transition-colors cursor-pointer"
                  :title="$t('chat.select_tool_batch')"
                >
                  <component
                    :is="isToolSelected(tc, tcIdx) ? CheckSquare : Square"
                    class="w-3.5 h-3.5"
                    :class="isToolSelected(tc, tcIdx) ? 'text-indigo-600 dark:text-indigo-400' : 'text-slate-400 dark:text-slate-500'"
                  />
                </button>

                <div class="flex items-center gap-1.5 min-w-0">
                  <span class="font-bold text-slate-800 dark:text-slate-100 flex items-center gap-1.5 text-xs truncate">
                    <Wrench class="w-3.5 h-3.5 text-indigo-500 dark:text-indigo-400 flex-shrink-0" />
                    <span class="font-sans font-semibold text-slate-900 dark:text-slate-100 truncate">{{ getToolDisplayName(tc) }}</span>
                  </span>
                  <span v-if="getToolDisplayName(tc) !== tc.name" class="font-mono text-[10px] text-slate-400 dark:text-indigo-300/60 truncate hidden sm:inline" :title="tc.name">
                    ({{ tc.name }})
                  </span>
                </div>

                <span v-if="tc.server_name" class="text-[10px] px-1.5 py-0.2 rounded bg-indigo-50 dark:bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-500/20 font-medium flex-shrink-0">
                  {{ getServerDisplayName(tc) }}
                </span>
              </div>

              <!-- Status Badge & Expand/Collapse Chevron -->
              <div class="flex items-center gap-2 flex-shrink-0">
                <!-- Streaming / Generating Badge -->
                <span
                  v-if="tc.status === 'streaming'"
                  class="text-[10px] font-semibold text-purple-700 dark:text-purple-300 bg-purple-100/90 dark:bg-purple-500/15 border border-purple-300 dark:border-purple-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <Loader2 class="w-3 h-3 text-purple-600 dark:text-purple-400 animate-spin" />
                  <span>{{ $t('chat.generating_tool_call') }}</span>
                </span>

                <!-- Pending Approval Badge -->
                <span
                  v-else-if="tc.status === 'pending_approval'"
                  class="text-[10px] font-semibold text-amber-800 dark:text-amber-300 bg-amber-100/90 dark:bg-amber-500/15 border border-amber-300 dark:border-amber-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <ShieldAlert class="w-3 h-3 text-amber-600 dark:text-amber-400" />
                  <span>{{ $t('chat.awaiting_authorization') }}</span>
                </span>

                <!-- Executing Badge -->
                <span
                  v-else-if="tc.status === 'executing'"
                  class="text-[10px] font-semibold text-indigo-700 dark:text-indigo-300 bg-indigo-100/90 dark:bg-indigo-500/15 border border-indigo-300 dark:border-indigo-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <Loader2 class="w-3 h-3 text-indigo-600 dark:text-indigo-400 animate-spin" />
                  <span>{{ $t('chat.status_executing') }}</span>
                </span>

                <!-- Rejected Badge -->
                <span
                  v-else-if="tc.status === 'rejected'"
                  class="text-[10px] font-semibold text-rose-700 dark:text-rose-300 bg-rose-100/90 dark:bg-rose-500/15 border border-rose-300 dark:border-rose-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <X class="w-3 h-3 text-rose-600 dark:text-rose-400" />
                  <span>{{ $t('chat.status_rejected') }}</span>
                </span>

                <!-- Error Badge -->
                <span
                  v-else-if="tc.status === 'error'"
                  class="text-[10px] font-semibold text-rose-700 dark:text-rose-400 bg-rose-100/90 dark:bg-rose-500/15 border border-rose-300 dark:border-rose-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <AlertTriangle class="w-3 h-3 text-rose-600 dark:text-rose-400" />
                  <span>{{ $t('common.error') }}</span>
                </span>

                <!-- Completed Badge -->
                <span
                  v-else
                  class="text-[10px] font-semibold text-emerald-700 dark:text-emerald-300 bg-emerald-100/90 dark:bg-emerald-500/15 border border-emerald-300 dark:border-emerald-500/30 px-2 py-0.5 rounded-full flex items-center gap-1 shadow-2xs"
                >
                  <Check class="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
                  <span>{{ $t('chat.status_executed') }}</span>
                </span>

                <!-- Expand/Collapse Chevron -->
                <div class="w-5 h-5 rounded-md flex items-center justify-center text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-transform">
                  <component :is="isToolExpanded(tc, tcIdx) ? ChevronDown : ChevronRight" class="w-3.5 h-3.5" />
                </div>
              </div>
            </div>

            <!-- Collapsible Body Content -->
            <div
              v-if="isToolExpanded(tc, tcIdx)"
              class="tool-call-body px-3 pb-3 pt-1 border-t border-slate-200/80 dark:border-white/[0.06] space-y-2.5 bg-slate-50/70 dark:bg-black/25"
            >
              <!-- Description -->
              <p class="text-[11px] text-slate-600 dark:text-slate-400 leading-tight">
                {{ getToolDescription(tc) }}
              </p>

              <!-- Streaming Parameters Placeholder -->
              <div
                v-if="tc.status === 'streaming'"
                class="p-2.5 rounded-xl bg-purple-500/10 border border-purple-500/25 flex items-center gap-2 text-xs text-purple-300 dark:text-purple-200"
              >
                <Loader2 class="w-3.5 h-3.5 animate-spin text-purple-400 flex-shrink-0" />
                <span class="text-[11px]">{{ $t('chat.generating_tool_params') }}</span>
              </div>

              <!-- Arguments Preview -->
              <div v-if="tc.status !== 'streaming' && hasArguments(tc.arguments)" class="space-y-1.5">
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[10px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider block">
                    {{ $t('chat.requested_parameters') }}
                  </span>
                  <button
                    @click.stop="toggleRawJson(tc, tcIdx)"
                    class="text-[10px] text-slate-400 hover:text-indigo-400 transition-colors flex items-center gap-1 cursor-pointer font-mono"
                    :title="$t('chat.toggle_json_view')"
                  >
                    <Code2 class="w-3 h-3" />
                    <span>{{ isRawJsonVisible(tc, tcIdx) ? $t('chat.view_formatted') : $t('chat.view_json') }}</span>
                  </button>
                </div>

                <!-- Formatted Parameters Grid/List -->
                <div
                  v-if="!isRawJsonVisible(tc, tcIdx) && isPlainObjectArguments(tc.arguments)"
                  class="rounded-xl bg-white/70 dark:bg-[#0b0e18]/80 border border-slate-200 dark:border-[#1e253a] overflow-hidden divide-y divide-slate-100 dark:divide-white/[0.04] shadow-2xs"
                >
                  <div
                    v-for="(val, key) in parseArguments(tc.arguments)"
                    :key="key"
                    class="px-3 py-1.5 flex items-center justify-between gap-3 text-[11px] hover:bg-slate-50/60 dark:hover:bg-white/[0.02] transition-colors"
                  >
                    <div class="flex items-center gap-1.5 min-w-0 flex-shrink-0">
                      <span class="font-medium text-slate-800 dark:text-slate-200">
                        {{ getFieldDisplayName(tc, String(key)) }}
                      </span>
                      <span v-if="getFieldDisplayName(tc, String(key)) !== String(key)" class="font-mono text-[9.5px] text-indigo-600/75 dark:text-indigo-300/60" :title="String(key)">
                        ({{ key }})
                      </span>
                    </div>
                    <div class="font-mono text-right break-all min-w-0 flex-1">
                      <span
                        v-if="typeof val === 'boolean'"
                        :class="val ? 'text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-500/10 border border-emerald-200 dark:border-emerald-500/20' : 'text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-500/10 border border-rose-200 dark:border-rose-500/20'"
                        class="px-1.5 py-0.5 rounded text-[10px] font-semibold"
                      >
                        {{ val ? 'true' : 'false' }}
                      </span>
                      <span
                        v-else-if="typeof val === 'number'"
                        class="text-cyan-700 dark:text-cyan-300 font-bold px-1.5 py-0.5 rounded bg-cyan-50 dark:bg-cyan-500/10 border border-cyan-200 dark:border-cyan-500/20 text-[10.5px]"
                      >
                        {{ val }}
                      </span>
                      <span
                        v-else
                        class="text-slate-800 dark:text-slate-200 font-sans"
                      >
                        {{ formatArgumentValue(val) }}
                      </span>
                    </div>
                  </div>
                </div>

                <!-- Raw JSON View (Fallback or Toggled) -->
                <pre
                  v-else
                  class="p-2 rounded-xl bg-white dark:bg-[#0b0e18] border border-slate-200 dark:border-[#1e253a] font-mono text-[11px] text-slate-800 dark:text-slate-300 overflow-x-auto whitespace-pre-wrap max-h-32 shadow-2xs"
                >{{ formatJson(tc.arguments) }}</pre>
              </div>

              <!-- Action Buttons for Pending Approval, Executing or Error -->
              <div v-if="tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error'" class="pt-2 border-t border-amber-300/60 dark:border-amber-500/20 flex items-center justify-between gap-2 flex-wrap">
                <span class="text-[11px] text-amber-800 dark:text-amber-200/80 font-medium">
                  <template v-if="tc.status === 'error'">{{ $t('chat.tool_call_error_msg') }}</template>
                  <template v-else-if="tc.status === 'executing'">{{ $t('chat.tool_call_executing_msg') }}</template>
                  <template v-else>{{ $t('chat.ai_needs_permission') }}</template>
                </span>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <button
                    :disabled="message.is_streaming || tc.status === 'executing'"
                    @click.stop="openRejectSingleModal(tc)"
                    class="px-3 py-1.5 rounded-xl bg-rose-50 hover:bg-rose-100 dark:bg-rose-500/15 dark:hover:bg-rose-500/25 border border-rose-200/90 dark:border-rose-500/30 text-rose-700 dark:text-rose-200 text-xs font-semibold transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed shadow-xs"
                    :title="$t('chat.reject_tool_shortcut', { key: deleteKeyLabel })"
                  >
                    <X class="w-3.5 h-3.5 text-rose-600 dark:text-rose-400" />
                    <span>{{ $t('common.cancel') }}</span>
                    <kbd class="ml-1 px-1.5 py-0.5 rounded bg-rose-200/70 dark:bg-rose-500/25 border border-rose-300/60 dark:border-rose-500/30 text-[9.5px] font-mono font-medium text-rose-800 dark:text-rose-200 leading-none">{{ deleteKeyLabel }}</kbd>
                  </button>
                  <button
                    v-if="tc.status === 'pending_approval' && (tc.server_id === 'skills' || tc.name === 'run_command' || tc.name === 'run_skill_command' || tc.name === 'run_skill_script')"
                    :disabled="message.is_streaming"
                    @click.stop="$emit('approveTool', { toolCall: tc, message, alwaysAllow: true })"
                    class="px-3 py-1.5 rounded-xl bg-amber-500/10 hover:bg-amber-500/20 text-amber-800 dark:text-amber-200 border border-amber-300/80 dark:border-amber-500/30 text-xs font-semibold transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed shadow-2xs"
                    :title="$t('chat.always_allow_tooltip')"
                  >
                    <Zap class="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
                    <span>{{ $t('chat.always_allow_and_run') }}</span>
                  </button>
                  <button
                    :disabled="message.is_streaming || tc.status === 'executing'"
                    @click.stop="$emit('approveTool', { toolCall: tc, message })"
                    class="px-3.5 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-md shadow-emerald-600/30 transition-all active:scale-95 cursor-pointer flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed"
                    :title="$t('chat.approve_tool_shortcut', { key: enterKeyLabel })"
                  >
                    <Loader2 v-if="tc.status === 'executing'" class="w-3.5 h-3.5 animate-spin" />
                    <Check v-else class="w-3.5 h-3.5" />
                    <span>{{ message.is_streaming ? $t('chat.waiting_ai') : (tc.status === 'executing' ? $t('chat.status_executing') : tc.status === 'error' ? $t('common.retry') : $t('chat.approve_and_run')) }}</span>
                    <kbd v-if="tc.status !== 'executing' && !message.is_streaming" class="ml-1.5 px-1.5 py-0.5 rounded bg-black/25 dark:bg-emerald-700/60 border border-white/20 dark:border-emerald-500/30 text-[9.5px] font-mono font-medium text-white leading-none">{{ enterKeyLabel }}</kbd>
                  </button>
                </div>
              </div>

              <!-- Rejection Reason Details for Rejected Tool -->
              <div v-if="tc.status === 'rejected'" class="pt-2 border-t border-rose-200 dark:border-rose-500/20 space-y-2">
                <div class="flex items-center justify-between text-[11px] text-rose-700 dark:text-rose-300 font-medium">
                  <div class="flex items-center gap-1.5">
                    <X class="w-3.5 h-3.5 text-rose-600 dark:text-rose-400" />
                    <span class="font-semibold">{{ $t('chat.execution_rejected_by_user') }}</span>
                  </div>
                  <button
                    :disabled="message.is_streaming"
                    @click.stop="$emit('approveTool', { toolCall: tc, message })"
                    class="text-[10.5px] text-indigo-600 dark:text-indigo-300 hover:text-indigo-800 dark:hover:text-indigo-200 hover:underline cursor-pointer disabled:opacity-40 flex items-center gap-1 font-semibold"
                  >
                    <Check class="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
                    <span>{{ $t('chat.reconsider_and_run') }}</span>
                  </button>
                </div>

                <div v-if="tc.rejection_reason" class="p-2.5 rounded-xl bg-rose-50 dark:bg-rose-950/25 border border-rose-200 dark:border-rose-500/30 text-[11px] text-rose-800 dark:text-rose-200/90 leading-relaxed font-sans shadow-2xs">
                  <span class="text-rose-700 dark:text-rose-300 font-bold block mb-0.5">{{ $t('chat.refusal_reason_card_title') }}</span>
                  <p class="whitespace-pre-wrap">{{ tc.rejection_reason }}</p>
                </div>
              </div>

              <!-- Error Details for Failed Tool -->
              <div v-if="tc.status === 'error' && tc.result" class="pt-1.5 border-t border-rose-200 dark:border-rose-500/20">
                <span class="text-[10.5px] text-rose-800 dark:text-rose-300 font-mono block p-2 rounded-xl bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-500/30 whitespace-pre-wrap shadow-2xs">
                  {{ tc.result?.error || formatJson(tc.result) }}
                </span>
              </div>

              <!-- Result View for Completed Tool -->
              <div v-if="tc.status === 'completed' && tc.result" class="pt-1.5 border-t border-emerald-200 dark:border-emerald-500/20">
                <div class="flex items-center justify-between gap-2 flex-wrap">
                  <button
                    @click="toggleResultView(getToolKey(tc, tcIdx))"
                    class="text-[11px] text-emerald-700 dark:text-emerald-300 hover:text-emerald-800 dark:hover:text-emerald-200 flex items-center gap-1 font-medium cursor-pointer"
                  >
                    <component :is="expandedResults[getToolKey(tc, tcIdx)] ? ChevronDown : ChevronRight" class="w-3 h-3" />
                    <span>{{ expandedResults[getToolKey(tc, tcIdx)] ? $t('chat.hide_tool_result') : $t('chat.view_tool_result_json') }}</span>
                  </button>
                  <button
                    :disabled="message.is_streaming"
                    @click.stop="$emit('reExecuteTool', { toolCall: tc, message })"
                    class="text-[10.5px] text-slate-500 hover:text-indigo-600 dark:text-slate-400 dark:hover:text-indigo-300 flex items-center gap-1 font-medium cursor-pointer transition-colors"
                    :title="$t('chat.re_execute_tooltip')"
                  >
                    <RotateCcw class="w-3 h-3" />
                    <span>{{ $t('chat.re_execute') }}</span>
                  </button>
                </div>
                <pre
                  v-if="expandedResults[getToolKey(tc, tcIdx)]"
                  class="mt-1.5 p-2 rounded-xl bg-white dark:bg-[#0b0e18] border border-emerald-300 dark:border-emerald-500/30 font-mono text-[10.5px] text-emerald-800 dark:text-emerald-200 max-h-48 overflow-y-auto whitespace-pre-wrap shadow-2xs"
                >{{ formatJson(tc.result) }}</pre>
              </div>
            </div>
          </div>

          <!-- Bottom Navigation Footer (visível quando há 4 ou mais ferramentas) -->
          <div
            v-if="message.tool_calls.length >= 4"
            class="flex items-center justify-between gap-2 pt-2 px-1 text-[11px] text-slate-500 dark:text-slate-400 border-t border-slate-200 dark:border-white/[0.06]"
          >
            <span class="text-[10.5px] font-mono">
              {{ $t('chat.tools_end_label', { count: message.tool_calls.length }) }}
            </span>
            <div class="flex items-center gap-2">
              <button
                type="button"
                @click="scrollToFirstTool"
                class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20 text-[10.5px] font-medium transition-all cursor-pointer active:scale-95 shadow-2xs"
                :title="$t('chat.scroll_first_tool')"
              >
                <ArrowUp class="w-3.5 h-3.5" />
                <span>{{ $t('chat.scroll_first_tool_btn') }}</span>
              </button>
              <button
                v-if="pendingToolsCount === 0"
                type="button"
                @click="isToolsListCollapsed = true"
                class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-slate-200/70 dark:bg-white/5 hover:bg-slate-300 dark:hover:bg-white/10 text-slate-700 dark:text-slate-300 border border-slate-300/80 dark:border-white/10 text-[10.5px] font-medium transition-all cursor-pointer active:scale-95"
                :title="$t('chat.hide_tools_list_title')"
              >
                <ChevronUp class="w-3.5 h-3.5" />
                <span>{{ $t('chat.collapse_tools_list') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Action Banner for Token Limit Reached (finish_reason: length) -->
      <div
        v-if="isTokenLimitReached"
        class="mt-3.5 p-3 rounded-2xl bg-amber-500/10 border border-amber-500/25 flex flex-col gap-2.5 select-none"
      >
        <div class="flex items-start gap-2.5">
          <div class="w-7 h-7 rounded-xl bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-amber-400 shrink-0 mt-0.5">
            <AlertTriangle class="w-4 h-4" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-amber-300">
              {{ $t('chat.token_limit_reached_title', { count: tokenLimitCount }) }}
            </div>
            <p class="text-[11.5px] text-slate-300 mt-0.5 leading-relaxed">
              {{ $t('chat.token_limit_banner_desc') }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-1.5 border-t border-amber-500/15 flex-wrap">
          <button
            @click="$emit('continueGeneration', message)"
            class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-1.5 shadow-md shadow-indigo-600/20 transition-all cursor-pointer active:scale-95"
            :title="$t('chat.continue_response_tooltip')"
          >
            <Play class="w-3.5 h-3.5 fill-white" />
            <span>{{ $t('chat.continue_response') }}</span>
          </button>
          <button
            @click="$emit('openParams')"
            class="px-3 py-1.5 rounded-xl bg-[#181d2f] hover:bg-[#20273f] text-slate-300 hover:text-white text-xs font-medium border border-[#273150] flex items-center gap-1.5 transition-all cursor-pointer active:scale-95"
            :title="$t('chat.increase_tokens_params')"
          >
            <SlidersHorizontal class="w-3.5 h-3.5 text-indigo-400" />
            <span>{{ $t('chat.increase_limit_btn') }}</span>
          </button>
        </div>
      </div>

      <!-- Quick Action Banner for Missing Model Warning -->
      <div
        v-if="isModelMissingError"
        class="mt-3 pt-3 border-t border-amber-500/20 flex items-center gap-2 flex-wrap select-none"
      >
        <button
          @click="$emit('selectTab', 'models')"
          class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-1.5 shadow-md shadow-indigo-600/20 transition-all cursor-pointer active:scale-95"
        >
          <Cpu class="w-3.5 h-3.5" />
          <span>{{ $t('chat.go_to_models') }}</span>
        </button>
        <button
          @click="$emit('retryLastMessage')"
          class="px-3 py-1.5 rounded-xl bg-[#181d2f] hover:bg-[#20273f] text-slate-300 hover:text-white text-xs font-medium border border-[#273150] flex items-center gap-1.5 transition-all cursor-pointer active:scale-95"
        >
          <RotateCcw class="w-3.5 h-3.5 text-indigo-400" />
          <span>{{ $t('chat.try_again') }}</span>
        </button>
      </div>
    </div>

    <!-- Modal de Justificativa de Recusa de Ferramenta(s) -->
    <Teleport to="body">
      <div
        v-if="rejectionModal.isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md select-none"
        @click.self="rejectionModal.isOpen = false"
      >
        <div
          class="bg-[#111422] border border-[#232a42] rounded-2xl w-full max-w-lg shadow-2xl p-5 text-slate-100 flex flex-col gap-4 animate-scaleUp relative"
        >
          <!-- Header -->
          <div class="flex items-center justify-between border-b border-white/[0.08] pb-3">
            <div class="flex items-center gap-3">
              <div class="w-9 h-9 rounded-xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400 flex-shrink-0 shadow-sm">
                <ShieldAlert class="w-5 h-5" />
              </div>
              <div>
                <h3 class="font-bold text-sm text-slate-100 leading-snug">
                  <template v-if="rejectionModal.mode === 'single'">
                    {{ $t('chat.refusal_modal_title_single') }}
                  </template>
                  <template v-else-if="rejectionModal.mode === 'selected'">
                    {{ $t('chat.refusal_modal_title_selected', { count: rejectionModal.toolCalls.length }) }}
                  </template>
                  <template v-else>
                    {{ $t('chat.refusal_modal_title_all', { count: rejectionModal.toolCalls.length }) }}
                  </template>
                </h3>
                <p class="text-[11px] text-slate-400">
                  {{ $t('chat.refusal_modal_subtitle') }}
                </p>
              </div>
            </div>
            <button
              @click="rejectionModal.isOpen = false"
              class="w-7 h-7 rounded-lg flex items-center justify-center text-slate-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Tools Target Preview Chips -->
          <div class="space-y-1.5 max-h-36 overflow-y-auto pr-1">
            <span class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 block">
              {{ rejectionModal.toolCalls.length === 1 ? $t('chat.refusal_target_single') : $t('chat.refusal_target_multiple', { count: rejectionModal.toolCalls.length }) }}
            </span>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="(t, idx) in rejectionModal.toolCalls"
                :key="idx"
                class="text-[11px] font-mono px-2.5 py-1 rounded-lg bg-rose-500/10 text-rose-300 border border-rose-500/20 flex items-center gap-1.5"
              >
                <Wrench class="w-3 h-3 text-rose-400 flex-shrink-0" />
                <span class="font-semibold">{{ t.name }}</span>
                <span v-if="t.server_name" class="text-[9.5px] text-rose-400/70">({{ getServerDisplayName(t) }})</span>
              </span>
            </div>
          </div>

          <!-- Input Textarea for Rejection Reason -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-300 flex items-center justify-between">
              <span>{{ $t('chat.refusal_reason_label') }} <span class="text-slate-500 font-normal">{{ $t('chat.optional_recommended') }}</span></span>
            </label>
            <textarea
              v-model="rejectionModal.reason"
              :placeholder="$t('chat.refusal_reason_placeholder')"
              rows="3"
              autofocus
              @keydown.ctrl.enter.stop="confirmRejection()"
              @keydown.meta.enter.stop="confirmRejection()"
              class="w-full px-3 py-2 rounded-xl bg-[#090b14] border border-[#1e2439] focus:border-rose-500/50 focus:ring-2 focus:ring-rose-500/20 text-slate-200 text-xs placeholder-slate-500 resize-none outline-none leading-relaxed transition-all"
            />
            <div class="flex items-center justify-between text-[10px] text-slate-500">
              <span>{{ $t('chat.refusal_tip', { key: isMac ? '⌘+Enter' : 'Ctrl+Enter' }) }}</span>
              <span v-if="rejectionModal.reason.length > 0" class="text-slate-400 font-mono">{{ $t('chat.refusal_characters', { count: rejectionModal.reason.length }) }}</span>
            </div>
          </div>

          <!-- Action Buttons -->
          <div class="flex items-center justify-between pt-2 border-t border-white/[0.08] gap-2">
            <button
              @click="confirmRejection('')"
              class="px-3 py-1.5 rounded-xl text-slate-400 hover:text-slate-200 text-xs font-medium hover:bg-white/[0.04] transition-colors cursor-pointer"
              :title="$t('chat.refusal_no_reason_tooltip')"
            >
              {{ $t('chat.refusal_no_reason_btn') }}
            </button>

            <div class="flex items-center gap-2">
              <button
                @click="rejectionModal.isOpen = false"
                class="px-3.5 py-1.5 rounded-xl bg-[#171c2e] hover:bg-[#202740] border border-[#232a42] text-slate-300 text-xs font-medium transition-colors cursor-pointer"
              >
                {{ $t('common.cancel') }}
              </button>
              <button
                @click="confirmRejection()"
                class="px-4 py-1.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold shadow-md shadow-rose-600/30 transition-all active:scale-95 cursor-pointer flex items-center gap-1.5"
              >
                <X class="w-3.5 h-3.5" />
                <span>{{ $t('chat.confirm_refusal_btn', { count: rejectionModal.toolCalls.length }) }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import {
  Copy,
  Check,
  BrainCircuit,
  ChevronDown,
  ChevronRight,
  ChevronUp,
  Wrench,
  ShieldAlert,
  Loader2,
  X,
  Layers,
  FileCode2,
  Mic,
  Zap,
  Volume2,
  VolumeX,
  Square,
  CheckSquare,
  Search,
  RotateCcw,
  Cpu,
  AlertTriangle,
  Play,
  SlidersHorizontal,
  Pencil,
  Code2,
  Trash2,
  ArrowUp,
  ArrowDown
} from 'lucide-vue-next'
import {
  speakText,
  stopSpeech,
  isSpeaking as globalIsSpeaking,
  isPaused as globalIsPaused,
  activeSpeakingId
} from '~/utils/tts'
import { useAppLocale } from '~/composables/useLocale'
import type { ChatMessage, McpToolWithServer } from '~/types'

const { t } = useAppLocale()

const props = withDefaults(
  defineProps<{
    message: ChatMessage
    showEfficiencyMetrics?: boolean
    mcpTools?: McpToolWithServer[]
  }>(),
  {
    showEfficiencyMetrics: true,
    mcpTools: () => []
  }
)

const emit = defineEmits<{
  (e: 'approveTool', payload: any): void
  (e: 'rejectTool', payload: any): void
  (e: 'reExecuteTool', payload: any): void
  (e: 'approveSelectedTools', payload: any): void
  (e: 'rejectSelectedTools', payload: any): void
  (e: 'approveAllTools', payload: any): void
  (e: 'rejectAllTools', payload: any): void
  (e: 'resendMessage', payload: any): void
  (e: 'retryLastMessage'): void
  (e: 'selectTab', tab: string): void
  (e: 'continueGeneration', message: ChatMessage): void
  (e: 'openParams'): void
  (e: 'toolLabelUpdated', payload: any): void
  (e: 'deleteMessage', messageId: string): void
}>()

const isTokenLimitReached = computed(() => {
  if (isUser.value || props.message?.is_streaming) return false
  const fr = props.message?.metrics?.finish_reason || props.message?.finish_reason
  return fr === 'length'
})

const tokenLimitCount = computed(() => {
  return (props.message?.metrics && props.message.metrics.completion_tokens) || props.message?.tokens_count || 0
})

const isModelMissingError = computed(() => {
  if (isUser.value) return false
  const content = props.message?.content || ''
  return (
    content.includes('Nenhum modelo selecionado') ||
    content.includes('selecione e carregue um modelo') ||
    content.includes('No model selected') ||
    content.includes('select and load a model') ||
    content.includes('Ningún modelo seleccionado')
  )
})

const isThinkingExpanded = ref(false)
const copied = ref(false)
const confirmingDelete = ref(false)
let confirmDeleteTimeout: ReturnType<typeof setTimeout> | null = null

const handleDeleteClick = () => {
  if (!confirmingDelete.value) {
    confirmingDelete.value = true
    if (confirmDeleteTimeout) clearTimeout(confirmDeleteTimeout)
    confirmDeleteTimeout = setTimeout(() => {
      confirmingDelete.value = false
    }, 3000)
  } else {
    if (confirmDeleteTimeout) clearTimeout(confirmDeleteTimeout)
    confirmingDelete.value = false
    emit('deleteMessage', props.message.id)
  }
}
const liveThinkingRef = ref<HTMLElement | null>(null)
const expandedResults = reactive<Record<string, boolean>>({})
const expandedTools = reactive<Record<string, boolean>>({})
const expandedAttachments = reactive<Record<string | number, boolean>>({})

// Multi-selection state for batch tool actions
const selectedToolKeys = ref<Set<string>>(new Set())
const toolSearchQuery = ref('')
const isToolsListCollapsed = ref(false)

const isMac = ref(true)
onMounted(() => {
  if (typeof navigator !== 'undefined') {
    isMac.value = /(Mac|iPhone|iPod|iPad)/i.test(navigator.platform || navigator.userAgent || '')
  }
})
const enterKeyLabel = computed(() => 'Enter')
const deleteKeyLabel = computed(() => 'Esc')

// Rejection modal state
const rejectionModal = reactive<{
  isOpen: boolean
  mode: 'single' | 'selected' | 'all'
  toolCalls: any[]
  reason: string
  targetToolCall: any
}>({
  isOpen: false,
  mode: 'single', // 'single' | 'selected' | 'all'
  toolCalls: [],
  reason: '',
  targetToolCall: null
})

const messageId = computed(() => props.message.id || props.message.created_at || props.message.content?.slice(0, 30))

const isThisSpeaking = computed(() => {
  return globalIsSpeaking.value && activeSpeakingId.value === messageId.value
})

const isThisPaused = computed(() => {
  return globalIsPaused.value && activeSpeakingId.value === messageId.value
})

const handleTtsClick = () => {
  if (!props.message?.content) return
  speakText(messageId.value, props.message.content)
}

const toggleAttachmentContent = (idx: number | string) => {
  expandedAttachments[idx] = !expandedAttachments[idx]
}

const isAudioExt = (name?: string | null) => {
  if (!name) return false
  const n = name.toLowerCase()
  return ['.mp3', '.wav', '.m4a', '.ogg', '.aac', '.flac', '.opus'].some(ext => n.endsWith(ext))
}

const formatBytes = (bytes?: number | null) => {
  if (!bytes) return ''
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

const formatDuration = (sec?: number | null) => {
  if (!sec) return ''
  const mins = Math.floor(sec / 60)
  const remSec = Math.floor(sec % 60)
  return mins > 0 ? `${mins}m ${remSec}s` : `${remSec}s`
}

const getToolKey = (tc: any, idx = 0) => tc?.id || `${tc?.name}_${idx}`

const showRawJson = reactive<Record<string, boolean>>({})

const isRawJsonVisible = (tc: any, tcIdx: number) => {
  const key = getToolKey(tc, tcIdx)
  return !!showRawJson[key]
}

const toggleRawJson = (tc: any, tcIdx: number) => {
  const key = getToolKey(tc, tcIdx)
  showRawJson[key] = !showRawJson[key]
}

const isProceduralSkillCall = (tc: any) => {
  return tc.name === 'create_procedural_skill' ||
    tc.name === 'update_procedural_skill' ||
    tc.name === 'edit_procedural_skill' ||
    (tc.arguments && (Array.isArray(tc.arguments.steps) || Array.isArray(tc.arguments.triggers)))
}

const getServerDisplayName = (tc: any) => {
  if (tc.server_id === 'skills' || tc.server_name === 'Procedural Skills' || isProceduralSkillCall(tc)) {
    return t('memory.skill_command_server_name')
  }
  if (tc.name === 'atena_web_search' || tc.name === 'atena_fetch_webpage') {
    return t('memory.server_atena_web')
  }
  if (tc.name?.startsWith('atena_scratchpad_')) {
    return t('memory.server_atena_scratchpad')
  }
  if (
    tc.name?.startsWith('atena_schedule_') ||
    tc.name === 'atena_list_scheduled_tasks' ||
    tc.name === 'atena_cancel_scheduled_task' ||
    tc.name === 'atena_run_scheduled_task'
  ) {
    return t('memory.server_atena_scheduler')
  }
  if (tc.server_id === 'atena_native' || tc.server_id === 'atena' || tc.server_name?.includes('Atena Core')) {
    return t('memory.server_atena_native')
  }
  return tc.server_name || ''
}

const getToolDisplayName = (tc: any) => {
  if (tc.name === 'update_procedural_skill' || tc.name === 'edit_procedural_skill') return t('memory.skill_edit_title')
  if (isProceduralSkillCall(tc)) return t('memory.skill_create_title')
  if (tc.name === 'run_command' || tc.name === 'run_skill_command') return t('memory.skill_command_run')
  if (tc.name === 'run_skill_script') return t('memory.skill_script_run')
  if (tc.name === 'atena_search_episodes') return t('memory.native_search_episodes_title')
  if (tc.name === 'atena_read_episode') return t('memory.native_read_episode_title')
  if (tc.name === 'atena_search_memory') return t('memory.native_search_memory_title')
  if (tc.name === 'atena_web_search') return t('memory.native_web_search_title')
  if (tc.name === 'atena_fetch_webpage') return t('memory.native_fetch_webpage_title')
  if (tc.name === 'atena_scratchpad_write') return t('memory.native_scratchpad_write_title')
  if (tc.name === 'atena_scratchpad_read') return t('memory.native_scratchpad_read_title')
  if (tc.name === 'atena_scratchpad_clear') return t('memory.native_scratchpad_clear_title')
  if (tc.name === 'atena_schedule_task') return t('memory.native_schedule_task_title')
  if (tc.name === 'atena_list_scheduled_tasks') return t('memory.native_list_scheduled_tasks_title')
  if (tc.name === 'atena_cancel_scheduled_task') return t('memory.native_cancel_scheduled_task_title')
  if (tc.name === 'atena_run_scheduled_task') return t('memory.native_run_scheduled_task_title')
  if (tc.name === 'loading_tool') return t('chat.generating_tool_call')
  if (tc.label) return tc.label
  if (props.mcpTools && props.mcpTools.length > 0) {
    const match = props.mcpTools.find(
      (t: any) => t.tool?.name === tc.name && (!tc.server_id || t.server_id === tc.server_id)
    )
    if (match?.tool?.label) return match.tool.label
  }
  return tc.name
}

const heuristicFieldTranslation = (key: string) => {
  const map: Record<string, string> = {
    upcoming_days: 'Próximos Dias',
    overdue_days: 'Dias em Atraso',
    query: 'Termo de Busca',
    q: 'Termo de Busca',
    limit: 'Limite',
    offset: 'Deslocamento',
    page: 'Página',
    id: 'Identificador',
    identifier: 'Identificador',
    user_id: 'ID do Usuário',
    start_date: 'Data Inicial',
    end_date: 'Data Final',
    date: 'Data',
    month: 'Mês',
    year: 'Ano',
    amount: 'Valor',
    currency: 'Moeda',
    description: 'Descrição',
    desc: 'Descrição',
    title: 'Título',
    name: 'Nome',
    status: 'Status',
    category: 'Categoria',
    filter: 'Filtro',
    sort: 'Ordenação',
    order: 'Ordenação',
    path: 'Caminho',
    filepath: 'Caminho do Arquivo',
    filename: 'Nome do Arquivo',
    content: 'Conteúdo',
    text: 'Texto',
    url: 'Endereço (URL)',
    type: 'Tipo',
    enabled: 'Ativado'
  }
  if (map[key]) return map[key]
  return key
    .split('_')
    .map((w: string) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

const getFieldDisplayName = (tc: any, key: string) => {
  if (key === 'timeout_ms' || key === 'timeoutMs') return t('memory.param_timeout_ms')
  if (key === 'query') return t('memory.param_query')
  if (key === 'limit') return t('memory.param_limit')
  if (key === 'identifier') return t('memory.param_identifier')
  if (tc.server_id === 'skills' || tc.name === 'run_command' || tc.name === 'run_skill_command' || tc.name === 'run_skill_script' || isProceduralSkillCall(tc)) {
    if (key === 'command' || key === 'cmd') return t('memory.skill_param_command')
    if (key === 'script_file' || key === 'script' || key === 'script_name' || key === 'scriptName' || key === 'file_name' || key === 'fileName' || key === 'filename') return t('memory.skill_param_script_file')
    if (key === 'slug' || key === 'skillId' || key === 'skill_id' || key === 'id') return t('memory.skill_param_slug')
    if (key === 'args') return t('memory.skill_param_args')
    if (key === 'triggers' || key === 'gatilhos') return t('memory.skill_param_triggers')
    if (key === 'steps' || key === 'passos') return t('memory.skill_param_steps')
    if (key === 'scripts') return t('memory.skill_param_scripts')
    if (key === 'refinement_note' || key === 'refinementNote') return t('memory.skill_param_refinement_note')
    if (key === 'name') return t('memory.skill_param_name')
  }
  if (tc.field_labels && tc.field_labels[key]) {
    return tc.field_labels[key]
  }
  if (props.mcpTools && props.mcpTools.length > 0) {
    const match = props.mcpTools.find(
      (t: any) => t.tool?.name === tc.name && (!tc.server_id || t.server_id === tc.server_id)
    )
    if (match?.tool?.field_labels && match.tool.field_labels[key]) {
      return match.tool.field_labels[key]
    }
  }
  return heuristicFieldTranslation(key)
}

const parseArguments = (args: any) => {
  if (!args) return {}
  if (typeof args === 'object' && !Array.isArray(args)) {
    return args
  }
  if (typeof args === 'string') {
    try {
      const parsed = JSON.parse(args)
      if (typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)) {
        return parsed
      }
    } catch {
      return { valor: args }
    }
  }
  return { valor: args }
}

const isPlainObjectArguments = (args: any) => {
  if (!args) return false
  if (typeof args === 'object' && !Array.isArray(args)) {
    return Object.keys(args).length > 0
  }
  if (typeof args === 'string') {
    try {
      const parsed = JSON.parse(args)
      return typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed) && Object.keys(parsed).length > 0
    } catch {
      return false
    }
  }
  return false
}

const formatArgumentValue = (val: any) => {
  if (val === null || val === undefined) return 'null'
  if (typeof val === 'boolean') return val ? 'true' : 'false'
  if (typeof val === 'number') return String(val)
  if (typeof val === 'string') return val
  if (Array.isArray(val)) {
    if (val.every((item) => typeof item !== 'object')) {
      return val.join(', ')
    }
    return JSON.stringify(val)
  }
  return JSON.stringify(val)
}

const isToolExpanded = (tc: any, idx: number) => {
  const key = getToolKey(tc, idx)
  if (expandedTools[key] !== undefined) {
    return expandedTools[key]
  }
  return tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error'
}

const toggleToolExpand = (tc: any, idx: number) => {
  const key = getToolKey(tc, idx)
  const current = isToolExpanded(tc, idx)
  expandedTools[key] = !current
}

const areAllToolsExpanded = computed(() => {
  if (!props.message?.tool_calls?.length) return false
  return props.message.tool_calls.every((tc: any, idx: number) => isToolExpanded(tc, idx))
})

const toggleAllTools = () => {
  const nextState = !areAllToolsExpanded.value
  props.message?.tool_calls?.forEach((tc: any, idx: number) => {
    expandedTools[getToolKey(tc, idx)] = nextState
  })
}

const getToolCardId = (idx: number) => `tool-card-${String(messageId.value || 'msg').replace(/[^a-zA-Z0-9_-]/g, '_')}-${idx}`

const scrollToFirstTool = () => {
  if (isToolsListCollapsed.value) {
    isToolsListCollapsed.value = false
  }
  nextTick(() => {
    const el = document.getElementById(getToolCardId(0))
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
      el.classList.add('ring-2', 'ring-indigo-500', 'shadow-indigo-500/20')
      setTimeout(() => {
        el.classList.remove('ring-2', 'ring-indigo-500', 'shadow-indigo-500/20')
      }, 1500)
    }
  })
}

const scrollToLastTool = () => {
  if (isToolsListCollapsed.value) {
    isToolsListCollapsed.value = false
  }
  nextTick(() => {
    const total = filteredToolCalls.value.length || props.message?.tool_calls?.length || 0
    if (total === 0) return
    const el = document.getElementById(getToolCardId(total - 1))
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'end' })
      el.classList.add('ring-2', 'ring-indigo-500', 'shadow-indigo-500/20')
      setTimeout(() => {
        el.classList.remove('ring-2', 'ring-indigo-500', 'shadow-indigo-500/20')
      }, 1500)
    }
  })
}

const completedToolsCount = computed(() => {
  if (!props.message?.tool_calls) return 0
  return props.message.tool_calls.filter((t: any) => t.status === 'completed').length
})

const pendingToolsCount = computed(() => {
  if (!props.message?.tool_calls) return 0
  return props.message.tool_calls.filter((t: any) => t.status === 'pending_approval' || t.status === 'executing' || t.status === 'error' || t.status === 'streaming').length
})

const rejectedToolsCount = computed(() => {
  if (!props.message?.tool_calls) return 0
  return props.message.tool_calls.filter((t: any) => t.status === 'rejected').length
})

// Tool filtering by search query
const filteredToolCalls = computed(() => {
  if (!props.message?.tool_calls) return []
  if (!toolSearchQuery.value.trim()) return props.message.tool_calls
  const q = toolSearchQuery.value.toLowerCase().trim()
  return props.message.tool_calls.filter((tc: any) => {
    if (tc.name && tc.name.toLowerCase().includes(q)) return true
    if (tc.server_name && tc.server_name.toLowerCase().includes(q)) return true
    if (tc.arguments) {
      const argStr = typeof tc.arguments === 'string' ? tc.arguments : JSON.stringify(tc.arguments)
      if (argStr.toLowerCase().includes(q)) return true
    }
    return false
  })
})

// Multi-selection helper functions
const isToolSelected = (tc: any, idx: number) => {
  return selectedToolKeys.value.has(getToolKey(tc, idx))
}

const toggleToolSelection = (tc: any, idx: number) => {
  const key = getToolKey(tc, idx)
  const next = new Set(selectedToolKeys.value)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  selectedToolKeys.value = next
}

const selectAllPendingTools = () => {
  const next = new Set<string>()
  props.message?.tool_calls?.forEach((tc: any, idx: number) => {
    if (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error') {
      next.add(getToolKey(tc, idx))
    }
  })
  selectedToolKeys.value = next
}

const clearToolSelection = () => {
  selectedToolKeys.value = new Set<string>()
}

const areAllPendingToolsSelected = computed(() => {
  const pending = props.message?.tool_calls?.filter(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error'
  ) || []
  if (pending.length === 0) return false
  return pending.every((tc: any, idx: number) => selectedToolKeys.value.has(getToolKey(tc, idx)))
})

const toggleSelectAllPending = () => {
  if (areAllPendingToolsSelected.value) {
    clearToolSelection()
  } else {
    selectAllPendingTools()
  }
}

const selectedPendingToolsCount = computed(() => {
  const pending = props.message?.tool_calls?.filter(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error'
  ) || []
  return pending.filter((tc: any, idx: number) => selectedToolKeys.value.has(getToolKey(tc, idx))).length
})

const getSelectedToolCalls = () => {
  return (props.message?.tool_calls || []).filter(
    (tc: any, idx: number) => (tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error') &&
                 selectedToolKeys.value.has(getToolKey(tc, idx))
  )
}

// Rejection modal opening handlers
const openRejectSingleModal = (tc: any) => {
  rejectionModal.mode = 'single'
  rejectionModal.targetToolCall = tc
  rejectionModal.toolCalls = [tc]
  rejectionModal.reason = ''
  rejectionModal.isOpen = true
}

const openRejectSelectedModal = () => {
  const selected = getSelectedToolCalls()
  if (selected.length === 0) return
  rejectionModal.mode = 'selected'
  rejectionModal.targetToolCall = null
  rejectionModal.toolCalls = selected
  rejectionModal.reason = ''
  rejectionModal.isOpen = true
}

const openRejectAllModal = () => {
  const pending = (props.message?.tool_calls || []).filter(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing' || tc.status === 'error'
  )
  if (pending.length === 0) return
  rejectionModal.mode = 'all'
  rejectionModal.targetToolCall = null
  rejectionModal.toolCalls = pending
  rejectionModal.reason = ''
  rejectionModal.isOpen = true
}

const confirmRejection = (reasonOverride?: any) => {
  const finalReason = typeof reasonOverride === 'string' ? reasonOverride : rejectionModal.reason
  if (rejectionModal.mode === 'single' && rejectionModal.targetToolCall) {
    emit('rejectTool', {
      toolCall: rejectionModal.targetToolCall,
      message: props.message,
      reason: finalReason
    })
    const next = new Set(selectedToolKeys.value)
    next.delete(getToolKey(rejectionModal.targetToolCall, 0))
    selectedToolKeys.value = next
  } else if (rejectionModal.mode === 'selected') {
    emit('rejectSelectedTools', {
      toolCalls: rejectionModal.toolCalls,
      message: props.message,
      reason: finalReason
    })
    clearToolSelection()
  } else if (rejectionModal.mode === 'all') {
    emit('rejectAllTools', {
      message: props.message,
      reason: finalReason
    })
    clearToolSelection()
  }
  rejectionModal.isOpen = false
  rejectionModal.reason = ''
  rejectionModal.targetToolCall = null
  rejectionModal.toolCalls = []
}

const handleApproveSelected = () => {
  const selected = getSelectedToolCalls()
  if (selected.length === 0) return
  emit('approveSelectedTools', {
    toolCalls: selected,
    message: props.message
  })
  clearToolSelection()
}

const toggleResultView = (id: string) => {
  expandedResults[id] = !expandedResults[id]
}

const isUser = computed(() => props.message.role === 'user' || props.message.role === 'User')

// Thinking is actively streaming if generation is in progress, thinking text is present and message content has not started yet
const isActivelyThinking = computed(() => {
  return !isUser.value && props.message.is_streaming && Boolean(props.message.thinking) && !props.message.content
})

// Auto-scroll thinking container to the bottom so the last 3 lines are always visible
const scrollThinkingToBottom = () => {
  nextTick(() => {
    if (liveThinkingRef.value) {
      liveThinkingRef.value.scrollTop = liveThinkingRef.value.scrollHeight
    }
  })
}

watch(
  () => props.message.thinking,
  () => {
    if (isActivelyThinking.value) {
      scrollThinkingToBottom()
    }
  }
)

const formattedTime = computed(() => {
  if (!props.message.timestamp) return ''
  const date = new Date(props.message.timestamp)
  return isNaN(date.getTime()) ? '' : date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
})

const formatLanguageName = (lang?: string) => {
  if (!lang) return 'Código'
  const map: Record<string, string> = {
    js: 'JavaScript',
    javascript: 'JavaScript',
    ts: 'TypeScript',
    typescript: 'TypeScript',
    vue: 'Vue',
    rs: 'Rust',
    rust: 'Rust',
    py: 'Python',
    python: 'Python',
    json: 'JSON',
    html: 'HTML',
    css: 'CSS',
    scss: 'SCSS',
    sh: 'Shell',
    bash: 'Bash',
    zsh: 'Zsh',
    sql: 'SQL',
    md: 'Markdown',
    markdown: 'Markdown',
    toml: 'TOML',
    yaml: 'YAML',
    yml: 'YAML',
    c: 'C',
    cpp: 'C++',
    go: 'Go',
    java: 'Java',
    diff: 'Diff / Alterações',
    patch: 'Patch'
  }
  return map[lang.toLowerCase()] || (lang.charAt(0).toUpperCase() + lang.slice(1))
}

const getLanguageLabel = formatLanguageName

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true
})

const defaultLinkOpen = md.renderer.rules.link_open || function (tokens: any, idx: any, options: any, _env: any, self: any) {
  return self.renderToken(tokens, idx, options)
}
md.renderer.rules.link_open = function (tokens: any, idx: any, options: any, env: any, self: any) {
  tokens[idx].attrSet('target', '_blank')
  tokens[idx].attrSet('rel', 'noopener noreferrer')
  return defaultLinkOpen(tokens, idx, options, env, self)
}

const renderDiffBlock = (str: string, filename?: string) => {
  const lines = str.split('\n')
  let additions = 0
  let deletions = 0
  let detectedPath = filename || ''

  for (const line of lines) {
    if (!detectedPath) {
      const match = line.match(/^diff --git a\/([^\s]+) b\/([^\s]+)/) ||
                    line.match(/^\+\+\+ b\/([^\s]+)/) ||
                    line.match(/^\+\+\+ ([^\s]+)/)
      if (match && match[1]) {
        detectedPath = match[1]
      }
    }
    if (line.startsWith('+') && !line.startsWith('+++')) additions++
    else if (line.startsWith('-') && !line.startsWith('---')) deletions++
  }

  const renderedLines = lines.map((line: string) => {
    const escaped = md.utils.escapeHtml(line)
    if (line.startsWith('+') && !line.startsWith('+++')) {
      return `<div class="diff-line diff-add"><span class="diff-sign">+</span><span class="diff-text">${escaped.substring(1)}</span></div>`
    } else if (line.startsWith('-') && !line.startsWith('---')) {
      return `<div class="diff-line diff-del"><span class="diff-sign">-</span><span class="diff-text">${escaped.substring(1)}</span></div>`
    } else if (line.startsWith('@@')) {
      return `<div class="diff-line diff-hunk"><span class="diff-text">${escaped}</span></div>`
    } else {
      const sign = line.startsWith(' ') ? ' ' : ''
      const text = line.startsWith(' ') ? escaped.substring(1) : escaped
      return `<div class="diff-line diff-ctx"><span class="diff-sign">${sign}</span><span class="diff-text">${text}</span></div>`
    }
  }).join('')

  const encodedDiff = encodeURIComponent(str)
  const displayTitle = detectedPath || t('chat.file_modification')

  return `<div class="diff-viewer-container">
    <div class="diff-header">
      <div class="flex items-center gap-2 min-w-0">
        <svg class="w-3.5 h-3.5 text-amber-400 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><path d="M9 15h6"/><path d="M12 12v6"/></svg>
        <span class="font-bold text-[11.5px] text-slate-200 font-mono truncate" title="${displayTitle}">${displayTitle}</span>
        <span class="px-1.5 py-0.5 rounded text-[9.5px] font-mono font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/25">+${additions}</span>
        <span class="px-1.5 py-0.5 rounded text-[9.5px] font-mono font-bold bg-rose-500/15 text-rose-400 border border-rose-500/25">-${deletions}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          class="copy-code-widget-btn px-2.5 py-1 rounded-lg bg-[#161a2b] hover:bg-[#1e243d] border border-[#242c47] hover:border-indigo-500/40 text-[10.5px] text-slate-300 hover:text-white transition-all cursor-pointer flex items-center gap-1.5 active:scale-95 shadow-sm"
          data-code="${encodedDiff}"
          title="${t('chat.copy_diff')}"
        >
          <svg class="copy-icon-copy w-3 h-3 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
          <svg class="copy-icon-check w-3 h-3 text-emerald-400 hidden" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          <span class="copy-label font-medium">${t('chat.diff_copy_btn')}</span>
        </button>
      </div>
    </div>
    <div class="diff-body">${renderedLines}</div>
  </div>`
}

const renderCodeWidget = (str: string, lang?: string, filename?: string) => {
  let highlighted = ''
  if (lang && hljs.getLanguage(lang)) {
    try {
      highlighted = hljs.highlight(str, { language: lang, ignoreIllegals: true }).value
    } catch (e) {
      highlighted = md.utils.escapeHtml(str)
    }
  } else {
    highlighted = md.utils.escapeHtml(str)
  }

  let detectedFilename = filename ? filename.trim() : ''
  if (!detectedFilename && props.message?.content) {
    const match = props.message.content.match(/(?:arquivo|file|salvar\s+em|nome:\s*|salvar\s+como|chamado\s+)[`"']?([a-zA-Z0-9_\-\.\/]+\.[a-zA-Z0-9]+)[`"']?/i)
    if (match && match[1]) {
      detectedFilename = match[1]
    }
  }

  if (!detectedFilename && lang) {
    const l = lang.toLowerCase()
    if (l === 'html') detectedFilename = 'index.html'
    else if (l === 'css') detectedFilename = 'style.css'
    else if (l === 'js' || l === 'javascript') detectedFilename = 'script.js'
    else if (l === 'ts' || l === 'typescript') detectedFilename = 'app.ts'
    else if (l === 'json') detectedFilename = 'data.json'
    else if (l === 'python' || l === 'py') detectedFilename = 'main.py'
    else if (l === 'bash' || l === 'sh') detectedFilename = 'script.sh'
    else if (l === 'sql') detectedFilename = 'query.sql'
  }

  const encodedCode = encodeURIComponent(str)
  const escapedFilename = md.utils.escapeHtml(detectedFilename)
  const languageLabel = getLanguageLabel(lang || 'code')

  return `<div class="code-widget-container">
    <div class="code-widget-header">
      <div class="flex items-center gap-2 min-w-0">
        <span class="code-badge uppercase text-[10px] font-bold font-mono tracking-wider px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">${languageLabel}</span>
        ${escapedFilename ? `<span class="code-filename text-[11.5px] text-slate-300 font-mono truncate font-medium" title="${escapedFilename}">${escapedFilename}</span>` : ''}
      </div>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          class="copy-code-widget-btn px-2.5 py-1 rounded-lg bg-[#161a2b] hover:bg-[#1e243d] border border-[#242c47] hover:border-indigo-500/40 text-[10.5px] text-slate-300 hover:text-white transition-all cursor-pointer flex items-center gap-1.5 active:scale-95 shadow-sm"
          data-code="${encodedCode}"
          title="${t('chat.copy_code')}"
        >
          <svg class="copy-icon-copy w-3 h-3 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
          <svg class="copy-icon-check w-3 h-3 text-emerald-400 hidden" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          <span class="copy-label font-medium">${t('common.copy')}</span>
        </button>
      </div>
    </div>
    <div class="code-widget-body">
      <pre class="hljs"><code>${highlighted}</code></pre>
    </div>
  </div>`
}

md.renderer.rules.fence = (tokens: any[], idx: number) => {
  const token = tokens[idx]
  const info = token.info ? token.info.trim() : ''
  const str = token.content || ''

  let lang = ''
  let filename = ''
  if (info) {
    const parts = info.split(/[:\s]+/)
    lang = parts[0].toLowerCase()
    if (parts.length > 1) {
      filename = parts.slice(1).join(' ')
    }
  }

  // Check if this is a diff / patch block
  if (lang === 'diff' || lang === 'patch' || str.startsWith('diff --git') || (str.includes('--- a/') && str.includes('+++ b/'))) {
    return renderDiffBlock(str, filename)
  }

  return renderCodeWidget(str, lang, filename)
}

const handleMessageBodyClick = async (e: MouseEvent) => {
  const target = e.target as HTMLElement | null
  if (!target) return

  const copyBtn = target.closest('.copy-code-widget-btn')
  if (copyBtn) {
    e.preventDefault()
    e.stopPropagation()
    const encoded = copyBtn.getAttribute('data-code')
    if (encoded) {
      const code = decodeURIComponent(encoded)
      try {
        await navigator.clipboard.writeText(code)
        const label = copyBtn.querySelector('.copy-label')
        const copyIcon = copyBtn.querySelector('.copy-icon-copy')
        const checkIcon = copyBtn.querySelector('.copy-icon-check')

        if (label) label.textContent = t('chat.code_copied')
        if (copyIcon) copyIcon.classList.add('hidden')
        if (checkIcon) checkIcon.classList.remove('hidden')
        copyBtn.classList.add('text-emerald-400', 'border-emerald-500/40')

        setTimeout(() => {
          if (label) label.textContent = t('common.copy')
          if (copyIcon) copyIcon.classList.remove('hidden')
          if (checkIcon) checkIcon.classList.add('hidden')
          copyBtn.classList.remove('text-emerald-400', 'border-emerald-500/40')
        }, 2000)
      } catch (err) {
        console.error('Erro ao copiar código:', err)
      }
    }
  }
}

const sanitizeText = (txt?: string | null) => {
  if (!txt) return ''
  return txt
    // Strip memory & skill XML tags (both self-closing and unclosed streaming tails)
    .replace(/<memoria\b[^>]*\/?>/gi, '')
    .replace(/<memory\b[^>]*\/?>/gi, '')
    .replace(/<memorizar\b[^>]*\/?>/gi, '')
    .replace(/<habilidade\b[^>]*\/?>/gi, '')
    .replace(/<skill\b[^>]*\/?>/gi, '')
    .replace(/<forget\b[^>]*\/?>/gi, '')
    .replace(/<esquecer\b[^>]*\/?>/gi, '')
    .replace(/<remover\b[^>]*\/?>/gi, '')
    // Strip trailing incomplete tags while streaming
    .replace(/<memoria\b[^<]*$/gi, '')
    .replace(/<memory\b[^<]*$/gi, '')
    .replace(/<memorizar\b[^<]*$/gi, '')
    .replace(/<habilidade\b[^<]*$/gi, '')
    .replace(/<skill\b[^<]*$/gi, '')
    .replace(/<forget\b[^<]*$/gi, '')
    .replace(/<esquecer\b[^<]*$/gi, '')
    .replace(/<remover\b[^<]*$/gi, '')
    // Neutralize meta tags (meta refresh/redirection), base tags and script tags
    .replace(/<meta\b[^>]*\/?>/gi, '')
    .replace(/<base\b[^>]*\/?>/gi, '')
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    // Strip model control & special tokens (audio, video, vision, channels, turn delimiters)
    .replace(/<\|?(?:audio|video|image|vision|channel|thought|start_of_turn|end_of_turn|im_start|im_end|endoftext|eot_id|start_header_id|end_header_id)[^>]*\|?>/gi, '')
    .replace(/<\|[a-zA-Z0-9_\-:]+\|?>/gi, '')
    .replace(/<[a-zA-Z0-9_\-]+(?:\|>|\|)/gi, '')
    .trim()
}

const makeHtmlSafe = (html: string): string => {
  if (!html) return ''
  return html
    .replace(/<meta\b[^>]*\/?>/gi, '')
    .replace(/<base\b[^>]*\/?>/gi, '')
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi, '')
    .replace(/<iframe\b[^>]*\/?>/gi, '')
    .replace(/href\s*=\s*["']\s*javascript:[^"']*["']/gi, 'href="#"')
}

const renderMarkdown = (text?: string | null) => {
  if (!text) return ''
  return makeHtmlSafe(md.render(sanitizeText(text)))
}

const cleanContentText = computed(() => {
  let rawContent = props.message.content || ''
  if (isUser.value) {
    if (props.message.display_text) {
      rawContent = props.message.display_text
    } else if (rawContent) {
      // Clean temporal tags and legacy raw file delimiters
      rawContent = rawContent.replace(/^\[Horário local: [^\]]+\]\n?/, '')
      rawContent = (rawContent.split(/\n\n--- (?:\[Arquivo Anexado:|\[Documento PDF:|\[Transcrição de Áudio:|Documentos e Arquivos Anexados)/)[0] || '').trim()
    }
  }
  return sanitizeText(rawContent)
})

const renderedContent = computed(() => {
  const cleanContent = cleanContentText.value
  if (!cleanContent) {
    if (props.message.tool_calls && props.message.tool_calls.some((t: any) => t.status === 'pending_approval' || t.status === 'streaming')) {
      return ''
    }
    if (isActivelyThinking.value) {
      return ''
    }
    if (props.message.is_streaming) {
      return `<span class="inline-flex items-center gap-1.5 select-none"><span class="w-2 h-2 rounded-full bg-indigo-400 animate-ping"></span><span class="text-xs text-indigo-400 font-mono">${t('chat.generating_response')}</span></span>`
    }
    return ''
  }
  return makeHtmlSafe(md.render(cleanContent))
})

const hasMemoryTagsOnly = computed(() => {
  if (isUser.value || props.message.is_streaming) return false
  const raw = props.message.content || ''
  if (!raw.trim()) return false
  const hasMem = /<memoria\b|<memory\b|<memorizar\b|<forget\b|<esquecer\b|<remover\b/i.test(raw)
  return hasMem && !cleanContentText.value
})

const hasArguments = (args: any) => {
  if (!args) return false
  if (typeof args === 'object') {
    return Object.keys(args).length > 0
  }
  if (typeof args === 'string') {
    return args.trim() !== '' && args.trim() !== '{}'
  }
  return false
}

const formatJson = (val: any) => {
  if (!val) return '{}'
  if (typeof val === 'string') {
    try {
      const parsed = JSON.parse(val)
      return JSON.stringify(parsed, null, 2)
    } catch {
      return val
    }
  }
  return JSON.stringify(val, null, 2)
}

const getToolDescription = (tc: any) => {
  if (tc.name === 'update_procedural_skill' || tc.name === 'edit_procedural_skill') return t('memory.skill_edit_desc')
  if (isProceduralSkillCall(tc)) return t('memory.skill_create_desc')
  if (tc.name === 'run_command' || tc.name === 'run_skill_command') return t('memory.skill_command_desc')
  if (tc.name === 'run_skill_script') return t('memory.skill_script_desc')
  if (tc.name === 'atena_search_episodes') return t('memory.native_search_episodes_desc')
  if (tc.name === 'atena_read_episode') return t('memory.native_read_episode_desc')
  if (tc.name === 'atena_search_memory') return t('memory.native_search_memory_desc')
  if (tc.name === 'atena_web_search') return t('memory.native_web_search_desc')
  if (tc.name === 'atena_fetch_webpage') return t('memory.native_fetch_webpage_desc')
  if (tc.name === 'atena_scratchpad_write') return t('memory.native_scratchpad_write_desc')
  if (tc.name === 'atena_scratchpad_read') return t('memory.native_scratchpad_read_desc')
  if (tc.name === 'atena_scratchpad_clear') return t('memory.native_scratchpad_clear_desc')
  if (tc.name === 'atena_schedule_task') return t('memory.native_schedule_task_desc')
  if (tc.name === 'atena_list_scheduled_tasks') return t('memory.native_list_scheduled_tasks_desc')
  if (tc.name === 'atena_cancel_scheduled_task') return t('memory.native_cancel_scheduled_task_desc')
  if (tc.name === 'atena_run_scheduled_task') return t('memory.native_run_scheduled_task_desc')
  if (tc.description) return tc.description
  if (tc.status === 'pending_approval') return t('chat.awaiting_authorization')
  if (tc.status === 'executing') return t('chat.tool_call_executing_msg')
  if (tc.status === 'completed') return t('chat.status_executed')
  if (tc.status === 'rejected') return t('chat.status_rejected')
  return tc.name || ''
}

const copyText = async (txt?: string | null) => {
  try {
    await navigator.clipboard.writeText(txt || '')
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy text:', err)
  }
}
</script>
