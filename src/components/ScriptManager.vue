<template>
  <div class="script-manager">
    <div class="header">
      <h2>📝 脚本管理</h2>
      <div class="header-buttons">
        <el-button @click="showLogsDialog = true">📋 查看运行日志</el-button>
        <el-button type="primary" @click="addScript">+ 添加脚本</el-button>
      </div>
    </div>

    <el-table :data="scripts" style="width: 100%">
      <el-table-column prop="name" label="脚本名称" width="200" />
      <el-table-column label="快捷键" width="200">
        <template #default="{ row }">
          <el-input
            v-model="row.hotkey"
            placeholder="点击录制快捷键"
            @keydown="recordHotkey($event, row)"
            @blur="saveScript(row)"
            size="small"
          />
        </template>
      </el-table-column>
      <el-table-column label="状态" width="100">
        <template #default="{ row }">
          <el-switch v-model="row.enabled" @change="saveScript(row)" />
        </template>
      </el-table-column>
      <el-table-column label="操作">
        <template #default="{ row }">
          <el-button size="small" @click="editScript(row)">编辑</el-button>
          <el-button size="small" :loading="runningId === row.id" @click="runScript(row)">运行</el-button>
          <el-button size="small" type="warning" v-if="runningId === row.id" @click="stopScript">停止</el-button>
          <el-button size="small" type="danger" @click="deleteScript(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="showLogsDialog" title="📋 运行日志" width="800px">
      <el-table :data="logs" style="width: 100%" max-height="400">
        <el-table-column prop="scriptName" label="脚本名称" width="200" />
        <el-table-column prop="time" label="运行时间" width="150" />
        <el-table-column prop="result" label="结果">
          <template #default="{ row }">
            <span :style="{ color: row.success ? '#67C23A' : '#F56C6C' }">{{ row.result }}</span>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'

interface Script {
  id: string
  name: string
  code: string
  hotkey: string | null
  enabled: boolean
  filePath?: string
}

interface LogEntry {
  scriptName: string
  time: string
  result: string
  success: boolean
}

const scripts = ref<Script[]>([])
const logs = ref<LogEntry[]>([])
const showLogsDialog = ref(false)
const runningId = ref<string | null>(null)
const currentView = inject<Ref<string>>('currentView')!
const currentEditingScript = inject<Ref<Script | null>>('currentEditingScript')!

const loadScripts = async () => {
  try {
    scripts.value = await invoke<Script[]>('load_scripts')
    for (const script of scripts.value) {
      if (script.enabled && script.hotkey) {
        await invoke('register_hotkey', { hotkey: script.hotkey, scriptId: script.id })
      }
    }
  } catch (error) {
    ElMessage.error(`加载失败: ${error}`)
  }
}

const saveScript = async (script: Script) => {
  try {
    await invoke('save_script', { script })
    if (script.enabled && script.hotkey) {
      await invoke('register_hotkey', { hotkey: script.hotkey, scriptId: script.id })
    } else if (!script.enabled && script.hotkey) {
      await invoke('unregister_hotkey', { hotkey: script.hotkey })
    }
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  }
}

const editScript = (script: Script) => {
  currentEditingScript.value = {
    ...script,
    filePath: script.filePath || `scripts/${script.id}.json`
  }
  currentView.value = 'editor'
}

const addScript = () => {
  currentEditingScript.value = {
    id: Date.now().toString(),
    name: '',
    code: '',
    hotkey: null,
    enabled: false
  }
  currentView.value = 'editor'
}

const runScript = async (script: Script) => {
  const startTime = new Date().toLocaleTimeString()
  runningId.value = script.id

  const unlistenDone = await listen<string>('script-done', (e) => {
    unlistenDone()
    const output = e.payload || '执行成功'
    const success = !output.startsWith('错误:')
    logs.value.unshift({ scriptName: script.name, time: startTime, result: output, success })
    if (runningId.value === script.id) runningId.value = null
  })

  try {
    await invoke('execute_script', { code: script.code })
  } catch (error) {
    unlistenDone()
    logs.value.unshift({ scriptName: script.name, time: startTime, result: `错误: ${error}`, success: false })
    runningId.value = null
  }
}

const stopScript = async () => {
  await invoke('stop_script')
}

const deleteScript = async (script: Script) => {
  try {
    await invoke('delete_script', { id: script.id })
    await loadScripts()
    ElMessage.success('删除成功')
  } catch (error) {
    ElMessage.error(`删除失败: ${error}`)
  }
}

const recordHotkey = (event: KeyboardEvent, script: Script) => {
  event.preventDefault()
  const oldHotkey = script.hotkey
  const keys = []
  if (event.ctrlKey) keys.push('Ctrl')
  if (event.shiftKey) keys.push('Shift')
  if (event.altKey) keys.push('Alt')
  if (event.key && !['Control', 'Shift', 'Alt'].includes(event.key)) {
    keys.push(event.key.toUpperCase())
  }
  if (keys.length > 1) {
    script.hotkey = keys.join('+')
    if (script.enabled && oldHotkey && oldHotkey !== script.hotkey) {
      invoke('unregister_hotkey', { hotkey: oldHotkey })
    }
  }
}

onMounted(async () => {
  await loadScripts()
  await invoke('start_hotkey_listener')

  const unlisten = await listen('hotkey-triggered', (event: any) => {
    console.log('收到热键事件:', event.payload)
    const [hotkey, scriptId] = event.payload
    console.log('热键:', hotkey, '脚本ID:', scriptId)
    const script = scripts.value.find(s => s.id === scriptId)
    console.log('找到的脚本:', script)
    if (script) {
      console.log('执行脚本:', script.name)
      runScript(script)
    } else {
      console.log('未找到脚本，当前脚本列表:', scripts.value.map(s => ({ id: s.id, name: s.name })))
    }
  })

  onUnmounted(() => {
    unlisten()
  })
})
</script>

<style scoped>
.script-manager {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 24px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h2 {
  margin: 0;
  color: #39C5BB;
  font-size: 24px;
  font-weight: 600;
}

.header-buttons {
  display: flex;
  gap: 12px;
}

:deep(.el-table) {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  overflow: hidden;
}

:deep(.el-table th) {
  background: linear-gradient(135deg, #7FDBDA, #39C5BB);
  color: white;
  font-weight: 600;
}

:deep(.el-input__inner) {
  border-radius: 6px;
}

:deep(.el-button--primary) {
  background: linear-gradient(135deg, #39C5BB, #1ABC9C);
  border: none;
}

:deep(.el-button--primary:hover) {
  background: linear-gradient(135deg, #7FDBDA, #39C5BB);
}
</style>
