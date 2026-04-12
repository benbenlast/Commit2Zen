/**
 * LLM 管理器 - 单例模式
 * 负责 LLM 提供商配置管理、任务分配和 API 调用
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  LLMConfig,
  LLMProvider,
  LLMTaskAssignment,
  LLMResponse,
  Message,
  ProviderType,
} from '../types/llm'

/** 默认配置 */
const DEFAULT_CONFIG: LLMConfig = {
  providers: {
    openai: {
      type: 'openai',
      enabled: false,
      apiKey: '',
      baseUrl: 'https://api.openai.com/v1',
      model: 'gpt-4o-mini',
      timeout: 30000,
      maxRetries: 3,
    },
    claude: {
      type: 'claude',
      enabled: false,
      apiKey: '',
      baseUrl: 'https://api.anthropic.com/v1',
      model: 'claude-3-5-sonnet-20241022',
      timeout: 60000,
      maxRetries: 3,
    },
    gemini: {
      type: 'gemini',
      enabled: false,
      apiKey: '',
      baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
      model: 'gemini-2.0-flash',
      timeout: 30000,
      maxRetries: 3,
    },
    ollama: {
      type: 'ollama',
      enabled: false,
      baseUrl: 'http://localhost:11434',
      model: 'llama3.2',
      timeout: 120000,
      maxRetries: 2,
    },
  },
  taskAssignments: [],
  defaultProvider: 'openai',
  maxRetries: 3,
  requestInterval: 1000,
}

/**
 * LLMManager 单例类
 */
export class LLMManager {
  private static instance: LLMManager | null = null

  private config: LLMConfig

  private lastRequestTime: number = 0

  private constructor() {
    this.config = { ...DEFAULT_CONFIG }
  }

  /**
   * 获取单例实例
   */
  static getInstance(): LLMManager {
    if (!LLMManager.instance) {
      LLMManager.instance = new LLMManager()
    }
    return LLMManager.instance
  }

  // ==================== 配置管理 ====================

  /**
   * 获取完整配置
   */
  getConfig(): LLMConfig {
    return { ...this.config }
  }

  /**
   * 设置完整配置
   */
  setConfig(config: LLMConfig): void {
    this.config = {
      ...DEFAULT_CONFIG,
      ...config,
      providers: {
        ...DEFAULT_CONFIG.providers,
        ...config.providers,
      },
    }
  }

  /**
   * 获取指定提供商的配置
   */
  getProvider(type: ProviderType): LLMProvider | undefined {
    return this.config.providers[type]
  }

  /**
   * 更新指定提供商的配置
   */
  updateProvider(type: ProviderType, updates: Partial<LLMProvider>): void {
    const provider = this.config.providers[type]
    if (!provider) {
      throw new Error(`未知的提供商类型: ${type}`)
    }
    this.config.providers[type] = { ...provider, ...updates }
  }

  /**
   * 获取启用的提供商列表
   */
  getEnabledProviders(): LLMProvider[] {
    return Object.values(this.config.providers).filter((p) => p.enabled)
  }

  /**
   * 获取默认提供商配置
   */
  getDefaultProvider(): LLMProvider | undefined {
    const defaultType = this.config.defaultProvider
    if (defaultType) {
      const provider = this.config.providers[defaultType]
      if (provider?.enabled) {
        return provider
      }
    }
    // 如果没有默认或默认未启用，返回第一个启用的
    return this.getEnabledProviders()[0]
  }

  // ==================== 任务分配 ====================

  /**
   * 获取适合指定任务的提供商
   */
  getProviderForTask(taskType: string): LLMProvider | undefined {
    // 查找匹配的任务分配规则
    const assignment = this.config.taskAssignments.find(
      (a) => a.taskType === taskType && a.enabled
    )

    if (assignment) {
      const provider = this.config.providers[assignment.providerType]
      if (provider?.enabled) {
        return provider
      }
    }

    // 如果没有匹配的规则，返回默认提供商
    return this.getDefaultProvider()
  }

  /**
   * 添加或更新任务分配规则
   */
  setTaskAssignment(assignment: LLMTaskAssignment): void {
    const index = this.config.taskAssignments.findIndex(
      (a) => a.taskType === assignment.taskType
    )
    if (index !== -1) {
      this.config.taskAssignments[index] = assignment
    } else {
      this.config.taskAssignments.push(assignment)
    }
  }

  /**
   * 移除任务分配规则
   */
  removeTaskAssignment(taskType: string): void {
    this.config.taskAssignments = this.config.taskAssignments.filter(
      (a) => a.taskType !== taskType
    )
  }

  // ==================== LLM 调用 ====================

  /**
   * 为指定任务调用 LLM
   */
  async callLLMForTask(
    taskType: string,
    messages: Message[],
    systemPrompt?: string
  ): Promise<LLMResponse> {
    const provider = this.getProviderForTask(taskType)
    if (!provider) {
      throw new Error(`没有可用的提供商来处理任务: ${taskType}`)
    }

    const finalMessages = this.buildMessages(messages, systemPrompt)
    return this.callProvider(provider, finalMessages)
  }

