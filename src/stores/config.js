import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useConfigStore = defineStore('config', {
  state: () => ({
    zentaoAccounts: [],
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
        // 后端返回 snake_case 字段名，需要手动映射到前端 camelCase
        this.zentaoAccounts = config.zentao_accounts || []
        this.git = config.git || this.git
        this.output = config.output || this.output
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

    async loadAccounts() {
      try {
        const accounts = await invoke('list_zentao_accounts')
        this.zentaoAccounts = accounts
        return accounts
      } catch (e) {
        console.error('加载账号列表失败:', e)
        throw e
      }
    },

    async addAccount(account) {
      try {
        const newAccount = await invoke('add_zentao_account', { account })
        this.zentaoAccounts.push(newAccount)
        return newAccount
      } catch (e) {
        console.error('添加账号失败:', e)
        throw e
      }
    },

    async updateAccount(account) {
      try {
        const updatedAccount = await invoke('update_zentao_account', { account })
        const index = this.zentaoAccounts.findIndex(a => a.id === account.id)
        if (index !== -1) {
          this.zentaoAccounts[index] = updatedAccount
        }
        return updatedAccount
      } catch (e) {
        console.error('更新账号失败:', e)
        throw e
      }
    },

    async deleteAccount(id) {
      try {
        await invoke('delete_zentao_account', { id })
        this.zentaoAccounts = this.zentaoAccounts.filter(a => a.id !== id)
        return true
      } catch (e) {
        console.error('删除账号失败:', e)
        throw e
      }
    },

    async testConnection(url, account, password) {
      try {
        const token = await invoke('test_zentao_connection', { url, account, password })
        return token
      } catch (e) {
        console.error('测试连接失败:', e)
        throw e
      }
    },
  },
})
