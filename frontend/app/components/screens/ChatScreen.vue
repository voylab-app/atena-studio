<template>
  <div class="h-full flex flex-col bg-[#090a0f] overflow-hidden relative"
    @dragenter.stop.prevent="handleDragEnter"
    @dragover.stop.prevent="handleDragOver"
    @dragleave.stop.prevent="handleDragLeave"
    @drop.stop.prevent="handleFileDrop">
    <!-- Video Blocked / Error Toast Notification -->
    <transition enter-active-class="transition duration-300 ease-out"
      enter-from-class="transform -translate-y-4 opacity-0" enter-to-class="transform translate-y-0 opacity-100"
      leave-active-class="transition duration-200 ease-in" leave-from-class="transform translate-y-0 opacity-100"
      leave-to-class="transform -translate-y-4 opacity-0">
      <div v-if="toastMessage"
        class="absolute top-4 left-1/2 -translate-x-1/2 z-50 max-w-md w-auto px-4 py-2.5 rounded-xl bg-[#1a1424]/95 border border-amber-500/50 shadow-2xl backdrop-blur-md flex items-center gap-3 text-xs text-amber-200 select-none animate-in fade-in">
        <AlertTriangle class="w-4 h-4 text-amber-400 flex-shrink-0" />
        <span class="flex-1 font-medium leading-tight">{{ toastMessage }}</span>
        <button @click="toastMessage = null"
          class="p-1 rounded-lg hover:bg-white/10 text-amber-300 hover:text-white transition-colors cursor-pointer">
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </transition>

    <!-- Drag Overlay Indicator -->
    <div v-if="isDragging"
      class="chat-drag-overlay absolute inset-0 bg-indigo-950/80 backdrop-blur-sm z-30 border-2 border-dashed border-indigo-400 m-3 rounded-2xl flex flex-col items-center justify-center text-center p-6 pointer-events-none select-none transition-all">
      <div class="drag-icon-box p-4 rounded-2xl bg-indigo-500/20 text-indigo-300 mb-3 border border-indigo-500/30 animate-bounce">
        <UploadCloud class="w-8 h-8" />
      </div>
      <h3 class="drag-title text-base font-bold text-white">{{ $t('chat.drop_files') }}</h3>
      <p class="drag-desc text-xs text-indigo-200 mt-1 max-w-sm">
        {{ $t('chat.drop_files_desc') }}
      </p>
      <span
        class="drag-warning mt-2 text-[10.5px] text-rose-300/80 bg-rose-950/50 px-2.5 py-0.5 rounded-full border border-rose-500/30">
        {{ $t('chat.videos_not_supported_badge') }}
      </span>
    </div>

    <!-- Messages Scrollable List -->
    <div ref="messagesContainer" class="flex-1 overflow-y-auto p-4 space-y-4 relative" @scroll="handleScroll">
      <!-- Empty Welcome Hero View -->
      <div v-if="!currentSession || currentSession.messages.length === 0"
        class="h-full flex flex-col items-center justify-center text-center p-6 space-y-6 select-none max-w-2xl mx-auto">
        <!-- Logo with refined ambient glow -->
        <div class="relative group">
          <div
            class="absolute -inset-2 rounded-3xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-teal-500/10 blur-xl opacity-70 group-hover:opacity-100 transition-opacity duration-500">
          </div>
          <div
            class="relative w-16 h-16 rounded-2xl overflow-hidden shadow-2xl shadow-indigo-500/20 ring-1 ring-white/10 flex items-center justify-center bg-[#0d101d]">
            <img src="/atena_logo.png" alt="Atena Studio" class="w-full h-full object-cover" />
          </div>
        </div>

        <!-- Greeting & Subtitle -->
        <div class="space-y-2">
          <h2 class="text-2xl font-bold text-slate-100 tracking-tight">
            {{ welcomeGreeting }}
          </h2>
          <p class="text-xs sm:text-sm text-slate-400 leading-relaxed max-w-md mx-auto">
            {{ $t('chat.welcome_subtitle') }}
          </p>
        </div>

        <!-- Capability Badges -->
        <div class="flex flex-wrap items-center justify-center gap-2">
          <span
            v-if="currentSession?.archived"
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-amber-500/15 border border-amber-500/35 text-[11px] font-semibold text-amber-300 shadow-sm">
            <Archive class="w-3.5 h-3.5 text-amber-400" />
            <span>{{ $t('chat.archived_badge') }}</span>
          </span>
          <span
            v-if="currentSession?.is_private"
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-violet-500/15 border border-violet-500/35 text-[11px] font-semibold text-violet-300 shadow-sm animate-pulse">
            <EyeOff class="w-3.5 h-3.5 text-violet-400" />
            <span>{{ $t('chat.private_badge_hero') }}</span>
          </span>
          <span
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-[#111422] border border-[#1e2338] text-[11px] font-medium text-slate-300 shadow-sm">
            <Eye class="w-3.5 h-3.5 text-sky-400" />
            <span>{{ $t('chat.multimodal_vision') }}</span>
          </span>
          <span
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-[#111422] border border-[#1e2338] text-[11px] font-medium text-slate-300 shadow-sm">
            <FileText class="w-3.5 h-3.5 text-rose-400" />
            <span>{{ $t('chat.pdf_docs') }}</span>
          </span>
          <span
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-[#111422] border border-[#1e2338] text-[11px] font-medium text-slate-300 shadow-sm">
            <Mic class="w-3.5 h-3.5 text-purple-400" />
            <span>{{ $t('chat.whisper_transcription') }}</span>
          </span>
        </div>

        <!-- Quick Starter Prompt Cards -->
        <div class="w-full grid grid-cols-1 sm:grid-cols-2 gap-3 text-left pt-2">
          <button v-for="(starter, idx) in starterPrompts" :key="idx" @click="useStarterPrompt(starter)"
            class="p-3.5 rounded-2xl bg-[#101422]/90 hover:bg-[#14192b] border border-[#1d2338] hover:border-indigo-500/40 text-xs text-slate-300 hover:text-white transition-all duration-200 group shadow-sm flex flex-col justify-between gap-2.5 cursor-pointer hover:shadow-xl hover:shadow-indigo-500/5 hover:-translate-y-0.5">
            <div class="flex items-center justify-between w-full">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg flex items-center justify-center shadow-sm"
                  :class="[starter.bg, starter.border]">
                  <component :is="starter.icon" class="w-3.5 h-3.5" :class="starter.color" />
                </div>
                <span class="font-semibold text-xs text-slate-200 group-hover:text-indigo-300 transition-colors">
                  {{ starter.title }}
                </span>
              </div>
              <ArrowUpRight
                class="w-3.5 h-3.5 text-slate-500 group-hover:text-indigo-400 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-all" />
            </div>
            <p
              class="text-[11.5px] text-slate-400 group-hover:text-slate-300 line-clamp-2 leading-relaxed transition-colors">
              {{ starter.prompt }}
            </p>
          </button>
        </div>
      </div>

      <!-- Chat Messages List -->
      <div v-else class="max-w-4xl lg:max-w-5xl mx-auto w-full space-y-3 px-1 sm:px-2">
        <!-- Archived Conversation Floating Banner -->
        <div v-if="currentSession?.archived"
          class="flex items-center justify-between gap-3 px-3.5 py-2.5 rounded-2xl bg-amber-950/40 border border-amber-500/35 text-amber-200 text-xs shadow-md backdrop-blur-md mb-3 select-none">
          <div class="flex items-center gap-2.5 min-w-0 flex-1">
            <Archive class="w-4 h-4 text-amber-400 flex-shrink-0" />
            <div class="truncate">
              <strong class="font-semibold text-amber-300">{{ $t('chat.archived_banner') }}</strong>
              <span class="text-amber-200/80 ml-1.5 text-[11px] hidden sm:inline">{{ $t('chat.archived_banner_desc') }}</span>
            </div>
          </div>
          <button
            @click="$emit('unarchiveSession', currentSession.id)"
            class="px-3 py-1 rounded-xl bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-200 hover:text-white text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer flex-shrink-0 active:scale-95 shadow-sm"
            :title="$t('chat.unarchive_tooltip')"
          >
            <ArchiveRestore class="w-3.5 h-3.5 text-amber-300" />
            <span>{{ $t('chat.unarchive') }}</span>
          </button>
        </div>

        <!-- Private Mode Floating Notice -->
        <div v-if="currentSession?.is_private"
          class="flex items-center gap-2.5 px-3.5 py-2 rounded-2xl bg-violet-950/40 border border-violet-500/30 text-violet-200 text-xs shadow-md backdrop-blur-md mb-3 select-none">
          <EyeOff class="w-4 h-4 text-violet-400 flex-shrink-0" />
          <div class="flex-1 text-[11.5px] leading-relaxed">
            <strong class="font-semibold text-violet-300">Chat Privado Ativo:</strong> Nenhuma memória ou registro do diário anterior é consultado, e nenhuma informação desta conversa será gravada no seu cérebro de memória ou no diário de bordo.
          </div>
        </div>

        <ChatBubble v-for="(msg, index) in currentSession.messages" :key="msg.id || `msg-idx-${index}`" :message="msg"
          :showEfficiencyMetrics="showEfficiencyMetrics"
          :mcpTools="mcpTools"
          @approveTool="$emit('approveTool', $event)"
          @rejectTool="$emit('rejectTool', $event)"
          @reExecuteTool="$emit('reExecuteTool', $event)"
          @approveSelectedTools="$emit('approveSelectedTools', $event)"
          @rejectSelectedTools="$emit('rejectSelectedTools', $event)"
          @approveAllTools="$emit('approveAllTools', $event)"
          @rejectAllTools="$emit('rejectAllTools', $event)" @resendMessage="$emit('resendMessage', $event)"
          @continueGeneration="handleContinueGeneration"
          @openParams="$emit('openParams')"
          @toolLabelUpdated="$emit('refreshTools')"
          @deleteMessage="handleDeleteMessage(msg, index)"
          @retryLastMessage="handleRetryLastMessage" @selectTab="$emit('selectTab', $event)" />
      </div>
    </div>

    <!-- Bottom Input Area (LM Studio Unified Capsule Style) -->
    <div class="px-3 sm:px-4 pb-4 pt-1 bg-gradient-to-t from-[#090a0f] via-[#090a0f]/90 to-transparent z-20">
      <div
        class="max-w-4xl lg:max-w-5xl mx-auto flex flex-col bg-[#111422]/95 backdrop-blur-xl rounded-2xl border border-[#1e2439] focus-within:border-indigo-500/50 focus-within:ring-2 focus-within:ring-indigo-500/15 transition-all shadow-2xl shadow-black/40 relative">
        <!-- Hidden Native File Input (Accepts all documents, images, audio, XML, code, etc.) -->
        <input ref="fileInputRef" type="file" multiple class="hidden" @change="handleFileInputChange" />

        <!-- Attachment Chips List (Inside Capsule) -->
        <div v-if="attachments.length > 0" class="flex flex-wrap gap-2 p-2.5 pb-0">
          <div v-for="(att, idx) in attachments" :key="att.id || idx"
            class="flex items-center gap-2 p-1.5 pr-2.5 rounded-xl bg-[#15192b] border border-[#262e45] text-xs text-slate-200 shadow-sm group transition-all"
            :class="{ 'opacity-75 border-indigo-500/40': att.isProcessing }">
            <!-- Image thumbnail preview -->
            <img v-if="att.kind === 'image'" :src="att.data_url" alt="Preview"
              class="w-7 h-7 rounded-lg object-cover border border-[#3b4566]" />

            <!-- XML Badge -->
            <div v-else-if="att.kind === 'xml' || (att.name && att.name.toLowerCase().endsWith('.xml'))"
              class="w-7 h-7 rounded-lg bg-amber-500/20 border border-amber-500/40 flex items-center justify-center text-amber-300 font-bold text-[9px] flex-shrink-0 tracking-wider">
              XML
            </div>

            <!-- JSON Badge -->
            <div v-else-if="att.kind === 'json' || (att.name && att.name.toLowerCase().endsWith('.json'))"
              class="w-7 h-7 rounded-lg bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-300 font-bold text-[9px] flex-shrink-0 tracking-wider">
              JSON
            </div>

            <!-- CSV Badge -->
            <div
              v-else-if="att.kind === 'csv' || (att.name && (att.name.toLowerCase().endsWith('.csv') || att.name.toLowerCase().endsWith('.tsv')))"
              class="w-7 h-7 rounded-lg bg-teal-500/20 border border-teal-500/40 flex items-center justify-center text-teal-300 font-bold text-[9px] flex-shrink-0 tracking-wider">
              CSV
            </div>

            <!-- PDF Icon -->
            <div v-else-if="att.kind === 'pdf' || (att.name && att.name.toLowerCase().endsWith('.pdf'))"
              class="w-7 h-7 rounded-lg bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400 font-bold text-[9px] flex-shrink-0">
              PDF
            </div>

            <!-- Audio Icon -->
            <div v-else-if="att.kind === 'audio'"
              class="w-7 h-7 rounded-lg bg-purple-500/15 border border-purple-500/30 flex items-center justify-center text-purple-400 flex-shrink-0">
              <Mic v-if="!att.isProcessing" class="w-4 h-4" />
              <Loader2 v-else class="w-4 h-4 animate-spin text-purple-300" />
            </div>

            <!-- Text / Code Icon -->
            <div v-else
              class="w-7 h-7 rounded-lg bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 flex-shrink-0">
              <FileCode2 class="w-4 h-4" />
            </div>

            <div class="truncate max-w-[180px]">
              <p class="truncate font-medium text-[11px] leading-tight text-slate-200">{{ att.name }}</p>
              <div class="flex items-center gap-1.5 text-[9.5px] font-mono text-slate-400 mt-0.5">
                <span>{{ formatBytes(att.size_bytes) }}</span>
                <span v-if="att.pages" class="text-rose-300">• {{ att.pages }} {{ $t('chat.page_abbr') }}</span>
                <span v-if="att.duration" class="text-purple-300">• {{ formatDuration(att.duration) }}</span>
                <span v-if="att.isProcessing" class="text-indigo-400 flex items-center gap-1">
                  <Loader2 class="w-2.5 h-2.5 animate-spin" />
                  {{ att.statusText || $t('chat.processing') }}
                </span>
              </div>
            </div>

            <button @click="removeAttachment(idx)"
              class="p-1 rounded-md hover:bg-rose-500/20 text-slate-400 hover:text-rose-300 transition-colors ml-1 cursor-pointer"
              :title="$t('chat.remove_attachment')">
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <!-- Pending Tools Warning Banner (Inside Capsule) -->
        <div v-if="hasPendingTools" class="mx-3 mt-2 px-3 py-1.5 rounded-xl bg-amber-500/10 border border-amber-500/25 flex items-center justify-between text-xs text-amber-300 select-none">
          <div class="flex items-center gap-2 min-w-0">
            <ShieldAlert class="w-4 h-4 text-amber-400 shrink-0" />
            <span class="truncate font-medium">
              {{ pendingToolsCount > 1 ? $t('chat.pending_tools_banner_many', { count: pendingToolsCount }) : $t('chat.pending_tools_banner_one') }}
            </span>
          </div>
          <button
            type="button"
            @click="scrollToPendingTools"
            class="text-[11px] font-semibold text-amber-300 hover:text-amber-200 underline decoration-amber-400/50 cursor-pointer shrink-0 ml-2"
          >
            {{ $t('chat.pending_tools_review_btn') }}
          </button>
        </div>

        <!-- Top Input Row: Textarea -->
        <div class="px-3 pt-2.5 pb-1 flex items-start gap-2">
          <textarea ref="textareaRef" v-model="inputText" rows="1"
            :placeholder="hasPendingTools ? $t('chat.pending_tools_placeholder') : $t('chat.input_placeholder')"
            class="flex-1 bg-transparent px-1 py-1 text-sm text-slate-100 placeholder-slate-500 resize-none outline-none min-h-[38px] max-h-40 font-sans select-text leading-relaxed"
            @keydown="handleTextareaKeyDown" @input="handleTextareaInput" @paste="handlePaste"></textarea>
        </div>

        <!-- Bottom Controls Bar (Inside Capsule like LM Studio) -->
        <div
          class="px-3 pb-2.5 pt-1.5 flex items-center justify-between gap-2 border-t border-[#1a1f33]/60 select-none">
          <!-- Left Actions & Capability Chips -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <!-- Attach Button (+) -->
            <button @click="triggerFileInput"
              class="w-7 h-7 flex items-center justify-center rounded-xl bg-[#141828] hover:bg-[#1c2238] border border-[#1e2439] hover:border-indigo-500/40 text-slate-400 hover:text-indigo-300 transition-all cursor-pointer active:scale-95 shadow-sm"
              :title="$t('chat.attach_file')">
              <Paperclip class="w-3.5 h-3.5" />
            </button>

            <!-- Persona Selector Chip -->
            <button @click="showPersonasModal = true"
              class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-medium border transition-all cursor-pointer select-none active:scale-95 shadow-sm bg-gradient-to-r from-indigo-500/10 to-purple-500/10 text-indigo-300 border-indigo-500/30 hover:border-indigo-400 hover:text-white"
              :title="$t('chat.persona_tooltip', { name: activePersona.name })">
              <Sparkles class="w-3 h-3 text-indigo-400" />
              <span class="max-w-[90px] truncate font-medium">{{ activePersona.name }}</span>
            </button>

            <!-- Interactive Reasoning Effort Selector (Modelos com variantes de esforço: High, Medium, Low) -->
            <div v-if="currentModelVariants.length > 1" class="relative">
              <button
                type="button"
                @click="showReasoningDropdown = !showReasoningDropdown"
                class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-medium border transition-all cursor-pointer select-none active:scale-95 shadow-sm bg-purple-500/15 text-purple-300 border-purple-500/30 hover:bg-purple-500/25 hover:border-purple-400"
                :title="$t('chat.reasoning_effort_tooltip', { effort: currentEffortLabel })"
              >
                <Brain class="w-3.5 h-3.5 text-purple-400 shrink-0" />
                <span>{{ $t('chat.reasoning') }}:</span>
                <span class="font-bold text-amber-300">{{ currentEffort === 'high' ? 'High' : currentEffort === 'low' ? 'Low' : 'Medium' }}</span>
                <ChevronDown class="w-2.5 h-2.5 text-purple-400/80 ml-0.5 transition-transform" :class="{ 'rotate-180': showReasoningDropdown }" />
              </button>

              <!-- Backdrop to close on click outside -->
              <div
                v-if="showReasoningDropdown"
                @click="showReasoningDropdown = false"
                class="fixed inset-0 z-40"
              ></div>

              <!-- Reasoning Effort Popover Menu -->
              <div
                v-if="showReasoningDropdown"
                class="absolute bottom-full left-0 mb-2 w-56 p-1.5 rounded-2xl bg-[#0e111a] border border-[#232942] shadow-2xl shadow-black/90 z-50 space-y-1 backdrop-blur-md dropdown-enter"
              >
                <div class="px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-400 flex items-center justify-between border-b border-[#1b2135] mb-1">
                  <span>{{ $t('chat.reasoning_effort') }}</span>
                  <Brain class="w-3 h-3 text-purple-400" />
                </div>

                <button
                  v-for="v in currentModelVariants"
                  :key="v.effort"
                  type="button"
                  @click="handleSelectEffort(v)"
                  class="w-full flex items-center justify-between px-2.5 py-2 rounded-xl text-xs transition-all cursor-pointer text-left font-medium"
                  :class="[
                    v.effort === currentEffort
                      ? 'bg-indigo-600 text-white font-semibold shadow-md shadow-indigo-600/30'
                      : 'text-slate-300 hover:bg-[#161c2e] hover:text-white'
                  ]"
                >
                  <div class="flex items-center gap-2">
                    <Check v-if="v.effort === currentEffort" class="w-3.5 h-3.5 text-white shrink-0" />
                    <span v-else class="w-3.5 h-3.5 shrink-0"></span>
                    <span>{{ v.label }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- Thinking Toggle Chip (Para modelos locais ou modelos sem variantes múltiplas) -->
            <button
              v-else
              @click="toggleThinking"
              :class="[
                'flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-medium border transition-all cursor-pointer select-none active:scale-95 shadow-sm',
                params?.enable_thinking !== false
                  ? 'bg-purple-500/15 text-purple-300 border-purple-500/30 hover:bg-purple-500/25'
                  : 'bg-[#141828] text-slate-500 border-[#1e2439] hover:text-slate-300'
              ]"
              :title="params?.enable_thinking !== false ? $t('chat.think_toggle_off') : $t('chat.think_toggle_on')"
            >
              <Brain class="w-3 h-3 text-purple-400" />
              <span>{{ params?.enable_thinking !== false ? $t('chat.think_on') : $t('chat.think_off') }}</span>
            </button>

            <!-- MCP Tools Interactive Chip -->
            <button @click="showToolsModal = true" :class="[
              'flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-medium border transition-all cursor-pointer select-none active:scale-95 shadow-sm',
              activeMcpToolsCount > 0
                ? 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30 hover:bg-indigo-500/25'
                : 'bg-[#141828] text-slate-500 border-[#1e2439] hover:text-slate-300'
            ]"
              :title="activeModel && activeModel.supports_tools === false && activeMcpToolsCount > 0 ? $t('chat.tools_no_function_calling') : $t('chat.tools_title')">
              <Wrench class="w-3 h-3 text-indigo-400" />
              <span>Tools</span>
              <span v-if="activeMcpToolsCount > 0"
                class="px-1 py-0.2 rounded-md bg-indigo-500/30 text-[9.5px] font-bold text-indigo-200">{{
                activeMcpToolsCount }}</span>
            </button>

            <!-- Cache Efficiency Stats Chip -->
            <button v-if="showEfficiencyMetrics !== false" @click="showEfficiencyModal = true"
              class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[11px] font-medium border transition-all cursor-pointer select-none active:scale-95 bg-teal-500/15 text-teal-300 border-teal-500/30 hover:bg-teal-500/25 shadow-sm hidden sm:flex"
              :title="$t('chat.efficiency_title')">
              <Zap class="w-3 h-3 text-teal-400" />
              <span>{{ $t('chat.efficiency_btn') }}</span>
            </button>
          </div>

          <!-- Right Actions (Context Tokens, More & Send Button) -->
          <div class="flex items-center gap-2 flex-shrink-0">
            <!-- Context Tokens Badge with Dropdown -->
            <div class="relative">
              <button type="button" @click="toggleContextDropdown" :class="[
                'flex items-center gap-1.5 px-2.5 py-1 rounded-xl text-[10.5px] font-mono font-medium border transition-all cursor-pointer select-none active:scale-95',
                isContextFull
                  ? 'bg-rose-500/15 text-rose-400 border-rose-500/30'
                  : contextPctNumber >= 80
                    ? 'bg-amber-500/15 text-amber-400 border-amber-500/30'
                    : 'bg-[#141828] text-slate-400 border-[#1e2439] hover:border-indigo-500/40 hover:text-slate-200'
              ]" title="Limite de Janela de Contexto">
                <span class="font-bold text-slate-200">{{ contextUsed }}/{{ contextLimit >= 1024 ?
                  Math.round(contextLimit / 1024) + 'k' : contextLimit }}</span>
                <span class="text-slate-500 text-[9.5px]">tok</span>
                <span :class="[
                  'text-[9.5px] font-bold px-1.5 py-0.2 rounded-md font-mono',
                  isContextFull
                    ? 'text-rose-400 bg-rose-500/20'
                    : contextPctNumber >= 80
                      ? 'text-amber-400 bg-amber-500/20'
                      : 'text-indigo-300 bg-indigo-500/15'
                ]">
                  {{ contextPct }}%
                </span>
                <ChevronDown class="w-2.5 h-2.5 text-slate-500 ml-0.5" />
              </button>

              <!-- Quick Context Dropdown Popover -->
              <div v-if="showContextDropdown"
                class="absolute bottom-full right-0 mb-2 w-72 p-3.5 rounded-2xl bg-[#0e111a] border border-[#22283e] shadow-2xl z-50 space-y-3 backdrop-blur-md dropdown-enter">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-slate-100 flex items-center gap-1.5">
                    <SlidersHorizontal class="w-3.5 h-3.5 text-indigo-400" />
                    <span>Limite de Contexto</span>
                  </span>
                  <button type="button" @click="showContextDropdown = false"
                    class="text-slate-400 hover:text-slate-200 p-0.5 rounded-lg hover:bg-[#181d2e] cursor-pointer">
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>

                <!-- Context Usage Bar & Percentage -->
                <div class="space-y-1.5 p-2 rounded-xl bg-[#141826]/70 border border-[#1e2439]">
                  <div class="flex items-center justify-between text-[11px] font-mono">
                    <span class="text-slate-400">Uso Atual</span>
                    <div class="flex items-center gap-1.5">
                      <span class="font-bold text-slate-200">{{ contextUsed }} / {{ contextLimit >= 1024 ? Math.round(contextLimit / 1024) + 'k' : contextLimit }} tok</span>
                      <span :class="[
                        'px-1.5 py-0.5 rounded text-[10px] font-bold',
                        isContextFull ? 'text-rose-400 bg-rose-500/20' : contextPctNumber >= 80 ? 'text-amber-400 bg-amber-500/20' : 'text-indigo-300 bg-indigo-500/20'
                      ]">{{ contextPct }}%</span>
                    </div>
                  </div>
                  <div class="h-1.5 w-full bg-[#0a0d14] rounded-full overflow-hidden border border-white/5">
                    <div class="h-full transition-all duration-300 rounded-full"
                      :class="isContextFull ? 'bg-rose-500' : contextPctNumber >= 80 ? 'bg-amber-500' : 'bg-indigo-500'"
                      :style="{ width: `${Math.min(contextPctNumber, 100)}%` }" />
                  </div>
                </div>

                <!-- Presets Grid -->
                <div class="grid grid-cols-3 gap-1.5">
                  <button v-for="p in [2048, 4096, 8192, 16384, 32768, 65536]" :key="p" type="button"
                    @click="setQuickContext(p)" :class="[
                      'py-1.5 rounded-xl text-[10.5px] font-mono font-semibold border transition-all cursor-pointer text-center',
                      contextLimit === p
                        ? 'bg-indigo-600 text-white border-indigo-500 shadow-sm'
                        : 'bg-[#141826] text-slate-400 border-[#1e2439] hover:text-slate-200 hover:bg-[#1a2034]'
                    ]">
                    {{ p >= 1024 ? `${Math.round(p / 1024)}k` : p }}
                  </button>
                </div>

                <!-- Custom Context Input -->
                <div class="flex items-center gap-1.5 pt-2 border-t border-[#1e2439]">
                  <input type="number" v-model.number="customContextInput" step="512" min="512" max="1048576"
                    placeholder="Ex: 8192"
                    class="flex-1 px-3 py-1.5 rounded-xl bg-[#141826] border border-[#1e2439] focus:border-indigo-500 text-xs font-mono text-slate-100 outline-none"
                    @keydown.enter="applyCustomContext" />
                  <button type="button" @click="applyCustomContext"
                    class="px-3 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-all cursor-pointer active:scale-95">
                    Definir
                  </button>
                </div>

                <p class="text-[10px] text-slate-400 leading-tight">
                  💡 Contextos menores reduzem o uso de VRAM e aceleram a inferência.
                </p>
              </div>
            </div>

            <!-- More Options Button (Export + Clear) -->
            <div class="relative">
              <button type="button" @click="showMoreActions = !showMoreActions"
                class="w-7 h-7 flex items-center justify-center rounded-xl bg-[#141828] text-slate-400 hover:text-slate-200 hover:bg-[#1c2238] transition-all cursor-pointer active:scale-95 border border-[#1e2439]"
                title="Mais opções">
                <MoreHorizontal class="w-3.5 h-3.5" />
              </button>

              <!-- Dropdown Menu -->
              <div v-if="showMoreActions"
                class="absolute bottom-full right-0 mb-2 w-48 rounded-2xl bg-[#0e111a] border border-[#22283e] shadow-2xl z-50 overflow-hidden dropdown-enter">
                <button v-if="currentSession && currentSession.messages && currentSession.messages.length > 0"
                  @click="exportMarkdown(); showMoreActions = false"
                  class="w-full flex items-center gap-2.5 px-3.5 py-2.5 text-xs text-slate-300 hover:text-white hover:bg-[#181d2e] transition-all cursor-pointer">
                  <Download class="w-3.5 h-3.5 text-indigo-400" />
                  <span>Exportar (.md)</span>
                </button>
                <div class="h-px bg-[#1e2439] mx-2"></div>
                <button @click="$emit('clearChat'); showMoreActions = false"
                  class="w-full flex items-center gap-2.5 px-3.5 py-2.5 text-xs text-slate-400 hover:text-rose-300 hover:bg-rose-500/10 transition-all cursor-pointer">
                  <Trash2 class="w-3.5 h-3.5" />
                  <span>Limpar conversa</span>
                </button>
              </div>
            </div>

            <!-- Send / Stop Circular Action Button (LM Studio Style) -->
            <button v-if="isGenerating" @click="$emit('stopGeneration')"
              class="stop-generation-btn w-8 h-8 rounded-xl bg-rose-600 hover:bg-rose-500 text-white flex items-center justify-center shadow-md shadow-rose-600/30 transition-all active:scale-95 cursor-pointer"
              :title="$t('chat.stop_generating')">
              <Square class="w-3.5 h-3.5 fill-white text-white" />
            </button>

            <button v-else @click="handleSend" :disabled="!canSend" :class="[
              'w-8 h-8 rounded-xl flex items-center justify-center transition-all shadow-sm',
              !canSend
                ? 'bg-[#141828] text-slate-600 cursor-not-allowed border border-[#1e2439]'
                : 'bg-indigo-600 hover:bg-indigo-500 text-white cursor-pointer active:scale-95 shadow-indigo-600/30'
            ]" :title="hasPendingTools ? (pendingToolsCount > 1 ? $t('chat.pending_tools_tooltip_many', { count: pendingToolsCount }) : $t('chat.pending_tools_tooltip_one')) : $t('chat.send_message')">
              <Loader2 v-if="isProcessingAnyAttachment" class="w-3.5 h-3.5 animate-spin text-slate-400" />
              <ShieldAlert v-else-if="hasPendingTools" class="w-3.5 h-3.5 text-amber-400" />
              <ArrowUpRight v-else class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Floating Circular Scroll-to-Bottom Button -->
    <transition enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 translate-y-3 scale-90" enter-to-class="opacity-100 translate-y-0 scale-100"
      leave-active-class="transition-all duration-150 ease-in" leave-from-class="opacity-100 translate-y-0 scale-100"
      leave-to-class="opacity-0 translate-y-3 scale-90">
      <button v-if="showScrollBottom" @click="handleScrollToBottomClick"
        class="absolute bottom-36 right-6 z-30 w-10 h-10 rounded-full bg-[#161a2b]/95 hover:bg-[#20273d] text-indigo-300 hover:text-white border border-indigo-500/40 hover:border-indigo-400 shadow-2xl shadow-black/80 backdrop-blur-md flex items-center justify-center transition-all hover:scale-110 active:scale-95 cursor-pointer group"
        title="Rolar para o final da conversa">
        <!-- Activity Pulse Indicator if AI is generating in background -->
        <span v-if="isGenerating" class="absolute -top-1 -right-1 flex h-3.5 w-3.5">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
          <span class="relative inline-flex rounded-full h-3.5 w-3.5 bg-indigo-500 border-2 border-[#090a0f]"></span>
        </span>

        <ArrowDown
          class="w-4 h-4 text-indigo-300 group-hover:text-white transition-transform group-hover:translate-y-0.5" />
      </button>
    </transition>

    <!-- MCP Tools Configuration Modal -->
    <McpToolsModal v-if="showToolsModal" @close="showToolsModal = false" @updated="$emit('refreshTools')" />

    <!-- Personas / Agent Profiles Modal -->
    <PersonasModal v-if="showPersonasModal" :params="params" @close="showPersonasModal = false" @personaChanged="handlePersonaChanged" />

    <!-- Chat Efficiency Statistics Modal -->
    <ChatEfficiencyModal v-if="showEfficiencyModal" :show="showEfficiencyModal" :session="currentSession"
      :activeModel="activeModel" :params="params" @close="showEfficiencyModal = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import {
  Sparkles,
  Trash2,
  Send,
  Square,
  Paperclip,
  Brain,
  Wrench,
  X,
  FileText,
  FileCode2,
  Mic,
  UploadCloud,
  ArrowDown,
  ArrowUpRight,
  Download,
  AlertTriangle,
  Loader2,
  ChevronDown,
  SlidersHorizontal,
  MoreHorizontal,
  Shield,
  ShieldAlert,
  Image,
  Eye,
  EyeOff,
  Lightbulb,
  PenLine,
  Globe,
  MessageCircleHeart,
  Zap,
  Check,
  Archive,
  ArchiveRestore
} from 'lucide-vue-next'

