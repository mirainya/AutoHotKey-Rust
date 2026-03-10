<template>
  <el-dialog
    v-model="visible"
    :title="isEdit ? '编辑脚本' : '添加脚本'"
    width="80%"
    @close="handleClose"
  >
    <el-form :model="form" label-width="80px">
      <el-form-item label="脚本名称">
        <el-input v-model="form.name" placeholder="请输入脚本名称" />
      </el-form-item>

      <el-form-item label="快捷键">
        <el-input
          v-model="form.hotkey"
          placeholder="例如: F9, Ctrl+A"
          clearable
        />
      </el-form-item>

      <el-form-item label="脚本代码">
        <div style="border: 1px solid #dcdfe6; border-radius: 4px; overflow: hidden;">
          <vue-monaco-editor
            v-model:value="form.code"
            language="rhai"
            theme="vs-dark"
            height="500px"
            :options="{
              minimap: { enabled: false },
              fontSize: 14,
              tabSize: 2,
              automaticLayout: true
            }"
          />
        </div>
      </el-form-item>

      <el-form-item label="启用">
        <el-switch v-model="form.enabled" />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { VueMonacoEditor } from '@guolao/vue-monaco-editor'
import { setupRhaiLanguage } from '../utils/rhaiLanguage'

interface Script {
  id: string
  name: string
  code: string
  hotkey: string | null
  enabled: boolean
}

interface Props {
  modelValue: boolean
  script: Script | null
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', script: Script): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const isEdit = computed(() => props.script !== null)

const form = ref<Script>({
  id: '',
  name: '',
  code: '',
  hotkey: null,
  enabled: true
})

watch(() => props.script, (newScript) => {
  if (newScript) {
    form.value = { ...newScript }
  } else {
    form.value = {
      id: Date.now().toString(),
      name: '',
      code: '',
      hotkey: null,
      enabled: true
    }
  }
}, { immediate: true })

const handleSave = () => {
  if (!form.value.name.trim()) {
    return
  }
  emit('save', { ...form.value })
}

const handleClose = () => {
  visible.value = false
}

onMounted(() => {
  setupRhaiLanguage()
})
</script>
