# Commit2Zen 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个 Node.js 命令行工具,收集 git 提交记录,按分支分组汇总,并通过禅道 REST API 自动创建任务。

**Architecture:** 单文件 ES Module 脚本 (`commit2zen.mjs`),包含配置加载、Git 日志收集、分支分组、禅道 API 客户端和报告生成五个模块,零外部依赖。

**Tech Stack:** Node.js (ES Modules), fetch API, child_process, fs/promises, path

---

## 文件结构

```
Commit2Zen/
├── commit2zen.mjs              # 主脚本 (所有逻辑)
├── config.example.json         # 配置示例
├── .gitignore                  # Git 忽略规则
├── docs/
│   └── superpowers/
│       └── plans/
│           └── 2026-04-05-commit2zen-plan.md  # 本文件
└── reports/                    # 执行报告目录 (自动生成)
```

---

### Task 1: 配置加载和验证模块

**Files:**
- Create: `commit2zen.mjs` (初始骨架 + 配置模块)
- Create: `config.example.json`

- [ ] **Step 1: 创建配置示例文件**

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

- [ ] **Step 2: 创建主脚本骨架和配置加载函数**

```javascript
#!/usr/bin/env node
import { readFileSync, existsSync, mkdirSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// ============================================================
// 配置模块
// ============================================================

const DEFAULT_CONFIG = {
  zentao: {
    taskType: 'dev'
  },
  git: {
    maxCommits: 100,
    includeMerged: false,
    branchPattern: '.*'
  },
  output: {
    reportDir: 'reports',
    verbose: true
  }
};

function loadConfig(configPath = join(__dirname, 'config.json')) {
  if (!existsSync(configPath)) {
    console.error('❌ 配置文件不存在:', configPath);
    console.error('请复制 config.example.json 为 config.json 并填入你的配置');
    process.exit(1);
  }

  try {
    const raw = readFileSync(configPath, 'utf-8');
    const userConfig = JSON.parse(raw);
    return {
      zentao: { ...DEFAULT_CONFIG.zentao, ...userConfig.zentao },
      git: { ...DEFAULT_CONFIG.git, ...userConfig.git },
      output: { ...DEFAULT_CONFIG.output, ...userConfig.output }
    };
  } catch (error) {
    if (error instanceof SyntaxError) {
      console.error('❌ 配置文件 JSON 格式错误:', error.message);
    } else {
      console.error('❌ 读取配置文件失败:', error.message);
    }
    process.exit(1);
  }
}

function validateConfig(config) {
  const required = ['url', 'account', 'password', 'projectId', 'assignedTo'];
  const missing = required.filter(key => !config.zentao[key]);

  if (missing.length > 0) {
    console.error('❌ 配置缺少必填字段:', missing.join(', '));
    process.exit(1);
  }

  if (!config.zentao.url.startsWith('http://') && !config.zentao.url.startsWith('https://')) {
    console.error('❌ 禅道 URL 必须以 http:// 或 https:// 开头');
    process.exit(1);
  }

  return config;
}

// ============================================================
// 主入口
// ============================================================

async function main() {
  console.log('🚀 Commit2Zen - Git 提交汇总到禅道\n');

  const config = loadConfig();
  validateConfig(config);

  console.log('✅ 配置加载成功');
  console.log('   禅道:', config.zentao.url);
  console.log('   项目 ID:', config.zentao.projectId);
  console.log('');
}

main().catch(error => {
  console.error('❌ 未捕获的错误:', error.message);
  process.exit(1);
});
```

- [ ] **Step 3: 创建 .gitignore 文件**

```gitignore
config.json
reports/
node_modules/
*.log
```

- [ ] **Step 4: 测试配置加载**

复制配置示例并运行:

```bash
cp config.example.json config.json
node commit2zen.mjs
```

期望输出:
```
🚀 Commit2Zen - Git 提交汇总到禅道

✅ 配置加载成功
   禅道: http://your-zentao.com
   项目 ID: 1
```

