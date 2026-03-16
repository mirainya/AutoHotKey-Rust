<template>
  <div class="editor-page">
    <div class="toolbar">
      <el-button-group>
        <el-tooltip content="新建" placement="bottom" :show-after="500">
          <el-button @click="handleNew">
            <el-icon><Plus /></el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="打开" placement="bottom" :show-after="500">
          <el-button @click="handleOpen">
            <el-icon><FolderOpened /></el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="保存" placement="bottom" :show-after="500">
          <el-button type="primary" @click="handleSave">
            <el-icon><Document /></el-icon>
          </el-button>
        </el-tooltip>
      </el-button-group>
      <el-button-group style="margin-left: 12px">
        <el-tooltip content="运行" placement="bottom" :show-after="500">
          <el-button type="success" :disabled="isRunning" @click="handleRun">
            <el-icon><CaretRight /></el-icon>
          </el-button>
        </el-tooltip>
        <el-button type="danger" title="停止" v-if="isRunning" @click="handleStop">停止</el-button>
        <el-button type="warning" title="继续" v-if="isPaused" @click="handleDebugContinue">继续</el-button>
        <el-button type="info" title="步进" v-if="isPaused" @click="handleStepOver">步进</el-button>
        <el-tooltip content="调试" placement="bottom" :show-after="500">
          <el-button @click="handleDebug">
            <el-icon><VideoPlay /></el-icon>
          </el-button>
        </el-tooltip>
      </el-button-group>
      <el-button-group style="margin-left: 12px">
        <el-tooltip content="格式化" placement="bottom" :show-after="500">
          <el-button @click="handleFormat">
            <el-icon><MagicStick /></el-icon>
          </el-button>
        </el-tooltip>
      </el-button-group>
    </div>
    <div class="editor-container">
      <el-tabs v-model="activeTab" type="card" closable @tab-remove="handleTabRemove" @tab-add="handleNew" class="editor-tabs" :style="{ height: editorHeight + 'px' }">
        <el-tab-pane v-for="tab in tabs" :key="tab.id" :label="tab.name" :name="tab.id">
          <template #label>
            <span @dblclick="handleRename(tab)">{{ tab.name }}</span>
          </template>
          <Codemirror
            v-model="tab.code"
            :extensions="extensions"
            @ready="(payload: any) => { cmView = payload.view }"
          />
        </el-tab-pane>
      </el-tabs>
      <div class="resizer" @mousedown="startResize"></div>
      <div class="console" :style="{ height: consoleHeight + 'px' }">
        <div class="console-header">
          <div class="console-tabs">
            <span class="console-tab" :class="{ active: consoleTab === 'output' }" @click="consoleTab = 'output'">控制台</span>
            <span class="console-tab" :class="{ active: consoleTab === 'vars' }" @click="consoleTab = 'vars'">
              变量
              <span v-if="Object.keys(debugVars).length" class="var-badge">{{ Object.keys(debugVars).length }}</span>
            </span>
          </div>
          <div class="console-filters" v-show="consoleTab === 'output'">
            <el-button size="small" :type="consoleFilter === 'all' ? 'primary' : ''" text @click="consoleFilter = 'all'">全部</el-button>
            <el-button size="small" :type="consoleFilter === 'error' ? 'danger' : ''" text @click="consoleFilter = 'error'">错误</el-button>
            <el-button size="small" :type="consoleFilter === 'warn' ? 'warning' : ''" text @click="consoleFilter = 'warn'">警告</el-button>
            <el-button size="small" :type="consoleFilter === 'debug' ? 'info' : ''" text @click="consoleFilter = 'debug'">调试</el-button>
          </div>
          <div class="console-actions" v-show="consoleTab === 'output'">
            <el-button size="small" text @click="copyConsole" title="复制">📋</el-button>
            <el-button size="small" text @click="clearConsole" title="清空">🗑️</el-button>
          </div>
        </div>
        <pre v-show="consoleTab === 'output'" class="console-content" ref="consoleRef"><template v-if="filteredConsoleLines.length">