import ChatBubble from '../ChatBubble.vue'
import McpToolsModal from '../McpToolsModal.vue'
import ChatEfficiencyModal from '../ChatEfficiencyModal.vue'
import PersonasModal from '../PersonasModal.vue'
import { formatSessionToMarkdown, downloadMarkdownFile } from '~/utils/exportMarkdown'
import { activePersona, initPersonas } from '~/utils/personas'
import { useAppLocale } from '~/composables/useLocale'
import type { ChatSession, ModelInfo, GenerationParams, AppConfig, Project, Persona } from '~/types'

const { t } = useAppLocale()
import {
  isVideoFile,
  isAudioFile,
  isImageFile,
  extractPdfText,
  renderPdfPageToImage,
  transcribeAudioFile,
  readTextFile
} from '~/utils/fileProcessor'

const props = withDefaults(
  defineProps<{
    currentSession?: ChatSession | null
    activeModel?: ModelInfo | null
    allModels?: ModelInfo[]
    isGenerating?: boolean
    params?: GenerationParams | null
    activeMcpToolsCount?: number
    showEfficiencyMetrics?: boolean
    config?: AppConfig | null
    mcpTools?: any[]
    projects?: Project[]
  }>(),
  {
    currentSession: () => ({ id: '', title: '', created_at: '', messages: [] }),
    activeModel: null,
    allModels: () => [],
    isGenerating: false,
    params: () => ({
      system_prompt: '',
      context_length: 8192,
      enable_thinking: true,
      thinking_budget: 2048,
      kv_cache_quant: 'q8_0',
      enable_prompt_cache: true,
      flash_attention: true
    }),
    activeMcpToolsCount: 0,
    showEfficiencyMetrics: true,
    config: () => ({}),
    mcpTools: () => [],
    projects: () => []
  }
)