- [ ] **Step 5: 测试配置错误处理**

```bash
# 测试缺失配置
mv config.json config.json.bak
node commit2zen.mjs
# 应显示配置文件不存在错误

# 恢复配置
mv config.json.bak config.json
```

- [ ] **Step 6: 提交**

```bash
git add commit2zen.mjs config.example.json .gitignore
git commit -m "feat: 添加配置加载和验证模块"
```

---

### Task 2: Git 日志收集模块

**Files:**
- Modify: `commit2zen.mjs` (添加 gitLog 模块)

- [ ] **Step 1: 添加 Git 日志收集函数**

在 `commit2zen.mjs` 中,在配置模块后添加:

```javascript
// ============================================================
// Git 日志收集模块
// ============================================================

function isGitRepo() {
  try {
    execSync('git rev-parse --git-dir', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function collectGitLog(maxCommits = 100) {
  if (!isGitRepo()) {
    console.error('❌ 当前目录不是 Git 仓库');
    process.exit(1);
  }

  try {
    const log = execSync(
      `git log -n ${maxCommits} --all --format="%H|%an|%ad|%s|%D" --date=iso`,
      { encoding: 'utf-8' }
    );

    const lines = log.split('\n').filter(line => line.trim());

    if (lines.length === 0) {
      console.warn('⚠️  没有找到提交记录');
      return [];
    }

    return lines.map(line => {
      const parts = line.split('|');
      const [hash, author, date, message, refs] = parts;

      // 解析分支信息
      const branches = refs
        ? refs.match(/-> ([^\s,]+)/g)?.map(r => r.replace('-> ', '')) || []
        : [];

      // 获取修改的文件列表
      let files = [];
      try {
        const filesOutput = execSync(
          `git diff-tree --no-commit-id --name-only -r ${hash}`,
          { encoding: 'utf-8' }
        );
        files = filesOutput.split('\n').filter(f => f.trim());
      } catch {
        // 忽略错误,可能是根提交
      }

      return {
        hash: hash.substring(0, 7),
        author,
        date: date.trim(),
        message,
        branches,
        files
      };
    });
  } catch (error) {
    console.error('❌ 获取 Git 日志失败:', error.message);
    process.exit(1);
  }
}
```

- [ ] **Step 2: 更新 main 函数测试 Git 日志收集**

将 `main()` 函数更新为:

```javascript
async function main() {
  console.log('🚀 Commit2Zen - Git 提交汇总到禅道\n');

  const config = loadConfig();
  validateConfig(config);

  console.log('✅ 配置加载成功');
  console.log('   禅道:', config.zentao.url);
  console.log('   项目 ID:', config.zentao.projectId);
  console.log('');

  console.log('📦 收集 Git 提交记录...');
  const commits = collectGitLog(config.git.maxCommits);

  if (commits.length === 0) {
    console.log('ℹ️  没有需要处理的提交');
    return;
  }

  console.log(`✅ 找到 ${commits.length} 个提交`);
  commits.slice(0, 3).forEach(commit => {
    console.log(`   - ${commit.hash} ${commit.message}`);
  });
  if (commits.length > 3) {
    console.log(`   ... 还有 ${commits.length - 3} 个提交`);
  }
  console.log('');
}
```

- [ ] **Step 3: 初始化 Git 仓库并测试**

```bash
git init
git add .
git commit -m "initial commit"
node commit2zen.mjs
```

期望输出:
```
🚀 Commit2Zen - Git 提交汇总到禅道

✅ 配置加载成功
   禅道: http://your-zentao.com
   项目 ID: 1

📦 收集 Git 提交记录...
✅ 找到 1 个提交
   - abc1234 initial commit
```

- [ ] **Step 4: 提交**

```bash
git add commit2zen.mjs
git commit -m "feat: 添加 Git 日志收集模块"
```

---

