<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between gap-4">
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
          <Clock class="w-4 h-4 text-emerald-400 flex-shrink-0" />
          <span class="truncate">{{ $t('memory.automation_title') }}</span>
        </h3>
        <p class="text-xs text-slate-400 mt-0.5">
          {{ $t('memory.automation_subtitle') }}
        </p>
      </div>

      <div class="flex items-center gap-2 flex-shrink-0">
        <button
          type="button"
          @click="openHistoryModal"
          class="px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 bg-[#171c2c] hover:bg-[#222940] border border-[#232b42] text-slate-200 transition-all cursor-pointer"
        >
          <History class="w-3.5 h-3.5 text-slate-400" />
          <span>{{ $t('memory.btn_execution_history') }}</span>
        </button>

        <button
          type="button"
          @click="openNewTaskModal"
          class="px-3.5 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm shadow-emerald-600/30 transition-all cursor-pointer active:scale-95"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>{{ $t('memory.btn_add_task') }}</span>
        </button>
      </div>
    </div>

    <!-- Status Strip -->
    <div class="flex items-center justify-between px-3.5 py-2.5 rounded-2xl bg-[#0e101a] border border-[#1b2033] text-xs text-slate-400 flex-wrap gap-2 shadow-sm">
      <div class="flex items-center gap-2.5 flex-wrap">
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
          <span class="text-slate-200 font-medium">Scheduler Daemon:</span>
          <span class="text-emerald-400 font-semibold">Active (0% idle CPU)</span>
        </div>
        <span class="text-slate-600">•</span>
        <div class="flex items-center gap-1.5">
          <span class="text-slate-200 font-medium">Active Tasks:</span>
          <span class="text-slate-300">{{ activeTasksCount }} / {{ tasks.length }}</span>
        </div>
      </div>
      <div class="text-[11px] text-slate-500">
        Rust / Tokio Background Worker
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="py-12 flex flex-col items-center justify-center gap-3 text-slate-400">
      <Loader2 class="w-6 h-6 animate-spin text-emerald-400" />
      <span class="text-xs">Loading scheduled tasks...</span>
    </div>

    <!-- Empty State -->
    <div v-else-if="tasks.length === 0" class="py-12 px-4 rounded-2xl bg-[#0e101a]/50 border border-[#1b2033] border-dashed flex flex-col items-center justify-center text-center gap-3">
      <div class="w-10 h-10 rounded-2xl bg-emerald-500/10 flex items-center justify-center text-emerald-400">
        <Clock class="w-5 h-5" />
      </div>
      <div class="max-w-xs">
        <h4 class="text-xs font-semibold text-slate-200">No scheduled routines yet</h4>
        <p class="text-[11px] text-slate-400 mt-1">
          Create proactive routines such as daily morning summaries, system monitors, or associative memory sleep cycles.
        </p>
      </div>
      <button
        @click="openNewTaskModal"
        class="mt-2 px-3 py-1.5 rounded-xl text-xs font-medium bg-[#171c2c] hover:bg-[#222940] border border-[#232b42] text-slate-200 flex items-center gap-1.5 transition-all cursor-pointer"
      >
        <Plus class="w-3.5 h-3.5 text-emerald-400" />
        <span>{{ $t('memory.btn_add_task') }}</span>
      </button>
    </div>

    <!-- Tasks List -->
    <div v-else class="space-y-3">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="p-4 rounded-2xl bg-[#0e101a] border border-[#1b2033] hover:border-[#28304d] transition-all flex flex-col gap-3 shadow-sm group"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-start gap-3 min-w-0">
            <div
              :class="[
                'w-8 h-8 rounded-xl flex items-center justify-center flex-shrink-0 mt-0.5',
                task.action_type === 'autonomous_prompt'
                  ? 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
                  : task.action_type === 'memory_sleep'
                  ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
                  : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
              ]"
            >
              <Bot v-if="task.action_type === 'autonomous_prompt'" class="w-4 h-4" />
              <Moon v-else-if="task.action_type === 'memory_sleep'" class="w-4 h-4" />
              <Zap v-else class="w-4 h-4" />
            </div>

            <div class="min-w-0">
              <div class="flex items-center gap-2 flex-wrap">
                <h4 class="text-xs font-bold text-slate-200 truncate">{{ task.name }}</h4>
                <span class="px-2 py-0.5 rounded-md text-[10px] font-mono bg-[#141826] border border-[#20273d] text-slate-300">
                  {{ task.cron_expr }}
                </span>
                <span
                  :class="[
                    'px-1.5 py-0.5 rounded text-[9px] font-medium uppercase tracking-wider',
                    task.action_type === 'autonomous_prompt' ? 'bg-purple-500/20 text-purple-300' :
                    task.action_type === 'memory_sleep' ? 'bg-blue-500/20 text-blue-300' :
                    'bg-emerald-500/20 text-emerald-300'
                  ]"
                >
                  {{ task.action_type }}
                </span>

                <!-- Delivery Channel Badge -->
                <span
                  :class="[
                    'px-1.5 py-0.5 rounded text-[9px] font-medium flex items-center gap-1',
                    task.delivery_channel === 'telegram' ? 'bg-sky-500/20 text-sky-300 border border-sky-500/30' :
                    task.delivery_channel === 'both' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' :
                    task.delivery_channel === 'silent' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30' :
                    'bg-slate-700/40 text-slate-300 border border-slate-600/30'
                  ]"
                >
                  <Send v-if="task.delivery_channel === 'telegram'" class="w-2.5 h-2.5" />
                  <BellOff v-else-if="task.delivery_channel === 'silent'" class="w-2.5 h-2.5" />
                  <MessageSquare v-else class="w-2.5 h-2.5" />
                  <span>{{ formatDeliveryLabel(task.delivery_channel) }}</span>
                </span>
              </div>
              <p v-if="task.description" class="text-[11px] text-slate-400 mt-0.5 truncate">
                {{ task.description }}
              </p>
            </div>
          </div>

          <!-- Toggle & Actions -->
          <div class="flex items-center gap-2 flex-shrink-0">
            <button
              @click="toggleTask(task)"
              :class="[
                'w-8 h-4.5 rounded-full transition-colors relative cursor-pointer',
                task.enabled ? 'bg-indigo-600' : 'bg-slate-700'
              ]"
              :title="task.enabled ? 'Enabled' : 'Disabled'"
            >
              <span
                :class="[
                  'absolute top-0.5 left-0.5 w-3.5 h-3.5 rounded-full bg-white transition-transform',
                  task.enabled ? 'translate-x-3.5' : 'translate-x-0'
                ]"
              />
            </button>

            <button
              @click="openTaskHistory(task.id)"
              class="p-1.5 rounded-lg bg-[#141826] hover:bg-[#1d2338] border border-[#222940] text-slate-300 hover:text-white transition-all cursor-pointer"
              :title="$t('memory.btn_execution_history')"
            >
              <History class="w-3.5 h-3.5 text-sky-400" />
            </button>

            <button
              @click="runNow(task)"
              :disabled="runningTaskId === task.id"
              class="p-1.5 rounded-lg bg-[#141826] hover:bg-[#1d2338] border border-[#222940] text-slate-300 hover:text-white transition-all cursor-pointer disabled:opacity-50"
              :title="$t('memory.btn_run_now')"
            >
              <Loader2 v-if="runningTaskId === task.id" class="w-3.5 h-3.5 animate-spin text-emerald-400" />
              <Play v-else class="w-3.5 h-3.5 text-emerald-400" />
            </button>

            <button
              @click="deleteTask(task.id)"
              class="p-1.5 rounded-lg bg-[#141826] hover:bg-rose-950/40 border border-[#222940] hover:border-rose-800/40 text-slate-400 hover:text-rose-300 transition-all cursor-pointer"
              title="Delete"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <!-- Last Result Preview if available -->
        <div v-if="task.last_result" class="p-2.5 rounded-xl bg-[#090b12] border border-[#161b2c] text-[11px] text-slate-300">
          <div class="flex items-center justify-between text-[10px] text-slate-500 mb-1">
            <span class="font-semibold text-slate-400">Último Retorno Registrado:</span>
          </div>
          <p class="font-mono text-[10px] text-slate-400 line-clamp-2 leading-relaxed select-text">
            {{ task.last_result }}
          </p>
        </div>

        <!-- Timestamps footer -->
        <div class="flex items-center justify-between text-[10px] text-slate-500 pt-2 border-t border-[#161a29]">
          <div class="flex items-center gap-1.5">
            <Clock class="w-3 h-3 text-slate-600" />
            <span>{{ $t('memory.task_last_run') }}:</span>
            <span class="text-slate-400 font-mono">{{ formatTimestamp(task.last_run) }}</span>
          </div>
          <div v-if="task.next_run" class="flex items-center gap-1.5">
            <span>{{ $t('memory.task_next_run') }}:</span>
            <span class="text-emerald-400/80 font-mono">{{ formatTimestamp(task.next_run) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Task Modal -->
    <div
      v-if="showModal"
      class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4"
    >
      <div class="w-full max-w-md bg-[#0e101a] border border-[#1b2033] rounded-2xl p-5 space-y-4 shadow-2xl">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
            <Clock class="w-4 h-4 text-emerald-400" />
            <span>{{ $t('memory.btn_add_task') }}</span>
          </h3>
          <button @click="showModal = false" class="text-slate-400 hover:text-white text-xs cursor-pointer">✕</button>
        </div>

        <div class="space-y-3 text-xs">
          <div>
            <label class="block text-slate-300 font-medium mb-1">{{ $t('memory.task_name') }}</label>
            <input
              v-model="newTask.name"
              type="text"
              placeholder="e.g. Daily Morning Briefing"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 placeholder-slate-500 outline-none focus:border-emerald-500 transition-all font-sans"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-medium mb-1">Description (Optional)</label>
            <input
              v-model="newTask.description"
              type="text"
              placeholder="Brief description of the routine objective"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 placeholder-slate-500 outline-none focus:border-emerald-500 transition-all font-sans"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-medium mb-1">{{ $t('memory.task_action') }}</label>
            <select
              v-model="newTask.action_type"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 outline-none focus:border-emerald-500 transition-all font-sans cursor-pointer"
            >
              <option value="autonomous_prompt">Autonomous Prompt (AI Generation & Synthesis)</option>
              <option value="memory_sleep">Associative Memory Sleep Cycle (Consolidate Facts & Prune)</option>
              <option value="skill">Procedural Skill Execution</option>
            </select>
          </div>

          <!-- Delivery Channel Option -->
          <div>
            <label class="block text-slate-300 font-medium mb-1">{{ $t('memory.delivery_channel') }}</label>
            <select
              v-model="newTask.delivery_channel"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 outline-none focus:border-emerald-500 transition-all font-sans cursor-pointer"
            >
              <option value="chat">💬 {{ $t('memory.delivery_channel_chat') }}</option>
              <option value="telegram">✈️ {{ $t('memory.delivery_channel_telegram') }}</option>
              <option value="both">💬+✈️ {{ $t('memory.delivery_channel_both') }}</option>
              <option value="silent">🔕 {{ $t('memory.delivery_channel_silent') }}</option>
            </select>
          </div>

          <div v-if="newTask.delivery_channel === 'telegram' || newTask.delivery_channel === 'both'">
            <label class="block text-slate-300 font-medium mb-1">Telegram Chat ID (Opcional - padrão: usuário autorizado)</label>
            <input
              v-model="newTask.delivery_target"
              type="text"
              placeholder="e.g. 123456789"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 placeholder-slate-500 outline-none focus:border-emerald-500 transition-all font-mono"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-medium mb-1">{{ $t('memory.task_cron') }}</label>
            <input
              v-model="newTask.cron_expr"
              type="text"
              placeholder="@daily, @hourly, @every 30m, or cron '0 9 * * *'"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 placeholder-slate-500 outline-none focus:border-emerald-500 transition-all font-mono"
            />
            <div class="flex gap-1.5 mt-1.5 flex-wrap">
              <button
                type="button"
                v-for="preset in ['@daily', '@hourly', '@every 30m', '0 9 * * *']"
                :key="preset"
                @click="newTask.cron_expr = preset"
                class="px-2 py-0.5 rounded text-[10px] font-mono bg-[#161b2c] text-slate-400 hover:text-slate-200 border border-[#20273d] transition-all cursor-pointer"
              >
                {{ preset }}
              </button>
            </div>
          </div>

          <div v-if="newTask.action_type === 'autonomous_prompt'">
            <label class="block text-slate-300 font-medium mb-1">Autonomous Goal / Prompt</label>
            <textarea
              v-model="promptInput"
              rows="3"
              placeholder="e.g. Generate a concise daily summary of active memories and pending tasks."
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 placeholder-slate-500 outline-none focus:border-emerald-500 transition-all font-sans resize-none"
            />
          </div>

          <div v-else-if="newTask.action_type === 'skill'">
            <label class="block text-slate-300 font-medium mb-1">{{ $t('memory.task_select_skill') }}</label>
            <select
              v-model="selectedSkillId"
              class="w-full px-3 py-2 rounded-xl bg-[#141826] border border-[#22283b] text-slate-100 outline-none focus:border-emerald-500 transition-all font-sans cursor-pointer"
            >
              <option value="" disabled>{{ $t('memory.task_select_skill_placeholder') }}</option>
              <option v-for="sk in availableSkills" :key="sk.id" :value="sk.id">
                {{ sk.name }} ({{ sk.id }})
              </option>
            </select>
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-[#1b2033]">
          <button
            type="button"
            @click="showModal = false"
            class="px-3 py-1.5 rounded-xl text-xs font-medium text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            @click="saveNewTask"
            :disabled="!newTask.name.trim() || !newTask.cron_expr.trim()"
            class="px-4 py-1.5 rounded-xl text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white shadow-sm shadow-emerald-600/30 transition-all cursor-pointer"
          >
            Save Routine
          </button>
        </div>
      </div>
    </div>

    <!-- Execution History Modal -->
    <div
      v-if="showHistoryModal"
      class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4"
    >
      <div class="w-full max-w-3xl max-h-[88vh] bg-[#0e101a] border border-[#1b2033] rounded-2xl p-5 space-y-4 shadow-2xl flex flex-col">
        <div class="flex items-center justify-between flex-shrink-0 gap-3">
          <h3 class="text-sm font-bold text-slate-100 flex items-center gap-2">
            <History class="w-4 h-4 text-emerald-400" />
            <span>{{ $t('memory.btn_execution_history') }}</span>
          </h3>

          <div class="flex items-center gap-2.5">
            <select
              v-model="selectedTaskFilter"
              @change="loadTaskRuns"
              class="px-2.5 py-1 rounded-xl bg-[#141826] border border-[#22283b] text-slate-300 text-xs outline-none focus:border-emerald-500 cursor-pointer"
            >
              <option :value="null">{{ $t('memory.filter_all_tasks') }}</option>
              <option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.name }}</option>
            </select>
            <button @click="showHistoryModal = false" class="text-slate-400 hover:text-white text-xs cursor-pointer p-1">✕</button>
          </div>
        </div>

        <!-- History List -->
        <div class="flex-1 overflow-y-auto space-y-3 pr-1 min-h-[250px]">
          <div v-if="taskRuns.length === 0" class="py-12 text-center text-xs text-slate-500">
            {{ $t('memory.no_runs_yet') }}
          </div>

          <div
            v-for="run in taskRuns"
            :key="run.id"
            class="p-3.5 rounded-xl bg-[#141826] border border-[#20273d] text-xs space-y-2.5 shadow-sm"
          >
            <!-- Header Row -->
            <div class="flex items-center justify-between gap-2 flex-wrap">
              <div class="flex items-center gap-2 flex-wrap">
                <span
                  :class="[
                    'w-2 h-2 rounded-full flex-shrink-0',
                    run.status === 'success' ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-rose-400 shadow-sm shadow-rose-400/50'
                  ]"
                />
                <span class="font-bold text-slate-100">{{ run.task_name }}</span>
                <span class="px-1.5 py-0.5 rounded text-[9px] uppercase font-mono bg-[#1a2034] text-slate-400 border border-[#262f4c]">
                  {{ formatDeliveryLabel(run.delivery_channel) }}
                </span>
                <span class="text-[10px] text-slate-500">({{ run.steps_count }} passos)</span>

                <button
                  v-if="run.session_id"
                  @click="openChatSession(run.session_id)"
                  class="px-2 py-0.5 rounded-lg bg-indigo-500/10 hover:bg-indigo-500/25 text-indigo-300 border border-indigo-500/30 flex items-center gap-1 transition-colors cursor-pointer text-[10px]"
                  :title="$t('memory.open_in_chat')"
                >
                  <MessageSquare class="w-2.5 h-2.5" />
                  <span>{{ $t('memory.open_in_chat') }}</span>
                </button>
              </div>
              <span class="font-mono text-[10px] text-slate-400">
                {{ formatTimestamp(run.executed_at) }}
              </span>
            </div>

            <!-- Tools used badges -->
            <div v-if="parseTools(run.tools_used).length > 0" class="flex items-center gap-1.5 flex-wrap pt-0.5">
              <span class="text-[10px] text-slate-400 font-medium">{{ $t('memory.tools_used') }}:</span>
              <span
                v-for="tool in parseTools(run.tools_used)"
                :key="tool"
                class="px-1.5 py-0.5 rounded text-[9px] font-mono bg-emerald-500/10 text-emerald-300 border border-emerald-500/20"
              >
                {{ tool }}
              </span>
            </div>

            <!-- Detailed Tool Steps Accordion -->
            <div v-if="parseStepsDetail(run.steps_detail).length > 0" class="pt-1">
              <button
                type="button"
                @click="toggleRunSteps(run.id)"
                class="flex items-center justify-between w-full px-2.5 py-1.5 rounded-lg bg-[#0d101c] hover:bg-[#131828] border border-[#1b2238] text-[10px] text-slate-300 transition-colors cursor-pointer"
              >
                <span class="flex items-center gap-1.5 font-semibold text-emerald-400">
                  <Wrench class="w-3 h-3 text-emerald-400" />
                  <span>{{ $t('memory.tool_execution_details') }} ({{ parseStepsDetail(run.steps_detail).length }})</span>
                </span>
                <ChevronDown :class="['w-3.5 h-3.5 text-slate-400 transition-transform duration-200', expandedStepsByRun[run.id] ? 'rotate-180' : '']" />
              </button>

              <div v-if="expandedStepsByRun[run.id]" class="mt-2 space-y-2 pl-2 border-l-2 border-emerald-500/30">
                <div
                  v-for="step in parseStepsDetail(run.steps_detail)"
                  :key="step.step"
                  class="p-2.5 rounded-lg bg-[#080b13] border border-[#171e33] space-y-2"
                >
                  <div class="flex items-center justify-between gap-2 flex-wrap">
                    <div class="flex items-center gap-1.5">
                      <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-emerald-500/20 text-emerald-300 font-mono">
                        Passo {{ step.step }}
                      </span>
                      <span class="font-mono text-slate-100 font-semibold text-[10px]">{{ step.tool_name }}</span>
                    </div>
                    <span
                      :class="[
                        'px-1.5 py-0.5 rounded text-[9px] font-mono',
                        step.status === 'completed'
                          ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'
                          : 'bg-rose-500/10 text-rose-400 border border-rose-500/30'
                      ]"
                    >
                      {{ step.status }}
                    </span>
                  </div>

                  <!-- Thought preview -->
                  <div v-if="step.thought" class="text-[10px] text-slate-400 italic bg-[#0c101d] p-1.5 rounded border border-[#182037]">
                    <span class="font-semibold text-slate-500 not-italic mr-1">{{ $t('memory.agent_thought') }}:</span>
                    {{ step.thought }}
                  </div>

                  <!-- Arguments -->
                  <div class="space-y-0.5">
                    <span class="text-[9px] text-slate-500 font-medium">{{ $t('memory.tool_arguments') }}:</span>
                    <pre class="p-2 rounded bg-[#04060a] border border-[#131726] text-[9px] text-slate-300 font-mono overflow-x-auto select-text max-h-28 overflow-y-auto">{{ JSON.stringify(step.arguments, null, 2) }}</pre>
                  </div>

                  <!-- Result -->
                  <div v-if="step.result !== undefined" class="space-y-0.5">
                    <span class="text-[9px] text-slate-500 font-medium">{{ $t('memory.tool_result') }}:</span>
                    <pre class="p-2 rounded bg-[#04060a] border border-[#131726] text-[9px] text-emerald-300/90 font-mono overflow-x-auto max-h-36 overflow-y-auto select-text">{{ typeof step.result === 'string' ? step.result : JSON.stringify(step.result, null, 2) }}</pre>
                  </div>
                </div>
              </div>
            </div>

            <!-- Output preview -->
            <div class="space-y-0.5 pt-0.5">
              <span class="text-[10px] text-slate-400 font-semibold">{{ $t('memory.final_answer') }}:</span>
              <div class="p-2.5 rounded-lg bg-[#0a0d17] border border-[#171d31] font-mono text-[10px] text-slate-300 whitespace-pre-wrap max-h-32 overflow-y-auto select-text">
                {{ run.output }}
              </div>
            </div>
          </div>
        </div>

        <div class="flex justify-end pt-2 border-t border-[#1b2033] flex-shrink-0">
          <button
            type="button"
            @click="showHistoryModal = false"
            class="px-4 py-1.5 rounded-xl text-xs font-semibold bg-slate-700 hover:bg-slate-600 text-white transition-colors cursor-pointer"
          >
            Fechar
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Clock, Plus, Trash2, Play, Bot, Moon, Zap, Loader2, History, Send, MessageSquare, BellOff, ChevronDown, Wrench } from 'lucide-vue-next'