<template v-for="(line, i) in filteredConsoleLines" :key="i"><span :class="['console-line', line.type]">{{ line.text }}
</span></template></template><template v-else>等待运行...</template></pre>
        <div v-show="consoleTab === 'vars'" class="vars-panel">
          <div v-if="!isPaused && !Object.keys(debugVars).length" class="vars-empty">
            调试暂停时显示变量信息
          </div>
          <div v-else-if="!Object.keys(debugVars).length" class="vars-empty">
            当前作用域无用户变量
          </div>
          <div v-else class="vars-list">
            <div v-for="(val, key) in debugVars" :key="key" class="var-item">
              <span class="var-name">{{ key }}</span>
              <span class="var-value" :title="val">{{ val }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, watch, onUnmounted, type Ref, computed, shallowRef } from 'vue'
import { Codemirror } from 'vue-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { oneDark } from '@codemirror/theme-one-dark'
import { linter, type Diagnostic } from '@codemirror/lint'
import { lineNumbers, Decoration, type DecorationSet, EditorView, ViewPlugin, type ViewUpdate, gutter, GutterMarker } from '@codemirror/view'
import { bracketMatching, foldGutter, foldKeymap } from '@codemirror/language'
import { search, searchKeymap } from '@codemirror/search'
import { keymap } from '@codemirror/view'
import { StateEffect, StateField, RangeSet } from '@codemirror/state'
import { ahkAutocomplete } from '../utils/ahkCompletions'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, FolderOpened, Document, CaretRight, VideoPlay, MagicStick } from '@element-plus/icons-vue'
import { type Script } from '../stores/scriptStore'
import { useScriptStore } from '../stores/scriptStore'
import { useConfigStore } from '../stores/configStore'
import { tauriInvoke } from '../utils/tauri'

// 断点 gutter
const toggleBreakpoint = StateEffect.define<{ pos: number; on: boolean }>()

class BreakpointMarker extends GutterMarker {
  toDOM() {
    const el = document.createElement('div')
    el.className = 'cm-breakpoint-marker'
    el.textContent = '●'
    return el
  }
}

const breakpointState = StateField.define<RangeSet<GutterMarker>>({
  create() { return RangeSet.empty },
  update(set, tr) {
    set = set.map(tr.changes)
    for (const e of tr.effects) {
      if (e.is(toggleBreakpoint)) {
        if (e.value.on) {
          set = set.update({ add: [new BreakpointMarker().range(e.value.pos)] })
        } else {
          const remove: any[] = []
          const cursor = set.iter(e.value.pos)
          while (cursor.value && cursor.from === e.value.pos) {
            remove.push(cursor)
            cursor.next()
          }
          set = set.update({ filter: (from) => from !== e.value.pos })
        }
      }
    }
    return set
  }
})

const breakpointGutter = gutter({
  class: 'cm-breakpoint-gutter',
  markers: (view) => view.state.field(breakpointState),
  initialSpacer: () => new BreakpointMarker(),
  domEventHandlers: {
    mousedown(view, line) {
      const pos = line.from
      let hasBreakpoint = false
      view.state.field(breakpointState).between(pos, pos, () => { hasBreakpoint = true })
      view.dispatch({ effects: toggleBreakpoint.of({ pos, on: !hasBreakpoint }) })
      return true
    }
  }
})

// 断点行高亮（调试暂停时）
const setActiveLine = StateEffect.define<number | null>()
const activeLineField = StateField.define<DecorationSet>({
  create() { return Decoration.none },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(setActiveLine)) {
        if (e.value === null) return Decoration.none
        const line = tr.state.doc.line(Math.min(e.value, tr.state.doc.lines))
        return Decoration.set([
          Decoration.line({ class: 'cm-active-breakpoint-line' }).range(line.from)
        ])
      }
    }
    return deco
  }
})

