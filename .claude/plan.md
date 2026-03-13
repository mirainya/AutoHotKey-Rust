# AutoHotKey-Rust 完善方案

## 阶段一：后端基础设施（稳定性 & 安全性）

### 1.1 统一错误处理
- 新建 `src-tauri/src/error.rs`，定义 `AppError` 枚举
- 包含变体：`Io`、`Serde`、`Enigo`、`Screenshot`、`Script`、`Window`、`Custom`
- 实现 `Into<tauri::ipc::InvokeError>`，所有命令返回 `Result<T, AppError>`
- 替换所有 `.map_err(|e| e.to_string())` 和 `.unwrap()`

### 1.2 添加日志框架
- Cargo.toml 添加 `tracing` + `tracing-subscriber`
- `main.rs` 初始化日志（文件 + 控制台），日志写入 app_data_dir/logs/
- 替换所有 `println!` 为 `tracing::info!/warn!/error!`

### 1.3 修复热键线程泄漏
- `hotkey.rs` 中用 `JoinHandle` 保存线程句柄
- `stop_hotkey_listener` 设置 flag 后等待线程退出（rdev::listen 的限制需要特殊处理）
- 防止重复 spawn

### 1.4 脚本执行安全增强
- 添加执行超时机制（读取配置的 scriptTimeout）
- 超时后强制终止 JS 运行时
- 限制 `readFile`/`writeFile` 只能访问 app_data_dir 下的文件（路径沙箱）

---

## 阶段二：补全缺失接口

### 2.1 窗口操作增强
- `find_window_by_class(class_name)` — 按窗口类名查找
- `send_keys(hwnd, keys)` — 组合键发送（如 "Ctrl+A"）
- 完善 `charToVkCode`：添加 numpad、标点符号、media keys 支持

### 2.2 脚本管理增强
- `run_script_by_id(id)` — 按 ID 加载并执行已保存脚本
- `get_script_status()` — 返回当前脚本运行状态（idle/running/error）

### 2.3 JS 运行时新增 API
- `httpGet(url)` / `httpPost(url, body)` — 脚本内 HTTP 请求（需加 `reqwest`）
- `playSound(path)` — 播放提示音（Windows API `PlaySound`）
- `msgBox(title, text)` — 已有，确认 JS 运行时中可用
- `getClipboard()` / `setClipboard(text)` — 确认 JS 运行时中可用

---

## 阶段三：前端架构优化

### 3.1 引入 Pinia 状态管理
- 安装 pinia，创建 stores/：
  - `scriptStore` — 脚本列表、运行状态、日志
  - `configStore` — 配置项（从后端加载/保存）
  - `resourceStore` — 像素模式等资源

### 3.2 统一 Tauri invoke 封装
- 创建 `src/utils/tauri.ts`，封装 invoke 调用
- 统一错误提示（ElMessage.error）
- 添加 loading 状态管理

### 3.3 Settings 配置持久化
- 后端新增 `save_config` / `load_config` 命令
- 配置保存到 app_data_dir/config.json
- Settings 组件改为从后端读写配置
- 脚本超时等配置实际生效

### 3.4 日志上限控制
- ScriptManager 中 logs 数组受 config.maxLogs 限制
- 超出时自动移除最旧的日志

---

## 阶段四：高级功能

### 4.1 脚本导入/导出
- 后端：`import_script(path)` / `export_script(id, path)`
- 前端：ScriptManager 添加导入/导出按钮，使用 tauri-plugin-dialog 选择文件

### 4.2 定时任务
- 后端：`schedule_script(id, cron_expr)` / `unschedule_script(id)` / `list_schedules()`
- 简单实现：用 tokio 定时器或 std::thread + sleep 循环检查
- 前端：ScriptManager 中添加定时设置 UI

### 4.3 系统托盘快捷运行
- 托盘菜单动态添加已启用的脚本
- 点击托盘菜单项直接运行对应脚本

### 4.4 多显示器支持
- `get_all_screens()` — 返回所有显示器信息
- 截图和像素操作支持指定显示器索引

---

## 执行策略
- 每个阶段完成后暂停，让主人确认再继续下一阶段
- 每个子任务完成后进行编译检查
- 保持与现有代码风格一致
