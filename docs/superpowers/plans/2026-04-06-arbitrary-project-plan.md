# 任意项目支持 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 扩展 Commit2Zen 支持通过配置文件或命令行参数指定任意项目路径进行 Git 提交统计。

**Architecture:** 新增项目上下文管理模块 (路径解析 + 验证),修改配置模块和 main 函数,保持向后兼容。

**Tech Stack:** Node.js (ES Modules), fs, path, process.chdir

---

### Task 1: 扩展配置模块和添加项目上下文管理

**Files:**
- Modify: `commit2zen.mjs:14-27` (DEFAULT_CONFIG)
- Modify: `commit2zen.mjs:71-82` (添加项目上下文管理模块)
- Modify: `config.example.json` (添加 projectPath 字段)

- [ ] **Step 1: 更新 DEFAULT_CONFIG 添加 projectPath 字段**

```javascript
const DEFAULT_CONFIG = {
  projectPath: null, // 新增: null 表示使用当前目录
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
```

- [ ] **Step 2: 添加项目上下文管理模块**

在 Git 日志收集模块前 (line 71 之前) 添加:

```javascript
// ============================================================
// 项目上下文管理模块
// ============================================================

function resolveProjectPath(configPath) {
  // 1. 命令行参数优先
  const cliPath = process.argv[2];
  if (cliPath) {
    return path.resolve(cliPath);
  }
  
  // 2. 配置文件次之
  if (configPath) {
    return path.resolve(configPath);
  }
  
  // 3. 默认当前目录
  return process.cwd();
}

function validateProjectPath(projectPath) {
  if (!existsSync(projectPath)) {
    console.error(`❌ 项目路径不存在: ${projectPath}`);
    process.exit(1);
  }
  
  const originalCwd = process.cwd();
  
  // 切换到项目目录
  process.chdir(projectPath);
  
  // 验证是否为 Git 仓库
  if (!isGitRepo()) {
    console.error(`❌ 不是 Git 仓库: ${projectPath}`);
    process.chdir(originalCwd);
    process.exit(1);
  }
  
  return originalCwd;
}
```

- [ ] **Step 3: 更新导入语句添加 path.resolve**

修改 line 3:

```javascript
import { join, dirname, resolve } from 'path';
```

- [ ] **Step 4: 更新 config.example.json**

```json
{
  "projectPath": "/path/to/your/project",
  "zentao": {
    "url": "http://192.168.1.23/zentao",
    "account": "lifc",
    "password": "lifc",
    "projectId": 165,
    "assignedTo": "lifc",
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

- [ ] **Step 5: 测试配置加载和路径解析**

```bash
# 测试默认行为 (当前目录)
node commit2zen.mjs

