use std::collections::HashSet;
use once_cell::sync::Lazy;

/// 已知非项目目录名称
static KNOWN_NON_PROJECT_DIRS: Lazy<HashSet<&str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    
    // Windows 系统目录
    set.insert("Windows");
    set.insert("Program Files");
    set.insert("Program Files (x86)");
    set.insert("ProgramData");
    set.insert("$Recycle.Bin");
    set.insert("System Volume Information");
    set.insert("Recovery");
    set.insert("Boot");
    set.insert("EFI");
    set.insert("PerfLogs");
    set.insert("Users");
    
    // 用户配置/缓存目录
    set.insert("AppData");
    set.insert("Local Settings");
    set.insert("Application Data");
    set.insert("Temporary Internet Files");
    
    // 构建产物和依赖
    set.insert("node_modules");
    set.insert("target");
    set.insert("dist");
    set.insert("build");
    set.insert(".next");
    set.insert(".output");
    set.insert("__pycache__");
    set.insert(".venv");
    set.insert("venv");
    set.insert(".tox");
    set.insert(".gradle");
    set.insert("vendor");
    
    // 其他非项目目录
    set.insert("$RECYCLE.BIN");
    set.insert("Config.Msi");
    set.insert("MSOCache");
    
    set
});

/// 判断是否应该跳过该目录
pub fn should_skip_directory(name: &str) -> bool {
    // 跳过隐藏目录（但 .git 除外，因为它是 Git 仓库标志）
    if name.starts_with('.') && name != ".git" {
        return true;
    }
    
    KNOWN_NON_PROJECT_DIRS.contains(name)
}
