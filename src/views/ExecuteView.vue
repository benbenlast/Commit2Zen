<template>
  <n-space vertical :size="24" style="padding: 24px;">
    <n-steps :current="currentStep" :status="stepStatus">
      <n-step title="选择 Git 项目" description="扫描或手动选择" />
      <n-step title="配置禅道" description="登录并选择项目" />
      <n-step title="预览信息" description="查看提交和分支" />
      <n-step title="执行" description="创建禅道任务" />
    </n-steps>

    <!-- Step 1: Git Project Selection -->
    <n-card v-if="currentStep === 1" title="选择 Git 项目">
      <n-space vertical>
        <n-space>
          <n-button @click="selectAndScanFolder" :loading="isScanning" type="primary">
            📁 选择扫描目录
          </n-button>
          <n-input v-model:value="manualPath" placeholder="或手动输入路径" style="width: 400px;" />
          <n-button @click="scanManualPath" :loading="isScanning">扫描此目录</n-button>
        </n-space>

        <!-- 扫描进度 -->
        <n-card v-if="scanProgress" size="small" :bordered="false">
          <n-space vertical>
            <n-progress
              type="line"
              :percentage="Math.round(scanProgress.percentage)"
              :show-indicator="true"
              :status="scanProgress.status === 'cancelled' ? 'error' : 'success'"
            />
            <n-text depth="3" v-if="scanProgress.current_directory">
              正在扫描: {{ scanProgress.current_directory }}
            </n-text>
            <n-space justify="space-between">
              <n-text depth="3">
                已扫描 {{ scanProgress.directoriesScanned }} 个目录，发现 {{ scanProgress.reposFound }} 个仓库
              </n-text>
              <n-button v-if="isScanning" size="small" type="error" @click="cancelScan">
                取消扫描
              </n-button>
            </n-space>
          </n-space>
        </n-card>

        <n-list v-if="gitStore.scannedRepos.length > 0" bordered>
          <n-list-item v-for="repo in gitStore.scannedRepos" :key="repo.path">
            <n-space justify="space-between" align="center">
              <div>
                <n-text strong>{{ repo.name }}</n-text>
                <br />
                <n-text depth="3" style="font-size: 12px;">{{ repo.path }}</n-text>
              </div>
              <n-button size="small" @click="selectGitProject(repo)">选择此项目</n-button>
            </n-space>
          </n-list-item>
        </n-list>

        <n-space v-if="selectedGitRepo">
          <n-tag type="success">已选择: {{ selectedGitRepo.name }}</n-tag>
          <n-button @click="currentStep = 2" type="primary">下一步</n-button>
        </n-space>
      </n-space>
    </n-card>

    <!-- Step 2: Zentao Configuration -->
    <n-card v-if="currentStep === 2" title="禅道项目配置">
      <n-space vertical>
        <!-- 选择已保存的禅道账号 -->
        <n-text strong>选择禅道账号</n-text>
        <n-radio-group v-model:value="selectedAccountId" @update:value="onAccountSelected">
          <n-space vertical>
            <n-radio v-for="account in configStore.zentaoAccounts" :key="account.id" :value="account.id">
              {{ account.name }} ({{ account.account }})
            </n-radio>
          </n-space>
        </n-radio-group>
        <n-empty v-if="configStore.zentaoAccounts.length === 0" description="暂无保存的账号，请先去配置页添加" />
        
        <n-button @click="router.push({ name: 'config' })" secondary size="small">
          管理账号
        </n-button>

        <!-- 选择账号后显示项目选择器 -->
        <template v-if="selectedAccount">
          <n-divider />
          
          <n-space v-if="loginStatus === 'logging_in'">
            <n-spin size="small" />
            <n-text>连接中...</n-text>
          </n-space>

          <n-alert v-if="loginStatus === 'error'" type="error">
            {{ loginError }}
          </n-alert>

          <n-alert v-if="loginStatus === 'connected'" type="success">
            ✓ 已连接 - {{ selectedAccount.account }}@{{ selectedAccount.name }}
          </n-alert>

          <n-form-item v-if="loginStatus === 'connected'" label="选择目标项目">
            <n-select
              v-model:value="selectedProjectId"
              :options="projectOptions"
              :loading="projectsLoading"
              placeholder="选择目标项目"
              style="width: 400px;"
            />
          </n-form-item>

          <n-space v-if="loginStatus === 'connected' && selectedProjectId">
            <n-tag type="success">已选择项目</n-tag>
            <n-button @click="collectAndPreview" :loading="collecting" type="primary">下一步：预览</n-button>
          </n-space>
        </template>
      </n-space>
    </n-card>

    <!-- Step 3: Preview -->
    <n-card v-if="currentStep === 3" title="预览信息">
      <n-space vertical>
        <n-statistic label="总提交数" :value="commits.length" />

        <n-data-table
          :columns="commitColumns"
          :data="commits"
          :pagination="{ pageSize: 10 }"
          :bordered="false"
        />

        <n-divider />

        <n-h3>分支汇总</n-h3>
        <n-grid :cols="2" :x-gap="12">
          <n-gi v-for="group in branchGroups" :key="group.branch">
            <n-card :title="group.branch" size="small">
              <n-descriptions :column="2" size="small">
                <n-descriptions-item label="提交数">{{ group.commit_count }}</n-descriptions-item>
                <n-descriptions-item label="作者数">{{ group.authors.length }}</n-descriptions-item>
                <n-descriptions-item label="文件数">{{ group.summary.total_files }}</n-descriptions-item>
                <n-descriptions-item label="时间范围">{{ group.date_range.start.slice(0, 10) }}</n-descriptions-item>
              </n-descriptions>
            </n-card>
          </n-gi>
        </n-grid>

        <n-button type="primary" size="large" @click="executeWorkflow" :loading="executing" block>
          创建禅道任务
        </n-button>
      </n-space>
    </n-card>

    <!-- Step 4: Results -->
    <n-card v-if="currentStep === 4" title="执行结果">
      <n-space vertical>
        <n-result
          v-if="!executing && taskResults.length > 0"
          status="success"
          title="执行完成"
          :description="`成功创建 ${successCount} 个任务，失败 ${failCount} 个`"
        />

        <n-data-table
          :columns="resultColumns"
          :data="taskResults"
          :bordered="false"
        />

        <n-space>
          <n-button @click="resetWorkflow">重新开始</n-button>
          <n-button @click="router.push({ name: 'history' })">查看历史</n-button>
        </n-space>
      </n-space>
    </n-card>
  </n-space>