### Task 3: 分支分组模块

**Files:**
- Modify: `commit2zen.mjs` (添加 branchGrouper 模块)

- [ ] **Step 1: 添加分支分组函数**

在 Git 日志模块后添加:

```javascript
// ============================================================
// 分支分组模块
// ============================================================

function groupCommitsByBranch(commits) {
  const branchMap = new Map();

  for (const commit of commits) {
    // 优先使用显式分支信息,否则标记为未分类
    const branchName = commit.branches.length > 0
      ? commit.branches[0]
      : '未分类';

    if (!branchMap.has(branchName)) {
      branchMap.set(branchName, []);
    }
    branchMap.get(branchName).push(commit);
  }

  // 转换为分组结果
  return Array.from(branchMap.entries()).map(([branch, branchCommits]) => {
    const authors = [...new Set(branchCommits.map(c => c.author))];
    const dates = branchCommits.map(c => new Date(c.date));
    const startDate = new Date(Math.min(...dates));
    const endDate = new Date(Math.max(...dates));

    return {
      branch: branch,
      commitCount: branchCommits.length,
      authors: authors,
      dateRange: {
        start: startDate.toISOString().split('T')[0],
        end: endDate.toISOString().split('T')[0]
      },
      commits: branchCommits,
      summary: generateBranchSummary(branchCommits)
    };
  });
}

function generateBranchSummary(commits) {
  const totalFiles = [...new Set(commits.flatMap(c => c.files))];

  return {
    totalCommits: commits.length,
    totalAuthors: new Set(commits.map(c => c.author)).size,
    totalFiles: totalFiles.length,
    commitTypes: classifyCommits(commits),
    topFiles: totalFiles.slice(0, 10)
  };
}

function classifyCommits(commits) {
  const types = {
    feat: 0,
    fix: 0,
    docs: 0,
    refactor: 0,
    test: 0,
    chore: 0,
    other: 0
  };

  for (const commit of commits) {
    const match = commit.message.match(/^(feat|fix|docs|refactor|test|chore)/);
    if (match) {
      types[match[1]]++;
    } else {
      types.other++;
    }
  }

  return types;
}
```

- [ ] **Step 2: 更新 main 函数测试分支分组**

将 main 函数中收集日志后的部分更新为:

```javascript
  console.log('📦 收集 Git 提交记录...');
  const commits = collectGitLog(config.git.maxCommits);

  if (commits.length === 0) {
    console.log('ℹ️  没有需要处理的提交');
    return;
  }

  console.log(`✅ 找到 ${commits.length} 个提交`);
  console.log('');

  console.log('📂 按分支分组...');
  const branches = groupCommitsByBranch(commits);

  console.log(`✅ 分为 ${branches.length} 个分支:`);
  for (const branch of branches) {
    console.log(`   🌿 ${branch.branch} (${branch.commitCount} 个提交, ${branch.authors.join(', ')})`);
  }
  console.log('');
```

- [ ] **Step 3: 提交**

```bash
git add commit2zen.mjs
git commit -m "feat: 添加分支分组模块"
```

---

### Task 4: 禅道 API 客户端模块

**Files:**
- Modify: `commit2zen.mjs` (添加 zentaoClient 模块)

- [ ] **Step 1: 添加禅道 API 客户端函数**

在分支分组模块后添加:

