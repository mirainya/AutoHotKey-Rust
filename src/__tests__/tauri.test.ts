import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock element-plus
vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
  },
}))

import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { tauriInvoke, tauriInvokeOrThrow } from '../utils/tauri'

const mockedInvoke = vi.mocked(invoke)

beforeEach(() => {
  vi.clearAllMocks()
})

describe('tauriInvoke', () => {
  it('成功时返回结果', async () => {
    mockedInvoke.mockResolvedValue({ id: '1', name: 'test' })

    const result = await tauriInvoke<{ id: string; name: string }>('load_scripts')

    expect(result).toEqual({ id: '1', name: 'test' })
    expect(mockedInvoke).toHaveBeenCalledWith('load_scripts', undefined)
  })

  it('传递参数', async () => {
    mockedInvoke.mockResolvedValue(null)

    await tauriInvoke('save_script', { script: { id: '1' } })

    expect(mockedInvoke).toHaveBeenCalledWith('save_script', { script: { id: '1' } })
  })

  it('失败时返回 null 并显示错误', async () => {
    mockedInvoke.mockRejectedValue('脚本不存在')

    const result = await tauriInvoke('load_scripts')

    expect(result).toBeNull()
    expect(ElMessage.error).toHaveBeenCalledWith('load_scripts: 脚本不存在')
  })

  it('处理 Error 对象', async () => {
    mockedInvoke.mockRejectedValue(new Error('网络错误'))

    const result = await tauriInvoke('save_script')

    expect(result).toBeNull()
    expect(ElMessage.error).toHaveBeenCalledWith('save_script: 网络错误')
  })

  it('处理无 message 的错误对象', async () => {
    mockedInvoke.mockRejectedValue({})

    const result = await tauriInvoke('test_cmd')

    expect(result).toBeNull()
    expect(ElMessage.error).toHaveBeenCalledWith('test_cmd: 未知错误')
  })
})

describe('tauriInvokeOrThrow', () => {
  it('成功时返回结果', async () => {
    mockedInvoke.mockResolvedValue([1, 2, 3])

    const result = await tauriInvokeOrThrow<number[]>('get_data')

    expect(result).toEqual([1, 2, 3])
  })

  it('失败时抛出异常并显示错误', async () => {
    mockedInvoke.mockRejectedValue('操作失败')

    await expect(tauriInvokeOrThrow('bad_cmd')).rejects.toBe('操作失败')
    expect(ElMessage.error).toHaveBeenCalledWith('bad_cmd: 操作失败')
  })
})
