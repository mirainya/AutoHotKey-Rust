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
          />
        </el-tab-pane>
      </el-tabs>
      <div class="resizer" @mousedown="startResize"></div>
      <div class="console" :style="{ height: consoleHeight + 'px' }">
        <div class="console-header">控制台输出</div>
        <pre class="console-content">{{ consoleOutput || '等待运行...' }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, watch, onUnmounted, type Ref, computed } from 'vue'
import { Codemirror } from 'vue-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { oneDark } from '@codemirror/theme-one-dark'
import { linter, type Diagnostic } from '@codemirror/lint'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, FolderOpened, Document, CaretRight, VideoPlay, MagicStick } from '@element-plus/icons-vue'

// 简单的JavaScript语法检查
const jsLinter = linter((view) => {
  const diagnostics: Diagnostic[] = []
  const code = view.state.doc.toString()

  // 检查未闭合的括号
  const openBrackets = (code.match(/\{/g) || []).length
  const closeBrackets = (code.match(/\}/g) || []).length
  if (openBrackets !== closeBrackets) {
    diagnostics.push({
      from: 0,
      to: code.length,
      severity: 'error',
      message: '括号未闭合'
    })
  }

  return diagnostics
})

const extensions = [javascript(), oneDark, jsLinter]

interface Script {
  id: string
  name: string
  code: string
  hotkey: string | null
  enabled: boolean
  filePath?: string
}

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
const editorHeight = ref(400)
const consoleHeight = ref(200)
const isRunning = ref(false)
let unlistenOutput: (() => void) | null = null
let unlistenDone: (() => void) | null = null

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
    await invoke('save_script', { script: currentTab.value })
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

const handleRun = async () => {
  if (!currentTab.value) return
  consoleOutput.value = ''
  isRunning.value = true

  unlistenOutput = await listen<string>('script-output', (e) => {
    consoleOutput.value += e.payload + '\n'
  })
  unlistenDone = await listen<string>('script-done', (e) => {
    if (e.payload) consoleOutput.value += e.payload
    isRunning.value = false
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
  })

  try {
    await invoke('execute_script', { code: currentTab.value.code })
  } catch (error) {
    consoleOutput.value = `错误: ${error}`
    isRunning.value = false
    unlistenOutput?.(); unlistenOutput = null
    unlistenDone?.(); unlistenDone = null
  }
}

const handleStop = async () => {
  await invoke('stop_script')
}

onUnmounted(() => {
  unlistenOutput?.()
  unlistenDone?.()
})

const handleDebug = () => {
  if (!currentTab.value) return
  console.log('=== 调试信息 ===')
  console.log('脚本内容:', currentTab.value.code)
  console.log('脚本长度:', currentTab.value.code.length)
  ElMessage.info('调试信息已输出到控制台')
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
      await invoke('save_script', { script: tab })
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
  background: #39C5BB;
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
</style>
