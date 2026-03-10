<template>
  <div class="hotkey-manager">
    <el-card class="header-card">
      <h2>🔥 热键管理</h2>
      <el-button
        :type="listening ? 'danger' : 'primary'"
        @click="toggleListener"
      >
        {{ listening ? '停止监听' : '启动监听' }}
      </el-button>
    </el-card>

    <el-card class="hotkey-list">
      <h3>监听到的热键</h3>
      <el-tag
        v-for="(key, index) in pressedKeys"
        :key="index"
        class="hotkey-tag"
        size="large"
      >
        {{ key }}
      </el-tag>
      <el-empty v-if="pressedKeys.length === 0" description="暂无热键记录" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const listening = ref(false)
const pressedKeys = ref<string[]>([])
let unlisten: any = null

const toggleListener = async () => {
  if (listening.value) {
    await invoke('stop_hotkey_listener')
    listening.value = false
  } else {
    await invoke('start_hotkey_listener')
    listening.value = true
  }
}

onMounted(async () => {
  unlisten = await listen('hotkey-pressed', (event: any) => {
    pressedKeys.value.unshift(event.payload)
    if (pressedKeys.value.length > 10) {
      pressedKeys.value.pop()
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<style scoped>
.hotkey-manager {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.header-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.hotkey-list {
  min-height: 300px;
}

.hotkey-tag {
  margin: 8px;
  background: linear-gradient(135deg, #7FDBDA, #39C5BB);
  border: none;
  color: white;
  font-weight: bold;
}
</style>