// 错误行高亮 decoration
const setErrorLine = StateEffect.define<number | null>()
const errorLineField = StateField.define<DecorationSet>({
  create() { return Decoration.none },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(setErrorLine)) {
        if (e.value === null) return Decoration.none
        const line = tr.state.doc.line(Math.min(e.value, tr.state.doc.lines))
        return Decoration.set([
          Decoration.line({ class: 'cm-error-line' }).range(line.from)
        ])
      }
    }
    return deco
  }
})

// 后端语法检查 linter（防抖 500ms）
const backendLinter = linter(async (view) => {
  const code = view.state.doc.toString()
  if (!code.trim()) return []
  try {
    await tauriInvoke('check_script_syntax', { code })
    return []
  } catch (err: any) {
    const error = typeof err === 'string' ? JSON.parse(err) : err
    if (error.line) {
      const line = Math.min(error.line, view.state.doc.lines)
      const lineObj = view.state.doc.line(line)
      return [{
        from: lineObj.from,
        to: lineObj.to,
        severity: 'error' as const,
        message: error.message || String(err)
      }]
    }
    return [{
      from: 0, to: Math.min(code.length, 1),
      severity: 'error' as const,
      message: error.message || String(err)
    }]
  }
}, { delay: 500 })

const extensions = [
  javascript(),
  oneDark,
  backendLinter,
  errorLineField,
  breakpointState,
  breakpointGutter,
  activeLineField,
  ahkAutocomplete,
  lineNumbers(),
  bracketMatching(),
  foldGutter(),
  search(),
  keymap.of([...foldKeymap, ...searchKeymap]),
]

const currentView = inject<Ref<string>>('currentView')!
const currentEditingScript = inject<Ref<Script | null>>('currentEditingScript')!

const tabs = ref<Script[]>([{
  id: Date.now().toString(),
  name: '新脚本',
  code: '// 在这里编写JavaScript代码\n',
  hotkey: null,
  enabled: true
}])

const activeTab = ref(tabs.value[0].id)
const consoleOutput = ref('')
const consoleRef = ref<HTMLPreElement>()

interface ConsoleLine { text: string; type: 'info' | 'warn' | 'error' | 'debug' | 'timing' }
const consoleLines = ref<ConsoleLine[]>([])
const consoleFilter = ref<string>('all')
const consoleTab = ref<'output' | 'vars'>('output')

function timestamp() {
  const d = new Date()
  return `[${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}:${String(d.getSeconds()).padStart(2,'0')}]`
}

function appendConsole(msg: string, type: ConsoleLine['type'] = 'info') {
  consoleLines.value.push({ text: `${timestamp()} ${msg}`, type })
  // 自动滚动到底部
  setTimeout(() => {
    if (consoleRef.value) consoleRef.value.scrollTop = consoleRef.value.scrollHeight
  }, 0)
}

const clearConsole = () => { consoleLines.value = [] }
const filteredConsoleLines = computed(() => {
  if (consoleFilter.value === 'all') return consoleLines.value
  return consoleLines.value.filter(l => l.type === consoleFilter.value)
})
const copyConsole = () => {
  const text = consoleLines.value.map(l => l.text).join('\n')
  navigator.clipboard.writeText(text)
  ElMessage.success('已复制到剪贴板')
}

const editorHeight = ref(400)
const consoleHeight = ref(200)
const isRunning = ref(false)
let unlistenOutput: (() => void) | null = null
let unlistenDone: (() => void) | null = null
let unlistenMsgBox: (() => void) | null = null
let unlistenNotify: (() => void) | null = null
let unlistenWarn: (() => void) | null = null
let unlistenDebug: (() => void) | null = null

// 监听 msgBox / notify 事件
listen<{ title: string; text: string }>('script-msgbox', (e) => {
  ElMessageBox.alert(e.payload.text, e.payload.title, { confirmButtonText: '确定' })
}).then(fn => { unlistenMsgBox = fn })

listen<{ title: string; text: string }>('script-notify', (e) => {
  ElMessage({ message: `${e.payload.title}: ${e.payload.text}`, type: 'info', duration: 5000 })
}).then(fn => { unlistenNotify = fn })

