<template>
  <div class="settings">
    <div class="header">
      <h2>⚙️ 配置</h2>
    </div>

    <el-form :model="config" label-width="160px" style="max-width: 500px; margin-top: 20px;">
      <el-form-item label="最大日志条数">
        <el-input-number v-model="config.maxLogs" :min="10" :max="1000" @change="save" />
      </el-form-item>
      <el-form-item label="启动时加载脚本">
        <el-switch v-model="config.autoLoad" @change="save" />
      </el-form-item>
      <el-form-item label="启动时启用热键">
        <el-switch v-model="config.autoHotkey" @change="save" />
      </el-form-item>
      <el-form-item label="脚本执行超时(秒)">
        <el-input-number v-model="config.scriptTimeout" :min="0" :max="3600" @change="save" />
        <span style="margin-left:8px;color:#999;font-size:12px;">0 = 不限制</span>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'

const STORAGE_KEY = 'ahk_rust_config'

const defaults = {
  maxLogs: 100,
  autoLoad: true,
  autoHotkey: false,
  scriptTimeout: 0,
}

const stored = localStorage.getItem(STORAGE_KEY)
const config = reactive(stored ? { ...defaults, ...JSON.parse(stored) } : { ...defaults })

function save() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
}
</script>

<style scoped>
.settings { padding: 20px; }
.header h2 { margin: 0; }
</style>