const emit = defineEmits<{
  sendMessage: [payload: any]
  resendMessage: [message: any]
  selectTab: [tab: string]
  stopGeneration: []
  clearChat: []
  approveTool: [payload: any]
  rejectTool: [payload: any]
  reExecuteTool: [payload: any]
  approveSelectedTools: [payload: any]
  rejectSelectedTools: [payload: any]
  approveAllTools: [payload: any]
  rejectAllTools: [payload: any]
  loadModel: [model: any]
  refreshTools: []
  openParams: []
  assignProject: [projectId: any]
  deleteMessage: [payload: any]
  unarchiveSession: [id: string]
}>()

const handleRetryLastMessage = () => {
  if (!props.currentSession?.messages?.length) return
  const lastUserMsg = [...props.currentSession.messages].reverse().find((m) => m.role === 'user' || m.role === 'User')
  if (lastUserMsg) {
    emit('resendMessage', lastUserMsg)
  }
}

const handleContinueGeneration = () => {
  if (props.isGenerating) return
  // If max_tokens is constrained (<= 2048), bump it so continuation doesn't immediately truncate again
  if (props.params && (props.params.max_tokens ?? 0) <= 2048) {
    props.params.max_tokens = Math.min((props.params.max_tokens || 0) + 2048, 8192)
    showToast(t('chat.limit_raised', { tokens: props.params.max_tokens }))
  }
  emit('sendMessage', {
    text: 'continue de onde parou',
    display_text: 'continue de onde parou',
    attachments: []
  })
}

