# Commit2Zen v2.1 优化设计文档

## 概述

基于 AI 小说生成器项目的最佳实践，对 Commit2Zen 进行全面优化，重点引入大模型集成、UI/UX 升级和架构改进。

---

## 1. 大模型集成架构

### 1.1 支持的 AI 服务商

**OpenAI 兼容 API** (统一接口，支持多家):
- OpenAI (GPT-4/4o/4o-mini)
- DeepSeek (Chat/R1)
- 智谱 GLM (GLM-4/4V/3-Turbo)
- 通义千问 (Qwen-Max/Plus/Turbo)
- 月之暗面 Kimi
- 字节豆包
- 讯飞星火
- 腾讯混元
- 百度文心一言

**专用 API**:
- Claude (Anthropic)
- Google Gemini
- Ollama (本地部署)

### 1.2 数据结构设计

```typescript
// src/types/llm.ts
interface LLMConfig {
  currentProvider: string                    // 当前使用的服务商
  providers: Record<string, LLMProvider>     // 所有服务商配置
  taskAssignment: {                          // 智能任务分配
    enabled: boolean
    tasks: {
      commitSummary: string      // 提交摘要生成 → GPT-4o
      taskDescription: string    // 禅道任务描述 → Claude
      branchSuggestion: string   // 分支命名建议 → Gemini
      trendAnalysis: string      // 趋势分析 → DeepSeek
    }
  }
}

interface LLMProvider {
  type: 'openai' | 'claude' | 'gemini' | 'ollama'
  name: string
  enabled: boolean
  apiKey: string
  baseUrl: string
  model: string
  temperature: number
  maxTokens: number
}
```

### 1.3 核心功能

**LLMManager 类** (`src/utils/llm.ts`):
```typescript
class LLMManager {
  // 配置管理
  getConfig()
  setCurrentProvider(key: string)
  updateProviderConfig(key: string, config: Partial<LLMProvider>)
  
  // 任务分配
  getProviderForTask(taskType: string): LLMProvider
  callLLMForTask(taskType: string, messages: Message[]): Promise<string>
  
  // LLM 调用
  callOpenAICompatible(provider: LLMProvider, messages: Message[]): Promise<string>
  callClaude(provider: LLMProvider, messages: Message[]): Promise<string>
  callGemini(provider: LLMProvider, messages: Message[]): Promise<string>
  callOllama(provider: LLMProvider, messages: Message[]): Promise<string>
}
```

### 1.4 使用场景

1. **智能提交摘要**: 分析 Git 提交历史，生成项目变更摘要
2. **禅道任务描述**: 自动生成结构化的任务描述（替代当前手动拼接）
3. **分支命名建议**: 基于提交内容智能推荐分支名
4. **趋势分析报告**: 分析项目开发趋势、活跃度统计

---

## 2. UI/UX 升级方案

### 2.1 引入 UnoCSS 原子化 CSS

**安装**: `@unocss/vite` + `unocss`

**配置** (`uno.config.ts`):
```typescript
import { defineConfig, presetUno, presetAttributify } from 'unocss'

export default defineConfig({
  presets: [presetUno(), presetAttributify()],
  shortcuts: {
    'flex-center': 'flex items-center justify-center',
    'flex-between': 'flex items-center justify-between',
    'card-base': 'rounded-lg bg-white p-4 shadow-sm dark:bg-gray-800',
  },
})
```

### 2.2 主题系统升级

**新增主题配置** (`src/stores/theme.js`):
```javascript
const themeStore = defineStore('theme', {
  state: () => ({
    isDark: false,
    primaryColor: '#1890ff',
    borderRadius: '8px',
  }),
  actions: {
    toggleTheme() { this.isDark = !this.isDark },
    setPrimaryColor(color) { this.primaryColor = color },
  },
  persist: true,  // pinia-plugin-persistedstate
})
```

### 2.3 新增仪表盘页面

**DashboardView.vue** - 统计概览:
- 总项目数、总提交数、总任务数
- 近 7 天活跃度图表 (ECharts)
- 分支分布饼图
- 任务创建成功率
- 最近执行记录卡片

### 2.4 交互优化

- 增加页面切换动画
- 添加操作引导提示
- 步骤指示器增加说明文字
- 错误状态增加重试按钮

---

## 3. 配置管理改进

### 3.1 统一命名规范

**后端自动转换** (`src-tauri/src/models/config.rs`):
```rust
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "zentaoAccounts")]
    pub zentao_accounts: Vec<ZentaoAccount>,
    
    pub git: GitConfig,
    pub output: OutputConfig,
    
    #[serde(rename = "llmConfig")]
    pub llm_config: Option<LLMConfig>,
}
```

