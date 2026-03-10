# AutoHotKey-Rust

🤖 一个基于 Tauri 2.0 的现代化桌面自动化工具，使用 Rust 和 Vue.js 构建

## ✨ 特性

- 🔥 **热键管理** - 注册和管理全局热键，快速触发脚本
- 📸 **像素选择器** - 屏幕取色和图像识别功能
- ✏️ **脚本编辑器** - 基于 CodeMirror 的 JavaScript 脚本编辑器
- 📦 **资源收集器** - 管理和组织脚本资源
- ⚙️ **配置管理** - 灵活的系统配置选项
- 📖 **API 文档** - 内置完整的 API 文档

## 🛠️ 技术栈

### 前端
- **框架**: Vue 3.4 + TypeScript
- **UI 组件库**: Element Plus
- **构建工具**: Vite 5.0
- **代码编辑器**: CodeMirror 6
- **样式**: SCSS

### 后端
- **框架**: Tauri 2.0
- **语言**: Rust
- **脚本引擎**: rquickjs (JavaScript 运行时)
- **自动化库**: 
  - rdev (输入模拟)
  - enigo (跨平台输入控制)
  - screenshots (屏幕截图)
  - image (图像处理)

## 📦 安装

### 环境要求
- Node.js 18+ 
- Rust 1.70+
- Windows 10/11

### 克隆项目
```bash
git clone https://github.com/yourusername/autohotkey-rust.git
cd autohotkey-rust
```

### 安装依赖
```bash
npm install
```

## 🚀 开发

### 启动开发服务器
```bash
npm run dev
```

### 运行 Tauri 应用
```bash
npm run tauri dev
```

### 构建生产版本
```bash
npm run build
npm run tauri build
```

## 📝 功能说明

### 脚本管理
- 创建、编辑、删除 JavaScript 脚本
- 脚本持久化存储（JSON 格式）
- 支持脚本热键绑定
- 启用/禁用脚本控制
- 实时脚本执行和输出

### 核心功能
- **鼠标控制**：移动、点击、滚动
- **键盘输入**：按键、文本输入
- **窗口管理**：查找、激活、移动窗口
- **剪贴板操作**：读取和写入
- **屏幕截图**：全屏和窗口截图
- **图像识别**：像素匹配和查找

### 像素操作
- 捕获屏幕区域像素数据
- 保存和加载像素模式
- 在图像中查找匹配模式
- 获取像素颜色值

## 📁 项目结构

```
autohotkey-rust/
├── src/                      # 前端源代码
│   ├── components/          # Vue 组件
│   │   ├── ApiDocs.vue      # API 文档组件
│   │   ├── HotkeyManager.vue # 热键管理组件
│   │   ├── PixelSelector.vue # 像素选择器
│   │   ├── ResourceCollector.vue # 资源收集器
│   │   ├── ScriptEditor.vue # 脚本编辑器
│   │   ├── ScriptEditorPage.vue # 脚本编辑页面
│   │   ├── ScriptManager.vue # 脚本管理器
│   │   └── Settings.vue     # 设置组件
│   ├── styles/              # 样式文件
│   │   └── miku-theme.scss  # 主题样式
│   ├── utils/               # 工具函数
│   │   └── rhaiLanguage.ts  # Rhai 语言支持
│   ├── App.vue              # 主应用组件
│   └── main.ts              # 入口文件
├── src-tauri/               # Tauri 后端源代码
│   ├── capabilities/        # 权限配置
│   ├── icons/               # 应用图标
│   ├── scripts/             # 脚本文件
│   └── src/
│       ├── commands/        # Tauri 命令
│       │   ├── automation.rs # 自动化命令
│       │   ├── hotkey.rs    # 热键命令
│       │   ├── mod.rs       # 模块导出
│       │   ├── pixel.rs     # 像素操作命令
│       │   ├── screenshot.rs # 截图命令
│       │   ├── script.rs    # 脚本命令
│       │   └── window.rs    # 窗口命令
│       └── main.rs          # 后端入口
├── public/                  # 公共资源
├── index.html               # HTML 模板
├── package.json             # Node.js 依赖配置
├── tsconfig.json            # TypeScript 配置
└── vite.config.ts           # Vite 配置
```

## 🔌 API 示例

### JavaScript 脚本示例
```javascript
// 打印输出
print("开始执行脚本...");

// 鼠标操作
moveMouse(100, 100);
clickMouse(100, 100, "left");
scrollMouse(100, 100, 120);

// 键盘操作
typeText("Hello World");
keyPress("enter");
keyDown("shift");
keyUp("shift");

// 等待
sleep(1000);

// 检查停止状态
if (isStopped()) {
    print("脚本已停止");
}

// 像素操作
let color = getPixelColor(100, 100);
let positions = findPattern("myImage", 90); // 90% 相似度

// 窗口操作
let hwnd = findWindow("Untitled - Notepad");
activateWindow(hwnd);
moveWindow(hwnd, 0, 0, 800, 600);
postChar(hwnd, "A");

// 循环等待
let found = waitForPattern("target", 5000); // 5 秒超时
if (found) {
    print("找到目标！");
}
```

### 可用的 API 函数

#### 基础函数
- `print(msg)` - 输出消息
- `sleep(ms)` - 延迟指定毫秒
- `isStopped()` - 检查是否被停止
- `screenshot()` - 截取屏幕

#### 鼠标控制
- `moveMouse(x, y)` - 移动鼠标
- `clickMouse(x, y, button)` - 点击鼠标
- `scrollMouse(x, y, delta)` - 滚动鼠标

#### 键盘控制
- `typeText(text)` - 输入文本
- `keyPress(key)` - 按下并释放键
- `keyDown(key)` - 按下键
- `keyUp(key)` - 释放键

#### 像素操作
- `getPixelColor(x, y)` - 获取像素颜色
- `findPattern(name, tolerance)` - 查找保存的图案
- `findCustomPattern(pattern, tolerance)` - 查找自定义图案
- `waitForPattern(name, timeout)` - 等待图案出现

#### 窗口操作
- `findWindow(title)` - 查找窗口
- `getForegroundWindow()` - 获取前台窗口
- `getWindowInfo(hwnd)` - 获取窗口信息
- `activateWindow(hwnd)` - 激活窗口
- `moveWindow(hwnd, x, y, w, h)` - 移动窗口
- `showWindow(hwnd, state)` - 显示/隐藏窗口
- `postClick(hwnd, x, y, button)` - 后台发送点击
- `postKey(hwnd, vkCode)` - 后台发送虚拟键
- `postChar(hwnd, char)` - 后台发送字符

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📧 联系方式

如有问题或建议，请通过 GitHub Issues 联系我们。

---

**注意**: 本项目仅供学习和研究使用。请合理使用自动化工具，遵守相关法律法规。
