<template>
  <div class="settings">
    <div class="page-header">
      <h2 class="page-title">⚙️ 配置</h2>
    </div>

    <div class="settings-cards">
      <el-card class="settings-card" shadow="hover">
        <template #header>
          <div class="card-header">🎛️ 通用设置</div>
        </template>
        <el-form :model="configStore.config" label-width="160px">
          <el-form-item label="最大日志条数">
            <el-input-number v-model="configStore.config.maxLogs" :min="10" :max="1000" @change="configStore.save" />
          </el-form-item>
          <el-form-item label="启动时加载脚本">
            <el-switch v-model="configStore.config.autoLoad" @change="configStore.save" />
          </el-form-item>
          <el-form-item label="启动时启用热键">
            <el-switch v-model="configStore.config.autoHotkey" @change="configStore.save" />
          </el-form-item>
        </el-form>
      </el-card>

      <el-card class="settings-card" shadow="hover">
        <template #header>
          <div class="card-header">🚀 脚本执行</div>
        </template>
        <el-form :model="configStore.config" label-width="160px">
          <el-form-item label="脚本执行超时(秒)">
            <el-input-number v-model="configStore.config.scriptTimeout" :min="0" :max="3600" @change="configStore.save" />
            <span class="form-hint">0 = 不限制</span>
          </el-form-item>
          <el-form-item label="编辑器字号">
            <el-input-number v-model="configStore.config.editorFontSize" :min="10" :max="24" @change="configStore.save" />
          </el-form-item>
          <el-form-item label="Tab 宽度">
            <el-input-number v-model="configStore.config.editorTabSize" :min="1" :max="8" @change="configStore.save" />
          </el-form-item>
          <el-form-item label="控制台字号">
            <el-input-number v-model="configStore.config.consoleFontSize" :min="10" :max="24" @change="configStore.save" />
          </el-form-item>
        </el-form>
      </el-card>

      <el-card class="settings-card" shadow="hover">
        <template #header>
          <div class="card-header">💡 关于</div>
        </template>
        <div class="about-content">
          <div class="about-row">
            <span class="about-label">版本</span>
            <span class="about-value">v0.1.0</span>
          </div>
          <div class="about-row">
            <span class="about-label">技术栈</span>
            <span class="about-value">Tauri 2.0 + Vue 3 + Rust</span>
          </div>
          <div class="about-row">
            <span class="about-label">项目</span>
            <a class="about-link" href="https://github.com" target="_blank">GitHub</a>
          </div>
          <div class="about-row">
            <span class="about-label">UI 框架</span>
            <span class="about-value">Element Plus</span>
          </div>
          <div class="about-row">
            <span class="about-label">主题</span>
            <span class="about-value">初音未来 🎵</span>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from '../stores/configStore'

const configStore = useConfigStore()
</script>

<style scoped>
.settings {
  padding: 0;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  margin: 0;
  color: var(--miku-primary);
  font-size: 24px;
  font-weight: 600;
  padding-bottom: 10px;
  border-bottom: 2px solid transparent;
  background-image: linear-gradient(var(--miku-primary), var(--miku-dark));
  background-clip: text;
  display: inline-block;
  position: relative;
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

.settings-cards {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 600px;
}

.settings-card {
  border-radius: var(--radius-md);
  transition: box-shadow var(--transition-normal);
}

.card-header {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.form-hint {
  margin-left: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

.about-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.about-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.about-label {
  width: 80px;
  color: var(--text-secondary);
  font-size: 14px;
  flex-shrink: 0;
}

.about-value {
  color: var(--text-primary);
  font-size: 14px;
}

.about-link {
  color: var(--miku-primary);
  text-decoration: none;
  font-size: 14px;
}

.about-link:hover {
  color: var(--miku-dark);
  text-decoration: underline;
}
</style>