```javascript
// ============================================================
// 禅道 API 客户端模块
// ============================================================

async function zentaoLogin(url, account, password) {
  const loginUrl = `${url}/api.php/v1/users/login`;

  try {
    const response = await fetch(loginUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ account, password })
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    if (!data.token) {
      throw new Error(data.message || '登录响应中未找到 token');
    }

    return data.token;
  } catch (error) {
    console.error('❌ 禅道登录失败:', error.message);
    process.exit(1);
  }
}

async function zentaoCreateTask(url, token, taskData, retries = 3) {
  const taskUrl = `${url}/api.php/v1/tasks`;

  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 30000);

      const response = await fetch(taskUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(taskData),
        signal: controller.signal
      });

      clearTimeout(timeout);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      return result;
    } catch (error) {
      if (attempt < retries && (error.name === 'AbortError' || error.message.includes('fetch'))) {
        const delay = Math.pow(2, attempt - 1) * 1000;
        console.warn(`   ⚠️  请求失败,${delay / 1000}s 后重试 (${attempt}/${retries})...`);
        await new Promise(resolve => setTimeout(resolve, delay));
      } else {
        throw error;
      }
    }
  }
}

function buildTaskDescription(branchData) {
  const lines = [];

  lines.push(`## 分支: ${branchData.branch}`);
  lines.push('');
  lines.push(`### 提交记录 (${branchData.commitCount})`);
  lines.push('');

  for (let i = 0; i < branchData.commits.length; i++) {
    const commit = branchData.commits[i];
    lines.push(`${i + 1}. **${commit.hash}** - ${commit.message}`);
    lines.push(`   - 作者: ${commit.author}`);
    lines.push(`   - 日期: ${commit.date.split('+')[0].trim()}`);
    if (commit.files.length > 0) {
      lines.push(`   - 文件: ${commit.files.join(', ')}`);
    }
    lines.push('');
  }

  lines.push('### 统计');
  lines.push(`- 提交数: ${branchData.commitCount}`);
  lines.push(`- 作者: ${branchData.authors.join(', ')}`);
  lines.push(`- 时间范围: ${branchData.dateRange.start} ~ ${branchData.dateRange.end}`);
  lines.push(`- 修改文件数: ${branchData.summary.totalFiles}`);
  lines.push('');

  return lines.join('\n');
}

function buildTaskPayload(branchData, config) {
  return {
    name: `[${branchData.branch}] 工作内容汇总`,
    desc: buildTaskDescription(branchData),
    project: config.zentao.projectId,
    assignedTo: config.zentao.assignedTo,
    type: config.zentao.taskType,
    estStarted: branchData.dateRange.start,
    deadline: branchData.dateRange.end
  };
}
```

- [ ] **Step 2: 提交**

```bash
git add commit2zen.mjs
git commit -m "feat: 添加禅道 API 客户端模块"
```

---

### Task 5: 报告生成模块和主流程整合

**Files:**
- Modify: `commit2zen.mjs` (添加 reportGenerator 模块和完整主流程)

- [ ] **Step 1: 添加报告生成函数**

在禅道 API 模块后添加:

```javascript
// ============================================================
// 报告生成模块
// ============================================================

function generateReport(branches, results, config) {
  const timestamp = new Date().toISOString();
  const totalCommits = branches.reduce((sum, b) => sum + b.commitCount, 0);
  const tasksCreated = results.filter(r => r.success).length;
  const tasksFailed = results.filter(r => !r.success).length;

  const report = {
    timestamp,
    project: 'Commit2Zen',
    branches: branches.map((branch, i) => ({
      branch: branch.branch,
      commitCount: branch.commitCount,
      taskCreated: results[i].success,
      taskId: results[i].taskId || null,
      taskUrl: results[i].taskUrl || null,
      error: results[i].error || null
    })),
    summary: {
      totalBranches: branches.length,
      totalCommits,
      tasksCreated,
      tasksFailed
    }
  };

  return report;
}

function saveReport(report, config) {
  const reportDir = join(__dirname, config.output.reportDir);

  if (!existsSync(reportDir)) {
    mkdirSync(reportDir, { recursive: true });
  }

  const dateStr = new Date().toISOString().split('T')[0];
  const reportPath = join(reportDir, `${dateStr}-report.json`);

  try {
    writeFileSync(reportPath, JSON.stringify(report, null, 2), 'utf-8');
    console.log('📄 报告已保存:', reportPath);
  } catch (error) {
    console.error('⚠️  保存报告文件失败:', error.message);
  }
}