interface ScheduledTask {
  id: string
  name: string
  description?: string
  cron_expr: string
  action_type: string
  payload: string
  enabled: boolean
  last_run?: string
  next_run?: string
  delivery_channel?: string
  delivery_target?: string
  last_result?: string
  created_at: string
  updated_at: string
}

interface ScheduledTaskRun {
  id: string
  task_id: string
  task_name: string
  action_type: string
  delivery_channel: string
  status: string
  steps_count: number
  tools_used?: string
  steps_detail?: string
  output: string
  session_id?: string
  executed_at: string
}

interface TaskRunStep {
  step: number
  thought?: string
  tool_name: string
  arguments: any
  status: string
  result?: any
}

const tasks = ref<ScheduledTask[]>([])
const taskRuns = ref<ScheduledTaskRun[]>([])
const isLoading = ref(true)
const runningTaskId = ref<string | null>(null)
const showModal = ref(false)
const showHistoryModal = ref(false)
const selectedTaskFilter = ref<string | null>(null)
const expandedStepsByRun = ref<Record<string, boolean>>({})

const promptInput = ref('Provide a concise daily status briefing.')
const availableSkills = ref<{ id: string; name: string; scripts?: string[] }[]>([])
const selectedSkillId = ref('')
const selectedScriptFile = ref('')

