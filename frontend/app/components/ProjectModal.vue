<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[9999] flex items-center justify-center p-4 sm:p-6 bg-black/80 backdrop-blur-md select-none animate-in fade-in duration-150"
      @click.self="close"
    >
      <div
        class="bg-[#0e111d] border border-[#242b45] rounded-3xl w-full max-w-md shadow-2xl shadow-black/95 flex flex-col overflow-hidden text-slate-100 animate-in zoom-in-95 duration-150"
      >
        <!-- Modal Header -->
        <div class="px-6 py-5 bg-[#121627] border-b border-[#1e253e] flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-2xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shadow-sm">
              <FolderPlus v-if="mode === 'create'" class="w-5 h-5" />
              <FolderCog v-else class="w-5 h-5" />
            </div>
            <div>
              <h3 class="text-base font-bold text-slate-100 leading-tight">
                {{ mode === 'create' ? 'Novo Projeto' : 'Editar Projeto' }}
              </h3>
              <p class="text-xs text-slate-400 mt-0.5">
                {{ mode === 'create' ? 'Crie um projeto para organizar e categorizar seus chats' : 'Altere o nome do projeto' }}
              </p>
            </div>
          </div>

          <button
            @click="close"
            class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Modal Body -->
        <div class="p-6 space-y-4">
          <!-- Project Name -->
          <div class="space-y-2">
            <label class="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <span>Nome do Projeto</span>
              <span class="text-rose-400">*</span>
            </label>
            <input
              v-model="projectName"
              type="text"
              placeholder="Ex: Trabalho, Estudos, Finanças, Ideias"
              class="w-full px-4 py-2.5 rounded-2xl bg-[#141829] border border-[#222a44] focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/30 text-sm text-slate-100 placeholder-slate-500 outline-none transition-all shadow-inner"
              autofocus
              @keydown.enter="handleSave"
            />
          </div>

          <div v-if="errorMessage" class="p-3 rounded-2xl bg-rose-500/10 border border-rose-500/25 text-rose-300 text-xs flex items-center gap-2.5">
            <AlertTriangle class="w-4 h-4 flex-shrink-0 text-rose-400" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-6 py-4 bg-[#121627] border-t border-[#1e253e] flex items-center justify-between">
          <div v-if="mode === 'edit' && targetProject">
            <div v-if="isConfirmingDelete" class="flex items-center gap-2">
              <span class="text-xs text-rose-300 font-medium">Excluir projeto?</span>
              <button
                type="button"
                @click="confirmDelete"
                class="px-3 py-1.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold shadow-md shadow-rose-600/30 transition-all cursor-pointer active:scale-95 flex items-center gap-1"
              >
                <Trash2 class="w-3.5 h-3.5" />
                <span>Confirmar</span>
              </button>
              <button
                type="button"
                @click="isConfirmingDelete = false"
                class="px-2.5 py-1.5 rounded-xl bg-[#161a2c] hover:bg-[#1e233b] text-slate-400 hover:text-slate-200 text-xs font-semibold transition-all cursor-pointer"
              >
                Cancelar
              </button>
            </div>
            <button
              v-else
              type="button"
              @click="isConfirmingDelete = true"
              class="px-3.5 py-2 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 text-rose-300 border border-rose-500/30 text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer active:scale-95"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>Excluir</span>
            </button>
          </div>
          <div v-else></div>

          <div class="flex items-center gap-2.5">
            <button
              type="button"
              @click="close"
              class="px-4 py-2 rounded-xl bg-[#161a2c] hover:bg-[#1e233b] text-slate-300 text-xs font-semibold border border-[#242b45] transition-all cursor-pointer"
            >
              Cancelar
            </button>
            <button
              type="button"
              @click="handleSave"
              class="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer active:scale-95 flex items-center gap-2"
            >
              <Check class="w-4 h-4" />
              <span>{{ mode === 'create' ? 'Criar Projeto' : 'Salvar' }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  FolderPlus,
  FolderCog,
  Trash2,
  X,
  Check,
  AlertTriangle,
} from 'lucide-vue-next'
import { useProjects, type Project } from '../composables/useProjects'

const props = defineProps<{
  isOpen: boolean
  mode: 'create' | 'edit'
  targetProject?: Project | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved', project: Project): void
  (e: 'deleted', projectId: string): void
}>()

const { createProject, updateProject, deleteProject } = useProjects()

const projectName = ref('')
const errorMessage = ref('')
const isConfirmingDelete = ref(false)

watch(
  () => props.isOpen,
  (open) => {
    if (open) {
      errorMessage.value = ''
      isConfirmingDelete.value = false
      if (props.mode === 'edit' && props.targetProject) {
        projectName.value = props.targetProject.name
      } else {
        projectName.value = ''
      }
    }
  },
  { immediate: true }
)

const handleSave = () => {
  const name = projectName.value.trim()
  if (!name) {
    errorMessage.value = 'Por favor, informe um nome para o projeto.'
    return
  }

  if (props.mode === 'create') {
    const created = createProject(name)
    emit('saved', created)
  } else if (props.mode === 'edit' && props.targetProject) {
    updateProject(props.targetProject.id, {
      name,
    })
    emit('saved', {
      ...props.targetProject,
      name,
    })
  }

  close()
}

const confirmDelete = () => {
  if (props.targetProject) {
    const id = props.targetProject.id
    deleteProject(id)
    emit('deleted', id)
    isConfirmingDelete.value = false
    close()
  }
}

const close = () => {
  isConfirmingDelete.value = false
  emit('close')
}
</script>
