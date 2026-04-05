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

  if (!Number.isInteger(maxCommits) || maxCommits < 1) {
    console.error('❌ maxCommits 必须是正整数');
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

// ============================================================
// 禅道 API 客户端模块
// ============================================================

async function zentaoLogin(url, account, password) {
  const loginUrl = `${url}/api.php/v1/tokens`;

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
          'Token': `${token}`
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
      const isNetworkError =
        error.name === 'AbortError' ||
        error.message.includes('fetch') ||
        error.code === 'ECONNREFUSED' ||
        error.code === 'ENOTFOUND' ||
        error.code === 'ETIMEDOUT';

      if (attempt < retries && isNetworkError) {
        const delay = Math.pow(2, attempt - 1) * 1000;
        console.warn(`   ⚠️  请求失败,${delay / 1000}s 后重试 (${attempt}/${retries})...`);
        await new Promise(resolve => setTimeout(resolve, delay));
      } else {
        throw error;
      }
    }
  }

  throw new Error('所有重试均失败');
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

// ============================================================
// 主入口
// ============================================================

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

main().catch(error => {
  console.error('❌ 未捕获的错误:', error.message);
  process.exit(1);
});
