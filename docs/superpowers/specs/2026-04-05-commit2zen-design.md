# Commit2Zen 设计文档

## 概述

Commit2Zen 是一个自动化工具,用于收集当前项目的 git 提交记录,按功能/分支分组汇总,并自动在禅道上创建任务记录工作内容。

### 目标

- 自动收集 git 提交记录
- 按分支分组汇总工作内容
- 通过禅道 REST API 创建任务
- 生成执行报告

### 非目标

- 多项目管理(仅当前项目)
- 定时自动执行(手动触发)
- 图形界面(命令行工具)

## 架构设计

### 技术选型

- **运行时**: Node.js (ES Modules)
- **HTTP 客户端**: 内置 `fetch`
- **进程执行**: 内置 `child_process`
- **文件系统**: 内置 `fs/promises`
- **配置格式**: JSON

### 组件图

```
┌─────────────────────────────────────────────┐
│              commit2zen.mjs                 │
├─────────────────────────────────────────────┤
│  ┌───────────┐    ┌───────────┐             │
│  │ 配置加载器 │───▶│ Git收集器 │             │
│  └───────────┘    └─────┬─────┘             │
│                         │                   │
│                         ▼                   │
│                   ┌───────────┐             │
│                   │ 分支分组器 │             │
│                   └─────┬─────┘             │
│                         │                   │
│                         ▼                   │
│                   ┌───────────┐             │
│                   │ 禅道客户端 │────────────┤
│                   └─────┬─────┘             │
│                         │                   │
│                         ▼                   │
│                   ┌───────────┐             │
│                   │ 报告生成器 │             │
│                   └───────────┘             │
└─────────────────────────────────────────────┘
```

## 详细设计

### 1. Git 日志收集器

**功能**: 获取并解析 git 提交记录

**实现**:
```javascript
const { execSync } = require('child_process');

function collectGitLog() {
  const log = execSync(
    'git log --all --format="%H|%an|%ad|%s" --date=iso',
    { encoding: 'utf-8' }
  );
  
  return log.split('\n')
    .filter(line => line.trim())
    .map(line => {
      const [hash, author, date, ...messageParts] = line.split('|');
      return { hash, author, date, message: messageParts.join('|') };
    });
}
```

**输出格式**:
```javascript
{
  branch: 'feature/login',
  commits: [
    {
      hash: 'abc123',
      author: '张三',
      date: '2026-04-05 10:30:00',
      message: 'feat: 实现用户登录功能',
      files: ['src/auth.js', 'src/login.html']
    }
  ]
}
```

**错误处理**:
- 非 git 仓库: 抛出友好错误提示
- 无提交记录: 返回空数组并警告用户

### 2. 分支分组引擎

**功能**: 按分支名称分组提交记录

**实现**:
```javascript
function groupByBranch(commits) {
  const groups = new Map();
  
  for (const commit of commits) {
    const branch = commit.branch || '未分类';
    if (!groups.has(branch)) {
      groups.set(branch, []);
    }
    groups.get(branch).push(commit);
  }
  
  return Array.from(groups.entries()).map(([branch, commits]) => ({
    branch,
    commitCount: commits.length,
    authors: [...new Set(commits.map(c => c.author))],
    dateRange: {
      start: commits[commits.length - 1].date,
      end: commits[0].date
    },
    commits,
    summary: generateSummary(commits)
  }));
}
```

### 3. 禅道 API 客户端

**功能**: 与禅道 REST API 交互,创建任务

**禅道 API 端点**:
- 认证: `POST /api.php/v1/users/login`
- 创建任务: `POST /api.php/v1/tasks`
- 获取项目: `GET /api.php/v1/projects`