const newTask = ref({
  name: '',
  description: '',
  cron_expr: '@daily',
  action_type: 'autonomous_prompt',
  delivery_channel: 'chat',
  delivery_target: ''
})

watch(selectedSkillId, (newId) => {
  const sk = availableSkills.value.find((s) => s.id === newId)
  if (sk) {
    if (!newTask.value.name) {
      newTask.value.name = sk.name
    }
    if (sk.scripts && sk.scripts.length > 0) {
      selectedScriptFile.value = sk.scripts[0] || ''
    } else {
      selectedScriptFile.value = ''
    }
  }
})

const activeTasksCount = computed(() => tasks.value.filter((t) => t.enabled).length)

const loadTasks = async () => {
  isLoading.value = true
  try {
    const list = await invoke<ScheduledTask[]>('scheduler_get_tasks')
    tasks.value = list || []
  } catch (err) {
    console.error('Failed to load scheduled tasks:', err)
  } finally {
    isLoading.value = false
  }
}

const openHistoryModal = async () => {
  showHistoryModal.value = true
  await loadTaskRuns()
}

const openTaskHistory = (taskId: string) => {
  selectedTaskFilter.value = taskId
  openHistoryModal()
}

const loadTaskRuns = async () => {
  try {
    const args: { taskId?: string; limit: number } = { limit: 50 }
    if (selectedTaskFilter.value) {
      args.taskId = selectedTaskFilter.value
    }
    const runs = await invoke<ScheduledTaskRun[]>('scheduler_get_task_runs', args)
    taskRuns.value = runs || []
  } catch (err) {
    console.error('Failed to load task runs:', err)
  }
}

