import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useZentaoStore = defineStore('zentao', {
  state: () => ({
    token: null,
    projects: [],
    selectedProject: null,
    taskResults: [],
    creating: false,
    connected: false,
  }),

  getters: {
    successCount: (state) => state.taskResults.filter(r => r.task_created).length,
    failCount: (state) => state.taskResults.filter(r => !r.task_created).length,
  },

  actions: {
    async login(url, account, password) {
      try {
        this.token = await invoke('zentao_login', { url, account, password })
        this.connected = true
        return this.token
      } catch (e) {
        this.connected = false
        throw e
      }
    },

    async fetchProjects(url) {
      try {
        this.projects = await invoke('zentao_get_projects', {
          url,
          token: this.token,
        })
        return this.projects
      } catch (e) {
        console.error('获取项目列表失败:', e)
        throw e
      }
    },

    selectProject(project) {
      this.selectedProject = project
    },

    async createTask(url, branch, commits, projectId, assignedTo, taskType) {
      try {
        const result = await invoke('zentao_create_task', {
          url,
          token: this.token,
          branch,
          commits,
          projectId,
          assignedTo,
          taskType,
        })
        return result
      } catch (e) {
        console.error('创建任务失败:', e)
        throw e
      }
    },

    setResults(results) {
      this.taskResults = results
    },
  },
})
