<template>
  <aside class="w-[260px] min-w-[260px] h-full bg-[#0a0c13]/95 backdrop-blur-xl border-r border-[#1a1f30] flex flex-col select-none relative z-30">
    <!-- Brand / Logo Section -->
    <div class="px-3.5 py-3 flex items-center justify-between border-b border-[#1a1f30] relative">
      <div class="flex items-center gap-2.5">
        <div class="w-8 h-8 rounded-xl overflow-hidden shadow-md shadow-indigo-500/20 ring-1 ring-indigo-500/30 flex items-center justify-center bg-[#0e101c] flex-shrink-0">
          <img src="/atena_logo.png" alt="Atena Studio Logo" class="w-full h-full object-cover" />
        </div>
        <div>
          <h1 class="text-sm font-bold tracking-tight text-slate-100 leading-none">Atena Studio</h1>
          <p class="text-[10px] text-slate-400 font-medium tracking-wide mt-1">Local AI Runner</p>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <!-- Plugins Button -->
        <button
          @click="$emit('openPlugins')"
          class="p-1.5 rounded-xl border text-xs font-medium transition-all relative cursor-pointer active:scale-95 shadow-sm bg-[#121522] hover:bg-[#181d2e] border-[#1e2338] hover:border-indigo-500/30 text-slate-400 hover:text-indigo-300"
          :title="$t('sidebar.plugins_tooltip')"
        >
          <Blocks class="w-3.5 h-3.5" />
        </button>

        <!-- Notification Bell Button -->
        <button
          @click="toggleNotifications"
          :class="[
            'p-1.5 rounded-xl border text-xs font-medium transition-all relative cursor-pointer active:scale-95 shadow-sm',
            isNotificationsOpen
              ? 'bg-indigo-600 border-indigo-500 text-white shadow-indigo-600/25'
              : unreadCount > 0
              ? 'bg-[#121522] hover:bg-[#181d2e] border-amber-500/40 text-amber-300'
              : 'bg-[#121522] hover:bg-[#181d2e] border-[#1e2338] hover:border-indigo-500/30 text-slate-400 hover:text-slate-200'
          ]"
          :title="$t('sidebar.notifications_tooltip')"
        >
          <Bell class="w-3.5 h-3.5" />
          <span
            v-if="unreadCount > 0"
            class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-amber-400 text-slate-950 text-[8.5px] font-bold flex items-center justify-center animate-pulse shadow-md"
          >
            {{ unreadCount > 9 ? '9+' : unreadCount }}
          </span>
        </button>
      </div>

      <!-- Notifications Dropdown Flyout -->
      <div
        v-if="isNotificationsOpen"
        class="absolute left-2 top-14 w-80 max-h-[460px] bg-[#0f121d] border border-[#242b44] rounded-2xl shadow-2xl overflow-hidden flex flex-col z-50 animate-in fade-in zoom-in-95 duration-150"
      >
        <!-- Panel Header -->
        <div class="p-3 bg-[#131726] border-b border-[#20273d] flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Bell class="w-3.5 h-3.5 text-indigo-400" />
            <span class="text-xs font-bold text-slate-100">{{ $t('sidebar.notifications') }}</span>
            <span v-if="notifications.length > 0" class="text-[10px] font-mono text-slate-400 bg-[#1c2236] px-1.5 py-0.2 rounded">
              {{ notifications.length }}
            </span>
          </div>

          <div class="flex items-center gap-1">
            <button
              v-if="notifications.length > 0"
              @click="clearAllNotifications"
              class="text-[10.5px] text-slate-400 hover:text-rose-300 px-2 py-0.5 rounded hover:bg-rose-500/10 transition-colors cursor-pointer"
              :title="$t('sidebar.clear_notifications')"
            >
              {{ $t('sidebar.clear_notifications') }}
            </button>
            <button
              @click="isNotificationsOpen = false"
              class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-[#1f263d] transition-colors cursor-pointer"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <!-- Notification Items List -->
        <div class="flex-1 overflow-y-auto p-2 space-y-1.5 max-h-[380px]">
          <div
            v-if="notifications.length === 0"
            class="py-10 flex flex-col items-center justify-center text-center text-slate-500 space-y-1.5"
          >
            <BellOff class="w-7 h-7 text-slate-600" />
            <span class="text-xs text-slate-400 font-medium">{{ $t('sidebar.no_notifications') }}</span>
            <span class="text-[10.5px] text-slate-500">{{ $t('sidebar.no_notifications_desc') }}</span>
          </div>

          <div
            v-for="item in notifications"
            :key="item.id"
            :class="[
              'p-2.5 rounded-xl border transition-all flex items-start gap-2.5 group',
              item.read
                ? 'bg-[#121624]/60 border-[#1a2034] opacity-80'
                : 'bg-[#14192b] border-[#222b48] shadow-sm'
            ]"
          >
            <!-- Icon by type -->
            <div
              class="w-6 h-6 rounded-lg flex items-center justify-center flex-shrink-0 mt-0.5"
              :class="[
                item.type === 'download_completed'
                  ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30'
                  : item.type === 'download_cancelled'
                  ? 'bg-slate-500/15 text-slate-400 border border-slate-500/30'
                  : item.type === 'download_error'
                  ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30'
                  : item.type === 'model_deleted'
                  ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30'
                  : item.type === 'model_favorited'
                  ? 'bg-amber-500/15 text-amber-400 border border-amber-500/30'
                  : 'bg-indigo-500/15 text-indigo-400 border border-indigo-500/30'
              ]"
            >
              <CheckCircle2 v-if="item.type === 'download_completed'" class="w-3.5 h-3.5" />
              <Ban v-else-if="item.type === 'download_cancelled'" class="w-3.5 h-3.5" />
              <AlertTriangle v-else-if="item.type === 'download_error'" class="w-3.5 h-3.5" />
              <Trash2 v-else-if="item.type === 'model_deleted'" class="w-3.5 h-3.5" />
              <Star v-else-if="item.type === 'model_favorited'" class="w-3.5 h-3.5" />
              <Info v-else class="w-3.5 h-3.5" />
            </div>

            <!-- Message Content -->
            <div class="flex-1 truncate">
              <div class="flex items-center justify-between gap-1">
                <h5 class="text-xs font-bold text-slate-200 truncate">{{ item.title }}</h5>
                <span class="text-[9.5px] font-mono text-slate-500 flex-shrink-0">
                  {{ formatRelativeTime(item.timestamp) }}
                </span>
              </div>
              <p class="text-[11px] text-slate-400 mt-0.5 line-clamp-2 leading-tight">
                {{ item.message }}
              </p>
            </div>

            <!-- Item Delete Button -->
            <button
              @click.stop="removeNotification(item.id)"
              class="opacity-0 group-hover:opacity-100 p-1 text-slate-500 hover:text-rose-400 transition-opacity cursor-pointer"
              :title="$t('sidebar.dismiss_notification')"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Navigation Tabs Bar -->
    <div class="p-2 border-b border-[#1a1f30]">
      <div :class="['grid gap-1 bg-[#10131f] p-1 rounded-xl border border-[#1b2133]', enableMemory !== false ? 'grid-cols-4' : 'grid-cols-3']">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="$emit('update:activeTab', tab.id)"
          :class="[
            'flex flex-col items-center justify-center py-2 px-0.5 rounded-lg transition-all text-center group relative cursor-pointer select-none',
            activeTab === tab.id
              ? 'bg-gradient-to-b from-[#1c2236] to-[#151929] text-indigo-300 font-semibold shadow-sm border border-indigo-500/30'
              : 'text-slate-400 hover:text-slate-200 hover:bg-[#151928]'
          ]"
          :title="tab.label"
        >
          <component :is="tab.icon" class="w-4 h-4 mb-1 transition-transform group-hover:scale-110" />
          <span class="text-[9.5px] tracking-tight font-medium leading-none">{{ tab.label }}</span>
        </button>
      </div>
    </div>

    <!-- History / Sessions Area -->
    <div class="flex-1 overflow-hidden flex flex-col p-3 gap-2">
      <!-- Chat Creation Actions -->
      <div :class="['flex items-center gap-1.5', enableMemory !== false ? '' : '']">
        <button
          @click="$emit('newChat')"
          :class="[
            'flex items-center justify-center gap-1.5 py-2 px-2.5 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white rounded-xl text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all active:scale-[0.97] border border-indigo-400/25 group cursor-pointer',
            enableMemory !== false ? 'flex-1 min-w-0' : 'w-full'
          ]"
          :title="$t('sidebar.new_chat_tooltip')"
        >
          <Plus class="w-4 h-4 flex-shrink-0 transition-transform group-hover:rotate-90" />
          <span class="truncate">{{ $t('sidebar.new_chat') }}</span>
        </button>

        <!-- Private Chat Button (Visible when Cognitive Memory is active) -->
        <button
          v-if="enableMemory !== false"
          @click="$emit('newPrivateChat')"
          class="flex items-center justify-center gap-1.5 py-2 px-2.5 rounded-xl text-xs font-medium bg-[#131728] hover:bg-violet-950/40 border border-[#232a44] hover:border-violet-500/40 text-violet-300 hover:text-violet-200 transition-all active:scale-[0.97] shadow-sm group cursor-pointer flex-shrink-0"
          :title="$t('sidebar.private_chat_tooltip')"
        >
          <EyeOff class="w-3.5 h-3.5 text-violet-400 group-hover:scale-110 transition-transform" />
          <span class="text-[11px] font-medium hidden sm:inline">{{ $t('sidebar.private') }}</span>
        </button>
      </div>

      <!-- Search Box with ⌘K shortcut hint -->
      <div class="relative">
        <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none" />
        <input
          ref="searchInputRef"
          v-model="searchQuery"
          type="text"
          :placeholder="`${$t('common.search')} (⌘K)`"
          class="w-full pl-8 pr-7 py-1.5 rounded-xl bg-[#10131f] border border-[#1b2133] focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20 text-slate-200 placeholder-slate-500 text-[11px] outline-none transition-all"
        />
        <button
          v-if="searchQuery"
          @click="searchQuery = ''"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 p-0.5 cursor-pointer"
          :title="$t('common.cancel')"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Multi-selection Action Toolbar -->
      <div
        v-if="selectedSessionIds.size > 0"
        class="px-2.5 py-1.5 rounded-xl bg-[#121524] border border-[#222b44] flex items-center justify-between gap-2 shadow-xl animate-in slide-in-from-top-1 duration-150"
      >
        <span class="text-[10.5px] font-medium text-slate-300 truncate">
          {{ $t('sidebar.selected_count', { count: selectedSessionIds.size }) }}
        </span>

        <div class="flex items-center gap-1 shrink-0">
          <button
            type="button"
            @click="clearSelection"
            class="px-1.5 py-0.5 rounded text-[10px] text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
            :title="$t('sidebar.cancel_selection')"
          >
            {{ $t('common.cancel') }}
          </button>

          <button
            type="button"
            @click="archiveSelectedSessions"
            class="px-2 py-0.5 rounded-lg bg-[#181d30] hover:bg-[#202740] text-amber-300 hover:text-amber-200 border border-amber-500/30 text-[10px] font-medium flex items-center gap-1 transition-all active:scale-95 cursor-pointer shadow-sm"
            :title="$t('sidebar.archive_selected_tooltip')"
          >
            <Archive class="w-3 h-3 text-amber-400" />
            <span>{{ $t('sidebar.archive_selected') }}</span>
          </button>

          <button
            type="button"
            @click="deleteSelectedSessions"
            class="px-2 py-0.5 rounded-lg bg-[#181d30] hover:bg-rose-950/40 text-rose-300 hover:text-rose-200 border border-rose-500/30 text-[10px] font-medium flex items-center gap-1 transition-all active:scale-95 cursor-pointer shadow-sm"
            :title="$t('sidebar.delete_selected_tooltip')"
          >
            <Trash2 class="w-3 h-3 text-rose-400" />
            <span>{{ $t('sidebar.delete_selected') }}</span>
          </button>
        </div>
      </div>

      <!-- Dynamic Lists Container (Projetos + Recentes) -->
      <div class="flex-1 min-h-0 flex flex-col gap-2">
        <!-- Projetos Section (Ocupa apenas o necessário, limitado a no máximo 50%) -->
        <div class="flex-initial max-h-[50%] min-h-0 flex flex-col space-y-1">
          <div class="flex items-center justify-between px-1 pt-1 flex-shrink-0">
            <span class="text-[10px] font-semibold text-slate-400 uppercase tracking-wider">
              {{ $t('sidebar.projects') }}
            </span>
            <div class="flex items-center gap-1">
              <span v-if="projects.length > 0" class="text-[10px] font-mono font-medium text-slate-400 bg-[#121522] px-1.5 py-0.5 rounded-md border border-[#1e2439]">
                {{ projects.length }}
              </span>
              <button
                @click="openCreateProject"
                class="p-1 rounded-md text-slate-400 hover:text-indigo-300 hover:bg-[#181d2e] transition-colors cursor-pointer"
                :title="$t('sidebar.add_project')"
              >
                <Plus class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          <div class="flex-1 min-h-0 space-y-1 overflow-y-auto pr-0.5">
          <div
            v-for="project in projects"
            :key="project.id"
            class="space-y-0.5 rounded-xl transition-all duration-200"
            :class="[
              dragOverProjectId === project.id
                ? 'bg-indigo-600/20 ring-2 ring-indigo-500/70 border border-indigo-400 shadow-md shadow-indigo-500/20'
                : ''
            ]"
            @dragover.prevent="handleDragOverProject(project.id, $event)"
            @dragleave="handleDragLeaveProject(project.id, $event)"
            @drop.prevent="handleDropOnProject(project.id, $event)"
          >
            <div
              @click="handleProjectClick(project.id)"
              :class="[
                'group flex items-center justify-between px-2 py-1.5 rounded-xl cursor-pointer text-xs transition-all duration-200 border relative select-none',
                isProjectSelected(project.id)
                  ? 'bg-[#151a2d] border-indigo-500/40 text-slate-100 font-medium shadow-sm'
                  : 'border-transparent text-slate-300 hover:bg-[#121626] hover:text-white'
              ]"
              :title="project.name"
            >
              <!-- Left active indicator bar -->
              <div
                v-if="isProjectSelected(project.id)"
                class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-3.5 rounded-r bg-indigo-500"
              />

              <div class="flex items-center gap-1.5 truncate min-w-0 flex-1">
                <!-- Toggle expand chevron -->
                <button
                  @click.stop="toggleProjectExpand(project.id)"
                  class="p-0.5 rounded hover:bg-white/10 text-slate-400 hover:text-white transition-colors cursor-pointer"
                  :title="isProjectExpanded(project.id) ? $t('sidebar.collapse_chats') : $t('sidebar.expand_chats')"
                >
                  <component
                    :is="isProjectExpanded(project.id) ? ChevronDown : ChevronRight"
                    class="w-3 h-3 text-slate-400"
                  />
                </button>

                <Folder
                  :class="[
                    'w-3.5 h-3.5 flex-shrink-0 transition-colors',
                    isProjectSelected(project.id) ? 'text-amber-400' : 'text-slate-400 group-hover:text-amber-400'
                  ]"
                />
                <span class="truncate text-[11.5px] font-medium">{{ project.name }}</span>

                <!-- Chat count badge -->
                <span
                  v-if="getProjectSessions(project.id).length > 0"
                  class="text-[9.5px] font-mono text-slate-400 bg-[#101322] px-1 py-0.2 rounded border border-[#1d2338]"
                >
                  {{ getProjectSessions(project.id).length }}
                </span>
              </div>

              <!-- Actions on hover: New Chat in Project (+), Manage (⚙️) and Delete (🗑️) -->
              <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                <button
                  @click.stop="handleNewProjectChat(project.id)"
                  class="p-1 text-slate-400 hover:text-indigo-200 hover:bg-indigo-600/30 rounded-md transition-colors cursor-pointer"
                  :title="$t('sidebar.new_chat_in_project')"
                >
                  <Plus class="w-3.5 h-3.5 text-indigo-300" />
                </button>
                <button
                  @click.stop="openEditProject(project)"
                  class="p-1 text-slate-400 hover:text-indigo-300 hover:bg-[#1a2035] rounded-md transition-colors cursor-pointer"
                  :title="$t('sidebar.edit_project')"
                >
                  <Pencil class="w-3 h-3" />
                </button>
                <button
                  @click.stop="openDeleteProjectModal(project)"
                  class="p-1 text-slate-400 hover:text-rose-400 hover:bg-rose-500/20 rounded-md transition-colors cursor-pointer"
                  :title="$t('sidebar.delete_project')"
                >
                  <Trash2 class="w-3 h-3" />
                </button>
              </div>
            </div>

            <!-- Nested chats list for this project -->
            <div
              v-if="isProjectExpanded(project.id)"
              class="ml-3.5 pl-2 border-l border-indigo-500/20 space-y-0.5 py-0.5"
            >
              <div
                v-for="session in getProjectFilteredSessions(project.id)"
                :key="session.id"
                @click="handleSessionClick(session.id, $event)"
                :draggable="editingSessionId !== session.id"
                @dragstart="handleDragStart(session.id, $event)"
                @dragend="handleDragEnd"
                :class="[
                  'group flex items-center justify-between p-1.5 rounded-lg cursor-pointer text-xs transition-all duration-200 border relative cursor-grab active:cursor-grabbing',
                  draggedSessionId === session.id ? 'opacity-40 scale-95' : '',
                  selectedSessionIds.has(session.id)
                    ? 'bg-indigo-600/25 border-indigo-500/60 text-indigo-100 font-medium shadow-sm ring-1 ring-indigo-500/30'
                    : isSessionActive(session.id)
                      ? 'bg-indigo-600/20 border-indigo-500/40 text-indigo-100 font-medium shadow-sm'
                      : 'border-transparent text-slate-400 hover:bg-[#121626] hover:text-slate-200'
                ]"
                :title="$t('sidebar.drag_to_move_project_tooltip')"
              >
                <!-- Title / Selection Checkbox / Inline Edit -->
                <div class="flex items-center gap-1.5 truncate pr-1 flex-1 min-w-0">
                  <button
                    v-if="selectedSessionIds.size > 0"
                    type="button"
                    @click.stop="toggleSessionSelection(session.id)"
                    class="p-0.5 rounded transition-all cursor-pointer flex-shrink-0 text-indigo-400"
                    :title="selectedSessionIds.has(session.id) ? $t('sidebar.unselect_session_tooltip') : $t('sidebar.select_session_tooltip')"
                  >
                    <component
                      :is="selectedSessionIds.has(session.id) ? CheckSquare : Square"
                      class="w-3 h-3"
                    />
                  </button>

                  <MessageSquare
                    :class="[
                      'w-3 h-3 flex-shrink-0',
                      activeSessionId === session.id && activeTab === 'chat' ? 'text-indigo-300' : 'text-slate-500 group-hover:text-slate-400'
                    ]"
                  />
                  <!-- Inline editing input -->
                  <input
                    v-if="editingSessionId === session.id"
                    ref="editInputRef"
                    v-model="editingTitle"
                    type="text"
                    class="w-full bg-[#0d101a] border border-indigo-500/70 rounded px-1.5 py-0.5 text-[11px] text-slate-100 outline-none select-text"
                    @click.stop
                    @keydown.enter.stop="saveInlineEdit(session)"
                    @keydown.esc.stop="cancelInlineEdit"
                    @blur="saveInlineEdit(session)"
                  />
                  <span
                    v-else
                    class="truncate text-[11px]"
                    :title="session.title || $t('chat.new_chat')"
                    @dblclick.stop="startInlineEdit(session)"
                  >
                    {{ session.title || $t('chat.new_chat') }}
                  </span>
                </div>

                <!-- Actions on hover: single More button -->
                <div
                  v-if="editingSessionId !== session.id"
                  class="flex items-center opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
                >
                  <button
                    @click.stop="openSessionContextMenu(session, $event)"
                    class="p-0.5 hover:bg-white/10 rounded text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                    :title="$t('sidebar.more_options')"
                  >
                    <MoreHorizontal class="w-3 h-3" />
                  </button>
                </div>
              </div>

              <!-- Button to start new chat in project if empty -->
              <div
                v-if="getProjectSessions(project.id).length === 0"
                @click="handleNewProjectChat(project.id)"
                class="px-2 py-1 rounded-lg text-[10.5px] text-slate-500 hover:text-indigo-300 hover:bg-indigo-500/10 cursor-pointer flex items-center gap-1.5 transition-colors"
              >
                <Plus class="w-3 h-3" />
                <span>{{ $t('sidebar.new_chat') }}</span>
              </div>
            </div>
          </div>

          <div
            v-if="projects.length === 0"
            @click="openCreateProject"
            class="px-2 py-1.5 rounded-xl border border-dashed border-[#1f263e] hover:border-indigo-500/40 text-slate-500 hover:text-indigo-300 text-[11px] flex items-center justify-center gap-1.5 cursor-pointer transition-colors"
          >
            <Plus class="w-3 h-3" />
            <span>{{ $t('sidebar.add_project') }}</span>
          </div>
        </div>
      </div>

      <!-- Recentes Section (50% do espaço vertical) -->
      <div class="flex-1 min-h-0 flex flex-col space-y-1 border-t border-[#1a1f30]/60 pt-1.5">
        <div
          class="flex items-center justify-between px-1 pt-0.5 flex-shrink-0 transition-all rounded-lg"
          :class="[isDragOverRecents ? 'bg-indigo-500/15 ring-1 ring-indigo-500/40 p-1' : '']"
          @dragover.prevent="handleDragOverRecents"
          @dragleave="handleDragLeaveRecents"
          @drop.prevent="handleDropOnRecents"
        >
          <span class="text-[10px] font-semibold text-slate-400 uppercase tracking-wider">
            {{ $t('sidebar.recents') }}
          </span>
          <span class="text-[10px] font-mono font-medium text-slate-400 bg-[#121522] px-1.5 py-0.5 rounded-md border border-[#1e2439]">
            {{ unassignedSessions.length }}
          </span>
        </div>

        <div
          class="flex-1 min-h-0 overflow-y-auto space-y-1 pr-0.5 transition-all rounded-xl p-0.5"
          :class="[isDragOverRecents ? 'bg-indigo-500/10 ring-2 ring-dashed ring-indigo-500/50' : '']"
          @dragover.prevent="handleDragOverRecents"
          @dragleave="handleDragLeaveRecents"
          @drop.prevent="handleDropOnRecents"
        >
        <div
          v-for="session in unassignedSessions"
          :key="session.id"
          @click="handleSessionClick(session.id, $event)"
          :draggable="editingSessionId !== session.id"
          @dragstart="handleDragStart(session.id, $event)"
          @dragend="handleDragEnd"
          :class="[
            'group flex items-center justify-between p-2 rounded-xl cursor-pointer text-xs transition-all duration-200 border relative overflow-hidden cursor-grab active:cursor-grabbing',
            draggedSessionId === session.id ? 'opacity-40 scale-95' : '',
            selectedSessionIds.has(session.id)
              ? 'bg-[#181e36] border-indigo-500/60 text-indigo-100 font-medium shadow-sm ring-1 ring-indigo-500/30'
              : isSessionActive(session.id)
                ? 'bg-[#151a2d] border-indigo-500/40 text-slate-100 font-medium shadow-sm'
                : 'border-transparent text-slate-400 hover:bg-[#121626] hover:text-slate-200'
          ]"
          :title="$t('sidebar.drag_to_link_project_tooltip')"
        >
          <!-- Left active indicator dot -->
          <div
            v-if="isSessionActive(session.id)"
            class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-4 rounded-r bg-indigo-500"
          />

          <!-- Title / Selection Checkbox / Inline Edit -->
          <div class="flex items-center gap-1.5 truncate pr-1 flex-1 min-w-0">
            <button
              v-if="selectedSessionIds.size > 0"
              type="button"
              @click.stop="toggleSessionSelection(session.id)"
              class="p-0.5 rounded transition-all cursor-pointer flex-shrink-0 text-indigo-400"
              :title="selectedSessionIds.has(session.id) ? $t('sidebar.unselect_session_tooltip') : $t('sidebar.select_session_tooltip')"
            >
              <component
                :is="selectedSessionIds.has(session.id) ? CheckSquare : Square"
                class="w-3.5 h-3.5"
              />
            </button>

            <Pin
              v-if="session.pinned"
              class="w-3 h-3 text-amber-400 flex-shrink-0 fill-amber-400/20"
            />
            <EyeOff
              v-else-if="session.is_private"
              :class="[
                'w-3.5 h-3.5 flex-shrink-0 transition-colors',
                activeSessionId === session.id && activeTab === 'chat' ? 'text-violet-400' : 'text-violet-400/70 group-hover:text-violet-300'
              ]"
              :title="$t('sidebar.private_session_badge_tooltip')"
            />
            <MessageSquare
              v-else
              :class="[
                'w-3.5 h-3.5 flex-shrink-0 transition-colors',
                activeSessionId === session.id && activeTab === 'chat' ? 'text-indigo-400' : 'text-slate-500 group-hover:text-slate-300'
              ]"
            />

            <!-- Inline Editing Input -->
            <input
              v-if="editingSessionId === session.id"
              ref="editInputRef"
              v-model="editingTitle"
              type="text"
              class="w-full bg-[#0d101a] border border-indigo-500/70 rounded px-1.5 py-0.5 text-xs text-slate-100 outline-none select-text"
              @click.stop
              @keydown.enter.stop="saveInlineEdit(session)"
              @keydown.esc.stop="cancelInlineEdit"
              @blur="saveInlineEdit(session)"
            />

            <!-- Title Display -->
            <span
              v-else
              class="truncate text-[11.5px]"
              :title="session.title || $t('chat.new_chat')"
              @dblclick.stop="startInlineEdit(session)"
            >
              {{ session.title || $t('chat.new_chat') }}
            </span>
          </div>

          <!-- Actions on Hover: Single minimalist More button -->
          <div
            v-if="editingSessionId !== session.id"
            class="flex items-center opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
          >
            <button
              @click.stop="openSessionContextMenu(session, $event)"
              class="p-1 rounded-md text-slate-400 hover:text-slate-100 hover:bg-white/10 transition-colors cursor-pointer"
              :title="$t('sidebar.more_options')"
            >
              <MoreHorizontal class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <div v-if="unassignedSessions.length === 0 && projects.length > 0 && !searchQuery" class="text-center py-4 text-[11px] text-slate-500">
          <p>{{ $t('sidebar.no_sessions_outside_projects') }}</p>
        </div>
        <div v-else-if="filteredSessions.length === 0" class="text-center py-8 text-xs text-slate-500">
          <p v-if="searchQuery">{{ $t('sidebar.no_sessions_found', { query: searchQuery }) }}</p>
          <p v-else>{{ $t('sidebar.no_sessions') }}</p>
        </div>
      </div>
      </div>
      </div>

      <!-- Archived Chats Entry Button (Minimalist & only when > 0) -->
      <div v-if="archivedSessionsCount > 0" class="pt-1 flex-shrink-0 border-t border-[#1a1f30]/40">
        <button
          @click="$emit('openArchivedModal')"
          class="w-full flex items-center justify-between px-2 py-1.5 rounded-xl text-slate-400 hover:text-slate-200 hover:bg-[#131726] transition-all cursor-pointer group text-xs select-none"
          :title="$t('sidebar.archived_chats_tooltip')"
        >
          <div class="flex items-center gap-2">
            <Archive class="w-3.5 h-3.5 text-slate-500 group-hover:text-amber-400 transition-colors" />
            <span class="text-[11px] font-medium">{{ $t('sidebar.archived_chats') }}</span>
          </div>
          <span
            class="text-[9.5px] font-mono font-medium text-slate-400 bg-[#101320] group-hover:bg-[#181d2e] group-hover:text-amber-300 px-1.5 py-0.2 rounded border border-[#1b2133] transition-colors"
          >
            {{ archivedSessionsCount }}
          </span>
        </button>
      </div>
    </div>

    <!-- Bottom Hardware Telemetry Box -->
    <div class="p-3 border-t border-[#1a1f30] bg-[#080a10]/95">
      <div class="p-2.5 rounded-2xl bg-[#101320] border border-[#1b2135] space-y-2.5 text-xs shadow-inner">
        <div class="flex items-center justify-between gap-1.5">
          <div class="flex items-center gap-1.5 truncate min-w-0" :title="defaultDeviceName">
            <Cpu class="w-3.5 h-3.5 text-indigo-400 flex-shrink-0" />
            <span class="font-medium text-slate-200 truncate text-[11px]">{{ defaultDeviceName }}</span>
          </div>
          <span
            class="text-[9.5px] text-indigo-300 bg-indigo-500/10 px-1.5 py-0.5 rounded-md border border-indigo-500/20 font-semibold tracking-wide font-mono shrink-0 whitespace-nowrap"
            :title="accelerationBackendTooltip"
          >
            {{ shortAccelerationBackend }}
          </span>
        </div>

        <!-- AI Model / Engine Memory (Dedicated) -->
        <div>
          <div class="flex justify-between items-center text-[10px] text-slate-300 mb-1 font-medium">
            <span class="flex items-center gap-1 text-teal-300 font-semibold truncate pr-1">
              <Zap class="w-3 h-3 text-teal-400 shrink-0" />
              <span class="truncate">{{ $t('sidebar.ai_memory') }}</span>
            </span>
            <span class="font-mono text-teal-300 font-bold shrink-0 whitespace-nowrap">
              {{ formatGb(hardware.ai_ram_gb || hardware.used_vram_gb) }} GB
            </span>
          </div>
          <div class="w-full h-1.5 bg-[#090b12] rounded-full overflow-hidden border border-[#1a2033]">
            <div
              class="h-full rounded-full bg-gradient-to-r from-teal-400 to-indigo-500 transition-all duration-500"
              :style="{ width: `${aiRamPct}%` }"
            ></div>
          </div>
        </div>

        <!-- System RAM (Shared) with Visual Breakdown -->
        <div
          class="group cursor-help"
          :title="`${$t('sidebar.ai_tag')}: ${formatGb(hardware.ai_ram_gb)} GB | Apps: ${formatGb(hardware.system_other_ram_gb)} GB | ${$t('sidebar.free_tag')}: ${formatGb(hardware.free_ram_gb)} GB`"
        >
          <div class="flex justify-between items-center text-[10px] text-slate-400 mb-1">
            <span class="truncate pr-1">{{ $t('sidebar.system_ram') }}</span>
            <span class="font-mono text-slate-300 shrink-0 whitespace-nowrap">{{ formatGb(hardware.used_ram_gb) }}/{{ formatGb(hardware.total_ram_gb) }} GB ({{ ramPct }}%)</span>
          </div>
          <div class="w-full h-1.5 bg-[#090b12] rounded-full overflow-hidden border border-[#1a2033] flex">
            <!-- AI Segment (Teal) -->
            <div
              class="h-full bg-teal-400 transition-all duration-500"
              :style="{ width: `${aiRamPct}%` }"
            ></div>
            <!-- Other Apps Segment (Indigo) -->
            <div
              class="h-full bg-indigo-500/70 transition-all duration-500"
              :style="{ width: `${systemOtherPct}%` }"
            ></div>
          </div>
          <div class="flex justify-between text-[9px] text-slate-400 font-mono mt-1 px-0.5">
            <span class="text-teal-300">{{ $t('sidebar.ai_tag') }}: {{ formatGb(hardware.ai_ram_gb) }}G</span>
            <span class="text-indigo-300">{{ $t('sidebar.apps_tag') }}: {{ formatGb(hardware.system_other_ram_gb) }}G</span>
            <span class="text-slate-400">{{ $t('sidebar.free_tag') }}: {{ formatGb(hardware.free_ram_gb) }}G</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal de Gerenciamento de Projetos e Pastas -->
    <ProjectModal
      :is-open="isProjectModalOpen"
      :mode="projectModalMode"
      :target-project="targetProject"
      @close="isProjectModalOpen = false"
      @saved="isProjectModalOpen = false"
      @deleted="onProjectDeleted"
    />

    <!-- Modal de Confirmação de Exclusão de Projeto -->
    <DeleteProjectModal
      :is-open="isDeleteProjectModalOpen"
      :project="projectToDelete"
      :session-count="projectToDelete ? getProjectSessions(projectToDelete.id).length : 0"
      @close="isDeleteProjectModalOpen = false"
      @confirm="handleConfirmDeleteProject"
    />

    <!-- Context Menu Dropdown Teleport -->
    <Teleport to="body">
      <div
        v-if="contextMenuTarget && contextMenuPos"
        class="fixed z-[9999] w-36 bg-[#121627]/95 backdrop-blur-xl border border-[#232b45] rounded-xl shadow-2xl py-1 animate-in fade-in zoom-in-95 duration-100 text-xs text-slate-200 select-none shadow-black/80"
        :style="{ top: `${contextMenuPos.top}px`, left: `${contextMenuPos.left}px` }"
        @click.stop
      >
        <button
          @click="handleMenuPin(contextMenuTarget)"
          class="w-full px-2.5 py-1.5 flex items-center gap-2 hover:bg-[#1c2238] text-slate-300 hover:text-white transition-colors cursor-pointer text-left text-[11px]"
        >
          <Pin class="w-3 h-3 text-amber-400" :class="{ 'fill-amber-400': contextMenuTarget.pinned }" />
          <span>{{ contextMenuTarget.pinned ? $t('sidebar.unpin') : $t('sidebar.pin') }}</span>
        </button>

        <button
          @click="handleMenuRename(contextMenuTarget)"
          class="w-full px-2.5 py-1.5 flex items-center gap-2 hover:bg-[#1c2238] text-slate-300 hover:text-white transition-colors cursor-pointer text-left text-[11px]"
        >
          <Edit2 class="w-3 h-3 text-indigo-400" />
          <span>{{ $t('sidebar.rename') }}</span>
        </button>

        <button
          @click="handleMenuArchive(contextMenuTarget)"
          class="w-full px-2.5 py-1.5 flex items-center gap-2 hover:bg-[#1c2238] text-slate-300 hover:text-white transition-colors cursor-pointer text-left text-[11px]"
        >
          <Archive class="w-3 h-3 text-amber-400" />
          <span>{{ $t('sidebar.archive') }}</span>
        </button>

        <button
          @click="handleMenuExport(contextMenuTarget)"
          class="w-full px-2.5 py-1.5 flex items-center gap-2 hover:bg-[#1c2238] text-slate-300 hover:text-white transition-colors cursor-pointer text-left text-[11px]"
        >
          <Download class="w-3 h-3 text-teal-400" />
          <span>{{ $t('header.export') }}</span>
        </button>

        <div class="my-0.5 border-t border-[#1c2238]"></div>

        <button
          @click="handleMenuDelete(contextMenuTarget)"
          class="w-full px-2.5 py-1.5 flex items-center gap-2 hover:bg-rose-500/15 text-rose-400 hover:text-rose-300 transition-colors cursor-pointer text-left text-[11px]"
        >
          <Trash2 class="w-3 h-3 text-rose-400" />
          <span>{{ $t('common.delete') }}</span>
        </button>
      </div>

      <!-- Backdrop overlay to dismiss menu on outside click -->
      <div
        v-if="contextMenuTarget"
        class="fixed inset-0 z-[9998] bg-transparent cursor-default"
        @click="closeContextMenu"
      />
    </Teleport>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, inject, nextTick, onMounted, watch, type Ref, type ComputedRef } from 'vue'
