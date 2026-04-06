# Commit2Zen 任意项目支持设计文档

## 概述

将 Commit2Zen 从"仅统计当前项目"扩展为"支持任意指定项目",通过配置文件或命令行参数指定项目路径。

### 目标

- 支持通过配置文件 `projectPath` 字段指定项目路径
- 支持通过命令行参数传入项目路径
- 命令行参数优先于配置文件
- 未指定时默认使用当前目录(保持向后兼容)

### 非目标

- 多项目批量统计(未来可扩展)
- 监控/自动检测项目变更

## 架构设计

### 改动范围

| 模块 | 变更类型 | 说明 |
|------|---------|------|
| 配置加载模块 | 扩展 | 添加 `projectPath` 字段支持 |
| 项目上下文管理 | 新增 | 路径解析、验证、目录切换 |
| Git 日志收集模块 | 微调 | 移除当前目录检查(已在上下文模块处理) |
| Main 函数 | 修改 | 添加项目路径处理流程 |

### 新增函数

**1. `resolveProjectPath(configPath)`**

职责: 解析项目路径,按优先级返回

优先级:
1. 命令行参数 (`process.argv[2]`)
2. 配置文件 `config.projectPath`
3. 当前工作目录 (`process.cwd()`)

**2. `validateProjectPath(projectPath)`**

职责: 验证路径存在且是 Git 仓库,切换到该目录

行为:
- 检查路径是否存在
- 切换到该目录
- 验证是否为 Git 仓库
- 返回原始工作目录(用于后续恢复)

## 详细设计

### 1. 配置扩展

**默认配置更新:**

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

**配置示例文件 (`config.example.json`):**

```json
{
  "projectPath": "/path/to/your/project",
  "zentao": {
    "url": "http://192.168.1.23/zentao",
    "account": "your-account",
    "password": "your-password",
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

### 2. 项目上下文管理模块

**路径解析:**

```javascript
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
```

**路径验证:**

```javascript
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

### 3. Git 日志收集模块调整

**变更点:**

移除函数内的 `isGitRepo()` 检查,因为验证已在 `validateProjectPath()` 中完成。

```javascript
function collectGitLog(maxCommits = 100) {
  // 移除了 isGitRepo() 检查 (已在 validateProjectPath 中处理)
  
  if (!Number.isInteger(maxCommits) || maxCommits < 1) {
    console.error('❌ maxCommits 必须是正整数');
    process.exit(1);
  }

  try {
    const log = execSync(
      `git log -n ${maxCommits} --all --format="%H|%an|%ad|%s|%D" --date=iso`,
      { encoding: 'utf-8' }
    );
    // 其余逻辑不变...
  } catch (error) {
    console.error('❌ 获取 Git 日志失败:', error.message);
    process.exit(1);
  }
}
```

### 4. Main 函数流程

**新流程:**

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

  console.log(`✅ 找到 ${commits.length} 个提交`);
  console.log('');

  // 4. 按分支分组
  console.log('📂 按分支分组...');
  const branches = groupCommitsByBranch(commits);

  console.log(`✅ 分为 ${branches.length} 个分支:`);
  for (const branch of branches) {
    console.log(`   🌿 ${branch.branch} (${branch.commitCount} 个提交, ${branch.authors.join(', ')})`);
  }
  console.log('');

  // 5. 登录禅道
  console.log('🔐 登录禅道...');
  const token = await zentaoLogin(
    config.zentao.url,
    config.zentao.account,
    config.zentao.password
  );
  console.log('✅ 登录成功\n');

  // 6. 为每个分支创建任务
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

  // 7. 生成报告
  const report = generateReport(branches, results, config);
  saveReport(report, config);
  printSummary(report);
}
```

## 错误处理策略

| 场景 | 错误提示 | 行为 |
|------|---------|------|
| 路径不存在 | `❌ 项目路径不存在: /xxx/yyy` | 立即退出 (exit 1) |
| 不是 Git 仓库 | `❌ 不是 Git 仓库: /xxx/yyy` | 立即退出 (exit 1),恢复原目录 |
| 命令行参数无效 | `❌ 无效的项目路径: xxx` | 立即退出 (exit 1) |
| 配置中 projectPath 缺失 | 无错误 | 使用当前目录(默认行为) |
| Git 日志获取失败 | `❌ 获取 Git 日志失败: ...` | 立即退出 (exit 1) |

## 使用示例

### 示例 1: 配置文件指定路径

`config.json`:
```json
{
  "projectPath": "/Users/name/projects/my-app",
  "zentao": { ... }
}
```

运行:
```bash
node commit2zen.mjs
```

### 示例 2: 命令行参数覆盖

```bash
node commit2zen.mjs /path/to/another/project
```

即使配置文件中指定了 `projectPath`,命令行参数也会覆盖它。

### 示例 3: 默认行为(当前目录)

不配置 `projectPath`,不传命令行参数:
```bash
cd /my/project
node commit2zen.mjs
```

行为: 使用当前目录 (`/my/project`)

## 文件结构

```
Commit2Zen/
├── commit2zen.mjs              # 主脚本 (修改)
├── config.json                 # 配置文件 (可选添加 projectPath)
├── config.example.json         # 配置示例 (添加 projectPath)
├── .gitignore
├── README.md                   # 更新使用说明
└── reports/
```

## 成功标准

- [ ] 能通过配置文件 `projectPath` 指定项目
- [ ] 能通过命令行参数指定项目
- [ ] 命令行参数优先级高于配置文件
- [ ] 未指定时使用当前目录(向后兼容)
- [ ] 路径不存在时给出清晰错误提示
- [ ] 不是 Git 仓库时给出清晰错误提示
- [ ] 执行完毕后恢复原工作目录

## 向后兼容性

**完全向后兼容:**
- 不配置 `projectPath` 时,行为与之前完全相同(使用当前目录)
- 现有配置无需修改即可继续工作
- `config.example.json` 添加 `projectPath` 但标记为可选
