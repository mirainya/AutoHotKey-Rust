import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock localStorage
const storageMap = new Map<string, string>()
const mockLocalStorage = {
  getItem: vi.fn((key: string) => storageMap.get(key) ?? null),
  setItem: vi.fn((key: string, value: string) => { storageMap.set(key, value) }),
  removeItem: vi.fn((key: string) => { storageMap.delete(key) }),
  clear: vi.fn(() => { storageMap.clear() }),
  get length() { return storageMap.size },
  key: vi.fn((i: number) => [...storageMap.keys()][i] ?? null),
}
Object.defineProperty(globalThis, 'localStorage', { value: mockLocalStorage, writable: true })

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock element-plus
vi.mock('element-plus', () => ({
  ElMessage: { error: vi.fn(), success: vi.fn(), info: vi.fn() },
}))

// Mock tauriInvoke
vi.mock('../utils/tauri', () => ({
  tauriInvoke: vi.fn(),
}))

import { useConfigStore } from '../stores/configStore'
import { useScriptStore, type Script } from '../stores/scriptStore'
import { tauriInvoke } from '../utils/tauri'

const mockedTauriInvoke = vi.mocked(tauriInvoke)

beforeEach(() => {
  storageMap.clear()
  vi.clearAllMocks()
  setActivePinia(createPinia())
})

describe('configStore', () => {
  it('初始化默认配置', () => {
    const store = useConfigStore()

    expect(store.config.maxLogs).toBe(100)
    expect(store.config.autoLoad).toBe(true)
    expect(store.config.autoHotkey).toBe(false)
    expect(store.config.scriptTimeout).toBe(0)
  })

  it('从 localStorage 恢复配置', () => {
    storageMap.set('ahk_rust_config', JSON.stringify({ maxLogs: 50, scriptTimeout: 5000 }))

    const store = useConfigStore()

    expect(store.config.maxLogs).toBe(50)
    expect(store.config.scriptTimeout).toBe(5000)
    expect(store.config.autoLoad).toBe(true)
  })

  it('update 部分更新并持久化', () => {
    const store = useConfigStore()

    store.update({ maxLogs: 200, autoHotkey: true })

    expect(store.config.maxLogs).toBe(200)
    expect(store.config.autoHotkey).toBe(true)

    const saved = JSON.parse(storageMap.get('ahk_rust_config')!)
    expect(saved.maxLogs).toBe(200)
    expect(saved.autoHotkey).toBe(true)
  })
})

describe('scriptStore', () => {
  it('loadScripts 加载脚本列表', async () => {
    const scripts: Script[] = [
      { id: '1', name: 'test', code: 'print("hi")', hotkey: null, enabled: false },
    ]
    mockedTauriInvoke.mockResolvedValue(scripts as any)

    const store = useScriptStore()
    await store.loadScripts()

    expect(mockedTauriInvoke).toHaveBeenCalledWith('load_scripts')
    expect(store.scripts).toEqual(scripts)
  })

  it('loadScripts 失败时不覆盖已有数据', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const store = useScriptStore()
    store.scripts = [{ id: '1', name: 'existing', code: '', hotkey: null, enabled: false }]
    await store.loadScripts()

    expect(store.scripts).toHaveLength(1)
    expect(store.scripts[0].name).toBe('existing')
  })

  it('deleteScript 从列表中移除', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const store = useScriptStore()
    store.scripts = [
      { id: '1', name: 'a', code: '', hotkey: null, enabled: false },
      { id: '2', name: 'b', code: '', hotkey: null, enabled: false },
    ]

    await store.deleteScript('1')

    expect(mockedTauriInvoke).toHaveBeenCalledWith('delete_script', { id: '1' })
    expect(store.scripts).toHaveLength(1)
    expect(store.scripts[0].id).toBe('2')
  })

  it('saveScript 调用后端', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const store = useScriptStore()
    const script: Script = { id: '1', name: 'test', code: 'x', hotkey: null, enabled: true }
    await store.saveScript(script)

    expect(mockedTauriInvoke).toHaveBeenCalledWith('save_script', { script })
  })

  it('runScript 设置 runningId 并传入 timeout', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const configStore = useConfigStore()
    configStore.update({ scriptTimeout: 3000 })

    const store = useScriptStore()
    const script: Script = { id: '42', name: 'run-me', code: 'sleep(100)', hotkey: null, enabled: true }
    await store.runScript(script)

    expect(store.runningId).toBe('42')
    expect(mockedTauriInvoke).toHaveBeenCalledWith('execute_script', { code: 'sleep(100)', timeout: 3000 })
  })

  it('runScript timeout 为 0 时不传', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const store = useScriptStore()
    const script: Script = { id: '1', name: 'x', code: '', hotkey: null, enabled: true }
    await store.runScript(script)

    expect(mockedTauriInvoke).toHaveBeenCalledWith('execute_script', { code: '', timeout: undefined })
  })

  it('stopScript 重置 runningId', async () => {
    mockedTauriInvoke.mockResolvedValue(null as any)

    const store = useScriptStore()
    store.runningId = '1'
    await store.stopScript()

    expect(mockedTauriInvoke).toHaveBeenCalledWith('stop_script')
    expect(store.runningId).toBeNull()
  })

  it('addLog 遵守 maxLogs 限制', () => {
    const configStore = useConfigStore()
    configStore.update({ maxLogs: 3 })

    const store = useScriptStore()
    for (let i = 0; i < 5; i++) {
      store.addLog({ scriptName: `s${i}`, time: `${i}`, result: 'ok', success: true })
    }

    expect(store.logs).toHaveLength(3)
    expect(store.logs[0].scriptName).toBe('s4')
  })
})
