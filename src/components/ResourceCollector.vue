<template>
  <div class="resource-collector">
    <div class="page-header">
      <h2 class="page-title">🎨 资源采集工具</h2>
    </div>
    <el-tabs v-model="activeTab" type="border-card">
      <el-tab-pane label="像素采集" name="pixel">
        <el-card class="function-card" shadow="hover">
          <template #header>
            <div class="card-header">
              <span>🎯 采集功能</span>
            </div>
          </template>
          <div class="capture-controls">
            <div class="control-item">
              <span class="control-label">采集层级</span>
              <el-radio-group v-model="selectedLayers" size="default">
                <el-radio-button :value="1">1层(9点)</el-radio-button>
                <el-radio-button :value="2">2层(17点)</el-radio-button>
                <el-radio-button :value="3">3层(25点)</el-radio-button>
                <el-radio-button :value="5">5层(41点)</el-radio-button>
              </el-radio-group>
            </div>
            <el-button type="primary" size="large" @click="startPixelCapture">
              <el-icon><Camera /></el-icon>
              <span>开始采集像素</span>
            </el-button>
          </div>
          <div v-if="capturedPixels.length > 0" class="capture-result">
            <h4>采集结果（共 {{ capturedPixels.length }} 个点）：</h4>
            <div class="result-container">
              <div class="result-left">
                <el-table :data="capturedPixels" stripe style="width: 100%" size="small" max-height="400">
                  <el-table-column type="index" label="#" width="50" />
                  <el-table-column label="位置" min-width="120">
                    <template #default="{ row }">
                      ({{ row.x }}, {{ row.y }})
                    </template>
                  </el-table-column>
                  <el-table-column label="相对位置" min-width="120">
                    <template #default="{ row }">
                      ({{ row.x - captureRect.x - Math.floor(captureRect.width / 2) }}, {{ row.y - captureRect.y - Math.floor(captureRect.height / 2) }})
                    </template>
                  </el-table-column>
                  <el-table-column label="RGB" min-width="140">
                    <template #default="{ row }">
                      {{ row.r }}, {{ row.g }}, {{ row.b }}
                    </template>
                  </el-table-column>
                  <el-table-column label="预览" width="80">
                    <template #default="{ row }">
                      <div :style="{
                        width: '30px',
                        height: '20px',
                        backgroundColor: `rgb(${row.r}, ${row.g}, ${row.b})`,
                        border: '1px solid #dcdfe6',
                        borderRadius: '4px'
                      }"></div>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
              <div class="result-right">
                <div class="image-container">
                  <canvas ref="canvasRef" @mousedown="handleMouseDown"></canvas>
                </div>
              </div>
            </div>

            <div class="save-pattern-row">
              <el-input v-model="patternName" placeholder="输入模板名称" style="width: 200px" />
              <el-button type="primary" @click="savePattern">保存为模板</el-button>
            </div>
          </div>
        </el-card>

        <el-card class="function-card" shadow="hover">
          <template #header>
            <div class="card-header">
              <span>📦 模板库</span>
            </div>
          </template>
          <el-empty v-if="patterns.length === 0" description="暂无模板" />
          <el-row :gutter="16" v-else>
            <el-col :span="8" v-for="pattern in patterns" :key="pattern.name">
              <el-card class="pattern-card" shadow="hover">
                <template #header>
                  <div class="pattern-header">
                    <span class="pattern-name">{{ pattern.name }}</span>
                  </div>
                </template>
                <div class="pattern-info">
                  <div class="info-item">
                    <span class="label">像素点:</span>
                    <span>{{ pattern.pixels.length }} 个</span>
                  </div>
                  <div class="pixel-preview">
                    <div v-for="(pixel, idx) in pattern.pixels.slice(0, 5)" :key="idx"
                         :style="{ backgroundColor: `rgb(${pixel.r}, ${pixel.g}, ${pixel.b})` }"
                         class="pixel-dot"
                         :title="`RGB(${pixel.r}, ${pixel.g}, ${pixel.b})`">
                    </div>
                  </div>
                </div>
                <template #footer>
                  <div class="pattern-actions">
                    <el-button size="small" type="primary" @click="findPattern(pattern)">查找</el-button>
                    <el-button size="small" type="danger" @click="deletePattern(pattern.name)">删除</el-button>
                  </div>
                </template>
              </el-card>
            </el-col>
          </el-row>
        </el-card>
      </el-tab-pane>

      <el-tab-pane label="屏幕截图" name="screenshot">
        <el-card class="function-card" shadow="hover">
          <template #header>
            <div class="card-header">
              <span>📸 截图功能</span>
            </div>
          </template>
          <div class="screenshot-controls">
            <el-button type="primary" size="large" @click="captureScreen" :loading="capturing">
              <el-icon><Picture /></el-icon>
              <span>截取屏幕</span>
            </el-button>
            <el-alert v-if="lastCapture" :title="`截图已保存: ${lastCapture}`" type="success" />
          </div>
        </el-card>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Camera, Picture } from '@element-plus/icons-vue'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import { tauriInvoke } from '../utils/tauri'

