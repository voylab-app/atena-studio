import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Project {
  id: string
  name: string
  createdAt: number
  color?: string
}

const STORAGE_KEY_PROJECTS = 'atena_projects'
const STORAGE_KEY_ACTIVE = 'atena_active_project_id'
const STORAGE_KEY_EXPANDED = 'atena_expanded_projects'

const projects = ref<Project[]>([])
const activeProjectId = ref<string | null>(null)
const expandedProjectIds = ref<Set<string>>(new Set())
const isInitialized = ref(false)

export function useProjects() {
  const activeProject = computed(() => {
    return projects.value.find((p) => p.id === activeProjectId.value) || null
  })

  const saveProjects = () => {
    try {
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY_PROJECTS, JSON.stringify(projects.value))
        if (activeProjectId.value) {
          localStorage.setItem(STORAGE_KEY_ACTIVE, activeProjectId.value)
        } else {
          localStorage.removeItem(STORAGE_KEY_ACTIVE)
        }
      }
      invoke('db_set_setting', {
        key: 'atena_projects',
        value: JSON.stringify(projects.value)
      }).catch(() => {})
      invoke('db_set_setting', {
        key: 'atena_active_project_id',
        value: activeProjectId.value || ''
      }).catch(() => {})
    } catch (err) {
      console.error('Erro ao salvar projetos:', err)
    }
  }

  const saveExpandedProjects = () => {
    try {
      if (typeof window !== 'undefined') {
        localStorage.setItem(
          STORAGE_KEY_EXPANDED,
          JSON.stringify(Array.from(expandedProjectIds.value))
        )
      }
    } catch (err) {
      console.error('Erro ao salvar pastas expandidas no localStorage:', err)
    }
  }

  const isProjectExpanded = (projectId: string): boolean => {
    return expandedProjectIds.value.has(projectId)
  }

  const toggleProjectExpand = (projectId: string) => {
    const next = new Set(expandedProjectIds.value)
    if (next.has(projectId)) {
      next.delete(projectId)
    } else {
      next.add(projectId)
    }
    expandedProjectIds.value = next
    saveExpandedProjects()
  }

  const setProjectExpanded = (projectId: string, expanded: boolean) => {
    const next = new Set(expandedProjectIds.value)
    if (expanded) {
      next.add(projectId)
    } else {
      next.delete(projectId)
    }
    expandedProjectIds.value = next
    saveExpandedProjects()
  }

  const initProjects = () => {
    if (isInitialized.value) return
    if (typeof window === 'undefined') return

    try {
      const raw = localStorage.getItem(STORAGE_KEY_PROJECTS)
      if (raw) {
        projects.value = JSON.parse(raw)
      }
      const savedActive = localStorage.getItem(STORAGE_KEY_ACTIVE)
      if (savedActive && projects.value.some((p) => p.id === savedActive)) {
        activeProjectId.value = savedActive
      } else {
        activeProjectId.value = null
      }

      const rawExpanded = localStorage.getItem(STORAGE_KEY_EXPANDED)
      if (rawExpanded !== null) {
        try {
          const parsed = JSON.parse(rawExpanded)
          if (Array.isArray(parsed)) {
            expandedProjectIds.value = new Set(parsed)
          }
        } catch (e) {
          console.error('Erro ao ler pastas expandidas do localStorage:', e)
        }
      } else {
        // Primeira execução: padrão com todas as pastas expandidas
        expandedProjectIds.value = new Set(projects.value.map((p) => p.id))
        saveExpandedProjects()
      }
    } catch (err) {
      console.error('Erro ao inicializar projetos:', err)
    }

    // Sync authoritative project state from SQLite
    invoke<string | null>('db_get_setting', { key: 'atena_projects' })
      .then((dbRaw) => {
        if (dbRaw) {
          try {
            const parsed = JSON.parse(dbRaw)
            if (Array.isArray(parsed) && parsed.length > 0) {
              projects.value = parsed
            }
          } catch (_) {}
        } else if (projects.value.length > 0) {
          invoke('db_set_setting', {
            key: 'atena_projects',
            value: JSON.stringify(projects.value)
          }).catch(() => {})
        }
      })
      .catch(() => {})

    invoke<string | null>('db_get_setting', { key: 'atena_active_project_id' })
      .then((dbActive) => {
        if (dbActive && projects.value.some((p) => p.id === dbActive)) {
          activeProjectId.value = dbActive
        }
      })
      .catch(() => {})

    isInitialized.value = true
  }

  const selectProject = (id: string | null) => {
    if (activeProjectId.value === id) return
    activeProjectId.value = id
    saveProjects()
  }

  const createProject = (name: string): Project => {
    const trimmed = name.trim() || 'Novo Projeto'
    const newProject: Project = {
      id: `proj-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      name: trimmed,
      createdAt: Date.now(),
    }
    projects.value.unshift(newProject)
    activeProjectId.value = newProject.id

    const next = new Set(expandedProjectIds.value)
    next.add(newProject.id)
    expandedProjectIds.value = next

    saveProjects()
    saveExpandedProjects()
    return newProject
  }

  const updateProject = (id: string, updates: Partial<Omit<Project, 'id' | 'createdAt'>>) => {
    const proj = projects.value.find((p) => p.id === id)
    if (!proj) return
    if (updates.name !== undefined) proj.name = updates.name.trim()
    if (updates.color !== undefined) proj.color = updates.color
    saveProjects()
  }

  const deleteProject = (id: string) => {
    const idx = projects.value.findIndex((p) => p.id === id)
    if (idx === -1) return
    projects.value.splice(idx, 1)
    if (activeProjectId.value === id) {
      activeProjectId.value = null
    }

    if (expandedProjectIds.value.has(id)) {
      const next = new Set(expandedProjectIds.value)
      next.delete(id)
      expandedProjectIds.value = next
      saveExpandedProjects()
    }

    saveProjects()
  }

  return {
    projects,
    activeProjectId,
    activeProject,
    expandedProjectIds,
    isProjectExpanded,
    toggleProjectExpand,
    setProjectExpanded,
    initProjects,
    selectProject,
    createProject,
    updateProject,
    deleteProject,
  }
}
