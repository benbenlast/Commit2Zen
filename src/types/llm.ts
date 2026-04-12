/**
 * LLM 相关类型定义
 */

/** LLM 提供商类型 */
export type ProviderType = 'openai' | 'claude' | 'gemini' | 'ollama';

/** LLM 消息角色 */
export type MessageRole = 'system' | 'user' | 'assistant';

/** LLM 提供商配置 */
export interface LLMProvider {
  /** 提供商类型 */
  type: ProviderType;
  /** 是否启用 */
  enabled: boolean;
  /** API Key */
  apiKey?: string;
  /** API 基础 URL（用于兼容 OpenAI 接口的服务或 Ollama） */
  baseUrl?: string;
  /** 模型名称 */
  model: string;
  /** 请求超时时间（毫秒） */
  timeout?: number;
  /** 最大重试次数 */
  maxRetries?: number;
  /** 额外配置 */
  extra?: Record<string, unknown>;
}

/** LLM 任务分配配置 */
export interface LLMTaskAssignment {
  /** 任务类型标识 */
  taskType: string;
  /** 使用的提供商类型 */
  providerType: ProviderType;
  /** 任务描述 */
  description?: string;
  /** 系统提示词 */
  systemPrompt?: string;
  /** 是否启用 */
  enabled: boolean;
}

/** LLM 配置 */
export interface LLMConfig {
  /** 提供商配置映射 */
  providers: Record<ProviderType, LLMProvider>;
  /** 任务分配规则 */
  taskAssignments: LLMTaskAssignment[];
  /** 默认提供商 */
  defaultProvider?: ProviderType;
  /** 全局最大重试次数 */
  maxRetries?: number;
  /** 请求间隔（毫秒），用于限流 */
  requestInterval?: number;
}

/** 聊天消息 */
export interface Message {
  /** 消息角色 */
  role: MessageRole;
  /** 消息内容 */
  content: string;
}

/** LLM 调用响应 */
export interface LLMResponse {
  /** 响应内容 */
  content: string;
  /** 使用的模型 */
  model: string;
  /** 使用的提供商 */
  provider: ProviderType;
  /** 消耗 token（可选） */
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}

/** LLM 调用错误 */
export interface LLMError {
  /** 错误码 */
  code: string;
  /** 错误信息 */
  message: string;
  /** 提供商类型 */
  provider?: ProviderType;
  /** 原始错误（可选） */
  originalError?: unknown;
}
