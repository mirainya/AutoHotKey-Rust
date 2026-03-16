<template>
  <div class="script-manager">
    <div class="header">
      <h2 class="page-title">📝 脚本管理</h2>
      <div class="header-buttons">
        <el-input
          v-model="searchQuery"
          placeholder="搜索脚本..."
          clearable
          size="small"
          style="width: 200px"
          prefix-icon="Search"
        />
        <el-button @click="showLogsDialog = true">📋 查看运行日志</el-button>
        <el-button @click="importScript">📥 导入</el-button>
        <el-button type="primary" @click="addScript">+ 添加脚本</el-button>
      </div>
    </div>

    <el-table :data="filteredScripts" style="width: 100%">
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
      <el-table-column label="操作" width="220">
        <template #default="{ row }">
          <el-button size="small" @click="editScript(row)">编辑</el-button>
          <el-button
            v-if="store.runningId === row.id"
            size="small"
            type="warning"
            @click="store.stopScript()"
          >停止</el-button>
          <el-button
            v-else
            size="small"
            :loading="store.runningId === row.id"
            @click="runScript(row)"
          >运行</el-button>
          <el-dropdown trigger="click" @command="(cmd: string) => handleMoreCommand(cmd, row)">
            <el-button size="small" text>···</el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="export">📤 导出</el-dropdown-item>
                <el-dropdown-item command="delete" divided>🗑️ 删除</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="showLogsDialog" title="📋 运行日志" width="800px">
      <el-table :data="store.logs" style="width: 100%" max-height="400">
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
import { ref, computed, onMounted, onUnmounted, inject, type Ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { save, open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { useScriptStore, type Script } from '../stores/scriptStore'
import { tauriInvoke } from '../utils/tauri'

const store = useScriptStore()
const showLogsDialog = ref(false)
const searchQuery = ref('')
const currentView = inject<Ref<string>>('currentView')!
const currentEditingScript = inject<Ref<Script | null>>('currentEditingScript')!

const filteredScripts = computed(() => {
  if (!searchQuery.value) return store.scripts
  const q = searchQuery.value.toLowerCase()
  return store.scripts.filter(s =>
    s.name.toLowerCase().includes(q) ||
    (s.hotkey && s.hotkey.toLowerCase().includes(q))
  )
})

const saveScript = async (script: Script) => {
  await store.saveScript(script)
  if (script.enabled && script.hotkey) {
    await tauriInvoke('register_hotkey', { hotkey: script.hotkey, scriptId: script.id })
  } else if (!script.enabled && script.hotkey) {
    await tauriInvoke('unregister_hotkey', { hotkey: script.hotkey })
  }
  ElMessage.success('保存成功')
}

const editScript = (script: Script) => {
  currentEditingScript.value = { ...script }
  currentView.value = 'editor'
}

const addScript = () => {
  currentEditingScript.value = {
    id: Date.now().toString(),
    name: '',
    code: '',
    hotkey: null,
    enabled: false,
  }
  currentView.value = 'editor'
}

const runScript = async (script: Script) => {
  await store.runScript(script)
}

const deleteScript = async (script: Script) => {
  await store.deleteScript(script.id)
  ElMessage.success('删除成功')
}

const exportScript = async (script: Script) => {
  const path = await save({
    defaultPath: `${script.name}.js`,
    filters: [{ name: 'JavaScript', extensions: ['js'] }],
  })
  if (path) {
    await tauriInvoke('export_script', { id: script.id, path })
    ElMessage.success('导出成功')
  }
}

const importScript = async () => {
  const path = await open({
    filters: [{ name: 'JavaScript', extensions: ['js'] }],
    multiple: false,
  })
  if (path) {
    const name = String(path).split(/[/\\]/).pop()?.replace('.js', '') || 'imported'
    await tauriInvoke('import_script', { path: String(path), name })
    await store.loadScripts()
    ElMessage.success('导入成功')
  }
}

const handleMoreCommand = (command: string, script: Script) => {
  if (command === 'export') exportScript(script)
  else if (command === 'delete') deleteScript(script)
}

const recordHotkey = (event: KeyboardEvent, script: Script) => {
  event.preventDefault()
  const oldHotkey = script.hotkey
  const keys: string[] = []
  if (event.ctrlKey) keys.push('Ctrl')
  if (event.shiftKey) keys.push('Shift')
  if (event.altKey) keys.push('Alt')
  if (event.key && !['Control', 'Shift', 'Alt'].includes(event.key)) {
    keys.push(event.key.toUpperCase())
  }
  if (keys.length > 1) {
    script.hotkey = keys.join('+')
    if (script.enabled && oldHotkey && oldHotkey !== script.hotkey) {
      tauriInvoke('unregister_hotkey', { hotkey: oldHotkey })
    }
  }
}

let unlistenHotkey: (() => void) | null = null

onMounted(async () => {
  await store.loadScripts()
  await store.initListeners()
  await tauriInvoke('start_hotkey_listener')

  // 注册已启用脚本的热键
  for (const script of store.scripts) {
    if (script.enabled && script.hotkey) {
      await tauriInvoke('register_hotkey', { hotkey: script.hotkey, scriptId: script.id })
    }
  }

  unlistenHotkey = await listen<[string, string]>('hotkey-triggered', (event) => {
    const [, scriptId] = event.payload
    const script = store.scripts.find(s => s.id === scriptId)
    if (script) runScript(script)
  })
})

onUnmounted(() => {
  unlistenHotkey?.()
  store.cleanup()
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

.page-title {
  margin: 0;
  color: var(--miku-primary);
  font-size: 24px;
  font-weight: 600;
  position: relative;
  display: inline-block;
  padding-bottom: 10px;
}

.page-title::after {
  content: '';
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  height: 2px;
  background: linear-gradient(90deg, var(--miku-primary), var(--miku-light), transparent);
  border-radius: 1px;
}

.header-buttons {
  display: flex;
  gap: 12px;
}

:deep(.el-table) {
  background: rgba(255, 255, 255, 0.95);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

:deep(.el-table th) {
  background: linear-gradient(135deg, var(--miku-light), var(--miku-primary));
  color: white;
  font-weight: 600;
}

:deep(.el-input__inner) {
  border-radius: var(--radius-sm);
}

:deep(.el-button--primary) {
  background: linear-gradient(135deg, var(--miku-primary), var(--miku-dark));
  border: none;
}

:deep(.el-button--primary:hover) {
  background: linear-gradient(135deg, var(--miku-light), var(--miku-primary));
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}
</style>