// 监听 warn / debug 事件
listen<string>('script-output-warn', (e) => {
  appendConsole(e.payload, 'warn')
}).then(fn => { unlistenWarn = fn })

listen<string>('script-output-debug', (e) => {
  appendConsole(e.payload, 'debug')
}).then(fn => { unlistenDebug = fn })

const currentTab = computed(() => tabs.value.find(t => t.id === activeTab.value))

const startResize = (e: MouseEvent) => {
  const startY = e.clientY
  const startEditorHeight = editorHeight.value
  const startConsoleHeight = consoleHeight.value

  const onMouseMove = (e: MouseEvent) => {
    const delta = e.clientY - startY
    editorHeight.value = Math.max(200, startEditorHeight + delta)
    consoleHeight.value = Math.max(100, startConsoleHeight - delta)
  }

  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

watch(currentEditingScript, (script) => {
  if (script) {
    const existingTab = tabs.value.find(t => t.id === script.id)
    if (existingTab) {
      activeTab.value = existingTab.id
    } else {
      tabs.value.push({ ...script })
      activeTab.value = script.id
    }
  }
}, { immediate: true })

const handleSave = async () => {
  if (!currentTab.value) return
  try {
    if (!currentTab.value.filePath) {
      const file = await save({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        defaultPath: currentTab.value.name + '.json'
      })
      if (!file) return
      currentTab.value.filePath = file
      currentTab.value.name = file.split(/[/\\]/).pop()?.replace('.json', '') || currentTab.value.name
    }
    const scriptStore = useScriptStore()
    await scriptStore.saveScript(currentTab.value)
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  }
}

const handleNew = () => {
  const newTab: Script = {
    id: Date.now().toString(),
    name: '新脚本',
    code: '// 在这里编写JavaScript代码\n',
    hotkey: null,
    enabled: true
  }
  tabs.value.push(newTab)
  activeTab.value = newTab.id
}

const handleOpen = async () => {
  try {
    const file = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }]
    })
    if (file) {
      const content = await readTextFile(file as string)
      const script = JSON.parse(content) as Script
      const existingTab = tabs.value.find(t => t.id === script.id)
      if (existingTab) {
        activeTab.value = existingTab.id
        ElMessage.info('脚本已打开')
      } else {
        script.filePath = file as string
        tabs.value.push(script)
        activeTab.value = script.id
        ElMessage.success('打开成功')
      }
    }
  } catch (error) {
    ElMessage.error(`打开失败: ${error}`)
  }
}

// 保存 EditorView 引用，用于错误行高亮
const cmView = shallowRef<EditorView | null>(null)

function clearErrorLine() {
  if (cmView.value) {
    cmView.value.dispatch({ effects: setErrorLine.of(null) })
  }
}

function highlightErrorLine(line: number) {
  if (cmView.value && line > 0) {
    cmView.value.dispatch({ effects: setErrorLine.of(line) })
  }
}

const handleRun = async () => {
  if (!currentTab.value) return
  clearConsole()
  clearErrorLine()
  isRunning.value = true

  unlistenOutput = await listen<string>('script-output', (e) => {
    const isError = /^\[错误\]|^\[超时\]|^Error/i.test(e.payload)
    appendConsole(e.payload, isError ? 'error' : 'info')
  })

  interface ScriptDonePayload {
    success: boolean
    elapsed_ms?: number
    error?: { message: string; line?: number; column?: number }
  }

  unlistenDone = await listen<ScriptDonePayload>('script-done', (e) => {
    const p = e.payload
    if (p.success) {
      appendConsole(`✓ 执行完成 (${p.elapsed_ms ?? 0}ms)`, 'timing')
    } else {
      const err = p.error
      if (err) {
        const loc = err.line ? ` (第 ${err.line} 行)` : ''
        appendConsole(`✗ ${err.message}${loc} (${p.elapsed_ms ?? 0}ms)`, 'error')
        if (err.line) highlightErrorLine(err.line)
      }
    }
    isRunning.value = false
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
  })

  try {
    const configStore = useConfigStore()
    const timeout = configStore.config.scriptTimeout || undefined
    await tauriInvoke('execute_script', { code: currentTab.value.code, timeout })
  } catch (error) {
    appendConsole(`错误: ${error}`, 'error')
    isRunning.value = false
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
  }
}

