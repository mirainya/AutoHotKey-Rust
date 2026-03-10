<template>
  <div class="api-docs">
    <!-- 左侧目录 -->
    <aside class="api-sidebar">
      <input v-model="search" class="search-input" placeholder="🔍 搜索 API..." />
      <div v-for="cat in filteredCategories" :key="cat.name" class="cat-group">
        <div class="cat-title" @click="toggleCat(cat.name)">
          <span>{{ cat.name }}</span>
          <span class="cat-arrow">{{ collapsed.has(cat.name) ? '▶' : '▼' }}</span>
        </div>
        <template v-if="!collapsed.has(cat.name)">
          <div
            v-for="api in cat.apis"
            :key="api.name"
            :class="['api-link', { active: selected?.name === api.name }]"
            @click="selected = api"
          >
            <div class="api-link-name">{{ api.name }}</div>
            <div class="api-link-desc">{{ api.desc.slice(0, 18) }}...</div>
          </div>
        </template>
      </div>
    </aside>

    <!-- 右侧详情 -->
    <main class="api-detail" v-if="selected">
      <div class="fn-name">{{ selected.signature }}</div>
      <p class="fn-desc">{{ selected.desc }}</p>

      <template v-if="selected.params.length">
        <div class="section-title">参数</div>
        <table class="params-table">
          <tr v-for="p in selected.params" :key="p.name">
            <td class="pname">{{ p.name }}</td>
            <td class="ptype">{{ p.type }}</td>
            <td class="pdesc">{{ p.desc }}</td>
          </tr>
        </table>
      </template>

      <div class="section-title">返回值</div>
      <p class="returns">{{ selected.returns }}</p>

      <div class="section-title">示例</div>
      <pre class="example">{{ selected.example }}</pre>

      <template v-if="selected.notes">
        <div class="section-title">说明</div>
        <p class="notes">{{ selected.notes }}</p>
      </template>
    </main>
    <main class="api-detail empty" v-else>
      <p>← 从左侧选择一个 API 查看详情</p>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface Param { name: string; type: string; desc: string }
interface Api {
  name: string; signature: string; desc: string
  params: Param[]; returns: string; example: string; notes?: string
}
interface Category { icon: string; name: string; apis: Api[] }

const selected = ref<Api | null>(null)
const search = ref('')
const collapsed = ref<Set<string>>(new Set())
function toggleCat(name: string) {
  collapsed.value.has(name) ? collapsed.value.delete(name) : collapsed.value.add(name)
  collapsed.value = new Set(collapsed.value)
}

