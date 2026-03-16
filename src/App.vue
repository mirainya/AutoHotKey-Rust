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
      <div class="sidebar-footer">v0.1.0</div>
    </aside>

    <main class="content">
      <Transition name="page-fade" mode="out-in">
        <component :is="currentComponent" :key="currentView" />
      </Transition>
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

.sidebar {
  width: 200px;
  background: linear-gradient(180deg, var(--miku-primary) 0%, var(--miku-dark) 100%);
  color: white;
  display: flex;
  flex-direction: column;
}

.logo {
  padding: 24px 20px;
  font-size: 16px;
  font-weight: bold;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.nav-menu {
  flex: 1;
  padding: 20px 0;
}

.nav-item {
  padding: 16px 24px;
  cursor: pointer;
  transition: all var(--transition-normal);
  border-left: 4px solid transparent;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.1);
  border-left-color: white;
  transform: translateX(2px);
}

.nav-item.active {
  background: rgba(255, 255, 255, 0.2);
  border-left-color: white;
  font-weight: bold;
}

.sidebar-footer {
  padding: 12px 20px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  text-align: center;
}

.content {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
  background: rgba(255, 255, 255, 0.95);
}

/* 页面过渡动画 */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