function printSummary(report) {
  console.log('\n' + '='.repeat(60));
  console.log('📊 执行摘要');
  console.log('='.repeat(60));
  console.log(`分支数: ${report.summary.totalBranches}`);
  console.log(`提交数: ${report.summary.totalCommits}`);
  console.log(`创建任务: ${report.summary.tasksCreated}`);
  console.log(`失败任务: ${report.summary.tasksFailed}`);
  console.log('='.repeat(60));

  for (const branch of report.branches) {
    const icon = branch.taskCreated ? '✅' : '❌';
    console.log(`${icon} ${branch.branch}: ${branch.commitCount} 个提交`);
    if (branch.taskId) {
      console.log(`   任务 ID: ${branch.taskId}`);
    }
    if (branch.error) {
      console.log(`   错误: ${branch.error}`);
    }
  }
  console.log('');
}
```

- [ ] **Step 2: 更新 main 函数完成主流程**

将 main 函数完整替换为:

```javascript
async function main() {
  console.log('🚀 Commit2Zen - Git 提交汇总到禅道\n');

  // 1. 加载配置
  const config = loadConfig();
  validateConfig(config);

  console.log('✅ 配置加载成功');
  console.log('   禅道:', config.zentao.url);
  console.log('   项目 ID:', config.zentao.projectId);
  console.log('');

  // 2. 收集 Git 日志
  console.log('📦 收集 Git 提交记录...');
  const commits = collectGitLog(config.git.maxCommits);

  if (commits.length === 0) {
    console.log('ℹ️  没有需要处理的提交');
    return;
  }

  console.log(`✅ 找到 ${commits.length} 个提交`);
  console.log('');

  // 3. 按分支分组
  console.log('📂 按分支分组...');
  const branches = groupCommitsByBranch(commits);

  console.log(`✅ 分为 ${branches.length} 个分支:`);
  for (const branch of branches) {
    console.log(`   🌿 ${branch.branch} (${branch.commitCount} 个提交, ${branch.authors.join(', ')})`);
  }
  console.log('');

  // 4. 登录禅道
  console.log('🔐 登录禅道...');
  const token = await zentaoLogin(
    config.zentao.url,
    config.zentao.account,
    config.zentao.password
  );
  console.log('✅ 登录成功\n');

  // 5. 为每个分支创建任务
  console.log('📝 创建禅道任务...');
  const results = [];

  for (const branch of branches) {
    console.log(`   🌿 处理分支: ${branch.branch}...`);

    try {
      const payload = buildTaskPayload(branch, config);
      const result = await zentaoCreateTask(
        config.zentao.url,
        token,
        payload
      );

      results.push({
        success: true,
        taskId: result.id,
        taskUrl: `${config.zentao.url}/task-view-${result.id}.html`
      });

      console.log(`   ✅ 任务创建成功 (ID: ${result.id})`);
    } catch (error) {
      results.push({
        success: false,
        error: error.message
      });
      console.error(`   ❌ 任务创建失败: ${error.message}`);
    }
  }

  console.log('');

  // 6. 生成报告
  const report = generateReport(branches, results, config);
  saveReport(report, config);
  printSummary(report);
}
```

- [ ] **Step 3: 测试完整流程 (需要真实禅道实例)**

注意: 此步骤需要配置真实的禅道实例。如果没有禅道,可以跳过实际运行,仅验证代码语法:

```bash
node --check commit2zen.mjs
```

应无输出 (表示语法正确)。

- [ ] **Step 4: 提交**

```bash
git add commit2zen.mjs
git commit -m "feat: 添加报告生成模块和完整主流程"
```

---

### Task 6: 创建 README 和最终验证

**Files:**
- Create: `README.md`

- [ ] **Step 1: 创建 README.md**

```markdown
# Commit2Zen

自动收集 Git 提交记录,按分支分组汇总,并在禅道上创建任务。

## 功能特性