const categories: Category[] = [
  {
    icon: '🖱️', name: '鼠标',
    apis: [
      {
        name: 'moveMouse', signature: 'moveMouse(x, y)',
        desc: '将鼠标光标移动到屏幕指定坐标，不产生点击。',
        params: [
          { name: 'x', type: 'number', desc: '屏幕横坐标（像素）' },
          { name: 'y', type: 'number', desc: '屏幕纵坐标（像素）' },
        ],
        returns: 'void',
        example: 'moveMouse(100, 200)',
      },
      {
        name: 'clickMouse', signature: 'clickMouse(x, y, button)',
        desc: '移动鼠标到指定坐标并点击。',
        params: [
          { name: 'x', type: 'number', desc: '屏幕横坐标' },
          { name: 'y', type: 'number', desc: '屏幕纵坐标' },
          { name: 'button', type: '"left" | "right" | "middle"', desc: '鼠标按键' },
        ],
        returns: 'void',
        example: 'clickMouse(100, 200, "left")',
      },
      {
        name: 'scrollMouse', signature: 'scrollMouse(x, y, delta)',
        desc: '在指定坐标滚动鼠标滚轮。',
        params: [
          { name: 'x', type: 'number', desc: '屏幕横坐标' },
          { name: 'y', type: 'number', desc: '屏幕纵坐标' },
          { name: 'delta', type: 'number', desc: '滚动量，正数向上，负数向下' },
        ],
        returns: 'void',
        example: 'scrollMouse(500, 400, 3)   // 向上滚动3格\nscrollMouse(500, 400, -3)  // 向下滚动3格',
      },
    ]
  },
  {
    icon: '⌨️', name: '键盘',
    apis: [
      {
        name: 'keyPress', signature: 'keyPress(key)',
        desc: '按下并立即释放一个按键（完整的按键动作）。',
        params: [{ name: 'key', type: 'string', desc: '按键名，如 "a"、"enter"、"ctrl"、"f1"' }],
        returns: 'void',
        example: 'keyPress("enter")\nkeyPress("f5")',
      },
      {
        name: 'keyDown', signature: 'keyDown(key)',
        desc: '按下按键但不释放，常与 keyUp 配合实现组合键。',
        params: [{ name: 'key', type: 'string', desc: '按键名' }],
        returns: 'void',
        example: 'keyDown("ctrl")\nkeyPress("c")   // Ctrl+C\nkeyUp("ctrl")',
      },
      {
        name: 'keyUp', signature: 'keyUp(key)',
        desc: '释放之前按下的按键。',
        params: [{ name: 'key', type: 'string', desc: '按键名' }],
        returns: 'void',
        example: 'keyDown("shift")\nkeyPress("a")   // 输入大写 A\nkeyUp("shift")',
      },
      {
        name: 'typeText', signature: 'typeText(text)',
        desc: '模拟键盘逐字输入文本，适合在输入框中输入内容。',
        params: [{ name: 'text', type: 'string', desc: '要输入的字符串' }],
        returns: 'void',
        example: 'typeText("Hello World")',
      },
    ]
  },
  {
    icon: '🪟', name: '窗口',
    apis: [
      {
        name: 'findWindow', signature: 'findWindow(title)',
        desc: '按窗口标题查找窗口，返回窗口句柄（hwnd）。句柄是后续所有窗口操作的标识符。',
        params: [{ name: 'title', type: 'string', desc: '窗口标题（完整匹配）' }],
        returns: 'number — 窗口句柄，找不到返回 0',
        example: 'const hwnd = findWindow("记事本")\nif (hwnd) {\n  activateWindow(hwnd)\n}',
      },
      {
        name: 'getForegroundWindow', signature: 'getForegroundWindow()',
        desc: '获取当前处于前台（用户正在操作）的窗口句柄。',
        params: [],
        returns: 'number — 当前前台窗口句柄，失败返回 0',
        example: 'const hwnd = getForegroundWindow()\nprint("当前窗口: " + hwnd)',
      },
      {
        name: 'getWindowInfo', signature: 'getWindowInfo(hwnd)',
        desc: '获取窗口的详细信息，包括标题、类名、进程名、位置和大小。返回 JSON 字符串，需用 JSON.parse() 解析。',
        params: [{ name: 'hwnd', type: 'number', desc: '窗口句柄' }],
        returns: 'string — JSON: { hwnd, title, class_name, process_name, rect: {x,y,w,h} }',
        example: 'const hwnd = findWindow("记事本")\nconst info = JSON.parse(getWindowInfo(hwnd))\nprint(info.title)\nprint(info.process_name)   // notepad.exe\nprint(info.rect.w + "x" + info.rect.h)',
      },
      {
        name: 'activateWindow', signature: 'activateWindow(hwnd)',
        desc: '将指定窗口置于前台并激活，使其获得焦点。',
        params: [{ name: 'hwnd', type: 'number', desc: '窗口句柄' }],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\nactivateWindow(hwnd)\nsleep(200)\ntypeText("Hello")',
      },
      {
        name: 'moveWindow', signature: 'moveWindow(hwnd, x, y, w, h)',
        desc: '移动窗口到指定位置并调整大小。',
        params: [
          { name: 'hwnd', type: 'number', desc: '窗口句柄' },
          { name: 'x', type: 'number', desc: '窗口左上角横坐标' },
          { name: 'y', type: 'number', desc: '窗口左上角纵坐标' },
          { name: 'w', type: 'number', desc: '窗口宽度' },
          { name: 'h', type: 'number', desc: '窗口高度' },
        ],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\nmoveWindow(hwnd, 0, 0, 800, 600)',
      },
      {
        name: 'showWindow', signature: 'showWindow(hwnd, state)',
        desc: '控制窗口的显示状态。',
        params: [
          { name: 'hwnd', type: 'number', desc: '窗口句柄' },
          { name: 'state', type: '"minimize" | "maximize" | "hide" | "restore"', desc: '目标状态' },
        ],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\nshowWindow(hwnd, "minimize")\nsleep(1000)\nshowWindow(hwnd, "restore")',
      },
      {
        name: 'postClick', signature: 'postClick(hwnd, x, y, button)',
        desc: '向后台窗口发送鼠标点击消息，窗口无需在前台即可响应。坐标是相对于窗口客户区的坐标。',
        params: [
          { name: 'hwnd', type: 'number', desc: '窗口句柄' },
          { name: 'x', type: 'number', desc: '相对于窗口的横坐标' },
          { name: 'y', type: 'number', desc: '相对于窗口的纵坐标' },
          { name: 'button', type: '"left" | "right"', desc: '鼠标按键' },
        ],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\npostClick(hwnd, 100, 50, "left")',
        notes: '后台消息适用于支持消息驱动的窗口，部分游戏或特殊程序可能不响应。',
      },
      {
        name: 'postKey', signature: 'postKey(hwnd, vkCode)',
        desc: '向后台窗口发送按键消息，使用 Windows 虚拟键码（VK Code）。',
        params: [
          { name: 'hwnd', type: 'number', desc: '窗口句柄' },
          { name: 'vkCode', type: 'number', desc: 'Windows 虚拟键码（十六进制）' },
        ],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\npostKey(hwnd, 0x0D)   // Enter\npostKey(hwnd, 0x1B)   // Escape\npostKey(hwnd, 0x26)   // 方向键上\npostKey(hwnd, 0x41)   // A键（A=0x41, B=0x42...Z=0x5A）\npostKey(hwnd, 0x30)   // 数字0（0=0x30...9=0x39）',
        notes: '常用键码：Enter=0x0D, Space=0x20, Escape=0x1B, Tab=0x09\n方向键：上=0x26, 下=0x28, 左=0x25, 右=0x27\nF1-F12=0x70-0x7B',
      },
      {
        name: 'postChar', signature: 'postChar(hwnd, char)',
        desc: '向后台窗口发送字符输入消息，比 postKey 更简单，适合输入文字。',
        params: [
          { name: 'hwnd', type: 'number', desc: '窗口句柄' },
          { name: 'char', type: 'string', desc: '要发送的字符串' },
        ],
        returns: 'void',
        example: 'const hwnd = findWindow("记事本")\npostChar(hwnd, "Hello")',
      },
    ]
  },
  {
    icon: '🖼️', name: '像素/截图',
    apis: [
      {
        name: 'getPixelColor', signature: 'getPixelColor(x, y)',
        desc: '获取屏幕指定坐标的像素颜色，返回 RGB 值的 JSON 字符串。',
        params: [
          { name: 'x', type: 'number', desc: '屏幕横坐标' },
          { name: 'y', type: 'number', desc: '屏幕纵坐标' },
        ],
        returns: 'string — JSON: { r, g, b }，值范围 0-255',
        example: 'const c = JSON.parse(getPixelColor(100, 200))\nprint("R=" + c.r + " G=" + c.g + " B=" + c.b)\n\n// 判断某点是否为红色\nif (c.r > 200 && c.g < 50 && c.b < 50) {\n  print("检测到红色！")\n}',
      },
      {
        name: 'findPattern', signature: 'findPattern(name, tolerance)',
        desc: '在屏幕上查找已在"资源"页面保存的像素图案，返回所有匹配位置。',
        params: [
          { name: 'name', type: 'string', desc: '图案名称（在资源页面创建）' },
          { name: 'tolerance', type: 'number', desc: '颜色容差 0-255，建议 10-30' },
        ],
        returns: 'string — JSON: [[x,y], ...]，未找到返回 "[]"',
        example: 'const pts = JSON.parse(findPattern("开始按钮", 15))\nif (pts.length > 0) {\n  clickMouse(pts[0][0], pts[0][1], "left")\n} else {\n  print("未找到图案")\n}',
      },
      {
        name: 'waitForPattern', signature: 'waitForPattern(name, timeout_ms)',
        desc: '持续等待屏幕上出现指定图案，找到后立即返回坐标。超时则返回空数组，每 100ms 检测一次。',
        params: [
          { name: 'name', type: 'string', desc: '图案名称（在资源页面创建）' },
          { name: 'timeout_ms', type: 'number', desc: '最长等待时间（毫秒）' },
        ],
        returns: 'string — JSON: [[x,y], ...]，超时返回 "[]"',
        example: 'const pts = JSON.parse(waitForPattern("加载完成", 10000))\nif (pts.length > 0) {\n  print("加载完成，开始操作")\n  clickMouse(pts[0][0], pts[0][1], "left")\n} else {\n  print("等待超时")\n}',
      },
      {
        name: 'screenshot', signature: 'screenshot()',
        desc: '截取当前屏幕并保存到应用数据目录，返回保存的文件路径。',
        params: [],
        returns: 'string — 截图文件的完整路径',
        example: 'const path = screenshot()\nprint("截图保存到: " + path)',
      },
    ]
  },
  {
    icon: '🖥️', name: '系统',
    apis: [
      {
        name: 'getMousePos', signature: 'getMousePos()',
        desc: '获取当前鼠标光标的屏幕坐标。',
        params: [],
        returns: 'string — JSON: [x, y]',
        example: 'const pos = JSON.parse(getMousePos())\nprint("鼠标位置: " + pos[0] + ", " + pos[1])',
      },
      {
        name: 'getScreenSize', signature: 'getScreenSize()',
        desc: '获取主屏幕的分辨率，用于计算相对坐标。',
        params: [],
        returns: 'string — JSON: [width, height]',
        example: 'const size = JSON.parse(getScreenSize())\nconst cx = size[0] / 2  // 屏幕中心X\nconst cy = size[1] / 2\nclickMouse(cx, cy, "left")',
      },
      {
        name: 'getClipboard', signature: 'getClipboard()',
        desc: '读取系统剪贴板中的文本内容。',
        params: [],
        returns: 'string — 剪贴板文本，为空时返回空字符串',
        example: 'const text = getClipboard()\nprint("剪贴板内容: " + text)',
      },
      {
        name: 'setClipboard', signature: 'setClipboard(text)',
        desc: '将文本写入系统剪贴板。',
        params: [{ name: 'text', type: 'string', desc: '要写入的文本' }],
        returns: 'void',
        example: 'setClipboard("Hello World")\n// 然后可以用 Ctrl+V 粘贴\nkeyDown("ctrl")\nkeyPress("v")\nkeyUp("ctrl")',
      },
      {
        name: 'msgBox', signature: 'msgBox(text)',
        desc: '弹出系统消息对话框，脚本会暂停直到用户点击确定。适合调试或提示用户。',
        params: [{ name: 'text', type: 'string', desc: '要显示的消息内容' }],
        returns: 'void',
        example: 'msgBox("操作完成！")\n\n// 调试用\nconst hwnd = findWindow("记事本")\nmsgBox("找到窗口句柄: " + hwnd)',
      },
      {
        name: 'enumWindows', signature: 'enumWindows()',
        desc: '枚举所有可见窗口，返回窗口信息列表，可用于查找目标窗口的句柄。',
        params: [],
        returns: 'string — JSON: [{ hwnd, title, class_name, process_name, rect }]',
        example: 'const wins = JSON.parse(enumWindows())\nfor (const w of wins) {\n  if (w.title.includes("Chrome")) {\n    print("找到Chrome: " + w.hwnd)\n    activateWindow(w.hwnd)\n    break\n  }\n}',
      },
    ]
  },
  {
    icon: '⚙️', name: '流程控制',
    apis: [
      {
        name: 'sleep', signature: 'sleep(ms)',
        desc: '暂停脚本执行指定的毫秒数。在操作之间加入适当延迟可以提高稳定性。',
        params: [{ name: 'ms', type: 'number', desc: '暂停时间（毫秒），1000 = 1秒' }],
        returns: 'void',
        example: 'clickMouse(100, 200, "left")\nsleep(500)\nclickMouse(300, 400, "left")',
        notes: '即使在 sleep 期间，点击停止按钮也会立即中断脚本。',
      },
      {
        name: 'isStopped', signature: 'isStopped()',
        desc: '检查用户是否点击了停止按钮。在循环中定期检查，可以让脚本响应停止操作。',
        params: [],
        returns: 'boolean — true 表示已请求停止',
        example: 'while (!isStopped()) {\n  clickMouse(100, 200, "left")\n  sleep(1000)\n}\nprint("脚本已停止")',
      },
      {
        name: 'print', signature: 'print(msg)',
        desc: '将消息输出到脚本执行结果面板，用于调试和显示状态信息。',
        params: [{ name: 'msg', type: 'string', desc: '要输出的消息' }],
        returns: 'void',
        example: 'print("开始执行...")\nconst hwnd = findWindow("记事本")\nprint("窗口句柄: " + hwnd)',
      },
      {
        name: 'charToVkCode', signature: 'charToVkCode(char)',
        desc: '将字符或按键名转换为 Windows 虚拟键码，方便配合 postKey 使用，无需手动查表。',
        params: [{ name: 'char', type: 'string', desc: '单个字符或按键名（如 "a"、"enter"、"f1"）' }],
        returns: 'number — 对应的虚拟键码，无法识别返回 0',
        example: 'const hwnd = findWindow("记事本")\n\n// 用字符名转换，不用记键码\npostKey(hwnd, charToVkCode("enter"))  // Enter 键\npostKey(hwnd, charToVkCode("a"))      // A 键\npostKey(hwnd, charToVkCode("f5"))     // F5 键\npostKey(hwnd, charToVkCode("up"))     // 方向键上\n\n// 批量发送\nfor (const c of ["h","e","l","l","o"]) {\n  postKey(hwnd, charToVkCode(c))\n  sleep(50)\n}',
        notes: '支持的按键名：enter, space, escape/esc, tab, backspace\n方向键：up, down, left, right\n功能键：f1-f12\n字母 a-z、数字 0-9 直接传字符即可',
      },
    ]
  },
]