const showToolsModal = ref(false)
const showPersonasModal = ref(false)
const showEfficiencyModal = ref(false)
const showMoreActions = ref(false)
const inputText = ref('')
const drafts = ref<Record<string, string>>({})
const attachments = ref<any[]>([])
const isDragging = ref(false)
const dragCounter = ref(0)
const toastMessage = ref<string | null>(null)

const handleTextareaInput = (e?: Event) => {
  adjustTextareaHeight(e)
}

const handleTextareaKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey && !e.altKey && !e.metaKey && !e.ctrlKey) {
    e.preventDefault()
    if (hasPendingTools.value) {
      showToast(
        pendingToolsCount.value > 1
          ? t('chat.pending_tools_toast_many', { count: pendingToolsCount.value })
          : t('chat.pending_tools_toast_one')
      )
      return
    }
    handleSend()
  }
}

onMounted(() => {
  initPersonas()
  if (props.params && !props.params.system_prompt && activePersona.value) {
    props.params.system_prompt = activePersona.value.system_prompt || ''
  }
})

const handlePersonaChanged = (p: Persona | any) => {
  if (p && p.name) {
    showToast(t('chat.profile_activated', { name: p.name }))
  }
}

// Close dropdowns on outside click
const handleOutsideClick = (e: MouseEvent) => {
  if (showMoreActions.value && !(e.target as HTMLElement | null)?.closest('.relative')) {
    showMoreActions.value = false
  }
}
let toastTimer: ReturnType<typeof setTimeout> | null = null