import {
  MessageSquare,
  Layers,
  Brain,
  Zap,
  Activity,
  Wrench,
  Settings,
  Plus,
  Trash2,
  Cpu,
  Sparkles,
  Download,
  Bell,
  BellOff,
  X,
  CheckCircle2,
  Ban,
  AlertTriangle,
  Star,
  Info,
  Search,
  Edit2,
  Pin,
  Cloud,
  EyeOff,
  ShieldOff,
  Lock,
  Blocks,
  Folder,
  FolderPlus,
  Pencil,
  CheckSquare,
  Square,
  Check,
  ChevronRight,
  ChevronDown,
  Archive,
  ArchiveRestore,
  MoreHorizontal
} from 'lucide-vue-next'
import { formatSessionToMarkdown, downloadMarkdownFile } from '~/utils/exportMarkdown'
import {
  notifications,
  unreadCount,
  removeNotification,
  clearAllNotifications,
  markAllAsRead,
  formatRelativeTime
} from '~/utils/notifications'
import { useAppLocale } from '~/composables/useLocale'
import { useProjects, type Project } from '~/composables/useProjects'
import ProjectModal from '~/components/ProjectModal.vue'
import DeleteProjectModal from '~/components/DeleteProjectModal.vue'
import type { Session, Model, HardwareInfo } from '~/types'