const filteredCategories = computed(() => {
  if (!search.value) return categories
  const q = search.value.toLowerCase()
  return categories
    .map(cat => ({ ...cat, apis: cat.apis.filter(a => a.name.toLowerCase().includes(q) || a.desc.includes(q)) }))
    .filter(cat => cat.apis.length > 0)
})
</script>

<style scoped>
.api-docs { display: flex; height: 100%; overflow: hidden; font-size: 13px; }

.api-sidebar {
  width: 160px; min-width: 160px; overflow-y: auto;
  border-right: 1px solid #d0eeec; padding: 8px 0; background: #f0fafa;
}
.search-input {
  width: calc(100% - 16px); margin: 4px 8px 8px; padding: 5px 8px;
  border: 1px solid #b2dfdb; border-radius: 4px; font-size: 12px;
  background: white; color: #333; outline: none;
}
.search-input:focus { border-color: #39C5BB; }
.cat-title { padding: 10px 12px 6px; font-size: 13px; color: #1ABC9C; font-weight: bold; cursor: pointer; display: flex; justify-content: space-between; align-items: center; user-select: none; }
.cat-title:hover { background: #e0f7f5; }
.cat-arrow { font-size: 10px; color: #aaa; }
.api-link {
  padding: 5px 12px 5px 20px; cursor: pointer;
}
.api-link:hover { background: #e0f7f5; }
.api-link.active { background: #c8f0ec; border-left: 3px solid #39C5BB; }
.api-link-name { color: #333; font-size: 12px; font-weight: 500; }
.api-link.active .api-link-name { color: #1ABC9C; font-weight: bold; }
.api-link-desc { color: #999; font-size: 11px; margin-top: 1px; }

.api-detail { flex: 1; overflow-y: auto; padding: 24px 28px; color: #333; }
.api-detail.empty { display: flex; align-items: center; justify-content: center; color: #aaa; }

.fn-name { font-family: monospace; font-size: 18px; color: #1ABC9C; margin-bottom: 8px; }
.fn-desc { color: #555; line-height: 1.6; margin-bottom: 16px; }

.section-title { font-size: 11px; text-transform: uppercase; color: #39C5BB; letter-spacing: 1px; margin: 16px 0 8px; border-bottom: 1px solid #d0eeec; padding-bottom: 4px; }

.params-table { width: 100%; border-collapse: collapse; }
.params-table td { padding: 5px 8px; vertical-align: top; border-bottom: 1px solid #f0fafa; }
.pname { font-family: monospace; color: #e67e22; width: 130px; }
.ptype { font-family: monospace; color: #27ae60; width: 200px; font-size: 12px; }
.pdesc { color: #555; }

.returns { font-family: monospace; color: #27ae60; font-size: 12px; }
.example { background: #1e2a2a; color: #e0f7f5; padding: 12px; border-radius: 6px; font-size: 12px; line-height: 1.6; overflow-x: auto; white-space: pre; }
.notes { color: #666; font-size: 12px; line-height: 1.6; background: #f0fafa; padding: 8px 12px; border-left: 3px solid #39C5BB; border-radius: 2px; white-space: pre-line; }
</style>