# 测试命令行参数
node commit2zen.mjs /tmp
```

期望: 不报错,正常显示配置加载信息。

- [ ] **Step 6: 提交**

```bash
git add commit2zen.mjs config.example.json
git commit -m "feat: 添加项目上下文管理模块和 projectPath 配置支持"
```

---

### Task 2: 修改 Git 日志收集模块和 Main 函数

**Files:**
- Modify: `commit2zen.mjs:84-88` (移除 isGitRepo 检查)
- Modify: `commit2zen.mjs:415-430` (更新 main 函数)

- [ ] **Step 1: 移除 collectGitLog 中的 isGitRepo 检查**

将 line 84-88 修改为:

```javascript
function collectGitLog(maxCommits = 100) {
  // isGitRepo 检查已在 validateProjectPath 中完成
  
  if (!Number.isInteger(maxCommits) || maxCommits < 1) {
    console.error('❌ maxCommits 必须是正整数');
    process.exit(1);
  }

  try {
```

- [ ] **Step 2: 更新 main 函数添加项目路径处理**

将 line 415-430 修改为:

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

  // 2. 解析并验证项目路径
  const projectPath = resolveProjectPath(config.projectPath);
  console.log('📁 项目路径:', projectPath);
  const originalCwd = validateProjectPath(projectPath);
  console.log('✅ 项目验证通过\n');

  // 3. 收集 Git 日志
  console.log('📦 收集 Git 提交记录...');
  const commits = collectGitLog(config.git.maxCommits);
  
  // 恢复原目录
  process.chdir(originalCwd);

  if (commits.length === 0) {
    console.log('ℹ️  没有需要处理的提交');
    return;
  }
```

- [ ] **Step 3: 测试三种场景**

```bash
# 测试 1: 当前目录 (向后兼容)
cd /path/to/some/project
node commit2zen.mjs
# 应正常收集当前项目的提交

# 测试 2: 命令行参数指定项目
node commit2zen.mjs /path/to/another/project
# 应显示:
# 📁 项目路径: /path/to/another/project
# ✅ 项目验证通过
# 并收集该项目的提交

# 测试 3: 路径不存在
node commit2zen.mjs /nonexistent/path
# 应显示: ❌ 项目路径不存在: /nonexistent/path

# 测试 4: 不是 Git 仓库
node commit2zen.mjs /tmp
# 应显示: ❌ 不是 Git 仓库: /tmp
```

- [ ] **Step 4: 提交**

```bash
git add commit2zen.mjs
git commit -m "feat: 修改 Git 日志收集和 main 函数支持任意项目路径"
```

---

### Task 3: 更新 README 文档

**Files:**
- Modify: `README.md` (添加任意项目支持的使用说明)

- [ ] **Step 1: 更新 README 添加使用说明**

在"安装和使用"部分添加:

```markdown
## 指定项目路径

默认情况下,Commit2Zen 统计当前目录的 Git 提交。你可以通过以下方式指定任意项目:

### 方式 1: 配置文件

编辑 `config.json`:

```json
{
  "projectPath": "/path/to/your/project",
  "zentao": { ... }
}
```

### 方式 2: 命令行参数

```bash
node commit2zen.mjs /path/to/your/project
```

命令行参数优先级高于配置文件。

### 方式 3: 当前目录 (默认)

```bash
cd /path/to/your/project
node commit2zen.mjs
```
```

- [ ] **Step 2: 验证文档语法**

```bash
node --check commit2zen.mjs
```

- [ ] **Step 3: 提交**

```bash
git add README.md
git commit -m "docs: 更新 README 添加任意项目支持的使用说明"
```

---

## 自审检查

### 1. 规格覆盖检查

| 规格要求 | 对应 Task | 状态 |
|---------|----------|------|
| 配置文件 projectPath 字段 | Task 1 | ✅ |
| 命令行参数支持 | Task 1 | ✅ |
| 命令行优先级高于配置 | Task 1 (resolveProjectPath) | ✅ |
| 默认使用当前目录 | Task 1 | ✅ |
| 路径不存在错误提示 | Task 1 (validateProjectPath) | ✅ |
| 不是 Git 仓库错误提示 | Task 1 (validateProjectPath) | ✅ |
| 移除 Git 日志模块中的检查 | Task 2 | ✅ |
| Main 函数添加项目路径处理 | Task 2 | ✅ |
| 执行后恢复原目录 | Task 2 (process.chdir) | ✅ |
| README 文档更新 | Task 3 | ✅ |

### 2. 占位符扫描

- 无 "TBD", "TODO", "implement later"
- 所有步骤包含完整代码
- 无模糊描述
- 所有函数签名一致

### 3. 类型一致性检查

- `resolveProjectPath(configPath)` 接收 `string | null`, 返回 `string` - 一致
- `validateProjectPath(projectPath)` 接收 `string`, 返回 `string` (originalCwd) - 一致
- `process.chdir(originalCwd)` 恢复目录 - 一致
- `config.projectPath` 从 DEFAULT_CONFIG 初始化为 `null` - 一致

所有检查通过,计划完整一致。