- ✅ 自动收集 Git 提交记录
- ✅ 按分支分组汇总工作内容
- ✅ 通过禅道 REST API 创建任务
- ✅ 生成执行报告 (JSON)
- ✅ 零外部依赖,开箱即用

## 前置要求

- Node.js 18+ (内置 fetch API)
- 可访问的禅道实例

## 安装和使用

### 1. 配置

复制配置示例并填入你的禅道信息:

```bash
cp config.example.json config.json
```

编辑 `config.json`:

```json
{
  "zentao": {
    "url": "http://your-zentao.com",
    "account": "your-account",
    "password": "your-password",
    "projectId": 1,
    "assignedTo": "your-account",
    "taskType": "dev"
  }
}
```

### 2. 运行

```bash
node commit2zen.mjs
```

### 3. 查看结果

- 控制台输出执行摘要
- 详细报告保存在 `reports/YYYY-MM-DD-report.json`
- 登录禅道查看创建的任务

## 配置选项

| 字段 | 必填 | 说明 | 默认值 |
|------|------|------|--------|
| zentao.url | ✅ | 禅道地址 | - |
| zentao.account | ✅ | 禅道账号 | - |
| zentao.password | ✅ | 禅道密码 | - |
| zentao.projectId | ✅ | 项目 ID | - |
| zentao.assignedTo | ✅ | 任务指派人 | - |
| zentao.taskType | ❌ | 任务类型 | dev |
| git.maxCommits | ❌ | 最大提交数 | 100 |
| git.includeMerged | ❌ | 包含合并提交 | false |
| output.reportDir | ❌ | 报告目录 | reports |
| output.verbose | ❌ | 详细输出 | true |

## 错误处理

- 网络请求失败会自动重试 (最多 3 次,指数退避)
- 单个分支任务创建失败不影响其他分支
- 所有错误记录在报告文件中

## 安全注意

- `config.json` 包含敏感信息,已加入 `.gitignore`
- 不要在版本控制中提交配置文件
- 建议限制 `config.json` 文件权限: `chmod 600 config.json`

## 许可证

MIT
```

- [ ] **Step 2: 最终代码检查**

```bash
node --check commit2zen.mjs
```

- [ ] **Step 3: 提交所有剩余文件**

```bash
git add README.md .gitignore
git commit -m "docs: 添加 README 和使用说明"
```

- [ ] **Step 4: 验证 Git 历史**

```bash
git log --oneline
```

应看到 6 个提交,每个对应一个 Task。

---

## 自审检查

### 1. 规格覆盖检查

| 规格要求 | 对应 Task | 状态 |
|---------|----------|------|
| Git 日志收集 | Task 2 | ✅ |
| 分支分组汇总 | Task 3 | ✅ |
| 禅道 API 集成 | Task 4 | ✅ |
| 配置管理 | Task 1 | ✅ |
| 执行报告生成 | Task 5 | ✅ |
| 错误处理和重试 | Task 4 | ✅ |
| 零外部依赖 | 全部 | ✅ |
| 安全考虑 (gitignore) | Task 1 | ✅ |

### 2. 占位符扫描

- 无 "TBD", "TODO", "implement later"
- 所有步骤包含完整代码
- 无 "Add appropriate error handling" 等模糊描述
- 无 "Similar to Task N" 引用
- 所有函数签名在整个计划中保持一致

### 3. 类型一致性检查

- `collectGitLog()` 返回 `Array<{hash, author, date, message, branches, files}>` - 所有引用一致
- `groupCommitsByBranch()` 返回 `Array<{branch, commitCount, authors, dateRange, commits, summary}>` - 所有引用一致
- `zentaoLogin()` 返回 `string` (token) - 一致
- `buildTaskPayload()` 使用 `config.zentao.*` 字段 - 与配置模块一致
- 报告结构 `generateReport()` 参数与 Task 5 使用处一致

所有检查通过,计划完整一致。
