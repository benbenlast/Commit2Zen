<template>
  <div style="padding: 24px;">
    <n-card title="历史报告">
      <n-space vertical>
        <n-list v-if="reports.length > 0" bordered>
          <n-list-item v-for="report in reports" :key="report.path">
            <n-space justify="space-between" align="center">
              <div>
                <n-text strong>{{ report.date }}</n-text>
                <br />
                <n-text depth="3" style="font-size: 12px;">{{ report.filename }}</n-text>
              </div>
              <n-space>
                <n-button size="small" @click="viewReport(report)">查看</n-button>
                <n-button size="small" type="error" @click="deleteReport(report)">删除</n-button>
              </n-space>
            </n-space>
          </n-list-item>
        </n-list>

        <n-empty v-else description="暂无历史报告" />

        <!-- Report Detail Modal -->
        <n-modal v-model:show="showDetail" preset="card" title="报告详情" style="width: 800px;">
          <n-descriptions v-if="selectedReport" :column="2" bordered>
            <n-descriptions-item label="时间">{{ selectedReport.timestamp }}</n-descriptions-item>
            <n-descriptions-item label="项目">{{ selectedReport.project }}</n-descriptions-item>
            <n-descriptions-item label="分支数">{{ selectedReport.summary.total_branches }}</n-descriptions-item>
            <n-descriptions-item label="提交数">{{ selectedReport.summary.total_commits }}</n-descriptions-item>
            <n-descriptions-item label="成功任务">{{ selectedReport.summary.tasks_created }}</n-descriptions-item>
            <n-descriptions-item label="失败任务">{{ selectedReport.summary.tasks_failed }}</n-descriptions-item>
          </n-descriptions>

          <n-divider />

          <n-data-table
            v-if="selectedReport"
            :columns="columns"
            :data="selectedReport.branches"
            :pagination="{ pageSize: 10 }"
            size="small"
          />
        </n-modal>
      </n-space>
    </n-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()
const reports = ref([])
const showDetail = ref(false)
const selectedReport = ref(null)

const columns = [
  { title: '分支', key: 'branch' },
  { title: '提交数', key: 'commit_count' },
  {
    title: '状态',
    key: 'task_created',
    render: (row) => row.task_created ? '成功' : '失败',
  },
  {
    title: '任务',
    key: 'task_id',
    render: (row) => row.task_url
      ? `<a href="${row.task_url}" target="_blank">任务 #${row.task_id}</a>`
      : '-',
  },
]

onMounted(async () => {
  await loadReports()
})

const loadReports = async () => {
  try {
    reports.value = await invoke('get_report_history', { reportDir: 'reports' })
  } catch (e) {
    message.error(`加载失败: ${e}`)
  }
}

const viewReport = async (report) => {
  try {
    const content = await invoke('read_report', { path: report.path })
    selectedReport.value = JSON.parse(content)
    showDetail.value = true
  } catch (e) {
    message.error(`查看失败: ${e}`)
  }
}

const deleteReport = async (report) => {
  // 实现删除功能
  message.info('删除功能待实现')
}
</script>