interface PixelInfo {
  x: number
  y: number
  r: number
  g: number
  b: number
}

interface PixelPattern {
  name: string
  pixels: PixelInfo[]
}

const activeTab = ref('pixel')
const capturing = ref(false)
const lastCapture = ref('')
const capturedPixels = ref<PixelInfo[]>([])
const captureImage = ref('')
const captureSize = ref({ width: 0, height: 0 })
const patternName = ref('')
const patterns = ref<PixelPattern[]>([])
const captureRect = ref({ x: 0, y: 0, width: 0, height: 0 })
const selectedLayers = ref(1)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const selectedPixelIndex = ref(-1)
const pixelSize = ref(30)
const canvasOffset = ref({ startX: 0, startY: 0 })
let unlisten: (() => void) | null = null

const captureScreen = async () => {
  capturing.value = true
  try {
    const path = await tauriInvoke<string>('capture_screen')
    lastCapture.value = path
    ElMessage.success('截图成功！')
  } catch (error) {
    ElMessage.error(`截图失败: ${error}`)
  } finally {
    capturing.value = false
  }
}

const startPixelCapture = async () => {
  try {
    console.log('开始创建选择器窗口')
    const webview = new WebviewWindow('pixel-selector', {
      url: '/selector.html',
      title: '',
      fullscreen: true,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      visible: false
    })
    await webview.once('tauri://created', () => {
      console.log('选择器窗口创建成功')
    })
    await webview.once('tauri://error', (e) => {
      console.error('选择器窗口创建失败', e)
      ElMessage.error(`窗口创建失败: ${e}`)
    })
  } catch (error) {
    console.error('创建选择器失败', error)
    ElMessage.error(`打开选择器失败: ${error}`)
  }
}

const onSelectionComplete = async (rect: { x: number, y: number, width: number, height: number }) => {
  try {
    captureRect.value = rect
    const result = await tauriInvoke<{ pixels: PixelInfo[], image_base64: string, width: number, height: number }>('capture_pixels', {
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
      layers: selectedLayers.value
    })
    capturedPixels.value = result.pixels
    captureImage.value = `data:image/png;base64,${result.image_base64}`
    captureSize.value = { width: result.width, height: result.height }
    ElMessage.success(`像素采集成功！共采集 ${result.pixels.length} 个点`)
  } catch (error) {
    ElMessage.error(`采集失败: ${error}`)
  }
}

