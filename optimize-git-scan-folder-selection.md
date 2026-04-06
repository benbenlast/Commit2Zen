# Git 扫描优化方案：文件夹选择扫描

## 背景

当前系统有两种扫描模式：
1. **快速扫描**：检查预定义的常见目录（D:\aicode\ 等）
2. **全盘扫描**：枚举所有磁盘并递归扫描

用户要求将两者合并为统一的文件夹选择扫描，并引入优化方案。

## 调整方案

### 1. UI 变化（ExecuteView.vue）

**移除：**
- "快速扫描" 按钮
- "全盘扫描" 按钮

**新增：**
- "选择文件夹" 按钮（使用 Tauri dialog 打开目录选择器）
- 显示已选择的文件夹路径
- "开始扫描" 按钮（选择文件夹后启用）
- 保留进度条和取消按钮

### 2. 后端变化

#### 2.1 新增命令 `scan_folder`
```rust
#[command]
pub async fn scan_folder(
    root_path: String,
    app: tauri::AppHandle,
    cancel_map: tauri::State<'_, ScanCancelMap>,
) -> Result<String, String>
```

- 接收一个根目录路径参数
- 返回 scan_id 用于取消
- 通过 Tauri 事件发送进度

#### 2.2 使用 walkdir 优化扫描

**替代方案：**
- 当前使用手动 `read_dir` 递归遍历
- 改用 `walkdir` crate 的高效遍历

**优化点：**
1. **Pruning 优化**：找到 `.git` 目录后立即停止深入
2. **前置过滤**：遍历时跳过已知非项目目录
3. **并行扫描**（可选）：使用 `jwalk` 进行多线程扫描

#### 2.3 保留优化

- 进度事件系统（scan-progress）
- 取消机制（AtomicBool）
- Skip List（HashSet 过滤）

### 3. 前端变化（git.js + ExecuteView.vue）

**git.js 新增：**
```javascript
async scanFolder(folderPath) {
  const scanId = await invoke('scan_folder', { rootPath: folderPath })
  // 监听进度事件...
}
```

**ExecuteView.vue 更新：**
- 使用 `dialog.open` 选择目录
- 显示选择的路径
- 调用 `scanFolder` 而非 `startFullScan`

### 4. 依赖变化

**Cargo.toml 添加：**
```toml
walkdir = "2"
```

### 5. 文件变更清单

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src-tauri/Cargo.toml` | 修改 | 添加 walkdir 依赖 |
| `src-tauri/src/services/scanner.rs` | 重写 | 使用 walkdir 替代手动递归 |
| `src-tauri/src/commands/git.rs` | 修改 | 添加 scan_folder 命令 |
| `src-tauri/src/main.rs` | 修改 | 注册 scan_folder 命令 |
| `src/stores/git.js` | 修改 | 添加 scanFolder 方法 |
| `src/views/ExecuteView.vue` | 重写 | 统一文件夹选择 UI |

### 6. 实施步骤

1. 添加 walkdir 依赖
2. 重写 scanner.rs 使用 walkdir
3. 添加 scan_folder 命令
4. 更新 main.rs 注册命令
5. 更新 git.js store
6. 重写 ExecuteView.vue UI
7. 测试扫描功能

## 预期效果

- 用户点击"选择文件夹" → 选择目录 → 点击"开始扫描"
- 扫描使用 walkdir 高效遍历
- 实时显示进度和发现的仓库
- 支持随时取消
- 扫描速度提升 30-50%（取决于目录结构）