const handleStop = async () => {
  await tauriInvoke('stop_script')
}

// 从用户代码中提取 var/let/const 声明的变量名
function extractVarNames(code: string): string[] {
  const names = new Set<string>()
  const re = /\b(?:var|let|const)\s+([a-zA-Z_$][\w$]*)/g
  let m: RegExpExecArray | null
  while ((m = re.exec(code)) !== null) {
    names.add(m[1])
  }
  // 也捕获 for 循环中的解构: for (let i = ...)
  // 以及多变量声明: var a = 1, b = 2
  const multiRe = /\b(?:var|let|const)\s+(.+?)(?:;|$)/gm
  while ((m = multiRe.exec(code)) !== null) {
    const decl = m[1]
    const varRe = /([a-zA-Z_$][\w$]*)\s*(?:=|,|$)/g
    let vm: RegExpExecArray | null
    while ((vm = varRe.exec(decl)) !== null) {
      names.add(vm[1])
    }
  }
  return [...names]
}

onUnmounted(() => {
  unlistenOutput?.()
  unlistenDone?.()
  unlistenMsgBox?.()
  unlistenNotify?.()
  unlistenWarn?.()
  unlistenDebug?.()
  unlistenBreakpointHit?.()
  unlistenBreakpointVars?.()
})

// 获取当前编辑器中所有断点的行号
function getBreakpointLines(): number[] {
  if (!cmView.value) return []
  const lines: number[] = []
  const bpSet = cmView.value.state.field(breakpointState)
  const cursor = bpSet.iter()
  while (cursor.value) {
    const lineNum = cmView.value.state.doc.lineAt(cursor.from).number
    lines.push(lineNum)
    cursor.next()
  }
  return lines.sort((a, b) => a - b)
}

function clearActiveLine() {
  if (cmView.value) {
    cmView.value.dispatch({ effects: setActiveLine.of(null) })
  }
}

function highlightActiveLine(line: number) {
  if (cmView.value && line > 0) {
    cmView.value.dispatch({ effects: setActiveLine.of(line) })
  }
}

const isPaused = ref(false)
const debugVars = ref<Record<string, string>>({})
let unlistenBreakpointHit: (() => void) | null = null
let unlistenBreakpointVars: (() => void) | null = null

const handleDebugContinue = async () => {
  clearActiveLine()
  isPaused.value = false
  appendConsole('▶ 继续执行...', 'debug')
  await tauriInvoke('debug_continue')
}

const handleStepOver = async () => {
  clearActiveLine()
  isPaused.value = false
  appendConsole('⏭ 步进...', 'debug')
  await tauriInvoke('debug_step_over')
}