const toggleRunSteps = (runId: string) => {
  expandedStepsByRun.value[runId] = !expandedStepsByRun.value[runId]
}

const parseStepsDetail = (detailStr?: string): TaskRunStep[] => {
  if (!detailStr) return []
  try {
    const parsed = JSON.parse(detailStr)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

const openChatSession = (sessionId?: string) => {
  if (!sessionId) return
  showHistoryModal.value = false
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('open-chat-session', { detail: { sessionId } }))
  }
}

const parseTools = (toolsStr?: string): string[] => {
  if (!toolsStr) return []
  try {
    const parsed = JSON.parse(toolsStr)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

const formatDeliveryLabel = (channel?: string) => {
  if (channel === 'telegram') return 'Telegram'
  if (channel === 'both') return 'Chat + Telegram'
  if (channel === 'silent') return 'Silencioso'
  return 'Chat'
}

const toggleTask = async (task: ScheduledTask) => {
  try {
    const nextState = !task.enabled
    task.enabled = nextState
    await invoke('scheduler_toggle_task', {
      taskId: task.id,
      enabled: nextState
    })
  } catch (err) {
    console.error('Failed to toggle task:', err)
  }
}

const deleteTask = async (taskId: string) => {
  try {
    await invoke('scheduler_delete_task', { taskId })
    tasks.value = tasks.value.filter((t) => t.id !== taskId)
  } catch (err) {
    console.error('Failed to delete task:', err)
  }
}

const runNow = async (task: ScheduledTask) => {
  runningTaskId.value = task.id
  try {
    await invoke('scheduler_run_now', { taskId: task.id })
    // Refresh after a short delay
    setTimeout(loadTasks, 2000)
  } catch (err) {
    console.error('Failed to trigger task:', err)
  } finally {
    setTimeout(() => {
      runningTaskId.value = null
    }, 1500)
  }
}

const openNewTaskModal = async () => {
  newTask.value = {
    name: '',
    description: '',
    cron_expr: '@daily',
    action_type: 'autonomous_prompt',
    delivery_channel: 'chat',
    delivery_target: ''
  }
  promptInput.value = 'Provide a concise daily status briefing.'
  selectedSkillId.value = ''
  selectedScriptFile.value = ''
  try {
    const sList = await invoke<any[]>('skills_get_all')
    availableSkills.value = sList || []
  } catch (err) {
    console.error('Failed to load skills:', err)
  }
  showModal.value = true
}

const saveNewTask = async () => {
  const now = new Date().toISOString()
  const payload = newTask.value.action_type === 'autonomous_prompt'
    ? JSON.stringify({ prompt: promptInput.value })
    : newTask.value.action_type === 'skill'
    ? JSON.stringify({
        skill_slug: selectedSkillId.value,
        script_file: selectedScriptFile.value || undefined,
        tool_name: selectedScriptFile.value ? 'run_skill_script' : undefined
      })
    : JSON.stringify({})

  const taskToSave: ScheduledTask = {
    id: `task-${Date.now()}`,
    name: newTask.value.name.trim(),
    description: newTask.value.description.trim() || undefined,
    cron_expr: newTask.value.cron_expr.trim(),
    action_type: newTask.value.action_type,
    payload,
    enabled: true,
    delivery_channel: newTask.value.delivery_channel,
    delivery_target: newTask.value.delivery_target.trim() || undefined,
    created_at: now,
    updated_at: now
  }

  try {
    await invoke('scheduler_save_task', { task: taskToSave })
    tasks.value.push(taskToSave)
    showModal.value = false
  } catch (err) {
    console.error('Failed to save task:', err)
  }
}

const formatTimestamp = (ts?: string) => {
  if (!ts) return 'Never'
  try {
    const d = new Date(ts)
    return d.toLocaleString([], {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return ts
  }
}

onMounted(() => {
  loadTasks()
})
</script>