const showToast = (msg: string) => {
  toastMessage.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toastMessage.value = null
  }, 6000)
}

const hasPendingTools = computed(() => {
  const lastMsg = (props.currentSession?.messages || []).slice().reverse().find(
    (m) => m.role === 'assistant' && m.tool_calls && m.tool_calls.length > 0
  )
  if (!lastMsg || !lastMsg.tool_calls) return false
  return lastMsg.tool_calls.some(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing'
  )
})

const pendingToolsCount = computed(() => {
  const lastMsg = (props.currentSession?.messages || []).slice().reverse().find(
    (m) => m.role === 'assistant' && m.tool_calls && m.tool_calls.length > 0
  )
  if (!lastMsg || !lastMsg.tool_calls) return 0
  return lastMsg.tool_calls.filter(
    (tc: any) => tc.status === 'pending_approval' || tc.status === 'executing'
  ).length
})

const scrollToPendingTools = () => {
  const el = document.querySelector('.sticky.-top-\\[18px\\]') || document.querySelector('[id^="tool-card-"]')
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

const canSend = computed(() => {
  if (props.isGenerating) return false
  if (isContextFull.value) return false
  if (isProcessingAnyAttachment.value) return false
  if (hasPendingTools.value) return false
  return inputText.value.trim().length > 0 || attachments.value.length > 0
})

const isProcessingAnyAttachment = computed(() => {
  return attachments.value.some((a) => a.isProcessing === true)
})

const exportMarkdown = async () => {
  if (!props.currentSession) return
  const md = formatSessionToMarkdown(props.currentSession, props.activeModel)
  const safeTitle = (props.currentSession.title || 'conversa')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '_')
    .slice(0, 40)
  await downloadMarkdownFile(md, `${safeTitle || 'conversa'}_${new Date().toISOString().slice(0, 10)}.md`)
}