### 3.2 前端配置持久化

**安装**: `pinia-plugin-persistedstate`

**使用**:
```javascript
export const useConfigStore = defineStore('config', {
  state: () => ({
    zentaoAccounts: [],
    git: { max_commits: 100, ... },
    llmConfig: null,
  }),
  persist: {
    key: 'commit2zen-config',
    storage: localStorage,
  },
})
```

### 3.3 配置导入导出

- 增加导出按钮 (导出 JSON 文件)
- 增加导入按钮 (导入 JSON 文件)
- 配置预设模板 (开发/测试/生产)

---

## 4. 错误处理优化

### 4.1 Rust 端错误类型

```rust
// src-tauri/src/errors.rs
#[derive(Debug, Serialize)]
pub enum AppError {
    #[error("仓库未找到: {0}")]
    RepositoryNotFound(String),
    
    #[error("日期筛选错误: {0}")]
    DateFilterError(String),
    
    #[error("禅道 API 错误: {0}")]
    ZentaoApiError(String),
    
    #[error("LLM 调用失败: {0}")]
    LlmError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
}
```

### 4.2 前端错误映射

```javascript
const ERROR_MESSAGES = {
  'RepositoryNotFound': '未找到 Git 仓库，请检查路径',
  'DateFilterError': '日期范围无效',
  'ZentaoApiError': '禅道连接失败，请检查配置',
  'LlmError': 'AI 服务调用失败，请检查配置或重试',
}
```

---

## 5. 性能优化

### 5.1 并行任务创建

```rust
// 修改 execute_full_workflow
use tokio::task::JoinSet;

let mut tasks = JoinSet::new();
for group in &branch_groups {
    let url = account.url.clone();
    let token = token.clone();
    let task_data = build_task_payload(...);
    tasks.spawn(async move {
        create_task(&url, &token, &task_data).await
    });
}

while let Some(result) = tasks.join_next().await {
    // 处理结果
}
```

### 5.2 提交分页加载

```javascript
// 前端虚拟滚动
const commits = ref([])
const page = ref(1)
const pageSize = 50

const loadMoreCommits = async () => {
  const newCommits = await invoke('collect_git_log', {
    projectPath: path,
    maxCommits: pageSize,
    offset: (page.value - 1) * pageSize,
  })
  commits.value.push(...newCommits)
  page.value++
}
```

---

## 6. 项目结构更新

```
Commit2Zen/
├── src/
│   ├── types/                    # 新增 TypeScript 类型
│   │   └── llm.ts
│   ├── utils/                    # 新增工具函数
│   │   └── llm.ts               # LLMManager 类
│   ├── views/
│   │   ├── DashboardView.vue    # 新增仪表盘
│   │   └── LLMConfigView.vue    # 新增 LLM 配置页
│   └── stores/
│       └── theme.js             # 新增主题管理
│
├── src-tauri/
│   ├── src/
│   │   ├── errors.rs            # 新增错误类型
│   │   ├── services/
│   │   │   └── llm_service.rs   # 新增 LLM 服务
│   │   └── commands/
│   │       └── llm.rs           # 新增 LLM 命令
│   └── Cargo.toml               # 添加依赖
│
└── uno.config.ts                # 新增 UnoCSS 配置
```

---

## 7. 实施优先级

### Phase 1: 大模型集成 (本次)
- [ ] LLM 数据模型定义
- [ ] 后端 LLM 服务层
- [ ] 前端 LLMManager 类
- [ ] LLM 配置页面
- [ ] 集成到现有工作流 (任务描述生成)

### Phase 2: UI/UX 升级
- [ ] 安装 UnoCSS
- [ ] 主题系统优化
- [ ] 仪表盘页面
- [ ] 交互优化

### Phase 3: 配置和错误处理
- [ ] 命名统一
- [ ] 配置持久化
- [ ] 错误类型化
- [ ] 导入导出功能

### Phase 4: 性能优化
- [ ] 并行任务创建
- [ ] 提交分页
- [ ] 扫描优化

---

## 8. 技术栈更新

**新增依赖**:
- 前端: `unocss`, `echarts`, `vue-echarts`, `pinia-plugin-persistedstate`
- 后端: 无新增 (使用现有 reqwest)

**保留**:
- Vue 3 + Pinia + Naive UI
- Tauri 2.x + Rust
- Vite 构建

**移除**: 无

---

## 9. 兼容性说明

- 向后兼容现有配置格式
- 新增 LLM 配置为可选
- 渐进式迁移，不影响现有功能
- 最低 Node.js 版本: 18+
- 最低 Rust 版本: 1.70+
