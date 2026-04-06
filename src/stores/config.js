import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useConfigStore = defineStore('config', {
  state: () => ({
    zentao: {
      url: '',
      account: '',
      password: '',
      project_id: 1,
      assigned_to: '',
      task_type: 'dev',
    },
    git: {
      max_commits: 100,
      include_merged: false,
      branch_pattern: '.*',
    },
    output: {
      report_dir: 'reports',
      verbose: true,
    },
    loaded: false,
  }),

  actions: {
    async load() {
      try {
        const config = await invoke('load_config')
        this.$patch(config)
        this.loaded = true
        return true
      } catch (e) {
        console.error('加载配置失败:', e)
        return false
      }
    },

    async save() {
      try {
        await invoke('save_config', { config: this.$state })
        return true
      } catch (e) {
        console.error('保存配置失败:', e)
        return false
      }
    },

    async validate() {
      if (!this.zentao.url) return false
      if (!this.zentao.account) return false
      if (!this.zentao.password) return false
      if (this.zentao.project_id < 1) return false
      return true
    },
  },
})