const drawCanvas = () => {
  if (!canvasRef.value || !captureImage.value) return
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const img = new Image()
  img.onload = () => {
    const tempCanvas = document.createElement('canvas')
    const tempCtx = tempCanvas.getContext('2d')
    if (!tempCtx) return

    tempCanvas.width = img.width
    tempCanvas.height = img.height
    tempCtx.drawImage(img, 0, 0)
    const imageData = tempCtx.getImageData(0, 0, img.width, img.height)

    const centerX = Math.floor(img.width / 2)
    const centerY = Math.floor(img.height / 2)
    const radius = 5

    const startX = Math.max(0, centerX - radius)
    const startY = Math.max(0, centerY - radius)
    const endX = Math.min(img.width, centerX + radius + 1)
    const endY = Math.min(img.height, centerY + radius + 1)
    const drawWidth = endX - startX
    const drawHeight = endY - startY

    canvasOffset.value = { startX, startY }

    canvas.width = drawWidth * pixelSize.value
    canvas.height = drawHeight * pixelSize.value

    for (let y = startY; y < endY; y++) {
      for (let x = startX; x < endX; x++) {
        const index = (y * img.width + x) * 4
        const r = imageData.data[index]
        const g = imageData.data[index + 1]
        const b = imageData.data[index + 2]

        ctx.fillStyle = `rgb(${r}, ${g}, ${b})`
        ctx.fillRect((x - startX) * pixelSize.value, (y - startY) * pixelSize.value, pixelSize.value, pixelSize.value)
      }
    }

    capturedPixels.value.forEach((pixel, index) => {
      const x = (pixel.x - captureRect.value.x - startX) * pixelSize.value
      const y = (pixel.y - captureRect.value.y - startY) * pixelSize.value

      ctx.fillStyle = index === selectedPixelIndex.value ? 'rgba(0, 150, 255, 0.7)' : 'rgba(255, 0, 0, 0.7)'
      ctx.fillRect(x, y, pixelSize.value, pixelSize.value)

      ctx.fillStyle = '#fff'
      ctx.strokeStyle = '#000'
      ctx.lineWidth = 2
      ctx.font = 'bold 14px Arial'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.strokeText(String(index + 1), x + pixelSize.value / 2, y + pixelSize.value / 2)
      ctx.fillText(String(index + 1), x + pixelSize.value / 2, y + pixelSize.value / 2)
    })
  }
  img.src = captureImage.value
}

const handleMouseDown = async (e: MouseEvent) => {
  if (!canvasRef.value) return
  const canvas = canvasRef.value
  const rect = canvas.getBoundingClientRect()

  const scaleX = canvas.width / rect.width
  const scaleY = canvas.height / rect.height
  const canvasX = (e.clientX - rect.left) * scaleX
  const canvasY = (e.clientY - rect.top) * scaleY

  const clickedIndex = capturedPixels.value.findIndex(pixel => {
    const px = (pixel.x - captureRect.value.x - canvasOffset.value.startX) * pixelSize.value
    const py = (pixel.y - captureRect.value.y - canvasOffset.value.startY) * pixelSize.value
    return canvasX >= px && canvasX < px + pixelSize.value && canvasY >= py && canvasY < py + pixelSize.value
  })

  if (clickedIndex !== -1) {
    selectedPixelIndex.value = clickedIndex
    drawCanvas()
  } else if (selectedPixelIndex.value !== -1) {
    const drawX = Math.round(canvasX / pixelSize.value)
    const drawY = Math.round(canvasY / pixelSize.value)
    const newX = drawX + canvasOffset.value.startX + captureRect.value.x
    const newY = drawY + canvasOffset.value.startY + captureRect.value.y

    capturedPixels.value[selectedPixelIndex.value].x = newX
    capturedPixels.value[selectedPixelIndex.value].y = newY

    try {
      const result = await tauriInvoke<PixelInfo>('get_pixel_color', { x: newX, y: newY })
      capturedPixels.value[selectedPixelIndex.value].r = result.r
      capturedPixels.value[selectedPixelIndex.value].g = result.g
      capturedPixels.value[selectedPixelIndex.value].b = result.b
    } catch (error) {
      console.error('获取颜色失败', error)
    }

    drawCanvas()
  }
}

