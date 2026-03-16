<template>
  <div class="editor-page">
    <div class="toolbar">
      <el-button-group>
        <el-button title="新建" @click="handleNew">
          <el-icon><Plus /></el-icon>
        </el-button>
        <el-button title="打开" @click="handleOpen">
          <el-icon><FolderOpened /></el-icon>
        </el-button>
        <el-button type="primary" title="保存" @click="handleSave">
          <el-icon><Document /></el-icon>
        </el-button>
      </el-button-group>
      <el-button-group style="margin-left: 12px">
        <el-button type="success" title="运行" :disabled="isRunning" @click="handleRun">
          <el-icon><CaretRight /></el-icon>
        </el-button>
        <el-button type="danger" title="停止" v-if="isRunning" @click="handleStop">停止</el-button>
        <el-button title="调试" @click="handleDebug">
          <el-icon><VideoPlay /></el-icon>
        </el-button>
      </el-button-group>
      <el-button-group style="margin-left: 12px">
        <el-button title="格式化" @click="handleFormat">
          <el-icon><MagicStick /></el-icon>
        </el-button>
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
          <span>控制台输出</span>
          <div class="console-filters">
            <el-button size="small" :type="consoleFilter === 'all' ? 'primary' : ''" text @click="consoleFilter = 'all'">全部</el-button>
            <el-button size="small" :type="consoleFilter === 'error' ? 'danger' : ''" text @click="consoleFilter = 'error'">错误</el-button>
            <el-button size="small" :type="consoleFilter === 'warn' ? 'warning' : ''" text @click="consoleFilter = 'warn'">警告</el-button>
            <el-button size="small" :type="consoleFilter === 'debug' ? 'info' : ''" text @click="consoleFilter = 'debug'">调试</el-button>
          </div>
          <div class="console-actions">
            <el-button size="small" text @click="copyConsole" title="复制">📋</el-button>
            <el-button size="small" text @click="clearConsole" title="清空">🗑️</el-button>
          </div>
        </div>
        <pre class="console-content" ref="consoleRef"><template v-if="filteredConsoleLines.length">
<template v-for="(line, i) in filteredConsoleLines" :key="i"><span :class="['console-line', line.type]">{{ line.text }}
</span></template></template><template v-else>等待运行...</template></pre>
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
import { lineNumbers, Decoration, type DecorationSet, EditorView, ViewPlugin, type ViewUpdate } from '@codemirror/view'
import { bracketMatching, foldGutter, foldKeymap } from '@codemirror/language'
import { search, searchKeymap } from '@codemirror/search'
import { keymap } from '@codemirror/view'
import { StateEffect, StateField } from '@codemirror/state'
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

onUnmounted(() => {
  unlistenOutput?.()
  unlistenDone?.()
  unlistenMsgBox?.()
  unlistenNotify?.()
  unlistenWarn?.()
  unlistenDebug?.()
})

const handleDebug = async () => {
  if (!currentTab.value) return
  clearConsole()
  clearErrorLine()
  appendConsole('语法检查中...', 'debug')
  try {
    await tauriInvoke('check_script_syntax', { code: currentTab.value.code })
    appendConsole('✓ 语法检查通过', 'timing')
    // 无断点时等同于普通运行
    handleRun()
  } catch (err: any) {
    let error: any
    try { error = typeof err === 'string' ? JSON.parse(err) : err } catch { error = { message: String(err) } }
    const loc = error.line ? ` (第 ${error.line} 行)` : ''
    appendConsole(`✗ 语法错误: ${error.message}${loc}`, 'error')
    if (error.line) highlightErrorLine(error.line)
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
  gap: 2px;
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
</style>
