<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-black/60 dark:bg-black/75 backdrop-blur-md select-none animate-fade-in">
    <div
      class="w-full max-w-2xl bg-white dark:bg-[#0f121d] border border-slate-200 dark:border-[#232a42] rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] transition-all"
    >
      <!-- Modal Header -->
      <div class="p-4 sm:p-5 bg-slate-50 dark:bg-[#131726] border-b border-slate-200 dark:border-[#20273d] flex items-center justify-between flex-shrink-0">
        <div class="flex items-center gap-3">
          <button
            v-if="isEditing"
            @click="cancelEdit"
            class="p-2 rounded-xl bg-slate-200/70 hover:bg-slate-300/70 dark:bg-[#1c2236] dark:hover:bg-[#252d48] text-slate-600 dark:text-slate-300 transition-colors cursor-pointer"
            title="Voltar para a lista"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <div v-else class="p-2.5 rounded-2xl bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20 shadow-sm">
            <Sparkles class="w-5 h-5" />
          </div>

          <div>
            <h3 class="text-sm sm:text-base font-bold text-slate-900 dark:text-slate-100">
              {{ isEditing ? (editForm.id ? `Editar Persona: ${editForm.name}` : 'Criar Nova Persona') : 'Perfis de Agente & Personas' }}
            </h3>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">
              {{ isEditing ? 'Personalize o nome, ícone, especialidade e system prompt desta persona' : 'Escolha ou personalize o comportamento e instruções do assistente' }}
            </p>
          </div>
        </div>

        <button
          @click="$emit('close')"
          class="p-2 rounded-xl text-slate-400 hover:text-slate-700 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#1f263d] transition-colors cursor-pointer"
          title="Fechar"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-4 sm:p-5 space-y-4">
        <!-- EDIT / CREATE FORM VIEW -->
        <div v-if="isEditing" class="space-y-4 animate-fade-in">
          <!-- Top Info Banner if default persona -->
          <div
            v-if="editForm.id && !editForm.id.startsWith('custom_')"
            class="p-3 rounded-2xl bg-indigo-50 dark:bg-indigo-950/30 border border-indigo-200 dark:border-indigo-500/30 flex items-center justify-between gap-2"
          >
            <div class="flex items-center gap-2 text-xs text-indigo-800 dark:text-indigo-300">
              <Sparkles class="w-4 h-4 text-indigo-500 flex-shrink-0" />
              <span>Você está personalizando uma <strong>persona padrão do sistema</strong>. As alterações ficam salvas no seu navegador.</span>
            </div>
            <button
              v-if="isDefaultPersonaModified(editForm.id)"
              type="button"
              @click="handleResetInForm"
              class="px-2.5 py-1 text-[11px] font-semibold text-amber-700 dark:text-amber-300 bg-amber-100 dark:bg-amber-500/20 hover:bg-amber-200 dark:hover:bg-amber-500/30 border border-amber-300 dark:border-amber-500/40 rounded-lg flex items-center gap-1 transition-all cursor-pointer flex-shrink-0"
              title="Restaurar prompt e configurações originais"
            >
              <RotateCcw class="w-3 h-3" />
              <span>Restaurar Original</span>
            </button>
          </div>

          <!-- Name & Tag Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label class="block text-[11px] font-semibold text-slate-700 dark:text-slate-300 mb-1">
                Nome da Persona <span class="text-rose-500">*</span>
              </label>
              <input
                v-model="editForm.name"
                type="text"
                placeholder="Ex: Auditor de Código"
                class="w-full px-3 py-2 rounded-xl bg-slate-50 dark:bg-[#0b0e18] border border-slate-300 dark:border-[#232a42] text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 outline-none focus:border-indigo-500 focus:bg-white dark:focus:bg-[#0b0e18] transition-all"
              />
            </div>
            <div>
              <label class="block text-[11px] font-semibold text-slate-700 dark:text-slate-300 mb-1">
                Especialidade / Categoria
              </label>
              <input
                v-model="editForm.tag"
                type="text"
                placeholder="Ex: AppSec & Testes"
                class="w-full px-3 py-2 rounded-xl bg-slate-50 dark:bg-[#0b0e18] border border-slate-300 dark:border-[#232a42] text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 outline-none focus:border-indigo-500 focus:bg-white dark:focus:bg-[#0b0e18] transition-all"
              />
            </div>
          </div>

          <!-- Icon & Color Customization -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 p-3.5 rounded-2xl bg-slate-50 dark:bg-[#131726] border border-slate-200 dark:border-[#20273d]">
            <!-- Icon Selector -->
            <div>
              <label class="block text-[10.5px] font-semibold text-slate-600 dark:text-slate-300 mb-1.5 uppercase tracking-wider">
                Ícone
              </label>
              <div class="flex items-center gap-1.5 flex-wrap">
                <button
                  v-for="icon in AVAILABLE_ICONS"
                  :key="icon"
                  type="button"
                  @click="editForm.iconName = icon"
                  :class="[
                    'w-8 h-8 rounded-xl flex items-center justify-center transition-all cursor-pointer border',
                    editForm.iconName === icon
                      ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm ring-2 ring-indigo-500/30'
                      : 'bg-white dark:bg-[#1a2034] text-slate-600 dark:text-slate-300 border-slate-200 dark:border-[#28324e] hover:border-indigo-400'
                  ]"
                >
                  <component :is="resolveIcon(icon)" class="w-4 h-4" />
                </button>
              </div>
            </div>

            <!-- Color Palette Selector -->
            <div>
              <label class="block text-[10.5px] font-semibold text-slate-600 dark:text-slate-300 mb-1.5 uppercase tracking-wider">
                Cor de Destaque
              </label>
              <div class="flex items-center gap-2 flex-wrap">
                <button
                  v-for="c in AVAILABLE_COLORS"
                  :key="c.id"
                  type="button"
                  @click="editForm.color = c.id"
                  :class="[
                    'w-8 h-8 rounded-xl flex items-center justify-center transition-all cursor-pointer border',
                    c.bgClass,
                    editForm.color === c.id ? 'ring-2 ring-indigo-500 scale-105 border-indigo-500' : 'border-transparent opacity-85 hover:opacity-100'
                  ]"
                  :title="c.label"
                >
                  <Check v-if="editForm.color === c.id" class="w-3.5 h-3.5 text-white" />
                </button>
              </div>
            </div>
          </div>

          <!-- Short Description -->
          <div>
            <label class="block text-[11px] font-semibold text-slate-700 dark:text-slate-300 mb-1">
              Descrição Resumida (exibida no card)
            </label>
            <input
              v-model="editForm.description"
              type="text"
              placeholder="Ex: Especialista focado em segurança, análise de vulnerabilidades e testes."
              class="w-full px-3 py-2 rounded-xl bg-slate-50 dark:bg-[#0b0e18] border border-slate-300 dark:border-[#232a42] text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 outline-none focus:border-indigo-500 focus:bg-white dark:focus:bg-[#0b0e18] transition-all"
            />
          </div>

          <!-- System Prompt / Instructions -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 flex items-center gap-1.5">
                <span>Instruções do Sistema (System Prompt)</span>
                <span
                  v-if="!editForm.system_prompt || !editForm.system_prompt.trim()"
                  class="text-[10px] font-semibold text-emerald-700 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-500/15 px-2 py-0.5 rounded-full border border-emerald-200 dark:border-emerald-500/30 flex items-center gap-1"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                  <span>0 tokens (Vazio)</span>
                </span>
                <span
                  v-else
                  class="text-[10px] font-mono font-semibold text-indigo-700 dark:text-indigo-300 bg-indigo-50 dark:bg-indigo-500/15 px-2 py-0.5 rounded-full border border-indigo-200 dark:border-indigo-500/30 flex items-center gap-1"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-indigo-500"></span>
                  <span>~{{ Math.ceil(editForm.system_prompt.trim().length / 4) }} tokens</span>
                </span>
              </label>
              <span class="text-[10px] font-mono text-slate-400">
                {{ (editForm.system_prompt || '').length }} caracteres
              </span>
            </div>
            <textarea
              v-model="editForm.system_prompt"
              rows="5"
              placeholder="Opcional. Deixe em branco para economizar tokens e utilizar o comportamento original do modelo..."
              class="w-full p-3 rounded-xl bg-slate-50 dark:bg-[#0b0e18] border border-slate-300 dark:border-[#232a42] text-xs text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 outline-none focus:border-indigo-500 focus:bg-white dark:focus:bg-[#0b0e18] transition-all resize-y font-sans leading-relaxed"
            ></textarea>
            <div class="flex items-center justify-between mt-1 text-[10.5px]">
              <p class="text-slate-500 dark:text-slate-400">
                Dica: Deixe este campo vazio para máxima economia de tokens e prefill ultra-rápido.
              </p>
              <button
                v-if="editForm.system_prompt && editForm.system_prompt.trim()"
                type="button"
                @click="editForm.system_prompt = ''"
                class="text-indigo-600 dark:text-indigo-400 hover:underline cursor-pointer font-semibold flex-shrink-0"
              >
                Limpar para 0 tokens
              </button>
            </div>
          </div>

          <!-- Form Buttons -->
          <div class="pt-2 flex items-center justify-end gap-2.5">
            <button
              type="button"
              @click="cancelEdit"
              class="px-4 py-2 rounded-xl text-xs font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#1a2034] transition-colors cursor-pointer"
            >
              Cancelar
            </button>
            <button
              type="button"
              @click="saveEdit"
              :disabled="!editForm.name.trim()"
              class="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer active:scale-95 flex items-center gap-1.5"
            >
              <Check class="w-3.5 h-3.5" />
              <span>{{ editForm.id ? 'Salvar Alterações' : 'Criar Persona' }}</span>
            </button>
          </div>
        </div>

        <!-- PERSONAS LIST VIEW -->
        <div v-else class="space-y-3">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div
              v-for="persona in allPersonas"
              :key="persona.id"
              @click="selectPersona(persona)"
              :class="[
                'p-4 rounded-2xl border transition-all cursor-pointer flex flex-col justify-between relative group select-none shadow-sm',
                activePersonaId === persona.id
                  ? 'bg-indigo-50/70 dark:bg-gradient-to-br dark:from-[#1c223a] dark:to-[#141829] border-indigo-500 ring-2 ring-indigo-500/20 dark:ring-indigo-500/30'
                  : 'bg-white dark:bg-[#121626] border-slate-200 dark:border-[#1f263c] hover:border-indigo-300 dark:hover:border-slate-500 hover:bg-slate-50/50 dark:hover:bg-[#161b30]'
              ]"
            >
              <!-- Active Badge -->
              <div
                v-if="activePersonaId === persona.id"
                class="absolute top-3 right-3 flex items-center gap-1 text-[10px] font-bold text-indigo-700 dark:text-indigo-300 bg-indigo-100/90 dark:bg-indigo-500/20 border border-indigo-300 dark:border-indigo-500/40 px-2 py-0.5 rounded-full shadow-xs"
              >
                <Check class="w-3 h-3 text-indigo-600 dark:text-indigo-400" />
                <span>Ativo</span>
              </div>

              <!-- Top: Icon, Name, Tag & Modified Indicator -->
              <div>
                <div class="flex items-start gap-3" :class="activePersonaId === persona.id ? 'pr-14' : ''">
                  <!-- Icon Container -->
                  <div
                    class="w-9 h-9 rounded-xl flex items-center justify-center flex-shrink-0 shadow-xs border"
                    :class="getIconBoxClasses(persona.color)"
                  >
                    <component :is="resolveIcon(persona.iconName)" class="w-4.5 h-4.5" />
                  </div>

                  <!-- Name and Category -->
                  <div class="min-w-0 flex-1">
                    <h4 class="text-xs sm:text-sm font-bold text-slate-900 dark:text-slate-100 flex items-center gap-1.5 truncate">
                      <span class="truncate">{{ persona.name }}</span>
                    </h4>

                    <div class="flex items-center gap-1.5 mt-1 flex-wrap">
                      <!-- Category / Specialty Tag (Clean light & dark mode styling) -->
                      <span
                        class="text-[9.5px] font-semibold px-2 py-0.5 rounded-full border shadow-2xs"
                        :class="getTagClasses(persona.color)"
                      >
                        {{ persona.tag }}
                      </span>

                      <!-- Modified Badge -->
                      <span
                        v-if="persona.is_modified"
                        class="text-[9px] font-medium px-1.5 py-0.2 rounded-md bg-amber-50 dark:bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-500/30"
                        title="Esta persona padrão foi editada por você"
                      >
                        Personalizado
                      </span>
                      <span
                        v-else-if="persona.is_custom"
                        class="text-[9px] font-medium px-1.5 py-0.2 rounded-md bg-indigo-50 dark:bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-500/30"
                      >
                        Própria
                      </span>

                      <!-- Token Cost Indicator Badge -->
                      <span
                        v-if="!persona.system_prompt || !persona.system_prompt.trim()"
                        class="text-[9px] font-mono font-semibold px-1.5 py-0.2 rounded-md bg-emerald-50 dark:bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-500/30"
                        title="0 tokens adicionais de sistema"
                      >
                        0 tok
                      </span>
                      <span
                        v-else
                        class="text-[9px] font-mono font-medium px-1.5 py-0.2 rounded-md bg-slate-100 dark:bg-white/5 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-white/10"
                        :title="`Aproximadamente ${Math.ceil(persona.system_prompt.trim().length / 4)} tokens extras de sistema`"
                      >
                        ~{{ Math.ceil(persona.system_prompt.trim().length / 4) }} tok
                      </span>
                    </div>
                  </div>
                </div>

                <!-- Description -->
                <p class="text-[11.5px] text-slate-600 dark:text-slate-400 mt-2.5 line-clamp-2 leading-relaxed">
                  {{ persona.description || persona.system_prompt || 'Prompt padrão do sistema.' }}
                </p>
              </div>

              <!-- Action Toolbar (Edit, Duplicate, Reset, Delete) -->
              <div
                class="mt-3 pt-2.5 border-t border-slate-100 dark:border-white/5 flex items-center justify-between gap-1 text-[11px]"
                @click.stop
              >
                <div class="flex items-center gap-1">
                  <!-- Edit Action Button -->
                  <button
                    type="button"
                    @click="openEditForm(persona)"
                    class="px-2 py-1 rounded-lg text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-500/10 font-medium flex items-center gap-1 transition-colors cursor-pointer"
                    title="Editar instruções, nome e parâmetros desta persona"
                  >
                    <Edit3 class="w-3.5 h-3.5" />
                    <span>Editar</span>
                  </button>

                  <!-- Duplicate Action Button -->
                  <button
                    type="button"
                    @click="handleDuplicate(persona)"
                    class="px-2 py-1 rounded-lg text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-white/5 font-medium flex items-center gap-1 transition-colors cursor-pointer"
                    title="Criar uma cópia personalizada desta persona"
                  >
                    <Copy class="w-3.5 h-3.5" />
                    <span>Duplicar</span>
                  </button>
                </div>

                <div class="flex items-center gap-1">
                  <!-- Reset Default Persona (if modified) -->
                  <button
                    v-if="persona.is_modified"
                    type="button"
                    @click="handleResetDefault(persona.id)"
                    class="px-2 py-1 rounded-lg text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10 font-medium flex items-center gap-1 transition-colors cursor-pointer"
                    title="Restaurar valores e instruções originais de fábrica"
                  >
                    <RotateCcw class="w-3.5 h-3.5" />
                    <span>Restaurar</span>
                  </button>

                  <!-- Delete Button for Custom Personas -->
                  <button
                    v-if="persona.is_custom"
                    type="button"
                    @click="handleDelete(persona.id)"
                    class="px-2 py-1 rounded-lg text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-500/10 font-medium flex items-center gap-1 transition-colors cursor-pointer"
                    title="Excluir esta persona personalizada"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                    <span>Excluir</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-4 bg-slate-50 dark:bg-[#131726] border-t border-slate-200 dark:border-[#20273d] flex items-center justify-between flex-shrink-0">
        <button
          v-if="!isEditing"
          type="button"
          @click="openCreateForm"
          class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-slate-200/70 hover:bg-slate-300/80 dark:bg-[#1c2236] dark:hover:bg-[#252d48] border border-slate-300 dark:border-[#2b3552] text-xs font-semibold text-slate-700 dark:text-slate-200 transition-all cursor-pointer active:scale-95 shadow-2xs"
        >
          <Plus class="w-3.5 h-3.5 text-indigo-500 dark:text-indigo-400" />
          <span>Criar Nova Persona</span>
        </button>
        <span v-else></span>

        <button
          v-if="!isEditing"
          type="button"
          @click="$emit('close')"
          class="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/20 transition-all cursor-pointer active:scale-95"
        >
          Concluir
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import {
  Sparkles,
  Code2,
  Brain,
  PenTool,
  GraduationCap,
  CheckCircle2,
  Wrench,
  Shield,
  Compass,
  MessageSquare,
  BookOpen,
  HeartHandshake,
  Plus,
  Trash2,
  Edit3,
  Copy,
  RotateCcw,
  Check,
  X,
  ChevronLeft
} from 'lucide-vue-next'
import {
  allPersonas,
  activePersonaId,
  personaOverrides,
  setActivePersona,
  updatePersona,
  deleteCustomPersona,
  resetDefaultPersona,
  duplicatePersona
} from '~/utils/personas'
import type { GenerationParams, Persona } from '~/types'

const props = defineProps<{
  params?: GenerationParams | null
}>()

const emit = defineEmits<{
  close: []
  personaChanged: [persona: Persona]
}>()

const AVAILABLE_ICONS = [
  'Sparkles',
  'Code2',
  'Brain',
  'PenTool',
  'GraduationCap',
  'CheckCircle2',
  'Wrench',
  'Shield',
  'Compass',
  'BookOpen',
  'MessageSquare',
  'HeartHandshake'
]

const AVAILABLE_COLORS = [
  { id: 'indigo', label: 'Índigo', bgClass: 'bg-indigo-600' },
  { id: 'purple', label: 'Roxo', bgClass: 'bg-purple-600' },
  { id: 'teal', label: 'Teal', bgClass: 'bg-teal-600' },
  { id: 'emerald', label: 'Esmeralda', bgClass: 'bg-emerald-600' },
  { id: 'amber', label: 'Âmbar', bgClass: 'bg-amber-600' },
  { id: 'sky', label: 'Azul Céu', bgClass: 'bg-sky-600' },
  { id: 'rose', label: 'Rosa', bgClass: 'bg-rose-600' }
]

const resolveIcon = (name: string) => {
  switch (name) {
    case 'Code2': return Code2
    case 'Brain': return Brain
    case 'PenTool': return PenTool
    case 'GraduationCap': return GraduationCap
    case 'CheckCircle2': return CheckCircle2
    case 'Wrench': return Wrench
    case 'Shield': return Shield
    case 'Compass': return Compass
    case 'BookOpen': return BookOpen
    case 'MessageSquare': return MessageSquare
    case 'HeartHandshake': return HeartHandshake
    default: return Sparkles
  }
}

const getIconBoxClasses = (color: string) => {
  switch (color) {
    case 'emerald':
      return 'bg-emerald-100/90 dark:bg-emerald-600/25 text-emerald-700 dark:text-emerald-300 border-emerald-300/80 dark:border-emerald-500/30'
    case 'purple':
      return 'bg-purple-100/90 dark:bg-purple-600/25 text-purple-700 dark:text-purple-300 border-purple-300/80 dark:border-purple-500/30'
    case 'teal':
      return 'bg-teal-100/90 dark:bg-teal-600/25 text-teal-700 dark:text-teal-300 border-teal-300/80 dark:border-teal-500/30'
    case 'amber':
      return 'bg-amber-100/90 dark:bg-amber-600/25 text-amber-800 dark:text-amber-300 border-amber-300/80 dark:border-amber-500/30'
    case 'sky':
      return 'bg-sky-100/90 dark:bg-sky-600/25 text-sky-700 dark:text-sky-300 border-sky-300/80 dark:border-sky-500/30'
    case 'rose':
      return 'bg-rose-100/90 dark:bg-rose-600/25 text-rose-700 dark:text-rose-300 border-rose-300/80 dark:border-rose-500/30'
    default:
      return 'bg-indigo-100/90 dark:bg-indigo-600/25 text-indigo-700 dark:text-indigo-300 border-indigo-300/80 dark:border-indigo-500/30'
  }
}

const getTagClasses = (color: string) => {
  switch (color) {
    case 'emerald':
      return 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-500/15 dark:text-emerald-300 dark:border-emerald-500/25'
    case 'purple':
      return 'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-500/15 dark:text-purple-300 dark:border-purple-500/25'
    case 'teal':
      return 'bg-teal-50 text-teal-700 border-teal-200 dark:bg-teal-500/15 dark:text-teal-300 dark:border-teal-500/25'
    case 'amber':
      return 'bg-amber-50 text-amber-800 border-amber-200 dark:bg-amber-500/15 dark:text-amber-300 dark:border-amber-500/25'
    case 'sky':
      return 'bg-sky-50 text-sky-700 border-sky-200 dark:bg-sky-500/15 dark:text-sky-300 dark:border-sky-500/25'
    case 'rose':
      return 'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-500/15 dark:text-rose-300 dark:border-rose-500/25'
    default:
      return 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-500/15 dark:text-indigo-300 dark:border-indigo-500/25'
  }
}

// Editing state
const isEditing = ref(false)
const editForm = reactive({
  id: '',
  name: '',
  tag: '',
  iconName: 'Sparkles',
  color: 'indigo',
  description: '',
  system_prompt: ''
})

const isDefaultPersonaModified = (id: string) => {
  return Boolean(personaOverrides.value && personaOverrides.value[id])
}

const selectPersona = (persona: Persona) => {
  setActivePersona(persona.id)
  if (props.params) {
    props.params.system_prompt = persona.system_prompt || ''
  }
  emit('personaChanged', persona)
}

const openCreateForm = () => {
  editForm.id = ''
  editForm.name = ''
  editForm.tag = ''
  editForm.iconName = 'Sparkles'
  editForm.color = 'indigo'
  editForm.description = ''
  editForm.system_prompt = ''
  isEditing.value = true
}

const openEditForm = (persona: Persona) => {
  editForm.id = persona.id
  editForm.name = persona.name
  editForm.tag = persona.tag || ''
  editForm.iconName = persona.iconName || 'Sparkles'
  editForm.color = persona.color || 'indigo'
  editForm.description = persona.description || ''
  editForm.system_prompt = persona.system_prompt || ''
  isEditing.value = true
}

const cancelEdit = () => {
  isEditing.value = false
}

const saveEdit = () => {
  if (!editForm.name.trim()) return

  const isNew = !editForm.id
  const payload: Persona = {
    id: isNew ? `custom_${Date.now()}` : editForm.id,
    name: editForm.name.trim(),
    tag: editForm.tag.trim() || (editForm.system_prompt.trim() ? 'Geral' : '0 tokens'),
    iconName: editForm.iconName || 'Sparkles',
    color: editForm.color || 'indigo',
    description: editForm.description.trim() || (editForm.system_prompt.trim() ? (editForm.system_prompt.trim().slice(0, 80) + '...') : 'Sem instruções de sistema adicionais (0 tokens extras).'),
    system_prompt: editForm.system_prompt.trim()
  }

  const updated = updatePersona(payload)

  // If we just edited or created the active persona, sync it with params immediately
  if (activePersonaId.value === payload.id || isNew) {
    selectPersona(updated || payload)
  }

  isEditing.value = false
}

const handleResetInForm = () => {
  if (!editForm.id) return
  const original = resetDefaultPersona(editForm.id)
  if (original) {
    editForm.name = original.name
    editForm.tag = original.tag
    editForm.iconName = original.iconName
    editForm.color = original.color
    editForm.description = original.description
    editForm.system_prompt = original.system_prompt

    if (activePersonaId.value === editForm.id && props.params) {
      props.params.system_prompt = original.system_prompt || ''
    }
  }
}

const handleResetDefault = (id: string) => {
  const original = resetDefaultPersona(id)
  if (original && activePersonaId.value === id && props.params) {
    props.params.system_prompt = original.system_prompt || ''
    emit('personaChanged', original)
  }
}

const handleDuplicate = (persona: Persona) => {
  const dup = duplicatePersona(persona)
  if (dup) {
    selectPersona(dup)
  }
}

const handleDelete = (id: string) => {
  deleteCustomPersona(id)
}
</script>
