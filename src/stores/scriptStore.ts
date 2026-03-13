import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { tauriInvoke } from '../utils/tauri'
import { useConfigStore } from './configStore'

export interface Script {
  id: string
  name: string
  code: string
  hotkey: string | null
  enabled: boolean
  tags?: string[]
  filePath?: string
}

export interface LogEntry {
  scriptName: string
  time: string
  result: string
  success: boolean
}

export const useScriptStore = defineStore('script', () => {
  const scripts = ref<Script[]>([])
  const logs = ref<LogEntry[]>([])
  const runningId = ref<string | null>(null)
  let unlistenLog: (() => void) | null = null

  async function loadScripts() {
    const result = await tauriInvoke<Script[]>('load_scripts')
    if (result) scripts.value = result
  }

  async function saveScript(script: Script) {
    await tauriInvoke('save_script', { script })
  }

  async function deleteScript(id: string) {
    await tauriInvoke('delete_script', { id })
    scripts.value = scripts.value.filter(s => s.id !== id)
  }

  async function runScript(script: Script) {
    runningId.value = script.id
    const configStore = useConfigStore()
    const timeout = configStore.config.scriptTimeout || undefined
    await tauriInvoke('execute_script', { code: script.code, timeout })
  }

  async function stopScript() {
    await tauriInvoke('stop_script')
    runningId.value = null
  }

  function addLog(entry: LogEntry) {
    const configStore = useConfigStore()
    logs.value.unshift(entry)
    if (logs.value.length > configStore.config.maxLogs) {
      logs.value = logs.value.slice(0, configStore.config.maxLogs)
    }
  }

  async function initListeners() {
    unlistenLog = await listen<LogEntry>('script-log', (event) => {
      addLog(event.payload)
      runningId.value = null
    })
  }

  function cleanup() {
    unlistenLog?.()
  }

  return {
    scripts, logs, runningId,
    loadScripts, saveScript, deleteScript,
    runScript, stopScript, addLog,
    initListeners, cleanup,
  }
})
