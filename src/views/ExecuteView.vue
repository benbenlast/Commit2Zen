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
          <n-button @click="scanRepos" :loading="scanning">
            扫描本地仓库
          </n-button>
          <n-input v-model:value="manualPath" placeholder="或手动输入路径" style="width: 400px;" />
          <n-button @click="selectManual">浏览...</n-button>
        </n-space>

        <n-list v-if="repos.length > 0" bordered>
          <n-list-item v-for="repo in repos" :key="repo.path">
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
        <n-form :model="zentaoForm" label-placement="left" label-width="100">
          <n-form-item label="禅道地址">
            <n-input v-model:value="zentaoForm.url" placeholder="http://192.168.1.23/zentao" />
          </n-form-item>
          <n-form-item label="账号">
            <n-input v-model:value="zentaoForm.account" />
          </n-form-item>
          <n-form-item label="密码">
            <n-input v-model:value="zentaoForm.password" type="password" show-password-on="click" />
          </n-form-item>
        </n-form>

        <n-space>
          <n-button @click="testConnection" :loading="connecting">
            测试连接
          </n-button>
        </n-space>

        <n-select
          v-if="zentaoProjects.length > 0"
          v-model:value="selectedProjectId"
          :options="projectOptions"
          placeholder="选择目标项目"
          style="width: 400px;"
        />

        <n-space v-if="zentaoForm.token && selectedProjectId">
          <n-tag type="success">已连接并选择项目</n-tag>
          <n-button @click="collectAndPreview" :loading="collecting">下一步：预览</n-button>
        </n-space>
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
import { ref, computed, h } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const message = useMessage()

// Step state
const currentStep = ref(1)
const stepStatus = ref('process')

// Git state
const repos = ref([])
const scanning = ref(false)
const manualPath = ref('')
const selectedGitRepo = ref(null)

// Zentao state
const zentaoForm = ref({ url: '', account: '', password: '', token: '' })
const connecting = ref(false)
const zentaoProjects = ref([])
const selectedProjectId = ref(null)

// Preview state
const collecting = ref(false)
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

const selectManual = async () => {
  try {
    const selected = await open({ directory: true })
    if (selected) {
      manualPath.value = selected
      selectedGitRepo.value = { path: selected, name: selected.split(/[\\/]/).pop() }
      message.success(`已选择: ${selectedGitRepo.value.name}`)
    }
  } catch (e) {
    message.error(`选择失败: ${e}`)
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

const testConnection = async () => {
  if (!zentaoForm.value.url || !zentaoForm.value.account || !zentaoForm.value.password) {
    message.error('请填写完整的禅道信息')
    return
  }

  connecting.value = true
  try {
    zentaoForm.value.token = await invoke('zentao_login', {
      url: zentaoForm.value.url,
      account: zentaoForm.value.account,
      password: zentaoForm.value.password,
    })
    message.success('禅道连接成功')

    // Fetch projects
    zentaoProjects.value = await invoke('zentao_get_projects', {
      url: zentaoForm.value.url,
      token: zentaoForm.value.token,
    })

    if (zentaoProjects.value.length > 0) {
      message.success(`获取到 ${zentaoProjects.value.length} 个项目`)
    }
  } catch (e) {
    message.error(`连接失败: ${e}`)
  } finally {
    connecting.value = false
  }
}

const collectAndPreview = async () => {
  collecting.value = true
  try {
    branchGroups.value = await invoke('group_commits_by_branch', {
      commits: commits.value,
      branchPattern: null,
    })
    currentStep.value = 3
  } catch (e) {
    message.error(`分支分组失败: ${e}`)
  } finally {
    collecting.value = false
  }
}

const executeWorkflow = async () => {
  if (!selectedGitRepo.value || !selectedProject.value) {
    message.error('请选择 Git 项目和禅道目标')
    return
  }

  executing.value = true
  stepStatus.value = 'process'
  currentStep.value = 4

  try {
    const config = {
      zentao: {
        url: zentaoForm.value.url,
        account: zentaoForm.value.account,
        password: zentaoForm.value.password,
        project_id: selectedProject.value.id,
        assigned_to: zentaoForm.value.account,
        task_type: 'dev',
      },
      git: { max_commits: 100, include_merged: false, branch_pattern: '.*' },
      output: { report_dir: 'reports', verbose: true },
    }

    const report = await invoke('execute_full_workflow', {
      config,
      projectPath: selectedGitRepo.value.path,
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
  zentaoForm.value = { url: '', account: '', password: '', token: '' }
  zentaoProjects.value = []
  selectedProjectId.value = null
  commits.value = []
  branchGroups.value = []
  taskResults.value = []
}
</script>
