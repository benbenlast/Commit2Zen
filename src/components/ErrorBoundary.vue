<template>
  <div v-if="hasError" class="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
    <n-result status="error" :title="title" :description="description">
      <template #footer>
        <n-button @click="retry" type="primary" size="small">
          重试
        </n-button>
      </template>
    </n-result>
  </div>
  <slot v-else />
</template>

<script setup>
import { ref, onErrorCaptured } from 'vue'
import { NResult, NButton } from 'naive-ui'

const props = defineProps({
  title: { type: String, default: '发生错误' },
  retry: { type: Function, default: null },
})

const hasError = ref(false)
const description = ref('')

onErrorCaptured((error) => {
  hasError.value = true
  description.value = error.message || String(error)
  return false
})
</script>