const { t } = useAppLocale()

const {
  projects,
  activeProjectId,
  activeProject,
  expandedProjectIds,
  isProjectExpanded,
  toggleProjectExpand,
  setProjectExpanded,
  selectProject,
  deleteProject,
  initProjects,
} = useProjects()

const isProjectModalOpen = ref(false)
const projectModalMode = ref<'create' | 'edit'>('create')
const targetProject = ref<Project | null>(null)

const isDeleteProjectModalOpen = ref(false)
const projectToDelete = ref<Project | null>(null)

const openCreateProject = () => {
  projectModalMode.value = 'create'
  targetProject.value = null
  isProjectModalOpen.value = true
}

const openEditProject = (project: Project) => {
  projectModalMode.value = 'edit'
  targetProject.value = project
  isProjectModalOpen.value = true
}

const openDeleteProjectModal = (project: Project) => {
  projectToDelete.value = project
  isDeleteProjectModalOpen.value = true
}

const onProjectDeleted = (id: string) => {
  emit('deleteProject', id)
}

const handleConfirmDeleteProject = (project: Project) => {
  if (!project) return
  const id = project.id
  deleteProject(id)
  onProjectDeleted(id)
  isDeleteProjectModalOpen.value = false
  projectToDelete.value = null
}

onMounted(() => {
  initProjects()
})

