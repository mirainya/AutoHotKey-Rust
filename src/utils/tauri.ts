import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e: any) {
    const msg = typeof e === 'string' ? e : e?.message || '未知错误'
    ElMessage.error(`${cmd}: ${msg}`)
    console.error(`[tauri] ${cmd} 失败:`, e)
    return null
  }
}

export async function tauriInvokeOrThrow<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e: any) {
    const msg = typeof e === 'string' ? e : e?.message || '未知错误'
    ElMessage.error(`${cmd}: ${msg}`)
    throw e
  }
}
