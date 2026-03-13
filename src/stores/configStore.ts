import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriInvoke } from '../utils/tauri'

export interface AppConfig {
  maxLogs: number
  autoLoad: boolean
  autoHotkey: boolean
  scriptTimeout: number
}

const STORAGE_KEY = 'ahk_rust_config'

const defaults: AppConfig = {
  maxLogs: 100,
  autoLoad: true,
  autoHotkey: false,
  scriptTimeout: 0,
}

export const useConfigStore = defineStore('config', () => {
  const stored = localStorage.getItem(STORAGE_KEY)
  const config = ref<AppConfig>(stored ? { ...defaults, ...JSON.parse(stored) } : { ...defaults })

  function save() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config.value))
  }

  function update(partial: Partial<AppConfig>) {
    Object.assign(config.value, partial)
    save()
  }

  return { config, save, update }
})