  /**
   * 调用指定提供商
   */
  async callProvider(
    provider: LLMProvider,
    messages: Message[]
  ): Promise<LLMResponse> {
    // 限流控制
    await this.enforceRateLimit()

    const maxRetries = provider.maxRetries ?? this.config.maxRetries ?? 3
    let lastError: Error | null = null

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        switch (provider.type) {
          case 'openai':
            return await this.callOpenAI(provider, messages)
          case 'claude':
            return await this.callClaude(provider, messages)
          case 'gemini':
            return await this.callGemini(provider, messages)
          case 'ollama':
            return await this.callOllama(provider, messages)
          default:
            throw new Error(`不支持的提供商: ${provider.type}`)
        }
      } catch (error) {
        lastError = error as Error
        console.warn(
          `[LLM] ${provider.type} 调用失败 (尝试 ${attempt + 1}/${maxRetries + 1}):`,
          error
        )

        if (attempt < maxRetries) {
          // 指数退避重试
          const delay = Math.min(1000 * Math.pow(2, attempt), 10000)
          await this.sleep(delay)
        }
      }
    }

    throw new Error(
      `${provider.type} 调用失败，已重试 ${maxRetries} 次: ${lastError?.message}`
    )
  }

  /**
   * 调用 OpenAI API
   */
  private async callOpenAI(
    provider: LLMProvider,
    messages: Message[]
  ): Promise<LLMResponse> {
    try {
      const result = await invoke('llm_call_openai', {
        apiKey: provider.apiKey,
        baseUrl: provider.baseUrl,
        model: provider.model,
        messages,
        timeout: provider.timeout,
      })
      return this.normalizeResponse(result, 'openai')
    } catch (error) {
      throw this.wrapError(error, 'openai')
    }
  }

  /**
   * 调用 Claude API
   */
  private async callClaude(
    provider: LLMProvider,
    messages: Message[]
  ): Promise<LLMResponse> {
    try {
      const result = await invoke('llm_call_claude', {
        apiKey: provider.apiKey,
        baseUrl: provider.baseUrl,
        model: provider.model,
        messages,
        timeout: provider.timeout,
      })
      return this.normalizeResponse(result, 'claude')
    } catch (error) {
      throw this.wrapError(error, 'claude')
    }
  }

  /**
   * 调用 Gemini API
   */
  private async callGemini(
    provider: LLMProvider,
    messages: Message[]
  ): Promise<LLMResponse> {
    try {
      const result = await invoke('llm_call_gemini', {
        apiKey: provider.apiKey,
        baseUrl: provider.baseUrl,
        model: provider.model,
        messages,
        timeout: provider.timeout,
      })
      return this.normalizeResponse(result, 'gemini')
    } catch (error) {
      throw this.wrapError(error, 'gemini')
    }
  }

  /**
   * 调用 Ollama API
   */
  private async callOllama(
    provider: LLMProvider,
    messages: Message[]
  ): Promise<LLMResponse> {
    try {
      const result = await invoke('llm_call_ollama', {
        baseUrl: provider.baseUrl,
        model: provider.model,
        messages,
        timeout: provider.timeout,
      })
      return this.normalizeResponse(result, 'ollama')
    } catch (error) {
      throw this.wrapError(error, 'ollama')
    }
  }

  /**
   * 测试提供商连接
   */
  async testConnection(providerType: ProviderType): Promise<boolean> {
    const provider = this.config.providers[providerType]
    if (!provider) {
      throw new Error(`未知的提供商类型: ${providerType}`)
    }

    try {
      const testMessages: Message[] = [
        { role: 'user', content: 'Hi' },
      ]

      await this.callProvider(provider, testMessages)
      return true
    } catch {
      return false
    }
  }

  // ==================== 持久化 ====================

  /**
   * 从后端加载配置
   */
  async loadConfig(): Promise<boolean> {
    try {
      const config = await invoke<LLMConfig>('llm_load_config')
      this.setConfig(config)
      return true
    } catch (error) {
      console.warn('[LLM] 加载配置失败，使用默认配置:', error)
      return false
    }
  }

  /**
   * 保存配置到后端
   */
  async saveConfig(): Promise<boolean> {
    try {
      await invoke('llm_save_config', { config: this.config })
      return true
    } catch (error) {
      console.error('[LLM] 保存配置失败:', error)
      return false
    }
  }

  // ==================== 私有工具方法 ====================

  /**
   * 构建完整的消息列表（添加系统提示词）
   */
  private buildMessages(messages: Message[], systemPrompt?: string): Message[] {
    const finalMessages: Message[] = []

    if (systemPrompt) {
      finalMessages.push({ role: 'system', content: systemPrompt })
    }

    finalMessages.push(...messages)
    return finalMessages
  }

  /**
   * 规范化后端返回的响应
   */
  private normalizeResponse(
    result: unknown,
    providerType: ProviderType
  ): LLMResponse {
    const data = result as Record<string, unknown>
    return {
      content: (data.content as string) || '',
      model: (data.model as string) || '',
      provider: providerType,
      usage: data.usage
        ? {
            promptTokens: (data.usage as Record<string, number>).prompt_tokens || 0,
            completionTokens: (data.usage as Record<string, number>).completion_tokens || 0,
            totalTokens: (data.usage as Record<string, number>).total_tokens || 0,
          }
        : undefined,
    }
  }

  /**
   * 包装错误信息
   */
  private wrapError(error: unknown, providerType: ProviderType): Error {
    const message = error instanceof Error ? error.message : String(error)
    return new Error(`[${providerType}] ${message}`)
  }

  /**
   * 限流控制
   */
  private async enforceRateLimit(): Promise<void> {
    const interval = this.config.requestInterval || 1000
    const now = Date.now()
    const elapsed = now - this.lastRequestTime

    if (elapsed < interval) {
      await this.sleep(interval - elapsed)
    }

    this.lastRequestTime = Date.now()
  }

  /**
   * 休眠工具
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms))
  }
}

/**
 * 获取 LLMManager 单例实例的便捷方法
 */
export function getLLMManager(): LLMManager {
  return LLMManager.getInstance()
}