const isNotificationsOpen = ref(false)
const searchQuery = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)
const editInputRef = ref<HTMLInputElement | null>(null)
const editingSessionId = ref<string | null>(null)
const editingTitle = ref('')

const toggleNotifications = () => {
  isNotificationsOpen.value = !isNotificationsOpen.value
  if (isNotificationsOpen.value) {
    markAllAsRead()
  }
}

const props = defineProps<{
  activeTab?: string
  sessions?: Session[]
  activeSessionId?: string | null
  hardware?: HardwareInfo | any
  activeModel?: Model | null
  enableMemory?: boolean
}>()

const supportsMlx = inject<Ref<boolean> | ComputedRef<boolean>>('supportsMlx', computed(() => false))
const platformInfo = inject<Ref<{ os: string; supports_mlx: boolean }>>('platformInfo', ref({ os: 'linux', supports_mlx: false }))

const emit = defineEmits<{
  'update:activeTab': [val: string]
  'selectSession': [id: string]
  'newChat': []
  'newProjectChat': [projectId: string]
  'newPrivateChat': []
  'deleteSession': [sessionOrId: Session | string]
  'deleteSessions': [sessions: Session[] | string[]]
  'archiveSession': [id: string]
  'archiveSessions': [ids: string[]]
  'openArchivedModal': []
  'renameSession': [payload: { id: string; title: string }]
  'togglePinSession': [id: string]
  'assignProject': [payload: { sessionId: string; projectId: string | null }]
  'deleteProject': [id: string]
  'openSetup': []
  'openPlugins': []
}>()