const messagesContainer = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)

const userScrolledUp = ref(false)
const showScrollBottom = ref(false)

const handleScroll = () => {
  if (!messagesContainer.value) return
  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  const distanceToBottom = scrollHeight - scrollTop - clientHeight
  const threshold = 70

  if (distanceToBottom > threshold) {
    userScrolledUp.value = true
    showScrollBottom.value = true
  } else {
    userScrolledUp.value = false
    showScrollBottom.value = false
  }
}

const scrollToBottom = (force = false, behavior = 'auto') => {
  if (!force && userScrolledUp.value) return
  nextTick(() => {
    if (messagesContainer.value) {
      if (behavior === 'smooth') {
        messagesContainer.value.scrollTo({
          top: messagesContainer.value.scrollHeight,
          behavior: 'smooth'
        })
      } else {
        messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
      }
    }
  })
}

const handleScrollToBottomClick = () => {
  userScrolledUp.value = false
  showScrollBottom.value = false
  scrollToBottom(true, 'smooth')
}

const welcomeGreeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 12) return t('chat.good_morning')
  if (hour < 18) return t('chat.good_afternoon')
  return t('chat.good_evening')
})

const starterPrompts = computed(() => [
  {
    title: t('chat.starter_writing_title'),
    prompt: t('chat.starter_writing_prompt'),
    icon: PenLine,
    color: 'text-sky-400',
    bg: 'bg-sky-500/10',
    border: 'border-sky-500/20'
  },
  {
    title: t('chat.starter_concept_title'),
    prompt: t('chat.starter_concept_prompt'),
    icon: Lightbulb,
    color: 'text-amber-400',
    bg: 'bg-amber-500/10',
    border: 'border-amber-500/20'
  },
  {
    title: t('chat.starter_plan_title'),
    prompt: t('chat.starter_plan_prompt'),
    icon: Globe,
    color: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
    border: 'border-emerald-500/20'
  },
  {
    title: t('chat.starter_brainstorm_title'),
    prompt: t('chat.starter_brainstorm_prompt'),
    icon: Sparkles,
    color: 'text-purple-400',
    bg: 'bg-purple-500/10',
    border: 'border-purple-500/20'
  }
])

const isAgyModel = (model?: any): boolean => {
  if (!model) return false
  return (
    model.backend === 'Antigravity' ||
    model.format === 'Agy' ||
    (model.id && model.id.startsWith('agy/')) ||
    (model.quantization && model.quantization === 'Antigravity')
  )
}