const handleDebug = async () => {
  if (!currentTab.value) return
  const bpLines = getBreakpointLines()
  if (bpLines.length === 0) {
    handleRun()
    return
  }

  clearConsole()
  clearErrorLine()
  clearActiveLine()
  debugVars.value = {}
  appendConsole('语法检查中...', 'debug')
  try {
    await tauriInvoke('check_script_syntax', { code: currentTab.value.code })
    appendConsole('✓ 语法检查通过', 'timing')
  } catch (err: any) {
    let error: any
    try { error = typeof err === 'string' ? JSON.parse(err) : err } catch { error = { message: String(err) } }
    const loc = error.line ? ` (第 ${error.line} 行)` : ''
    appendConsole(`✗ 语法错误: ${error.message}${loc}`, 'error')
    if (error.line) highlightErrorLine(error.line)
    return
  }

  // 从用户代码中提取变量名，生成显式捕获的 __snapshot 覆盖
  const varNames = extractVarNames(currentTab.value.code)
  const snapshotOverride = varNames.length > 0
    ? `__snapshot = function() { var r = {}; ${varNames.map(v =>
        `try { r["${v}"] = (typeof ${v} === 'object' && ${v} !== null) ? JSON.stringify(${v}) : String(${v}); } catch(e) {}`
      ).join(' ')} return JSON.stringify(r); };`
    : ''

  // 在每一行前注入 __step(lineNum) 调用（跳过空行和纯注释行）
  const lines = currentTab.value.code.split('\n')
  const injected = (snapshotOverride ? snapshotOverride + '\n' : '') + lines.map((line, i) => {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('//')) return line
    return `__step(${i + 1});\n${line}`
  }).join('\n')

  appendConsole(`⬤ 已设置 ${bpLines.length} 个断点: 行 ${bpLines.join(', ')}`, 'debug')
  isRunning.value = true

  // 监听断点命中事件
  unlistenBreakpointHit = await listen<number>('script-breakpoint-hit', (e) => {
    isPaused.value = true
    highlightActiveLine(e.payload)
    appendConsole(`⬤ 断点命中: 第 ${e.payload} 行`, 'debug')
  })

  // 监听变量快照事件
  unlistenBreakpointVars = await listen<string>('script-breakpoint-vars', (e) => {
    try {
      debugVars.value = JSON.parse(e.payload)
    } catch {
      debugVars.value = {}
    }
  })

  unlistenOutput = await listen<string>('script-output', (e) => {
    const isError = /^\[错误\]|^\[超时\]|^Error/i.test(e.payload)
    appendConsole(e.payload, isError ? 'error' : 'info')
  })

  interface ScriptDonePayload {
    success: boolean
    elapsed_ms?: number
    error?: { message: string; line?: number; column?: number }
  }

  unlistenDone = await listen<ScriptDonePayload>('script-done', (e) => {
    const p = e.payload
    if (p.success) {
      appendConsole(`✓ 调试执行完成 (${p.elapsed_ms ?? 0}ms)`, 'timing')
    } else {
      const err = p.error
      if (err) {
        const loc = err.line ? ` (第 ${err.line} 行)` : ''
        appendConsole(`✗ ${err.message}${loc} (${p.elapsed_ms ?? 0}ms)`, 'error')
        if (err.line) highlightErrorLine(err.line)
      }
    }
    isRunning.value = false
    isPaused.value = false
    debugVars.value = {}
    clearActiveLine()
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
    unlistenBreakpointHit?.(); unlistenBreakpointHit = null
    unlistenBreakpointVars?.(); unlistenBreakpointVars = null
  })

  try {
    const configStore = useConfigStore()
    const timeout = configStore.config.scriptTimeout || undefined
    await tauriInvoke('execute_script', { code: injected, timeout, breakpointLines: bpLines })
  } catch (error) {
    appendConsole(`错误: ${error}`, 'error')
    isRunning.value = false
    isPaused.value = false
    debugVars.value = {}
    clearActiveLine()
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
    unlistenBreakpointHit?.(); unlistenBreakpointHit = null
    unlistenBreakpointVars?.(); unlistenBreakpointVars = null
  }
}

const handleFormat = () => {
  if (!currentTab.value) return
  try {
    currentTab.value.code = currentTab.value.code
      .split('\n')
      .filter((line, index, arr) => {
        if (line.trim() === '') {
          return index === 0 || index === arr.length - 1 || arr[index - 1].trim() !== ''
        }
        return true
      })
      .join('\n')
    ElMessage.success('代码已格式化')
  } catch (error) {
    ElMessage.error('格式化失败')
  }
}

const handleTabRemove = (tabId: string) => {
  const index = tabs.value.findIndex(t => t.id === tabId)
  if (index === -1) return

  tabs.value.splice(index, 1)

  if (tabs.value.length === 0) {
    handleNew()
  } else if (activeTab.value === tabId) {
    activeTab.value = tabs.value[Math.max(0, index - 1)].id
  }
}