// Drag and Drop State for Sessions & Projects
const draggedSessionId = ref<string | null>(null)
const dragOverProjectId = ref<string | null>(null)
const isDragOverRecents = ref(false)

const handleDragStart = (sessionId: string, event: DragEvent) => {
  draggedSessionId.value = sessionId
  if (event && event.dataTransfer) {
    event.dataTransfer.setData('text/plain', sessionId)
    event.dataTransfer.effectAllowed = 'move'
  }
}

const handleDragEnd = () => {
  draggedSessionId.value = null
  dragOverProjectId.value = null
  isDragOverRecents.value = false
}

const handleDragOverProject = (projectId: string, event: DragEvent) => {
  if (draggedSessionId.value) {
    dragOverProjectId.value = projectId
    if (event && event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
  }
}

const handleDragLeaveProject = (projectId: string, event: DragEvent) => {
  if (dragOverProjectId.value === projectId) {
    dragOverProjectId.value = null
  }
}

const handleDropOnProject = (projectId: string, event: DragEvent) => {
  const sessionId = draggedSessionId.value || event?.dataTransfer?.getData('text/plain')
  if (sessionId) {
    emit('assignProject', { sessionId, projectId })
    setProjectExpanded(projectId, true)
  }
  draggedSessionId.value = null
  dragOverProjectId.value = null
}

const handleDragOverRecents = (event: DragEvent) => {
  if (draggedSessionId.value) {
    isDragOverRecents.value = true
    if (event && event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
  }
}

const handleDragLeaveRecents = () => {
  isDragOverRecents.value = false
}

const handleDropOnRecents = (event: DragEvent) => {
  const sessionId = draggedSessionId.value || event?.dataTransfer?.getData('text/plain')
  if (sessionId) {
    emit('assignProject', { sessionId, projectId: null })
  }
  draggedSessionId.value = null
  isDragOverRecents.value = false
}

const lastFocusedTarget = ref<{ type: string; id: string | null }>({ type: 'session', id: null })
const selectedSessionIds = ref<Set<string>>(new Set())
const lastClickedSessionId = ref<string | null>(null)
const selectedProjectId = ref<string | null>(null)

const isProjectSelected = (projectId: string) => {
  return selectedProjectId.value === projectId
}

const isSessionActive = (sessionId: string) => {
  return props.activeSessionId === sessionId && props.activeTab === 'chat' && !selectedProjectId.value
}

const allVisibleSessions = computed<Session[]>(() => {
  const result: Session[] = []
  projects.value.forEach((p) => {
    if (isProjectExpanded(p.id)) {
      result.push(...getProjectFilteredSessions(p.id))
    }
  })
  result.push(...unassignedSessions.value)
  return result
})

const selectSessionRange = (fromId: string, toId: string) => {
  const list = allVisibleSessions.value
  const fromIdx = list.findIndex((s) => s.id === fromId)
  const toIdx = list.findIndex((s) => s.id === toId)
  if (fromIdx === -1 || toIdx === -1) {
    toggleSessionSelection(toId)
    return
  }
  const [start, end] = fromIdx < toIdx ? [fromIdx, toIdx] : [toIdx, fromIdx]
  const next = new Set(selectedSessionIds.value)
  for (let i = start; i <= end; i++) {
    const s = list[i]
    if (s?.id) next.add(s.id)
  }
  selectedSessionIds.value = next
}

const toggleSessionSelection = (sessionId: string) => {
  selectedProjectId.value = null
  const next = new Set(selectedSessionIds.value)
  if (next.has(sessionId)) {
    next.delete(sessionId)
  } else {
    next.add(sessionId)
  }
  selectedSessionIds.value = next
  lastClickedSessionId.value = sessionId
  lastFocusedTarget.value = { type: 'session', id: sessionId }
}

// Floating Context Menu for Sessions
const contextMenuTarget = ref<Session | null>(null)
const contextMenuPos = ref<{ top: number; left: number } | null>(null)

const openSessionContextMenu = (session: Session, event: MouseEvent) => {
  if (contextMenuTarget.value?.id === session.id) {
    closeContextMenu()
    return
  }
  const btn = event.currentTarget as HTMLElement
  const rect = btn.getBoundingClientRect()
  contextMenuPos.value = {
    top: Math.min(rect.bottom + 4, window.innerHeight - 190),
    left: Math.min(rect.right - 140, window.innerWidth - 150)
  }
  contextMenuTarget.value = session
}

const closeContextMenu = () => {
  contextMenuTarget.value = null
  contextMenuPos.value = null
}

const handleMenuPin = (session: Session) => {
  emit('togglePinSession', session.id)
  closeContextMenu()
}

const handleMenuRename = (session: Session) => {
  closeContextMenu()
  startInlineEdit(session)
}

const handleMenuArchive = (session: Session) => {
  emit('archiveSession', session.id)
  closeContextMenu()
}

const handleMenuExport = (session: Session) => {
  closeContextMenu()
  handleExport(session)
}

const handleMenuDelete = (session: Session) => {
  emit('deleteSession', session.id)
  closeContextMenu()
}

const clearSelection = () => {
  selectedSessionIds.value = new Set()
  selectedProjectId.value = null
}

const deleteSelectedSessions = () => {
  if (selectedSessionIds.value.size === 0) return
  const ids = Array.from(selectedSessionIds.value)
  emit('deleteSessions', ids)
}

const handleSessionClick = (sessionId: string, event: MouseEvent) => {
  closeContextMenu()
  if (editingSessionId.value) return

  selectedProjectId.value = null

  const isMetaOrCtrl = event?.metaKey || event?.ctrlKey
  const isShift = event?.shiftKey

  lastFocusedTarget.value = { type: 'session', id: sessionId }

  if (isMetaOrCtrl) {
    toggleSessionSelection(sessionId)
    return
  }

  if (isShift && lastClickedSessionId.value) {
    selectSessionRange(lastClickedSessionId.value, sessionId)
    return
  }

  if (selectedSessionIds.value.size > 0 && !selectedSessionIds.value.has(sessionId)) {
    selectedSessionIds.value = new Set()
  }

  lastClickedSessionId.value = sessionId
  emit('selectSession', sessionId)
}

const triggerDeleteShortcut = () => {
  if (editingSessionId.value) return false

  if (selectedSessionIds.value.size > 1) {
    deleteSelectedSessions()
    return true
  }

  if (selectedSessionIds.value.size === 1) {
    const id = Array.from(selectedSessionIds.value)[0]
    if (id) {
      emit('deleteSession', id)
      return true
    }
  }

  if (selectedProjectId.value) {
    const proj = projects.value.find((p) => p.id === selectedProjectId.value)
    if (proj) {
      openDeleteProjectModal(proj)
      return true
    }
  }

  if (lastFocusedTarget.value?.type === 'project' && lastFocusedTarget.value.id) {
    const proj = projects.value.find((p) => p.id === lastFocusedTarget.value.id)
    if (proj) {
      openDeleteProjectModal(proj)
      return true
    }
  }

  if (lastFocusedTarget.value?.type === 'session' && lastFocusedTarget.value.id) {
    emit('deleteSession', lastFocusedTarget.value.id)
    return true
  }

  return false
}

// Expose method to focus search input from keyboard shortcuts (Cmd+K)
const focusSearch = () => {
  if (searchInputRef.value) {
    searchInputRef.value.focus()
    searchInputRef.value.select()
  }
}

defineExpose({
  focusSearch,
  triggerDeleteShortcut,
  clearSelection,
  selectedSessionIds,
  lastFocusedTarget,
  openDeleteProjectModal
})

const handleSelect = (sessionId: string) => {
  if (editingSessionId.value) return
  lastFocusedTarget.value = { type: 'session', id: sessionId }
  emit('selectSession', sessionId)
}

const startInlineEdit = (session: Session) => {
  editingSessionId.value = session.id
  editingTitle.value = session.title || t('sidebar.new_chat')
  nextTick(() => {
    if (editInputRef.value) {
      if (Array.isArray(editInputRef.value)) {
        ;(editInputRef.value[0] as any)?.focus()
        ;(editInputRef.value[0] as any)?.select()
      } else {
        editInputRef.value.focus()
        editInputRef.value.select()
      }
    }
  })
}

const saveInlineEdit = (session: Session) => {
  if (!editingSessionId.value) return
  const newTitle = editingTitle.value.trim()
  if (newTitle && newTitle !== session.title) {
    emit('renameSession', { id: session.id, title: newTitle })
  }
  editingSessionId.value = null
  editingTitle.value = ''
}

const cancelInlineEdit = () => {
  editingSessionId.value = null
  editingTitle.value = ''
}

const filteredSessions = computed<Session[]>(() => {
  if (!props.sessions) return []
  let list = props.sessions.filter((s) => !s.archived)
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter((s) => s.title?.toLowerCase().includes(q))
  }
  return [...list].sort((a, b) => {
    if (a.pinned && !b.pinned) return -1
    if (!a.pinned && b.pinned) return 1
    const timeA = new Date(a.updated_at || a.created_at || 0).getTime()
    const timeB = new Date(b.updated_at || b.created_at || 0).getTime()
    return (isNaN(timeB) ? 0 : timeB) - (isNaN(timeA) ? 0 : timeA)
  })
})

const archivedSessionsCount = computed(() => {
  return (props.sessions || []).filter((s) => Boolean(s.archived)).length
})

const archiveSelectedSessions = () => {
  if (selectedSessionIds.value.size === 0) return
  const ids = Array.from(selectedSessionIds.value)
  emit('archiveSessions', ids)
  clearSelection()
}

const handleProjectClick = (projectId: string) => {
  selectedProjectId.value = projectId
  selectProject(projectId)
  setProjectExpanded(projectId, true)
  lastFocusedTarget.value = { type: 'project', id: projectId }
  selectedSessionIds.value = new Set()
}

const handleNewProjectChat = (projectId: string) => {
  selectedProjectId.value = null
  selectProject(projectId)
  setProjectExpanded(projectId, true)
  lastFocusedTarget.value = { type: 'project', id: projectId }
  selectedSessionIds.value = new Set()
  emit('newProjectChat', projectId)
}

const getProjectSessions = (projectId: string): Session[] => {
  return (props.sessions || []).filter((s) => s.project_id === projectId && !s.archived)
}

const getProjectFilteredSessions = (projectId: string): Session[] => {
  return filteredSessions.value.filter((s) => s.project_id === projectId)
}

const unassignedSessions = computed<Session[]>(() => {
  const validProjectIds = new Set(projects.value.map((p) => p.id))
  return filteredSessions.value.filter((s) => !s.project_id || !validProjectIds.has(s.project_id))
})

const handleExport = async (session: Session) => {
  if (!session) return
  const md = formatSessionToMarkdown(session, props.activeModel)
  const safeTitle = (session.title || 'conversa')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '_')
    .slice(0, 40)
  await downloadMarkdownFile(md, `${safeTitle || 'conversa'}_${new Date().toISOString().slice(0, 10)}.md`)
}