const parseAgyBase = (model?: any) => {
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

const currentModelVariants = computed(() => {
  if (!props.activeModel || !props.allModels?.length) return []
  const agyInfo = parseAgyBase(props.activeModel)
  if (agyInfo) {
    const list = props.allModels.filter((m) => {
      const info = parseAgyBase(m)
      return info && info.baseId === agyInfo.baseId
    })
    const effortOrder: Record<string, number> = { high: 1, medium: 2, low: 3 }
    return list
      .map((m) => {
        const info = parseAgyBase(m)
        const eff = info?.effort || 'medium'
        const effLabel = eff === 'high' ? t('cloud.effort_high') : eff === 'medium' ? t('cloud.effort_medium') : t('cloud.effort_low')
        return {
          effort: eff,
          label: `${eff === 'high' ? 'High' : eff === 'medium' ? 'Medium' : 'Low'} (${effLabel})`,
          shortLabel: eff === 'high' ? 'High' : eff === 'medium' ? 'Medium' : 'Low',
          model: m
        }
      })
      .sort((a, b) => (effortOrder[a.effort] || 99) - (effortOrder[b.effort] || 99))
  }
  return []
})

const currentEffort = computed(() => {
  const agyInfo = parseAgyBase(props.activeModel)
  return agyInfo?.effort || 'medium'
})

const currentEffortLabel = computed(() => {
  const eff = currentEffort.value
  const effLabel = eff === 'high' ? t('cloud.effort_high') : eff === 'low' ? t('cloud.effort_low') : t('cloud.effort_medium')
  return `${eff === 'high' ? 'High' : eff === 'low' ? 'Low' : 'Medium'} (${effLabel})`
})

const showReasoningDropdown = ref(false)

const handleSelectEffort = (variant: any) => {
  showReasoningDropdown.value = false
  if (variant.model.id !== props.activeModel?.id) {
    emit('loadModel', variant.model)
    showToast(t('chat.reasoning_changed', { label: variant.label }))
  }
}

const toggleThinking = () => {
  if (props.params) {
    props.params.enable_thinking = !props.params.enable_thinking
  }
}

const formatBytes = (bytes?: number | null) => {
  if (!bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

const formatDuration = (sec?: number | null) => {
  if (!sec) return '0s'
  const mins = Math.floor(sec / 60)
  const remSec = Math.floor(sec % 60)
  return mins > 0 ? `${mins}m ${remSec}s` : `${remSec}s`
}

const triggerFileInput = () => {
  fileInputRef.value?.click()
}

const processFiles = async (fileList: File[]) => {
  if (!fileList || fileList.length === 0) return

  // 1. Deduplicate incoming fileList in the current batch
  const uniqueFiles: File[] = []
  const seen = new Set<string>()
  for (const file of fileList) {
    const key = `${file.name}-${file.size}-${file.lastModified || 0}`
    if (!seen.has(key)) {
      seen.add(key)
      uniqueFiles.push(file)
    }
  }

  for (const file of uniqueFiles) {
    // 2. Prevent adding exact duplicate already present in attachments
    const alreadyAttached = attachments.value.some(
      (a: any) => a.name === file.name && a.size_bytes === file.size
    )
    if (alreadyAttached) {
      continue
    }

    const fileNameLower = (file.name || '').toLowerCase()

    // 1. Explicitly Block Video Files
    if (isVideoFile(file)) {
      showToast(t('chat.videos_not_supported', { name: file.name }))
      continue
    }

    // 2. Image Files
    if (isImageFile(file)) {
      const attItem: any = {
        id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        kind: 'image',
        is_image: true,
        data_url: '',
        size_bytes: file.size,
        isProcessing: true
      }
      attachments.value = [...attachments.value, attItem]

      const reader = new FileReader()
      reader.onload = (e: ProgressEvent<FileReader>) => {
        attItem.data_url = (e.target?.result as string) || ''
        attItem.isProcessing = false
        attachments.value = [...attachments.value]
      }
      reader.onerror = () => {
        attItem.isProcessing = false
        attItem.error = 'Erro ao carregar imagem'
        attachments.value = [...attachments.value]
      }
      reader.readAsDataURL(file)
      continue
    }

    // 3. PDF Documents (Extract text via PDF.js + visual render if scanned)
    const isPdf = file.type === 'application/pdf' || fileNameLower.endsWith('.pdf')
    if (isPdf) {
      const attItem: any = {
        id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        kind: 'pdf',
        is_image: false,
        size_bytes: file.size,
        pages: null,
        text: '',
        page_images: [],
        isProcessing: true,
        statusText: 'Lendo PDF...'
      }
      attachments.value = [...attachments.value, attItem]

      try {
        const result = await extractPdfText(file)
        attItem.text = result.text
        attItem.pages = result.pages
        attItem.charCount = result.charCount
        attItem.isScanned = result.isScanned

        // If scanned or no text layer, try rendering first page as image for VLM support
        if (result.isScanned) {
          const imgUrl = await renderPdfPageToImage(file, 1)
          if (imgUrl) {
            attItem.page_images = [imgUrl]
          }
        }

        attItem.isProcessing = false
        attItem.statusText = ''
        attachments.value = [...attachments.value]
      } catch (err: any) {
        attItem.isProcessing = false
        attItem.error = err.message
        attachments.value = [...attachments.value]
        showToast(err.message || t('chat.pdf_error', { name: file.name }))
      }
      continue
    }

    // 4. Audio Files (Transcribe speech via Whisper)
    if (isAudioFile(file)) {
      const attItem: any = {
        id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        name: file.name,
        kind: 'audio',
        is_image: false,
        size_bytes: file.size,
        duration: null,
        text: '',
        isProcessing: true,
        statusText: 'Transcrevendo...'
      }
      attachments.value = [...attachments.value, attItem]

      try {
        const result = await transcribeAudioFile(file, (p) => {
          if (p.status === 'downloading') attItem.statusText = `Baixando modelo (${p.progress}%)`
          else if (p.status === 'decoding_audio') attItem.statusText = 'Decodificando áudio...'
          else if (p.status === 'transcribing') attItem.statusText = 'Transcrevendo áudio...'
          attachments.value = [...attachments.value]
        }, {
          language: props.config?.whisper_language || 'pt',
          model: props.config?.whisper_model || 'mlx-community/whisper-small-mlx'
        })
        attItem.text = result.text
        attItem.duration = result.duration
        attItem.isProcessing = false
        attItem.statusText = ''
        attachments.value = [...attachments.value]
      } catch (err: any) {
        attItem.isProcessing = false
        attItem.error = err.message
        attachments.value = [...attachments.value]
        showToast(err.message || t('chat.audio_error', { name: file.name }))
      }
      continue
    }

    // 5. Code & Text Files (XML, JSON, CSV, TXT, MD, Code, etc.) - ALL non-binary files
    const isXml = fileNameLower.endsWith('.xml')
    const isJson = fileNameLower.endsWith('.json')
    const isCsv = fileNameLower.endsWith('.csv') || fileNameLower.endsWith('.tsv')

    let fileKind = 'text'
    if (isXml) fileKind = 'xml'
    else if (isJson) fileKind = 'json'
    else if (isCsv) fileKind = 'csv'

    const attItem: any = {
      id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      name: file.name,
      kind: fileKind,
      is_image: false,
      size_bytes: file.size,
      text: '',
      isProcessing: true,
      statusText: 'Lendo conteúdo...'
    }
    attachments.value = [...attachments.value, attItem]

    try {
      const textContent = await readTextFile(file)
      attItem.text = textContent
      attItem.isProcessing = false
      attItem.statusText = ''
      attachments.value = [...attachments.value]
    } catch (err: any) {
      attItem.isProcessing = false
      attItem.error = err.message
      attachments.value = [...attachments.value]
      showToast(err.message || t('chat.file_error', { name: file.name }))
    }
  }
}

const handleFileInputChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  if (target?.files) {
    processFiles(Array.from(target.files))
  }
  if (target) {
    target.value = ''
  }
}

const handleDragEnter = (e: DragEvent) => {
  e.preventDefault()
  e.stopPropagation()
  dragCounter.value++
  if (e.dataTransfer?.types?.includes('Files')) {
    isDragging.value = true
  }
}

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  e.stopPropagation()
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'copy'
  }
  isDragging.value = true
}

const handleDragLeave = (e: DragEvent) => {
  e.preventDefault()
  e.stopPropagation()
  dragCounter.value--
  if (dragCounter.value <= 0) {
    dragCounter.value = 0
    isDragging.value = false
  }
}

let lastDropTimestamp = 0

const handleFileDrop = (e: DragEvent) => {
  e.preventDefault()
  e.stopPropagation()
  dragCounter.value = 0
  isDragging.value = false

  const now = Date.now()
  if (now - lastDropTimestamp < 500) {
    return
  }
  lastDropTimestamp = now

  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    processFiles(Array.from(e.dataTransfer.files))
  }
}

let lastPasteTimestamp = 0

const handlePaste = (e: ClipboardEvent) => {
  if (e.clipboardData?.files && e.clipboardData.files.length > 0) {
    e.preventDefault()
    e.stopPropagation()

    const now = Date.now()
    if (now - lastPasteTimestamp < 500) {
      return
    }
    lastPasteTimestamp = now

    const files = Array.from(e.clipboardData.files)
    processFiles(files)
  }
}

const removeAttachment = (idx: number) => {
  attachments.value = attachments.value.filter((_, i) => i !== idx)
}

const useStarterPrompt = (item: any) => {
  inputText.value = item.prompt
  nextTick(() => {
    textareaRef.value?.focus()
  })
}

