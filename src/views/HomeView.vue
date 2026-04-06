<template>
  <div style="padding: 32px;">
    <n-space vertical :size="24">
      <n-h1>Commit2Zen</n-h1>
      <n-text depth="3">Git 提交记录管理与禅道任务自动化工具</n-text>

      <n-space>
        <n-button type="primary" @click="router.push({ name: 'execute' })">
          开始执行
        </n-button>
        <n-button @click="router.push({ name: 'config' })">
          配置管理
        </n-button>
        <n-button @click="router.push({ name: 'history' })">
          历史记录
        </n-button>
      </n-space>

      <n-card title="最近执行" v-if="lastReport">
        <n-descriptions :column="4" bordered>
          <n-descriptions-item label="时间">{{ lastReport.timestamp }}</n-descriptions-item>
          <n-descriptions-item label="分支数">{{ lastReport.summary.total_branches }}</n-descriptions-item>
          <n-descriptions-item label="提交数">{{ lastReport.summary.total_commits }}</n-descriptions-item>
          <n-descriptions-item label="任务创建">{{ lastReport.summary.tasks_created }}</n-descriptions-item>
        </n-descriptions>
      </n-card>

      <n-card v-else>
        <n-empty description="暂无执行记录" />
      </n-card>
    </n-space>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const lastReport = ref(null)

onMounted(async () => {
  try {
    const history = await invoke('get_report_history', { reportDir: 'reports' })
    if (history.length > 0) {
      const latest = history[0]
      const content = await invoke('read_report', { path: latest.path })
      lastReport.value = JSON.parse(content)
    }
  } catch (e) {
    console.error('加载历史记录失败:', e)
  }
})
</script>
