import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useConfigStore = defineStore('config', {
  state: () => ({
    zentaoAccounts: [],
    git: {
      maxCommits: 100,
      includeMerged: false,
      branchPattern: '.*',
    },
    output: {
      reportDir: 'reports',
      verbose: true,
    },
    loaded: false,
  }),

  actions: {
    async load() {
      try {
        const config = await invoke('load_config')
        // 后端现在返回 camelCase 字段名 (通过 serde rename)
        // 直接使用 $patch 合并配置
        this.$patch({
          zentaoAccounts: config.zentaoAccounts || [],
          git: config.git || this.git,
          output: config.output || this.output,
          loaded: true,
        })
        return true
      } catch (e) {
        console.error('加载配置失败:', e)
        return false
      }
    },

    async save() {
      try {
        // 保存时需要将 camelCase 转换为后端期望的格式
        const payload = {
          zentaoAccounts: this.zentaoAccounts,
          git: {
            max_commits: this.git.maxCommits ?? this.git.max_commits ?? 100,
            include_merged: this.git.includeMerged ?? this.git.include_merged ?? false,
            branch_pattern: this.git.branchPattern ?? this.git.branch_pattern ?? '.*',
          },
          output: {
            report_dir: this.output.reportDir ?? this.output.report_dir ?? 'reports',
            verbose: this.output.verbose ?? true,
          },
        }
        await invoke('save_config', { config: payload })
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

  persist: {
    key: 'config-store',
    storage: localStorage,
    // 不持久化 loaded 状态，每次启动时重新从后端加载
    pick: ['zentaoAccounts', 'git', 'output'],
  },
})