const handleSend = () => {
  if (hasPendingTools.value) {
    showToast(
      pendingToolsCount.value > 1
        ? t('chat.pending_tools_toast_many', { count: pendingToolsCount.value })
        : t('chat.pending_tools_toast_one')
    )
    return
  }

  if (isProcessingAnyAttachment.value) {
    showToast(t('chat.wait_processing'))
    return
  }

  const text = inputText.value.trim()
  if (!canSend.value) return

  // Warn if no model is loaded
  if (!props.activeModel) {
    showToast(t('chat.no_active_model'))
  }

  // Format LLM message prompt with structured text attachments
  let fullPrompt = text
  const images: string[] = []
  const attachmentsPayload: any[] = []

  for (const att of attachments.value) {
    attachmentsPayload.push({
      name: att.name,
      kind: att.kind,
      is_image: att.is_image,
      size_bytes: att.size_bytes,
      pages: att.pages,
      duration: att.duration,
      text: att.text
    })

    if (att.kind === 'image') {
      images.push(att.data_url)
    } else {
      if (att.page_images && att.page_images.length > 0) {
        images.push(...att.page_images)
      }
      if (att.text) {
        if (att.kind === 'pdf') {
          fullPrompt += `\n\n--- [Documento PDF Anexado: ${att.name}${att.pages ? ` (${att.pages} páginas)` : ''}] ---\n${att.text}\n--- [Fim do Documento PDF: ${att.name}] ---`
        } else if (att.kind === 'audio') {
          fullPrompt += `\n\n--- [Transcrição de Áudio Anexado: ${att.name}${att.duration ? ` (${formatDuration(att.duration)})` : ''}] ---\n${att.text}\n--- [Fim da Transcrição de Áudio: ${att.name}] ---`
        } else if (att.kind === 'xml' || (att.name && att.name.toLowerCase().endsWith('.xml'))) {
          fullPrompt += `\n\n--- [Conteúdo do Arquivo XML Anexado: ${att.name}] ---\n${att.text}\n--- [Fim do Arquivo XML: ${att.name}] ---`
        } else if (att.kind === 'json' || (att.name && att.name.toLowerCase().endsWith('.json'))) {
          fullPrompt += `\n\n--- [Conteúdo do Arquivo JSON Anexado: ${att.name}] ---\n${att.text}\n--- [Fim do Arquivo JSON: ${att.name}] ---`
        } else {
          fullPrompt += `\n\n--- [Conteúdo do Arquivo Anexado: ${att.name}] ---\n${att.text}\n--- [Fim do Arquivo: ${att.name}] ---`
        }
      }
    }
  }

  // Fallback if user submitted only images/files with no prompt text
  if (!fullPrompt.trim() && images.length > 0) {
    fullPrompt = 'Analise a imagem anexada.'
  } else if (!fullPrompt.trim() && attachmentsPayload.length > 0) {
    const firstAtt = attachmentsPayload[0]
    if (firstAtt.kind === 'xml' || (firstAtt.name && firstAtt.name.toLowerCase().endsWith('.xml'))) {
      fullPrompt = `Por favor, analise a estrutura e os dados do arquivo XML "${firstAtt.name}" anexado.`
    } else {
      fullPrompt = `Analise o conteúdo do arquivo "${firstAtt.name}" anexado.`
    }
  }

  // Reset scroll state on new message submission
  userScrolledUp.value = false
  showScrollBottom.value = false

  emit('sendMessage', {
    text: fullPrompt,
    display_text: text || (attachmentsPayload[0] ? `Anexo: ${attachmentsPayload[0].name}` : ''),
    images: images.length > 0 ? images : undefined,
    attachments: attachmentsPayload
  })

  if (props.currentSession?.id) {
    delete drafts.value[props.currentSession.id]
  }
  inputText.value = ''
  attachments.value = []
  if (textareaRef.value) {
    textareaRef.value.style.height = 'auto'
  }

  scrollToBottom(true)
}

const adjustTextareaHeight = (e?: Event) => {
  const el = (e?.target as HTMLTextAreaElement) || textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${Math.min(el.scrollHeight, 140)}px`
}

// Quick Context limit dropdown state & methods
const showContextDropdown = ref(false)
const customContextInput = ref(props.params?.context_length || 8192)

const toggleContextDropdown = () => {
  showContextDropdown.value = !showContextDropdown.value
  if (showContextDropdown.value) {
    customContextInput.value = contextLimit.value
  }
}

const setQuickContext = (tokens: number) => {
  if (props.params) {
    props.params.context_length = tokens
    customContextInput.value = tokens
  }
  showContextDropdown.value = false
  showToast(t('chat.context_limit_adjusted', { tokens: tokens >= 1000 ? Math.round(tokens / 1024) + 'k' : tokens }))
}

const applyCustomContext = () => {
  const val = Number(customContextInput.value)
  if (val >= 256 && val <= 1048576 && props.params) {
    props.params.context_length = val
    showContextDropdown.value = false
    showToast(t('chat.context_limit_adjusted', { tokens: val >= 1024 ? Math.round(val / 1024) + 'k' : val }))
  }
}

const handleDeleteMessage = (msg: any, index: number) => {
  emit('deleteMessage', { id: msg?.id, index })
  showToast(t('chat.message_deleted'))
}

// Context calculation (prioritizing user-defined context_length)
const contextLimit = computed(() => {
  if (props.params?.context_length && props.params.context_length > 0) {
    return props.params.context_length
  }
  return props.activeModel?.context_length || 8192
})


const contextUsed = computed(() => {
  if (!props.currentSession?.messages) return 0
  let totalChars = (props.params?.system_prompt || '').length
  for (const m of props.currentSession.messages) {
    totalChars += (m.content || '').length + (m.thinking || '').length
    if (m.tool_calls && Array.isArray(m.tool_calls)) {
      for (const tc of m.tool_calls) {
        totalChars += (tc.name || '').length + JSON.stringify(tc.arguments || {}).length
        if (tc.result) totalChars += JSON.stringify(tc.result).length
      }
    }
  }
  return Math.ceil(totalChars / 4)
})

const contextPctNumber = computed(() => {
  if (contextLimit.value <= 0) return 0
  return (contextUsed.value / contextLimit.value) * 100
})

const contextPct = computed(() => {
  return contextPctNumber.value.toFixed(1)
})

const isContextFull = computed(() => contextUsed.value >= contextLimit.value)

// Watch session change to switch drafts and reset scroll
watch(
  () => props.currentSession?.id,
  (newId, oldId) => {
    if (oldId) {
      drafts.value[oldId] = inputText.value
    }
    inputText.value = newId ? (drafts.value[newId] || '') : ''
    nextTick(() => {
      adjustTextareaHeight()
    })
    userScrolledUp.value = false
    showScrollBottom.value = false
    scrollToBottom(true)
  }
)

// Watch new messages
watch(
  () => props.currentSession?.messages?.length,
  (newLen) => {
    const lastMsg = newLen ? props.currentSession?.messages?.[newLen - 1] : undefined
    if (lastMsg && (lastMsg.role === 'user' || lastMsg.role === 'User')) {
      userScrolledUp.value = false
      showScrollBottom.value = false
      scrollToBottom(true)
    } else {
      scrollToBottom()
    }
  }
)

// Watch streaming content
watch(
  () => props.currentSession?.messages?.[props.currentSession.messages.length - 1]?.content,
  () => {
    if (!userScrolledUp.value) {
      scrollToBottom()
    }
  }
)

// Watch streaming thinking
watch(
  () => props.currentSession?.messages?.[props.currentSession.messages.length - 1]?.thinking,
  () => {
    if (!userScrolledUp.value) {
      scrollToBottom()
    }
  }
)

// Watch tool calls updates
watch(
  () => props.currentSession?.messages?.[props.currentSession.messages.length - 1]?.tool_calls,
  () => {
    if (!userScrolledUp.value) {
      scrollToBottom()
    }
  },
  { deep: true }
)

// When generation completes, if user wasn't scrolled up, make sure it stays at bottom
watch(
  () => props.isGenerating,
  (generating) => {
    if (!generating && !userScrolledUp.value) {
      scrollToBottom(true)
    }
  }
)
</script>
