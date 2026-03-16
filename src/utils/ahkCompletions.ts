import { autocompletion, type CompletionContext, type Completion } from '@codemirror/autocomplete'

const apiCompletions: Completion[] = [
  // 鼠标
  { label: 'moveMouse', type: 'function', detail: '(x, y)', info: '移动鼠标到指定坐标' },
  { label: 'clickMouse', type: 'function', detail: '(x, y, button)', info: '点击鼠标，button: "left"|"right"|"middle"' },
  { label: 'doubleClick', type: 'function', detail: '(x, y, button)', info: '双击鼠标' },
  { label: 'dragMouse', type: 'function', detail: '(x1, y1, x2, y2, button)', info: '拖拽鼠标从(x1,y1)到(x2,y2)' },
  { label: 'scrollMouse', type: 'function', detail: '(x, y, delta)', info: '滚动鼠标，正数向上负数向下' },
  { label: 'getMousePos', type: 'function', detail: '()', info: '获取鼠标坐标，返回 JSON {"x","y"}' },

  // 键盘
  { label: 'keyPress', type: 'function', detail: '(key)', info: '按下并释放按键' },
  { label: 'keyDown', type: 'function', detail: '(key)', info: '按下按键不释放' },
  { label: 'keyUp', type: 'function', detail: '(key)', info: '释放按键' },
  { label: 'typeText', type: 'function', detail: '(text)', info: '模拟键盘输入文本' },
  { label: 'sendKeys', type: 'function', detail: '(keys)', info: '发送组合键，如 "Ctrl+C"' },

  // 窗口
  { label: 'findWindow', type: 'function', detail: '(title)', info: '按标题查找窗口，返回 hwnd' },
  { label: 'findWindowByClass', type: 'function', detail: '(className)', info: '按类名查找窗口' },
  { label: 'getForegroundWindow', type: 'function', detail: '()', info: '获取前台窗口句柄' },
  { label: 'getWindowInfo', type: 'function', detail: '(hwnd)', info: '获取窗口信息 JSON' },
  { label: 'getWindowTitle', type: 'function', detail: '(hwnd)', info: '获取窗口标题' },
  { label: 'activateWindow', type: 'function', detail: '(hwnd)', info: '激活窗口到前台' },
  { label: 'moveWindow', type: 'function', detail: '(hwnd, x, y, w, h)', info: '移动/调整窗口' },
  { label: 'showWindow', type: 'function', detail: '(hwnd, cmd)', info: '显示/隐藏/最大化窗口' },

  // 消息
  { label: 'postClick', type: 'function', detail: '(hwnd, x, y, button)', info: '后台点击窗口' },
  { label: 'postKey', type: 'function', detail: '(hwnd, vkCode)', info: '后台发送按键' },
  { label: 'postChar', type: 'function', detail: '(hwnd, char)', info: '后台发送字符' },

  // 像素 & 截图
  { label: 'getPixelColor', type: 'function', detail: '(x, y)', info: '获取像素颜色 "#RRGGBB"' },
  { label: 'captureWindowImage', type: 'function', detail: '(hwnd)', info: '截取窗口图像' },
  { label: 'waitForPixelColor', type: 'function', detail: '(x, y, color, timeout, tolerance)', info: '轮询等待像素变为指定颜色' },

  // 文件 & 剪贴板
  { label: 'readFile', type: 'function', detail: '(path)', info: '读取文件内容（沙箱内）' },
  { label: 'writeFile', type: 'function', detail: '(path, text)', info: '写入文件（沙箱内）' },
  { label: 'appendFile', type: 'function', detail: '(path, content)', info: '追加写入文件（沙箱内）' },
  { label: 'getClipboard', type: 'function', detail: '()', info: '获取剪贴板文本' },
  { label: 'setClipboard', type: 'function', detail: '(text)', info: '设置剪贴板文本' },

  // 工具
  { label: 'sleep', type: 'function', detail: '(ms)', info: '等待指定毫秒' },
  { label: 'isStopped', type: 'function', detail: '()', info: '检查是否已请求停止' },
  { label: 'getScreenSize', type: 'function', detail: '()', info: '获取屏幕分辨率 JSON {"width","height"}' },
  { label: 'print', type: 'function', detail: '(msg)', info: '输出到控制台' },
  { label: 'warn', type: 'function', detail: '(msg)', info: '输出警告到控制台（黄色）' },
  { label: 'debug', type: 'function', detail: '(msg)', info: '输出调试信息到控制台（灰色）' },
  { label: 'charToVkCode', type: 'function', detail: '(char)', info: '字符转虚拟键码' },
  { label: 'now', type: 'function', detail: '()', info: '获取当前时间戳（秒）' },
  { label: 'random', type: 'function', detail: '(min, max)', info: '生成随机整数' },
  { label: 'fileExists', type: 'function', detail: '(path)', info: '检查沙箱内文件是否存在' },
  { label: 'getEnv', type: 'function', detail: '(name)', info: '读取环境变量' },
  { label: 'getDateTime', type: 'function', detail: '()', info: '获取当前时间 "YYYY-MM-DD HH:MM:SS"' },
  { label: 'getTimestamp', type: 'function', detail: '()', info: '获取毫秒级时间戳' },
  { label: 'include', type: 'function', detail: '(path)', info: '加载沙箱内脚本文件内容，配合 eval 使用' },

  // 进程管理
  { label: 'run', type: 'function', detail: '(command)', info: '执行外部程序，返回 pid' },
  { label: 'runWait', type: 'function', detail: '(command)', info: '执行并等待，返回 JSON {exitCode, stdout}' },
  { label: 'processExists', type: 'function', detail: '(name)', info: '检查进程是否存在（仅Windows）' },
  { label: 'killProcess', type: 'function', detail: '(pid)', info: '终止进程（仅Windows）' },

  // 窗口扩展
  { label: 'enumWindows', type: 'function', detail: '()', info: '枚举所有窗口，返回 JSON 数组' },
  { label: 'waitForWindow', type: 'function', detail: '(title, timeout)', info: '等待窗口出现，返回 hwnd' },

  // 对话框 & 通知
  { label: 'msgBox', type: 'function', detail: '(title, text)', info: '弹出消息对话框' },
  { label: 'notify', type: 'function', detail: '(title, text)', info: '发送通知消息' },
  { label: 'beep', type: 'function', detail: '(freq, duration)', info: '播放蜂鸣声（仅Windows）' },
  { label: 'playSound', type: 'function', detail: '(path)', info: '播放 WAV 文件（仅Windows）' },

  // HTTP 请求
  { label: 'httpGet', type: 'function', detail: '(url)', info: 'HTTP GET 请求，返回响应文本' },
  { label: 'httpPost', type: 'function', detail: '(url, body, contentType)', info: 'HTTP POST 请求，返回响应文本' },

  // 定时器
  { label: 'setTimeout', type: 'function', detail: '(fn, ms)', info: '延迟执行函数（阻塞式）' },
  { label: 'setInterval', type: 'function', detail: '(fn, ms)', info: '循环执行函数，配合 clearInterval 停止' },
  { label: 'clearTimeout', type: 'function', detail: '(id)', info: '取消 setTimeout' },
  { label: 'clearInterval', type: 'function', detail: '(id)', info: '取消 setInterval 循环' },

  // 坐标模式
  { label: 'coordMode', type: 'function', detail: '(mode)', info: '设置坐标模式 "screen"|"window"' },
  { label: 'getCoordMode', type: 'function', detail: '()', info: '获取当前坐标模式' },
]

function ahkCompletions(context: CompletionContext) {
  const word = context.matchBefore(/\w*/)
  if (!word || (word.from === word.to && !context.explicit)) return null
  return {
    from: word.from,
    options: apiCompletions,
    validFor: /^\w*$/,
  }
}

export const ahkAutocomplete = autocompletion({
  override: [ahkCompletions],
  defaultKeymap: true,
})