</template>

<script setup>
import { ref, computed, h, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useGitStore } from '@/stores/git.js'
import { useConfigStore } from '@/stores/config.js'

const router = useRouter()
const message = useMessage()
const gitStore = useGitStore()
const configStore = useConfigStore()

// Step state
const currentStep = ref(1)
const stepStatus = ref('process')

// Git state
const repos = ref([])
const scanning = ref(false)
const manualPath = ref('')
const selectedGitRepo = ref(null)

// 扫描状态
const scanProgress = computed(() => gitStore.scanProgress)
const isScanning = computed(() => gitStore.isScanning)

// Zentao state
const selectedAccountId = ref(null)
const selectedAccount = ref(null)
const loginStatus = ref('idle') // idle | logging_in | connected | error
const loginError = ref('')
const projectsLoading = ref(false)
const zentaoProjects = ref([])
const selectedProjectId = ref(null)
const connecting = ref(false)
const collecting = ref(false)

// Preview state
const commits = ref([])
const branchGroups = ref([])

// Execution state
const executing = ref(false)
const taskResults = ref([])

const projectOptions = computed(() =>
  zentaoProjects.value.map(p => ({ label: p.name, value: p.id }))
)

const selectedProject = computed(() =>
  zentaoProjects.value.find(p => p.id === selectedProjectId.value)
)

const successCount = computed(() => taskResults.value.filter(r => r.task_created).length)
const failCount = computed(() => taskResults.value.filter(r => !r.task_created).length)

// Commit table columns
const commitColumns = [
  { title: 'Hash', key: 'hash', width: 80 },
  { title: '作者', key: 'author', width: 120 },
  { title: '日期', key: 'date', width: 180 },
  { title: '提交信息', key: 'message', ellipsis: { tooltip: true } },
]

// Result table columns
const resultColumns = [
  { title: '分支', key: 'branch', width: 150 },
  { title: '提交数', key: 'commit_count', width: 80 },
  {
    title: '状态',
    key: 'task_created',
    width: 100,
    render: (row) => row.task_created
      ? h('span', { style: 'color: #18A058;' }, '成功')
      : h('span', { style: 'color: #D03050;' }, '失败'),
  },
  {
    title: '任务链接',
    key: 'task_url',
    render: (row) => row.task_url
      ? h('a', { href: row.task_url, target: '_blank', style: 'color: #2080F0;' }, `任务 #${row.task_id}`)
      : '-',
  },
  {
    title: '错误',
    key: 'error',
    ellipsis: { tooltip: true },
    render: (row) => row.error ? h('span', { style: 'color: #D03050;' }, row.error) : null,
  },
]

// Methods
const selectAndScanFolder = async () => {
  try {
    const selected = await open({ directory: true })
    console.log('[调试] 选择的目录:', selected)
    if (selected) {
      await gitStore.startFolderScan(selected)
      message.success('扫描已启动')
    }
  } catch (e) {
    console.error('[调试] 选择失败:', e)
    message.error(`选择失败: ${e}`)
  }
}

const scanManualPath = async () => {
  if (manualPath.value.trim()) {
    try {
      await gitStore.startFolderScan(manualPath.value.trim())
      message.success('扫描已启动')
    } catch (e) {
      message.error(`扫描失败: ${e}`)
    }
  }
}

