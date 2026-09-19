import type { Project } from '../composables/useProjects'
import type { PluginInfo } from '../composables/usePlugins'

export type { Project, PluginInfo }

export type MessageRole = 'system' | 'user' | 'assistant'

export interface GenerationMetrics {
  prompt_eval_count?: number
  prompt_eval_duration_ms?: number
  eval_count?: number
  eval_duration_ms?: number
  tokens_per_second?: number
  time_to_first_token_ms?: number
  total_duration_ms?: number
  prompt_tokens?: number
  cached_tokens?: number
  prefill_tokens?: number
  completion_tokens?: number
  cache_efficiency_pct?: number
  generation_speed_tps?: number
  prefill_speed_tps?: number
  [key: string]: any
}

export interface ChatAttachment {
  name: string
  size?: number
  type?: string
  content?: string
  [key: string]: any
}

export interface ChatMessage {
  id: string
  role: MessageRole | string
  content: string
  display_text?: string
  thinking?: string | null
  tool_calls?: McpToolCall[] | null
  tool_call_id?: string | null
  images?: string[] | null
  attachments?: ChatAttachment[] | null
  timestamp: string
  tokens_count?: number
  generation_speed_tps?: number
  time_to_first_token_ms?: number
  is_streaming?: boolean
  metrics?: GenerationMetrics | null
  [key: string]: any
}

export interface ChatSession {
  id: string
  title: string
  model_id?: string | null
  model_name?: string | null
  project_id?: string | null
  created_at: string
  updated_at?: string | null
  messages: ChatMessage[]
  is_private?: boolean
  pinned?: boolean
  archived?: boolean
  archived_at?: string | null
  [key: string]: any
}

// Aliases for convenience across components
export type Message = ChatMessage
export type Session = ChatSession

export type ModelFormat = 'Mlx' | 'Gguf' | 'Ollama' | 'Agy' | 'Cloud'
export type BackendType = 'MlxLm' | 'Ollama' | 'LocalNative' | 'Antigravity' | 'CloudOpenAi'

export type ModelStatus =
  | 'NotDownloaded'
  | { Downloading: { progress: number; speed_mbps: number } }
  | 'Downloaded'
  | { Loading: { progress: number } }
  | { Loaded: { vram_usage_mb: number; ram_usage_mb: number } }

export interface ModelInfo {
  id: string
  name: string
  author?: string
  architecture?: string
  quantization?: string
  size_gb?: number
  context_length?: number
  description?: string
  status?: ModelStatus | string
  downloads_count?: number
  is_featured?: boolean
  format?: ModelFormat | string
  backend?: BackendType | string
  local_path?: string | null
  supports_tools?: boolean
  supports_vision?: boolean
  supports_thinking?: boolean
  author_avatar_url?: string | null
  [key: string]: any
}

export type Model = ModelInfo

export interface HardwareInfo {
  chip_name: string
  gpu_cores: number
  metal_version: string
  total_vram_gb: number
  used_vram_gb: number
  system_ram_gb: number
  used_system_ram_gb: number
  [key: string]: any
}

export interface GenerationParams {
  temperature?: number
  top_p?: number
  max_tokens?: number
  context_length?: number
  system_prompt?: string
  enable_thinking?: boolean
  thinking_budget?: number
  kv_cache_quant?: string
  enable_prompt_cache?: boolean
  flash_attention?: boolean
  [key: string]: any
}

export interface CloudProviderConfig {
  enabled: boolean
  api_key: string
  base_url?: string
  models?: string[]
  [key: string]: any
}

export interface CloudProvidersSettings {
  openai?: CloudProviderConfig
  anthropic?: CloudProviderConfig
  gemini?: CloudProviderConfig
  groq?: CloudProviderConfig
  openrouter?: CloudProviderConfig
  deepseek?: CloudProviderConfig
  disabled_models?: string[]
  [key: string]: any
}

export interface DetectedModelDirectory {
  name: string
  path: string
  exists: boolean
  source: string
}

export interface AppConfig {
  models_directory?: string
  models_directories?: string[]
  mlx_host?: string
  mlx_port?: number
  ollama_host?: string
  ollama_port?: number
  auto_load_last_model?: boolean
  timezone?: string
  inject_current_date?: boolean
  inject_message_time?: boolean
  enable_cognitive_memory?: boolean
  enable_facts_memory?: boolean
  enable_skills_memory?: boolean
  enable_episodic_memory?: boolean
  show_efficiency_metrics?: boolean
  whisper_model?: string
  whisper_language?: string
  cloud_providers?: CloudProvidersSettings
  [key: string]: any
}

export interface McpToolDefinition {
  name: string
  description?: string | null
  inputSchema?: any
  input_schema?: any
  label?: string | null
  field_labels?: Record<string, string>
}

export interface McpToolWithServer {
  server_id: string
  server_name: string
  tool: McpToolDefinition
  enabled: boolean
  permission_mode: string
}

export interface SkillStepItem {
  order: number
  instruction: string
  tool_name?: string | null
  command?: string | null
  script_file?: string | null
  cwd?: string | null
  timeout_ms?: number | null
}

export interface SkillItem {
  id: string
  name: string
  description: string
  triggers: string[]
  steps: SkillStepItem[]
  version: number
  executions_count: number
  success_count: number
  last_refined_at: number
  refinement_notes: string[]
  folder_path?: string | null
  scripts?: string[]
  env_vars?: Record<string, string> | null
  permission_mode?: string
  enabled?: boolean
  [key: string]: any
}

export interface McpToolCall {
  id: string
  name: string
  arguments: any
  server_id?: string | null
  server_name?: string | null
  permission_mode?: string | null
  status?: string | null
  rejection_reason?: string | null
  result?: any
  label?: string | null
  field_labels?: Record<string, string>
  [key: string]: any
}

export interface McpServerConfig {
  id: string
  name: string
  transport: 'stdio' | 'sse' | 'http' | string
  command?: string | null
  args?: string[] | null
  env?: Record<string, string> | null
  url?: string | null
  headers?: Record<string, string> | null
  enabled: boolean
  description?: string | null
  permission_mode?: string
  tool_permissions?: Record<string, string>
  tool_labels?: Record<string, string>
  tool_field_labels?: Record<string, Record<string, string>>
  disabled_tools?: string[]
  [key: string]: any
}

export interface Persona {
  id: string
  name: string
  tag: string
  iconName: string
  color: string
  description: string
  system_prompt: string
  [key: string]: any
}

export interface NotificationItem {
  id: string
  type: 'info' | 'success' | 'warning' | 'error' | string
  title: string
  message: string
  timestamp: string
  read: boolean
  data?: any
}

export interface Toast extends NotificationItem {
  toastId: string
}

export interface DownloadItem {
  id: string
  repo_id?: string
  model_id?: string
  name?: string
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | string
  progress?: number
  speed_mbps?: number
  total_bytes?: number
  downloaded_bytes?: number
  error?: string | null
  [key: string]: any
}

export interface ServerRequestLog {
  id: string
  timestamp: string
  method: string
  path: string
  status_code: number
  latency_ms: number
  tokens_prompt: number
  tokens_completion: number
  model?: string | null
  body_preview?: string | null
}

export interface DeveloperLogEntry {
  id: string
  timestamp: string
  level: 'INFO' | 'DEBUG' | 'WARN' | 'ERROR'
  tag?: string | null
  message: string
  details?: string | null
}

