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
}

main().catch(error => {
  console.error('❌ 未捕获的错误:', error.message);
  process.exit(1);
});
