<template>
  <div class="settings">
    <div class="header">
      <h2>⚙️ 配置</h2>
    </div>

    <el-form :model="configStore.config" label-width="160px" style="max-width: 500px; margin-top: 20px;">
      <el-form-item label="最大日志条数">
        <el-input-number v-model="configStore.config.maxLogs" :min="10" :max="1000" @change="configStore.save" />
      </el-form-item>
      <el-form-item label="启动时加载脚本">
        <el-switch v-model="configStore.config.autoLoad" @change="configStore.save" />
      </el-form-item>
      <el-form-item label="启动时启用热键">
        <el-switch v-model="configStore.config.autoHotkey" @change="configStore.save" />
      </el-form-item>
      <el-form-item label="脚本执行超时(秒)">
        <el-input-number v-model="configStore.config.scriptTimeout" :min="0" :max="3600" @change="configStore.save" />
        <span style="margin-left:8px;color:#999;font-size:12px;">0 = 不限制</span>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from '../stores/configStore'

const configStore = useConfigStore()
</script>

<style scoped>
.settings { padding: 20px; }
.header h2 { margin: 0; }
</style>
