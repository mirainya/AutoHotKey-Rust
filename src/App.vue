<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo">AutoHotkey Rust</div>
      <nav class="nav-menu">
        <div
          v-for="item in menuItems"
          :key="item.id"
          :class="['nav-item', { active: currentView === item.id }]"
          @click="currentView = item.id"
        >
          {{ item.icon }} {{ item.label }}
        </div>
      </nav>
    </aside>

    <main class="content">
      <component :is="currentComponent" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, markRaw, provide } from 'vue'
import ResourceCollector from './components/ResourceCollector.vue'
import ScriptManager from './components/ScriptManager.vue'
import ScriptEditorPage from './components/ScriptEditorPage.vue'
import Settings from './components/Settings.vue'
import ApiDocs from './components/ApiDocs.vue'
import { type Script } from './stores/scriptStore'

const currentView = ref('script')
const currentEditingScript = ref<Script | null>(null)

provide('currentView', currentView)
provide('currentEditingScript', currentEditingScript)

const menuItems = [
  { id: 'script', icon: '📝', label: '脚本' },
  { id: 'resource', icon: '📸', label: '资源' },
  { id: 'editor', icon: '✏️', label: '编辑器' },
  { id: 'settings', icon: '⚙️', label: '配置' },
  { id: 'docs', icon: '📖', label: 'API 文档' }
]

const componentMap: Record<string, any> = {
  script: markRaw(ScriptManager),
  resource: markRaw(ResourceCollector),
  editor: markRaw(ScriptEditorPage),
  settings: markRaw(Settings),
  docs: markRaw(ApiDocs),
}

const currentComponent = computed(() => {
  return componentMap[currentView.value] || 'div'
})
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}
</style>
