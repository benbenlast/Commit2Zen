import { defineStore } from 'pinia'
import { getLLMManager } from '../utils/llm'

export const useLLMStore = defineStore('llm', {
  state: () => ({
    llmConfig: null,
    currentProvider: null,
    loading: false,
    error: null,
    // 连接测试状态
    testingProviders: {},
  }),

  getters: {
    /**
     * 获取所有启用的提供商列表
     */
    enabledProviders: (state) => {
      if (!state.llmConfig) return []
      return Object.values(state.llmConfig.providers || {}).filter(
        (provider) => provider.enabled
      )
    },

    /**
     * 获取当前选中的提供商配置
     */
    currentProviderConfig: (state) => {
      if (!state.llmConfig || !state.currentProvider) return null
      return state.llmConfig.providers[state.currentProvider] || null
    },

    /**
     * 获取所有提供商配置（含未启用的）
     */
    allProviders: (state) => {
      if (!state.llmConfig) return {}
      return state.llmConfig.providers || {}
    },

    /**
     * 获取任务分配规则
     */
    taskAssignments: (state) => {
      if (!state.llmConfig) return []
      return state.llmConfig.taskAssignments || []
    },

    /**
     * 默认提供商
     */
    defaultProvider: (state) => {
      if (!state.llmConfig) return null
      return state.llmConfig.defaultProvider || null
    },

    /**
     * 获取适合指定任务的提供商
     */
    getProviderForTask: (state) => {
      return (taskType) => {
        if (!state.llmConfig) return undefined
        const assignments = state.llmConfig.taskAssignments || []
        const assignment = assignments.find(
          (a) => a.taskType === taskType && a.enabled
        )
        if (assignment) {
          const provider = state.llmConfig.providers[assignment.providerType]
          if (provider?.enabled) {
            return provider
          }
        }
        // 返回默认提供商或第一个启用的
        const defaultType = state.llmConfig.defaultProvider
        if (defaultType) {
          const provider = state.llmConfig.providers[defaultType]
          if (provider?.enabled) {
            return provider
          }
        }
        return Object.values(state.llmConfig.providers || {}).find((p) => p.enabled)
      }
    },
  },

  actions: {
    /**
     * 从后端加载 LLM 配置
     */
    async loadConfig() {
      this.loading = true
      this.error = null
      try {
        const manager = getLLMManager()
        const success = await manager.loadConfig()
        if (success) {
          this.llmConfig = manager.getConfig()
          // 设置默认选中的提供商
          this.currentProvider = this.llmConfig.defaultProvider || 'openai'
        }
        return success
      } catch (e) {
        this.error = e.message || '加载配置失败'
        console.error('[llm store] 加载配置失败:', e)
        return false
      } finally {
        this.loading = false
      }
    },

    /**
     * 保存 LLM 配置到后端
     */
    async saveConfig() {
      this.loading = true
      this.error = null
      try {
        const manager = getLLMManager()
        manager.setConfig(this.llmConfig)
        const success = await manager.saveConfig()
        if (success) {
          this.llmConfig = manager.getConfig()
        }
        return success
      } catch (e) {
        this.error = e.message || '保存配置失败'
        console.error('[llm store] 保存配置失败:', e)
        return false
      } finally {
        this.loading = false
      }
    },

    /**
     * 更新提供商配置
     */
    updateProviderConfig(providerType, updates) {
      if (!this.llmConfig) {
        this.llmConfig = getLLMManager().getConfig()
      }
      const manager = getLLMManager()
      manager.updateProvider(providerType, updates)
      this.llmConfig = manager.getConfig()
    },

    /**
     * 测试提供商连接
     */
    async testConnection(providerType) {
      this.testingProviders = { ...this.testingProviders, [providerType]: true }
      this.error = null
      try {
        const manager = getLLMManager()
        const success = await manager.testConnection(providerType)
        return success
      } catch (e) {
        this.error = e.message || '连接测试失败'
        console.error(`[llm store] ${providerType} 连接测试失败:`, e)
        return false
      } finally {
        this.testingProviders = { ...this.testingProviders, [providerType]: false }
      }
    },

    /**
     * 选择当前提供商
     */
    selectProvider(providerType) {
      this.currentProvider = providerType
    },

    /**
     * 设置默认提供商
     */
    setDefaultProvider(providerType) {
      if (!this.llmConfig) {
        this.llmConfig = getLLMManager().getConfig()
      }
      this.llmConfig.defaultProvider = providerType
      this.currentProvider = providerType
    },

    /**
     * 添加或更新任务分配规则
     */
    setTaskAssignment(assignment) {
      if (!this.llmConfig) {
        this.llmConfig = getLLMManager().getConfig()
      }
      const manager = getLLMManager()
      manager.setTaskAssignment(assignment)
      this.llmConfig = manager.getConfig()
    },

    /**
     * 移除任务分配规则
     */
    removeTaskAssignment(taskType) {
      if (!this.llmConfig) return
      const manager = getLLMManager()
      manager.removeTaskAssignment(taskType)
      this.llmConfig = manager.getConfig()
    },

    /**
     * 调用 LLM 处理任务
     */
    async callLLMForTask(taskType, messages, systemPrompt) {
      this.loading = true
      this.error = null
      try {
        const manager = getLLMManager()
        const result = await manager.callLLMForTask(taskType, messages, systemPrompt)
        return result
      } catch (e) {
        this.error = e.message || 'LLM 调用失败'
        console.error('[llm store] LLM 调用失败:', e)
        throw e
      } finally {
        this.loading = false
      }
    },

    /**
     * 重置配置为默认值
     */
    resetConfig() {
      const manager = getLLMManager()
      this.llmConfig = manager.getConfig()
      this.currentProvider = this.llmConfig.defaultProvider || 'openai'
      this.error = null
    },

    /**
     * 清除错误状态
     */
    clearError() {
      this.error = null
    },
  },

  persist: {
    key: 'llm-store',
    storage: localStorage,
    // 不持久化 loading, error, testingProviders 等运行时状态
    pick: ['llmConfig', 'currentProvider'],
  },
})