const cancelScan = async () => {
  await gitStore.cancelScan()
  message.info('扫描已取消')
}

const scanRepos = async () => {
  scanning.value = true
  try {
    repos.value = await invoke('scan_git_repositories')
    if (repos.value.length === 0) {
      message.info('未找到本地 Git 仓库，请手动输入路径')
    } else {
      message.success(`找到 ${repos.value.length} 个 Git 仓库`)
    }
  } catch (e) {
    message.error(`扫描失败: ${e}`)
  } finally {
    scanning.value = false
  }
}

const selectGitProject = async (repo) => {
  selectedGitRepo.value = repo
  message.success(`已选择: ${repo.name}`)

  // Pre-collect commits
  try {
    commits.value = await invoke('collect_git_log', {
      projectPath: repo.path,
      maxCommits: 100,
    })
    message.success(`获取到 ${commits.value.length} 条提交记录`)
  } catch (e) {
    message.error(`收集提交失败: ${e}`)
  }
}

const onAccountSelected = async (accountId) => {
  // 查找选中的账号
  const account = configStore.zentaoAccounts.find(a => a.id === accountId)
  if (!account) {
    loginStatus.value = 'error'
    loginError.value = '未找到选中的账号'
    return
  }

  selectedAccount.value = account
  loginStatus.value = 'logging_in'
  loginError.value = ''
  projectsLoading.value = true

  try {
    console.log('[禅道] 尝试登录:', account.url, account.account)
    // 使用账号信息登录
    const token = await invoke('zentao_login', {
      url: account.url,
      account: account.account,
      password: account.password,
    })

    console.log('[禅道] 登录成功，token:', token)
    loginStatus.value = 'connected'
    message.success(`禅道连接成功: ${account.name}`)

    // 获取项目列表
    console.log('[禅道] 获取项目列表:', account.url, token)
    const projects = await invoke('zentao_get_projects', {
      url: account.url,
      token: token,
    })
    console.log('[禅道] 项目列表返回:', JSON.stringify(projects))

    zentaoProjects.value = projects
    selectedProjectId.value = null

    if (zentaoProjects.value.length > 0) {
      message.success(`获取到 ${zentaoProjects.value.length} 个项目`)
    } else {
      message.warning('未找到可用项目')
    }
  } catch (e) {
    console.error('[禅道] 错误:', e)
    loginStatus.value = 'error'
    loginError.value = `连接失败: ${e}`
    message.error(`连接失败: ${e}`)
  } finally {
    projectsLoading.value = false
  }
}

const collectAndPreview = async () => {
  if (!selectedGitRepo.value) {
    message.error('请先选择 Git 项目')
    return
  }

  collecting.value = true
  try {
    // 重新收集提交记录
    commits.value = await invoke('collect_git_log', {
      projectPath: selectedGitRepo.value.path,
      maxCommits: configStore.git.max_commits,
    })

    // 按分支分组
    branchGroups.value = await invoke('group_commits_by_branch', {
      commits: commits.value,
      branchPattern: configStore.git.branchPattern || null,
    })

    currentStep.value = 3
    message.success(`预览已加载: ${commits.value.length} 条提交`)
  } catch (e) {
    message.error(`加载预览失败: ${e}`)
  } finally {
    collecting.value = false
  }
}

const executeWorkflow = async () => {
  if (!selectedGitRepo.value || !selectedProject.value || !selectedAccount.value) {
    message.error('请完成所有选择')
    return
  }

  executing.value = true
  stepStatus.value = 'process'
  currentStep.value = 4

  try {
    const report = await invoke('execute_full_workflow', {
      account: selectedAccount.value,
      projectId: selectedProject.value.id,
      projectPath: selectedGitRepo.value.path,
      gitConfig: {
        max_commits: configStore.git.max_commits,
        include_merged: configStore.git.include_merged,
        branch_pattern: configStore.git.branch_pattern || '.*',
      },
      outputConfig: {
        report_dir: configStore.output.report_dir,
        verbose: configStore.output.verbose,
      },
    })

    taskResults.value = report.branches
    stepStatus.value = 'finish'
    message.success('执行完成')
  } catch (e) {
    stepStatus.value = 'error'
    message.error(`执行失败: ${e}`)
  } finally {
    executing.value = false
  }
}

const resetWorkflow = () => {
  currentStep.value = 1
  stepStatus.value = 'process'
  selectedGitRepo.value = null
  selectedAccountId.value = null
  selectedAccount.value = null
  loginStatus.value = 'idle'
  loginError.value = ''
  zentaoProjects.value = []
  selectedProjectId.value = null
  commits.value = []
  branchGroups.value = []
  taskResults.value = []
  gitStore.resetScan()
}

// 组件卸载时清理事件监听
onUnmounted(() => {
  gitStore.resetScan()
})
</script>
