# Commit2Zen

Git 提交记录管理与禅道任务自动化工具。

## 功能特性

- 本地 Git 仓库扫描发现
- 禅道连接测试与项目选择
- Commit 记录预览与分支汇总
- 一键创建禅道任务
- 历史报告查看

### 前置要求

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) 18+

### 安装

```bash
# 安装前端依赖
npm install

# 安装 Tauri CLI
cargo install tauri-cli
```

### 开发模式

```bash
cargo tauri dev
```

### 构建安装包

```bash
cargo tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

---

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
