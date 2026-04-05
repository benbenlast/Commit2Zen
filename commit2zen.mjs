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