**认证流程**:
```javascript
async function authenticate(url, account, password) {
  const res = await fetch(`${url}/api.php/v1/users/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ account, password })
  });
  
  const data = await res.json();
  return data.token; // access token
}
```

**创建任务**:
```javascript
async function createTask(url, token, taskData) {
  const res = await fetch(`${url}/api.php/v1/tasks`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify(taskData)
  });
  
  return res.json();
}
```

**任务数据格式**:
```javascript
{
  name: '[feature/login] 工作内容汇总',
  desc: `## 分支: feature/login

### 提交记录 (3)

1. **abc123** - feat: 实现用户登录功能
   - 作者: 张三
   - 日期: 2026-04-05 10:30:00
   - 文件: src/auth.js, src/login.html

2. **def456** - fix: 修复登录表单验证
   ...

### 统计
- 提交数: 3
- 作者: 张三
- 时间范围: 2026-04-05 ~ 2026-04-06
`,
  project: 1,
  assignedTo: 'zhangsan',
  type: 'dev',
  estStarted: '2026-04-05',
  deadline: '2026-04-06'
}
```

**错误处理**:
- 网络不可达: 重试 3 次,指数退避
- 认证失败: 立即停止并提示
- API 返回错误: 记录错误但继续处理其他分支
- 超时: 30 秒超时,抛出错误

### 4. 配置管理

**配置文件结构** (`config.json`):
```json
{
  "zentao": {
    "url": "http://your-zentao.com",
    "account": "your-account",
    "password": "your-password",
    "projectId": 1,
    "assignedTo": "your-account",
    "taskType": "dev"
  },
  "git": {
    "maxCommits": 100,
    "includeMerged": false,
    "branchPattern": ".*"
  },
  "output": {
    "reportDir": "reports",
    "verbose": true
  }
}
```

**配置验证**:
- 必填字段检查: url, account, password, projectId, assignedTo
- URL 格式验证
- 提供配置示例文件 `config.example.json`

### 5. 执行报告

**控制台输出**:
- 使用颜色区分信息类型 (成功/警告/错误)
- 实时显示进度
- 汇总统计信息

**JSON 报告**:
```json
{
  "timestamp": "2026-04-05T10:30:00.000Z",
  "project": "Commit2Zen",
  "branches": [
    {
      "branch": "feature/login",
      "commitCount": 3,
      "taskCreated": true,
      "taskId": 123,
      "taskUrl": "http://zentao.com/task-123.html"
    }
  ],
  "summary": {
    "totalBranches": 2,
    "totalCommits": 5,
    "tasksCreated": 2,
    "tasksFailed": 0
  }
}
```

## 错误处理策略

### 错误分类

1. **致命错误** (立即停止):
   - 非 git 仓库
   - 配置文件缺失或无效
   - 禅道认证失败

2. **可恢复错误** (记录并继续):
   - 单个分支任务创建失败
   - 网络超时(重试后)
   - API 返回部分错误

3. **警告** (提示但不影响):
   - 无提交记录
   - 配置使用默认值
   - 报告文件写入失败

### 重试机制

- 网络请求失败: 最多重试 3 次
- 退避策略: 1s, 2s, 4s
- 记录每次重试日志

## 文件结构

```
Commit2Zen/
├── commit2zen.mjs              # 主脚本 (入口文件)
├── config.json                 # 配置文件 (用户填写)
├── config.example.json         # 配置示例
├── README.md                   # 使用说明
├── docs/
│   └── superpowers/
│       └── specs/
│           └── 2026-04-05-commit2zen-design.md
└── reports/                    # 执行报告 (自动生成)
    └── YYYY-MM-DD-report.json
```

## 使用流程

1. **初始化配置**
   ```bash
   cp config.example.json config.json
   # 编辑 config.json 填入禅道信息
   ```

2. **运行工具**
   ```bash
   node commit2zen.mjs
   ```

3. **查看结果**
   - 控制台输出执行摘要
   - 查看 `reports/` 目录下的详细报告
   - 登录禅道查看创建的任务

## 成功标准

- [ ] 能正确收集当前项目的 git 提交记录
- [ ] 能按分支分组汇总提交信息
- [ ] 能通过禅道 API 成功创建任务
- [ ] 任务描述包含完整的提交信息
- [ ] 生成清晰的执行报告
- [ ] 错误处理完善,不会因单个失败而中断
- [ ] 零外部依赖,开箱即用

## 安全考虑

- **密码存储**: 配置文件中的密码应限制文件权限 (600)
- **敏感信息**: 不在日志中输出密码和 token
- **Git 敏感数据**: 不提交 `config.json` 到版本控制

## 未来扩展

可能但不在此次实现中:

- 多项目支持
- 定时自动执行 (cron)
- Web UI 界面
- 导出为其他格式 (Excel, PDF)
- 与 git hook 集成
- 支持其他项目管理工具 (Jira, GitLab)
