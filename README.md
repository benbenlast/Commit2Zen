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