const savePattern = async () => {
  if (!patternName.value) {
    ElMessage.warning('请输入模板名称')
    return
  }
  try {
    const centerX = Math.floor(captureRect.value.width / 2)
    const centerY = Math.floor(captureRect.value.height / 2)

    const pattern: PixelPattern = {
      name: patternName.value,
      pixels: capturedPixels.value.map(pixel => ({
        x: pixel.x - captureRect.value.x - centerX,
        y: pixel.y - captureRect.value.y - centerY,
        r: pixel.r,
        g: pixel.g,
        b: pixel.b
      }))
    }
    await tauriInvoke('save_pixel_pattern', { pattern })
    ElMessage.success('模板保存成功！')
    patternName.value = ''
    loadPatterns()
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  }
}

const loadPatterns = async () => {
  try {
    patterns.value = await tauriInvoke<PixelPattern[]>('load_pixel_patterns') || []
  } catch (error) {
    ElMessage.error(`加载失败: ${error}`)
  }
}

const deletePattern = async (name: string) => {
  try {
    await ElMessageBox.confirm(`确定删除模板"${name}"吗？`, '确认删除', { type: 'warning' })
    await tauriInvoke('delete_pixel_pattern', { name })
    ElMessage.success('删除成功！')
    loadPatterns()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(`删除失败: ${error}`)
    }
  }
}

const findPattern = async (pattern: PixelPattern) => {
  try {
    const tolerance = await ElMessageBox.prompt('请输入颜色容差（0-255）', '查找像素模式', {
      inputValue: '10',
      inputPattern: /^\d+$/,
      inputErrorMessage: '请输入0-255之间的数字'
    })
    const results = await tauriInvoke<{ x: number, y: number }[]>('find_pixel_pattern', {
      pattern,
      tolerance: Number(tolerance.value)
    })
    if (results.length > 0) {
      ElMessage.success(`找到 ${results.length} 个匹配位置！第一个位置：(${results[0].x}, ${results[0].y})`)
    } else {
      ElMessage.info('未找到匹配的位置')
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(`查找失败: ${error}`)
    }
  }
}

watch([captureImage, capturedPixels], () => {
  nextTick(() => {
    drawCanvas()
  })
})

onMounted(async () => {
  loadPatterns()
  unlisten = await listen('pixel-selection-complete', (event: any) => {
    onSelectionComplete(event.payload)
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})
</script>

<style scoped>
.resource-collector {
  display: flex;
  flex-direction: column;
}

.page-header {
  text-align: center;
  padding: 20px 0;
  margin-bottom: 16px;
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--miku-primary);
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

/* 功能卡片 */
.function-card {
  margin-bottom: 20px;
  border-radius: var(--radius-md);
}

.function-card:last-child {
  margin-bottom: 0;
}

/* 卡片标题 */
.card-header {
  font-size: 16px;
  font-weight: 600;
}

.tool-section {
  margin-bottom: 24px;
}

.tool-section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-header h3 {
  margin: 0;
}

.capture-controls {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.control-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.control-label {
  font-size: 14px;
  color: var(--text-regular);
  font-weight: 500;
}

.capture-result {
  margin-top: 20px;
  padding: 16px;
  background: var(--miku-bg);
  border-radius: var(--radius-md);
}

.save-pattern-row {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-top: 16px;
}

.result-container {
  display: flex;
  gap: 16px;
  margin-top: 12px;
}

.result-left {
  flex: 1;
  min-width: 0;
}

.result-right {
  flex: 1;
  min-width: 0;
}

.image-container {
  position: relative;
  width: 100%;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  justify-content: center;
  align-items: center;
}

.image-container canvas {
  max-width: 100%;
  height: auto;
  display: block;
  cursor: crosshair;
}

.pattern-card {
  margin-bottom: 16px;
}

.pattern-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.pattern-name {
  font-weight: bold;
  font-size: 16px;
}

.pattern-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-item {
  display: flex;
  justify-content: space-between;
}

.info-item .label {
  color: var(--text-secondary);
  font-size: 14px;
}

.pixel-preview {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.pixel-dot {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid #dcdfe6;
  cursor: pointer;
}

.pattern-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

/* 屏幕截图控制区 */
.screenshot-controls {
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: flex-start;
}
</style>
