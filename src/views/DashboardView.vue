<template>
  <div class="p-6">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold mb-2">仪表盘</h1>
      <p class="text-muted">Commit2Zen 项目概览与统计分析</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-4 gap-4 mb-6">
      <div class="card-base">
        <div class="text-muted text-sm mb-1">总项目数</div>
        <div class="text-3xl font-bold text-primary">{{ stats.totalProjects }}</div>
        <div class="text-xs text-muted mt-1">Git 仓库</div>
      </div>
      <div class="card-base">
        <div class="text-muted text-sm mb-1">总提交数</div>
        <div class="text-3xl font-bold text-success">{{ stats.totalCommits }}</div>
        <div class="text-xs text-muted mt-1">收集记录</div>
      </div>
      <div class="card-base">
        <div class="text-muted text-sm mb-1">总任务数</div>
        <div class="text-3xl font-bold text-warning">{{ stats.totalTasks }}</div>
        <div class="text-xs text-muted mt-1">禅道任务</div>
      </div>
      <div class="card-base">
        <div class="text-muted text-sm mb-1">成功率</div>
        <div class="text-3xl font-bold" :class="stats.successRate >= 90 ? 'text-success' : stats.successRate >= 70 ? 'text-warning' : 'text-error'">
          {{ stats.successRate }}%
        </div>
        <div class="text-xs text-muted mt-1">任务创建</div>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="grid grid-cols-2 gap-4 mb-6">
      <!-- 近7天活跃度 -->
      <div class="card-base">
        <h3 class="text-base font-semibold mb-4">近 7 天活跃度</h3>
        <v-chart :option="activityChartOption" class="h-64" autoresize />
      </div>

      <!-- 分支分布 -->
      <div class="card-base">
        <h3 class="text-base font-semibold mb-4">分支类型分布</h3>
        <v-chart :option="branchChartOption" class="h-64" autoresize />
      </div>
    </div>

    <!-- 最近执行记录 -->
    <div class="card-base">
      <div class="flex-between mb-4">
        <h3 class="text-base font-semibold">最近执行记录</h3>
        <n-button size="small" @click="router.push({ name: 'history' })">
          查看全部
        </n-button>
      </div>
      <n-empty v-if="recentExecutions.length === 0" description="暂无执行记录" />
      <n-list v-else bordered>
        <n-list-item v-for="exec in recentExecutions" :key="exec.id">
          <div class="flex-between">
            <div>
              <div class="font-medium">{{ exec.projectName }}</div>
              <div class="text-xs text-muted">{{ exec.branchCount }} 个分支，{{ exec.commitCount }} 条提交</div>
            </div>
            <div class="text-right">
              <div :class="exec.success ? 'text-success' : 'text-error'" class="text-sm">
                {{ exec.success ? '成功' : '失败' }}
              </div>
              <div class="text-xs text-muted">{{ exec.date }}</div>
            </div>
          </div>
        </n-list-item>
      </n-list>
    </div>

    <!-- AI 使用统计 -->
    <div class="card-base mt-4" v-if="llmStore.enabledProviders.length > 0">
      <div class="flex-between mb-4">
        <h3 class="text-base font-semibold">AI 使用统计</h3>
        <n-button size="small" @click="router.push({ name: 'llm-config' })">
          配置 AI
        </n-button>
      </div>
      <div class="grid grid-cols-3 gap-4">
        <div v-for="provider in llmStore.enabledProviders" :key="provider.type" class="text-center p-3 bg-gray-50 dark:bg-gray-700 rounded">
          <div class="text-sm font-medium">{{ provider.name }}</div>
          <div class="text-xs text-muted mt-1">调用 {{ provider.callCount || 0 }} 次</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import VChart from 'vue-echarts'
import { useGraphic } from 'echarts'
import { useLLMStore } from '@/stores/llm.js'

const router = useRouter()
const llmStore = useLLMStore()

// 统计数据
const stats = ref({
  totalProjects: 0,
  totalCommits: 0,
  totalTasks: 0,
  successRate: 0,
})

const recentExecutions = ref([])

// 活跃度图表配置
const activityChartOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  xAxis: {
    type: 'category',
    data: ['周一', '周二', '周三', '周四', '周五', '周六', '周日'],
  },
  yAxis: { type: 'value' },
  series: [{
    name: '提交数',
    type: 'line',
    data: [12, 18, 15, 22, 28, 8, 5],
    smooth: true,
    itemStyle: { color: '#1890ff' },
    areaStyle: {
      color: {
        type: 'linear',
        x: 0, y: 0, x2: 0, y2: 1,
        colorStops: [
          { offset: 0, color: 'rgba(24, 144, 255, 0.3)' },
          { offset: 1, color: 'rgba(24, 144, 255, 0.05)' },
        ],
      },
    },
  }],
  grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
}))

// 分支分布图表配置
const branchChartOption = computed(() => ({
  tooltip: { trigger: 'item' },
  legend: { bottom: '5%', left: 'center' },
  series: [{
    name: '分支类型',
    type: 'pie',
    radius: ['40%', '70%'],
    avoidLabelOverlap: false,
    itemStyle: {
      borderRadius: 10,
      borderColor: '#fff',
      borderWidth: 2,
    },
    label: { show: false, position: 'center' },
    emphasis: {
      label: { show: true, fontSize: 16, fontWeight: 'bold' },
    },
    data: [
      { value: 45, name: 'feature' },
      { value: 30, name: 'bugfix' },
      { value: 15, name: 'release' },
      { value: 10, name: 'hotfix' },
    ],
  }],
}))

// 加载统计数据
onMounted(async () => {
  // TODO: 从后端获取真实统计数据
  // 这里使用模拟数据展示
  stats.value = {
    totalProjects: 12,
    totalCommits: 156,
    totalTasks: 89,
    successRate: 94,
  }

  // TODO: 从历史报告中加载真实数据
  recentExecutions.value = [
    {
      id: 1,
      projectName: 'Commit2Zen',
      branchCount: 3,
      commitCount: 45,
      success: true,
      date: '2026-04-12 14:30',
    },
    {
      id: 2,
      projectName: 'AI Novel Generator',
      branchCount: 5,
      commitCount: 78,
      success: true,
      date: '2026-04-11 16:20',
    },
    {
      id: 3,
      projectName: 'Web App',
      branchCount: 2,
      commitCount: 23,
      success: false,
      date: '2026-04-10 09:15',
    },
  ]
})
</script>