const tabs = computed(() => {
  const list = [
    { id: 'chat', label: t('sidebar.chat'), icon: MessageSquare },
    { id: 'models', label: t('sidebar.models'), icon: Layers },
    { id: 'settings', label: t('sidebar.settings'), icon: Settings }
  ]
  if (props.enableMemory !== false) {
    list.splice(2, 0, { id: 'memory', label: t('sidebar.memory'), icon: Brain })
  }
  return list
})

const formatGb = (val: any) => (val ? Number(val).toFixed(1) : '0.0')

const aiRamPct = computed(() => {
  const tot = props.hardware.total_ram_gb || 16
  const ai = props.hardware.ai_ram_gb || props.hardware.used_vram_gb || 0
  return Math.min(100, Math.round((ai / tot) * 100))
})

const systemOtherPct = computed(() => {
  const tot = props.hardware.total_ram_gb || 16
  const other = props.hardware.system_other_ram_gb || 0
  return Math.min(100, Math.round((other / tot) * 100))
})

const vramPct = computed(() => {
  if (!props.hardware.total_vram_gb) return 0
  return Math.min(100, Math.round((props.hardware.used_vram_gb / props.hardware.total_vram_gb) * 100))
})

const ramPct = computed(() => {
  if (!props.hardware.total_ram_gb) return 0
  return Math.min(100, Math.round((props.hardware.used_ram_gb / props.hardware.total_ram_gb) * 100))
})

