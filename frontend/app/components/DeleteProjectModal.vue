<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[9999] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md select-none animate-in fade-in duration-150"
      @click.self="$emit('close')"
    >
      <div
        class="w-full max-w-md bg-[#0f121f] border border-[#232a42] rounded-3xl shadow-2xl shadow-black/80 overflow-hidden flex flex-col animate-in zoom-in-95 duration-150"
      >
        <!-- Modal Header -->
        <div class="p-5 border-b border-[#1b2135] bg-[#121627] flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-2xl bg-rose-500/15 border border-rose-500/30 flex items-center justify-center text-rose-400 shadow-inner">
              <Trash2 class="w-5 h-5" />
            </div>
            <div>
              <h3 class="text-sm font-bold text-slate-100 tracking-tight">Excluir Projeto</h3>
              <p class="text-[11px] text-slate-400 mt-0.5">Confirmação de exclusão</p>
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
        <div class="p-5 space-y-4 text-xs text-slate-300">
          <!-- Project Info Box -->
          <div class="p-3.5 rounded-2xl bg-[#14192b] border border-[#202844] flex items-start gap-2.5">
            <Folder class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <div class="min-w-0 flex-1">
              <div class="font-semibold text-slate-200 truncate text-[13px]">
                {{ project?.name || 'Projeto' }}
              </div>
              <div class="text-[11px] text-slate-400 mt-0.5">
                {{ sessionCount }} {{ sessionCount === 1 ? 'conversa vinculada' : 'conversas vinculadas' }}
              </div>
            </div>
          </div>

          <!-- Notice -->
          <div class="p-3 rounded-2xl bg-[#131728] border border-[#1e253e] text-[11.5px] text-slate-400 leading-relaxed">
            Tem certeza de que deseja excluir este projeto? Suas conversas <strong class="text-slate-200">não serão apagadas</strong> e continuarão disponíveis normalmente na seção de <strong class="text-slate-200">Recentes</strong>.
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="p-4 border-t border-[#1b2135] bg-[#121627] flex items-center justify-end gap-2.5">
          <button
            type="button"
            @click="$emit('close')"
            class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-300 hover:text-white bg-[#161a2c] hover:bg-[#1e233b] border border-[#242b45] transition-all cursor-pointer"
          >
            Cancelar
          </button>

          <button
            type="button"
            @click="project && $emit('confirm', project)"
            class="px-4 py-2 rounded-xl bg-gradient-to-r from-rose-600 to-rose-500 hover:from-rose-500 hover:to-rose-400 text-white text-xs font-semibold shadow-lg shadow-rose-600/25 transition-all active:scale-[0.98] flex items-center gap-1.5 cursor-pointer"
          >
            <Trash2 class="w-3.5 h-3.5" />
            <span>Excluir Projeto</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { Folder, Trash2, X } from 'lucide-vue-next'
import type { Project } from '../composables/useProjects'

defineProps<{
  isOpen: boolean
  project?: Project | null
  sessionCount?: number
}>()

defineEmits<{
  (e: 'close'): void
  (e: 'confirm', project: Project): void
}>()
</script>
