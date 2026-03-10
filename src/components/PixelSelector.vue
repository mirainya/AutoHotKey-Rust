<template>
  <div v-if="visible" class="pixel-selector" @mousedown="onMouseDown" @mousemove="onMouseMove" @mouseup="onMouseUp">
    <div class="selector-hint">按住鼠标左键拖动选择区域，松开完成选择</div>
    <div v-if="selecting" class="selection-box" :style="selectionStyle">
      <div class="selection-info">
        {{ Math.round(rect.width) }} × {{ Math.round(rect.height) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}

const emit = defineEmits<{
  complete: [rect: SelectionRect]
  cancel: []
}>()

const visible = ref(false)
const selecting = ref(false)
const startX = ref(0)
const startY = ref(0)
const currentX = ref(0)
const currentY = ref(0)
const windowOffsetX = ref(0)
const windowOffsetY = ref(0)

const rect = computed(() => {
  const x = Math.min(startX.value, currentX.value)
  const y = Math.min(startY.value, currentY.value)
  const width = Math.abs(currentX.value - startX.value)
  const height = Math.abs(currentY.value - startY.value)
  return { x, y, width, height }
})

const selectionStyle = computed(() => ({
  left: `${rect.value.x - windowOffsetX.value}px`,
  top: `${rect.value.y - windowOffsetY.value}px`,
  width: `${rect.value.width}px`,
  height: `${rect.value.height}px`
}))

const onMouseDown = (e: MouseEvent) => {
  selecting.value = true
  startX.value = e.screenX
  startY.value = e.screenY
  currentX.value = e.screenX
  currentY.value = e.screenY
}

const onMouseMove = (e: MouseEvent) => {
  if (selecting.value) {
    currentX.value = e.screenX
    currentY.value = e.screenY
  }
}

const onMouseUp = () => {
  if (selecting.value && rect.value.width > 5 && rect.value.height > 5) {
    emit('complete', rect.value)
    close()
  }
  selecting.value = false
}

const open = () => {
  windowOffsetX.value = window.screenX
  windowOffsetY.value = window.screenY
  visible.value = true
}

const close = () => {
  visible.value = false
  selecting.value = false
}

defineExpose({ open, close })
</script>

<style scoped>
.pixel-selector {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.3);
  cursor: crosshair;
  z-index: 9999;
}

.selector-hint {
  position: absolute;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
}

.selection-box {
  position: absolute;
  border: 2px solid #409eff;
  background: rgba(64, 158, 255, 0.1);
}

.selection-info {
  position: absolute;
  bottom: -30px;
  right: 0;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  white-space: nowrap;
}
</style>