const defaultDeviceName = computed(() => {
  if (props.hardware?.device_name) return props.hardware.device_name
  if (supportsMlx.value || platformInfo.value?.os === 'macos') return 'Mac (Apple Silicon)'
  if (platformInfo.value?.os === 'windows') return 'Windows PC'
  return 'Linux PC'
})

const shortAccelerationBackend = computed(() => {
  const raw = props.hardware?.acceleration_backend
  if (raw) {
    if (/metal/i.test(raw)) return 'Metal 3'
    if (/cuda/i.test(raw)) return 'CUDA'
    if (/rocm/i.test(raw)) return 'ROCm'
    if (/directml/i.test(raw)) return 'DirectML'
    if (/oneapi/i.test(raw)) return 'oneAPI'
    if (raw === 'CPU') return 'CPU'
    return raw.length > 10 ? raw.split(/[\s/]+/)[0] || raw : raw
  }
  if (supportsMlx.value || platformInfo.value?.os === 'macos') return 'Metal 3'
  if (platformInfo.value?.os === 'windows') return 'DirectML'
  return 'CPU'
})

const accelerationBackendTooltip = computed(() => {
  if (props.hardware?.acceleration_backend) return props.hardware.acceleration_backend
  if (supportsMlx.value || platformInfo.value?.os === 'macos') return 'Apple Metal 3'
  if (platformInfo.value?.os === 'windows') return 'Windows (DirectML / CUDA)'
  return 'Linux (ROCm / CUDA / CPU)'
})
</script>
