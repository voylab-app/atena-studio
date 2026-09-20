<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-fade-in"
    @click.self="closeModal"
  >
    <div
      class="w-full max-w-2xl bg-[#0f121d] border border-slate-700/60 dark:border-[#22283e] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[88vh] select-none"
    >
      <!-- Modal Header -->
      <div class="p-4 bg-[#141828] border-b border-[#1f253d] flex items-center justify-between gap-3">
        <div class="flex items-center gap-2.5 min-w-0">
          <div class="w-8 h-8 rounded-xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 flex-shrink-0">
            <Wrench class="w-4 h-4" />
          </div>
          <div class="min-w-0">
            <h3 class="font-bold text-sm text-slate-100 truncate">
              {{ $t('modals.mcp_tool.edit_title') }}
            </h3>
            <p class="text-[11px] font-mono text-indigo-300/80 truncate flex items-center gap-1.5 mt-0.5">
              <span>{{ server?.name || $t('settings.mcp_default_server_name') }}</span>
              <span class="text-slate-500">•</span>
              <span class="text-indigo-400 flex items-center gap-1">
                <Terminal class="w-2.5 h-2.5 flex-shrink-0" />
                <span>{{ tool?.name }}</span>
              </span>
            </p>
          </div>
        </div>

        <button
          @click="closeModal"
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
          :title="$t('common.close')"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-4 overflow-y-auto space-y-4 flex-1 select-text">
        <!-- Tool Status Toggle -->
        <div class="p-3 rounded-2xl bg-[#121626] border border-[#1f253d] flex items-center justify-between gap-3">
          <div class="flex items-center gap-2.5 min-w-0">
            <span
              class="w-2.5 h-2.5 rounded-full flex-shrink-0"
              :class="formEnabled ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50' : 'bg-slate-500'"
            ></span>
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold text-slate-100">{{ $t('modals.mcp_tool.status_label') }}</span>
                <span
                  :class="[
                    'text-[10px] font-semibold px-2 py-0.2 rounded-full border',
                    formEnabled
                      ? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
                      : 'bg-slate-800 text-slate-400 border-slate-700'
                  ]"
                >
                  {{ formEnabled ? $t('modals.mcp_tool.status_active') : $t('modals.mcp_tool.status_inactive') }}
                </span>
              </div>
              <p class="text-[10.5px] text-slate-400 mt-0.5">
                {{ formEnabled ? $t('modals.mcp_tool.tool_active_desc') : $t('modals.mcp_tool.tool_inactive_desc') }}
              </p>
            </div>
          </div>

          <!-- Switch button -->
          <button
            type="button"
            role="switch"
            :aria-checked="formEnabled"
            @click="formEnabled = !formEnabled"
            :class="[
              'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none',
              formEnabled ? 'bg-indigo-600 shadow-sm shadow-indigo-600/30' : 'bg-[#1e2436]'
            ]"
            :title="formEnabled ? $t('modals.mcp_tool.disable_tool') : $t('modals.mcp_tool.enable_tool')"
          >
            <span
              :class="[
                'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                formEnabled ? 'translate-x-5' : 'translate-x-0'
              ]"
            />
          </button>
        </div>

        <!-- Tool Main Label -->
        <div class="space-y-1.5">
          <label class="block text-xs font-semibold text-slate-200">
            {{ $t('modals.mcp_tool.tool_label_input') }}
          </label>
          <input
            v-model="formLabel"
            type="text"
            :placeholder="$t('modals.mcp_tool.label_placeholder')"
            class="w-full px-3 py-2 text-xs rounded-xl bg-[#0b0e18] border border-[#22283e] focus:border-indigo-500 text-slate-100 placeholder-slate-500 outline-none transition-all shadow-inner"
          />
          <p class="text-[10.5px] text-slate-400">
            {{ $t('modals.mcp_tool.custom_label_desc') }}
          </p>
        </div>

        <!-- Tool Description preview -->
        <div v-if="tool?.description" class="p-2.5 rounded-xl bg-indigo-950/20 border border-indigo-500/15 text-[11px] text-slate-300 leading-relaxed">
          <span class="text-[10px] uppercase font-bold tracking-wider text-indigo-400 block mb-0.5">
            {{ $t('modals.mcp_tool.original_tech_desc') }}
          </span>
          {{ tool.description }}
        </div>

        <!-- Tool Field Translations Section -->
        <div class="space-y-2.5 pt-2 border-t border-slate-700/40 dark:border-white/[0.06]">
          <div class="flex items-center justify-between gap-2">
            <div>
              <h4 class="text-xs font-bold text-slate-200">
                {{ $t('modals.mcp_tool.params_fields_title') }}
              </h4>
              <p class="text-[10.5px] text-slate-400">
                {{ $t('modals.mcp_tool.params_fields_desc') }}
              </p>
            </div>
            <span v-if="propertiesList.length > 0" class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
              {{ propertiesList.length }} {{ propertiesList.length > 1 ? $t('modals.mcp_tool.fields') : $t('modals.mcp_tool.field') }}
            </span>
          </div>

          <!-- Empty Properties Notice -->
          <div
            v-if="propertiesList.length === 0"
            class="p-4 rounded-xl bg-[#121524] border border-[#1f253d] text-center text-xs text-slate-400 italic"
          >
            {{ $t('modals.mcp_tool.no_params_required') }}
          </div>

          <!-- Properties List -->
          <div v-else class="space-y-2">
            <div
              v-for="prop in propertiesList"
              :key="prop.key"
              class="p-3 rounded-xl bg-[#121524] border border-[#1f253d] space-y-2 transition-all hover:border-[#2b3352]"
            >
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <span class="font-mono text-xs font-bold text-indigo-300 truncate">
                    {{ prop.key }}
                  </span>
                  <span class="text-[9.5px] font-mono px-1.5 py-0.2 rounded bg-white/5 text-slate-400 border border-white/10">
                    {{ prop.type }}
                  </span>
                  <span
                    v-if="prop.isRequired"
                    class="text-[9.5px] font-semibold px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-300 border border-amber-500/25"
                  >
                    {{ $t('modals.mcp_tool.required') }}
                  </span>
                </div>
              </div>

              <p v-if="prop.description" class="text-[10.5px] text-slate-400 leading-snug">
                {{ prop.description }}
              </p>

              <div>
                <label class="block text-[10px] font-medium text-slate-300 mb-1">
                  {{ $t('modals.mcp_tool.friendly_label') }}
                </label>
                <input
                  v-model="formFieldLabels[prop.key]"
                  type="text"
                  :placeholder="`Ex: ${prop.fallback}`"
                  class="w-full px-2.5 py-1.5 text-xs rounded-lg bg-[#0b0e18] border border-[#22283e] focus:border-indigo-500 text-slate-100 placeholder-slate-600 outline-none transition-all shadow-inner"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="p-3.5 bg-[#141828] border-t border-[#1f253d] flex items-center justify-between gap-2 flex-wrap">
        <!-- AI Suggestion button -->
        <button
          @click="suggestWithAi"
          :disabled="isTranslatingWithAi"
          class="px-3 py-1.5 rounded-xl bg-indigo-500/15 hover:bg-indigo-500/25 border border-indigo-500/30 text-indigo-300 text-xs font-semibold transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed shadow-xs"
          :title="$t('modals.mcp_tool.suggest_ai_title')"
        >
          <Loader2 v-if="isTranslatingWithAi" class="w-3.5 h-3.5 animate-spin" />
          <Sparkles v-else class="w-3.5 h-3.5 text-indigo-400" />
          <span>{{ isTranslatingWithAi ? $t('modals.mcp_tool.consulting_ai') : $t('modals.mcp_tool.suggest_ai') }}</span>
        </button>

        <div class="flex items-center gap-2">
          <button
            @click="closeModal"
            class="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-slate-300 text-xs font-semibold transition-all cursor-pointer"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            @click="saveChanges"
            :disabled="isSaving"
            class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
          >
            <Loader2 v-if="isSaving" class="w-3.5 h-3.5 animate-spin" />
            <Check v-else class="w-3.5 h-3.5" />
            <span>{{ $t('modals.mcp_tool.save_changes') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Wrench, X, Check, Sparkles, Loader2, Terminal } from 'lucide-vue-next'
import { useAppLocale } from '~/composables/useLocale'
import type { McpServerConfig, McpToolDefinition, AppConfig } from '~/types'

const { currentLocale } = useAppLocale()

export interface EditMcpToolSavedPayload {
  serverId: string
  toolName: string
  label: string
  fieldLabels: Record<string, string>
  enabled: boolean
}

const props = defineProps<{
  isOpen: boolean
  server?: McpServerConfig | null
  tool?: McpToolDefinition | null
  config?: AppConfig | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved', payload: EditMcpToolSavedPayload): void
}>()

const formLabel = ref('')
const formFieldLabels: Ref<Record<string, string>> = ref({})
const formEnabled = ref(true)
const isTranslatingWithAi = ref(false)
const isSaving = ref(false)

const heuristicFieldTranslation = (key: string): string => {
  const mapPt: Record<string, string> = {
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
  const mapEs: Record<string, string> = {
    upcoming_days: 'Próximos Días',
    overdue_days: 'Días de Retraso',
    query: 'Término de Búsqueda',
    q: 'Término de Búsqueda',
    limit: 'Límite',
    offset: 'Desplazamiento',
    page: 'Página',
    id: 'Identificador',
    identifier: 'Identificador',
    user_id: 'ID de Usuario',
    start_date: 'Fecha Inicial',
    end_date: 'Fecha Final',
    date: 'Fecha',
    month: 'Mes',
    year: 'Año',
    amount: 'Monto',
    currency: 'Moneda',
    description: 'Descripción',
    desc: 'Descripción',
    title: 'Título',
    name: 'Nombre',
    status: 'Estado',
    category: 'Categoría',
    filter: 'Filtro',
    sort: 'Ordenación',
    order: 'Ordenación',
    path: 'Ruta',
    filepath: 'Ruta del Archivo',
    filename: 'Nombre del Archivo',
    content: 'Contenido',
    text: 'Texto',
    url: 'Dirección (URL)',
    type: 'Tipo',
    enabled: 'Activado'
  }
  const mapEn: Record<string, string> = {
    upcoming_days: 'Upcoming Days',
    overdue_days: 'Overdue Days',
    query: 'Search Query',
    q: 'Search Query',
    limit: 'Limit',
    offset: 'Offset',
    page: 'Page',
    id: 'Identifier',
    identifier: 'Identifier',
    user_id: 'User ID',
    start_date: 'Start Date',
    end_date: 'End Date',
    date: 'Date',
    month: 'Month',
    year: 'Year',
    amount: 'Amount',
    currency: 'Currency',
    description: 'Description',
    desc: 'Description',
    title: 'Title',
    name: 'Name',
    status: 'Status',
    category: 'Category',
    filter: 'Filter',
    sort: 'Sort',
    order: 'Order',
    path: 'Path',
    filepath: 'File Path',
    filename: 'File Name',
    content: 'Content',
    text: 'Text',
    url: 'Address (URL)',
    type: 'Type',
    enabled: 'Enabled'
  }

  const locale = currentLocale.value || 'pt-BR'
  const map = locale === 'pt-BR' ? mapPt : (locale === 'es' ? mapEs : mapEn)
  if (map[key]) return map[key]
  return key
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

export interface ToolPropertyItem {
  key: string
  type: string
  description: string
  isRequired: boolean
  fallback: string
}

const propertiesList = computed<ToolPropertyItem[]>(() => {
  if (!props.tool) return []
  let schema: any =
    props.tool.inputSchema ||
    props.tool.input_schema ||
    (props.tool as any).parameters ||
    (props.tool as any).schema
  if (typeof schema === 'string') {
    try {
      schema = JSON.parse(schema)
    } catch (e) {
      schema = null
    }
  }
  if (!schema || typeof schema !== 'object') return []

  const propsObj =
    schema.properties ||
    schema.parameters?.properties ||
    schema.args?.properties ||
    {}
  const reqList =
    schema.required ||
    schema.parameters?.required ||
    []

  return Object.entries(propsObj).map(([key, val]) => {
    const valObj: any = typeof val === 'object' && val !== null ? val : {}
    return {
      key,
      type: valObj.type || 'any',
      description: valObj.description || '',
      isRequired: Array.isArray(reqList) && reqList.includes(key),
      fallback: heuristicFieldTranslation(key)
    }
  })
})

const initForm = () => {
  if (!props.tool) return
  formLabel.value =
    props.server?.tool_labels?.[props.tool.name] ||
    props.tool.label ||
    props.tool.name

  const isDisabled = Array.isArray(props.server?.disabled_tools) && props.server.disabled_tools.includes(props.tool.name)
  formEnabled.value = !isDisabled

  const existingFields =
    props.server?.tool_field_labels?.[props.tool.name] ||
    props.tool.field_labels ||
    {}

  const fields: Record<string, string> = {}
  for (const p of propertiesList.value) {
    fields[p.key] = existingFields[p.key] || p.fallback
  }
  formFieldLabels.value = fields
}

watch(
  () => [props.isOpen, props.tool],
  ([newOpen]) => {
    if (newOpen) {
      initForm()
    }
  },
  { immediate: true }
)

const closeModal = () => {
  emit('close')
}

const suggestWithAi = async () => {
  if (!props.tool || isTranslatingWithAi.value) return
  isTranslatingWithAi.value = true
  try {
    const schema =
      props.tool.inputSchema ||
      props.tool.input_schema ||
      (props.tool as any).parameters ||
      (props.tool as any).schema ||
      { type: 'object' }
    const toolPayload = {
      name: props.tool.name,
      description: props.tool.description || null,
      inputSchema: typeof schema === 'string' ? JSON.parse(schema) : schema,
      label: props.tool.label || null,
      field_labels: props.tool.field_labels || {}
    }
    const res: any = await invoke('translate_mcp_tool_labels', {
      tools: [toolPayload],
      mlxHost: props.config?.mlx_host || '127.0.0.1',
      mlxPort: props.config?.mlx_port || 8080,
      ollamaHost: props.config?.ollama_host || '127.0.0.1',
      ollamaPort: props.config?.ollama_port || 11434,
      targetLocale: currentLocale.value
    })
    if (res?.tool_labels?.[props.tool.name]) {
      formLabel.value = res.tool_labels[props.tool.name]
    }
    if (res?.tool_field_labels?.[props.tool.name]) {
      const suggestedFields = res.tool_field_labels[props.tool.name]
      for (const [k, v] of Object.entries(suggestedFields)) {
        formFieldLabels.value[k] = v as string
      }
    }
  } catch (err) {
    console.error('Failed to suggest with AI:', err)
  } finally {
    isTranslatingWithAi.value = false
  }
}

const saveChanges = async () => {
  if (!props.server || !props.tool) return
  isSaving.value = true
  try {
    const label = formLabel.value.trim()
    const fieldLabels = { ...formFieldLabels.value }

    // Call backend command to update details directly
    await invoke('update_mcp_tool_details', {
      serverId: props.server.id,
      toolName: props.tool.name,
      label: label || null,
      fieldLabels: Object.keys(fieldLabels).length > 0 ? fieldLabels : null,
      enabled: formEnabled.value
    })

    emit('saved', {
      serverId: props.server.id,
      toolName: props.tool.name,
      label,
      fieldLabels,
      enabled: formEnabled.value
    })
    closeModal()
  } catch (err) {
    console.error('Failed to save MCP tool details:', err)
  } finally {
    isSaving.value = false
  }
}
</script>