const handleRename = async (tab: Script) => {
  try {
    const { value } = await ElMessageBox.prompt('请输入新名称', '重命名', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputValue: tab.name
    })
    if (value && value.trim()) {
      tab.name = value.trim()
      if (currentEditingScript.value?.id === tab.id) {
        currentEditingScript.value.name = tab.name
      }
      await tauriInvoke('save_script', { script: tab })
      ElMessage.success('重命名成功')
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(`重命名失败: ${error}`)
    }
  }
}
</script>

<style scoped>
.editor-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #282c34;
}

.toolbar {
  padding: 12px 16px;
  background: #21252b;
  border-bottom: 1px solid #181a1f;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.resizer {
  height: 4px;
  background: #181a1f;
  cursor: ns-resize;
  transition: background 0.2s;
}

.resizer:hover {
  background: var(--miku-primary);
}

.console {
  background: #1e1e1e;
  border-top: 1px solid #181a1f;
  display: flex;
  flex-direction: column;
}

.console-header {
  padding: 8px 16px;
  background: #252526;
  color: #cccccc;
  font-size: 12px;
  font-weight: 600;
  border-bottom: 1px solid #181a1f;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.console-filters {
  display: flex;
  gap: 4px;
}

.console-actions {
  display: flex;
  gap: 4px;
}

.console-content {
  flex: 1;
  padding: 12px;
  margin: 0;
  color: #d4d4d4;
  font-family: 'Consolas', monospace;
  font-size: 13px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.console-line.error {
  color: #f56c6c;
}

.console-line.warn {
  color: #e6a23c;
}

.console-line.debug {
  color: #909399;
}

.console-line.timing {
  color: #67c23a;
}

.console-line.info {
  color: #d4d4d4;
}

:deep(.el-tabs) {
  flex: 1;
  display: flex;
  flex-direction: column;
}

:deep(.el-tabs__item) {
  color: #999 !important;
}

:deep(.el-tabs__item.is-active) {
  color: #fff !important;
}

:deep(.el-tabs__content) {
  flex: 1;
  overflow: hidden;
}

:deep(.el-tab-pane) {
  height: 100%;
}

:deep(.cm-editor) {
  height: 100%;
}

:deep(.cm-scroller) {
  font-family: 'Consolas', 'Microsoft YaHei', 'SimHei', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
}

:deep(.cm-error-line) {
  background: rgba(245, 108, 108, 0.15);
}

:deep(.cm-breakpoint-gutter) {
  width: 16px;
  cursor: pointer;
}

:deep(.cm-breakpoint-marker) {
  color: var(--el-color-danger);
  font-size: 10px;
  line-height: 1.6;
  text-align: center;
}

:deep(.cm-active-breakpoint-line) {
  background: rgba(255, 193, 7, 0.2);
}

/* 控制台 tab 切换 */
.console-tabs {
  display: flex;
  gap: 2px;
}

.console-tab {
  padding: 2px 12px;
  cursor: pointer;
  border-radius: 4px 4px 0 0;
  color: #888;
  font-size: 12px;
  transition: color 0.2s, background 0.2s;
  user-select: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.console-tab:hover {
  color: #ccc;
}

.console-tab.active {
  color: var(--miku-primary, #39c5bb);
  background: rgba(57, 197, 187, 0.1);
}

.var-badge {
  background: var(--miku-primary, #39c5bb);
  color: #fff;
  font-size: 10px;
  padding: 0 5px;
  border-radius: 8px;
  line-height: 16px;
  min-width: 16px;
  text-align: center;
}

/* 变量面板 */
.vars-panel {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.vars-empty {
  color: #666;
  font-size: 13px;
  text-align: center;
  padding: 24px;
}

.vars-list {
  display: flex;
  flex-direction: column;
}

.var-item {
  display: flex;
  padding: 4px 16px;
  font-family: 'Consolas', monospace;
  font-size: 13px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  transition: background 0.15s;
}

.var-item:hover {
  background: rgba(57, 197, 187, 0.06);
}

.var-name {
  color: #e06c75;
  min-width: 120px;
  flex-shrink: 0;
  padding-right: 12px;
}

.var-value {
  color: #98c379;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
