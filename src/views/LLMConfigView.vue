<template>
  <div style="padding: 24px;">
    <!-- 服务商管理 -->
    <n-card title="LLM 服务商管理">
      <n-space>
        <!-- 左侧：服务商列表 -->
        <n-card size="small" style="width: 280px;">
          <template #header>
            <n-space justify="space-between">
              <n-text strong>服务商列表</n-text>
              <n-button size="small" @click="addProvider">+ 新增</n-button>
            </n-space>
          </template>
          <n-list hoverable clickable>
            <n-list-item
              v-for="(provider, type) in llmStore.allProviders"
              :key="type"
              @click="selectProvider(type)"
              :class="{ 'selected-provider': selectedProvider === type }"
            >
              <n-space vertical>
                <n-space justify="space-between">
                  <n-text strong>{{ getProviderLabel(type) }}</n-text>
                  <n-switch
                    :value="provider.enabled"
                    @update:value="(val) => toggleEnabled(type, val)"
                    size="small"
                    @click.stop
                  />
                </n-space>
                <n-text depth="3" style="font-size: 12px;">
                  {{ provider.model }}
                  <n-tag v-if="provider.enabled" type="success" size="tiny" style="margin-left: 4px;">
                    已启用
                  </n-tag>
                  <n-tag v-else type="default" size="tiny" style="margin-left: 4px;">
                    已禁用
                  </n-tag>
                </n-text>
              </n-space>
            </n-list-item>
          </n-list>
          <n-empty v-if="Object.keys(llmStore.allProviders).length === 0" description="暂无服务商配置" />
        </n-card>

        <!-- 右侧：服务商编辑表单 -->
        <n-card size="small" style="flex: 1;">
          <template #header>
            <n-space justify="space-between">
              <n-text strong>{{ selectedProvider ? getProviderLabel(selectedProvider) : '选择服务商以编辑' }}</n-text>
              <n-space v-if="selectedProvider">
                <n-tag :type="getProviderTagType(selectedProvider)">{{ selectedProvider }}</n-tag>
              </n-space>
            </n-space>
          </template>
          <n-form
            v-if="selectedProvider && editForm"
            :model="editForm"
            label-placement="left"
            label-width="120"
          >
            <n-form-item label="服务商类型">
              <n-select
                v-model:value="editForm.type"
                :options="providerTypeOptions"
                :disabled="true"
              />
            </n-form-item>
            <n-form-item label="显示名称">
              <n-input v-model:value="editForm.displayName" placeholder="例如：我的 OpenAI" />
            </n-form-item>
            <n-form-item label="API Key" v-if="editForm.type !== 'ollama'">
              <n-input
                v-model:value="editForm.apiKey"
                type="password"
                show-password-on="click"
                placeholder="sk-..."
              />
            </n-form-item>
            <n-form-item label="Base URL">
              <n-input v-model:value="editForm.baseUrl" placeholder="https://api.openai.com/v1" />
            </n-form-item>
            <n-form-item label="Model 名称">
              <n-input v-model:value="editForm.model" placeholder="gpt-4o-mini" />
            </n-form-item>
            <n-form-item label="Temperature">
              <div style="display: flex; align-items: center; gap: 8px; width: 100%;">
                <n-slider
                  v-model:value="editForm.temperature"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  style="flex: 1;"
                />
                <n-input-number
                  v-model:value="editForm.temperature"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  size="small"
                  style="width: 90px; flex-shrink: 0;"
                />
              </div>
            </n-form-item>
            <n-form-item label="Max Tokens">
              <n-input-number
                v-model:value="editForm.maxTokens"
                :min="1"
                :max="128000"
                :step="100"
                style="width: 200px;"
              />
            </n-form-item>
            <n-form-item label="启用">
              <n-switch v-model:value="editForm.enabled" />
            </n-form-item>
          </n-form>
          <n-empty v-else description="请从左侧选择一个服务商进行编辑" />

          <n-space v-if="selectedProvider && editForm" style="margin-top: 16px;">
            <n-button type="primary" @click="saveProvider" :loading="saving">
              保存配置
            </n-button>
            <n-button @click="testConnection" :loading="testing">
              测试连接
            </n-button>
            <n-button @click="resetProvider">
              重置为默认
            </n-button>
          </n-space>
        </n-card>
      </n-space>
    </n-card>

    <!-- 任务分配配置 -->
    <n-card title="任务分配配置" style="margin-top: 24px;">
      <n-collapse :default-expanded-names="['task-assignments']">
        <n-collapse-item name="task-assignments" title="任务 → LLM 映射">
          <n-space vertical>
            <n-space vertical v-for="task in taskList" :key="task.key" style="border: 1px solid #e0e0e6; border-radius: 8px; padding: 12px;">
              <n-space justify="space-between" align="center">
                <n-space align="center">
                  <n-text strong>{{ task.label }}</n-text>
                  <n-switch
                    :value="taskAssignments[task.key]?.enabled ?? false"
                    @update:value="(val) => toggleTaskTaskAssignment(task.key, val)"
                    size="small"
                  >
                    <template #checked>
                      启用
                    </template>
                    <template #unchecked>
                      禁用
                    </template>
                  </n-switch>
                </n-space>
                <n-select
                  v-model:value="taskAssignments[task.key].provider"
                  :options="enabledProviderOptions"
                  placeholder="选择 LLM 服务商"
                  style="width: 200px;"
                  :disabled="!(taskAssignments[task.key]?.enabled ?? false)"
                  @update:value="(val) => updateTaskAssignment(task.key, val)"
                />
              </n-space>
              <n-text depth="3" style="font-size: 12px;">{{ task.description }}</n-text>
            </n-space>

            <n-space>
              <n-button type="primary" @click="saveTaskAssignments" :loading="savingAssignments">
                保存任务分配
              </n-button>
            </n-space>
          </n-space>
        </n-collapse-item>
      </n-collapse>
    </n-card>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { useLLMStore } from '../stores/llm.js'
import { getLLMManager } from '../utils/llm'

const message = useMessage()
const dialog = useDialog()
const llmStore = useLLMStore()

const selectedProvider = ref(null)
const saving = ref(false)
const testing = ref(false)
const savingAssignments = ref(false)

const providerTypeOptions = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Claude (Anthropic)', value: 'claude' },
  { label: 'Gemini (Google)', value: 'gemini' },
  { label: 'Ollama (本地)', value: 'ollama' },
]

const taskAssignments = ref({
  commitSummary: { enabled: false, provider: null },
  taskDescription: { enabled: false, provider: null },
  branchSuggestion: { enabled: false, provider: null },
  trendAnalysis: { enabled: false, provider: null },
})

const taskList = [
  {
    key: 'commitSummary',
    label: 'Commit 摘要',
    description: '分析 Git 提交历史，生成项目变更摘要',
  },
  {
    key: 'taskDescription',
    label: '任务描述',
    description: '为禅道任务自动生成结构化的任务描述',
  },
  {
    key: 'branchSuggestion',
    label: '分支建议',
    description: '基于提交内容智能推荐分支名称',
  },
  {
    key: 'trendAnalysis',
    label: '趋势分析',
    description: '分析项目开发趋势和活跃度统计',
  },
]

const editForm = ref(null)

const enabledProviderOptions = computed(() => {
  const options = []
  for (const [type, provider] of Object.entries(llmStore.allProviders)) {
    if (provider.enabled) {
      options.push({
        label: `${getProviderLabel(type)} (${provider.model})`,
        value: type,
      })
    }
  }
  if (options.length === 0) {
    options.push({ label: '请先启用至少一个服务商', value: null, disabled: true })
  }
  return options
})

// 当 store 加载完成后，同步数据到表单
watch(
  () => llmStore.llmConfig,
  (config) => {
    if (config) {
      syncTaskAssignmentsFromStore()
    }
  },
  { immediate: true }
)

onMounted(async () => {
  if (llmStore.llmConfig) {
    syncTaskAssignmentsFromStore()
  }
})

function syncTaskAssignmentsFromStore() {
  const assignments = llmStore.taskAssignments || []
  taskAssignments.value = {
    commitSummary: { enabled: false, provider: null },
    taskDescription: { enabled: false, provider: null },
    branchSuggestion: { enabled: false, provider: null },
    trendAnalysis: { enabled: false, provider: null },
  }
  for (const assignment of assignments) {
    if (assignment.taskType in taskAssignments.value) {
      taskAssignments.value[assignment.taskType] = {
        enabled: assignment.enabled,
        provider: assignment.providerType || null,
      }
    }
  }
}

function getProviderLabel(type) {
  const labels = {
    openai: 'OpenAI',
    claude: 'Claude',
    gemini: 'Gemini',
    ollama: 'Ollama',
  }
  return labels[type] || type
}

function getProviderTagType(type) {
  const types = {
    openai: 'info',
    claude: 'warning',
    gemini: 'success',
    ollama: 'default',
  }
  return types[type] || 'default'
}

function selectProvider(type) {
  selectedProvider.value = type
  const provider = llmStore.allProviders[type]
  if (provider) {
    editForm.value = {
      type: provider.type,
      displayName: getProviderLabel(provider.type),
      enabled: provider.enabled,
      apiKey: provider.apiKey || '',
      baseUrl: provider.baseUrl || '',
      model: provider.model || '',
      temperature: provider.extra?.temperature ?? 0.7,
      maxTokens: provider.extra?.maxTokens ?? 4096,
    }
  }
}

function addProvider() {
  message.info('请选择左侧已有的服务商进行配置，或重置为默认值')
}

function toggleEnabled(type, enabled) {
  const provider = llmStore.allProviders[type]
  if (provider) {
    llmStore.updateProviderConfig(type, { enabled })
    if (selectedProvider.value === type && editForm.value) {
      editForm.value.enabled = enabled
    }
  }
}

async function saveProvider() {
  if (!selectedProvider.value || !editForm.value) return

  if (editForm.value.type !== 'ollama' && !editForm.value.apiKey) {
    message.error('请填写 API Key')
    return
  }
  if (!editForm.value.model) {
    message.error('请填写 Model 名称')
    return
  }

  saving.value = true
  try {
    const updates = {
      enabled: editForm.value.enabled,
      apiKey: editForm.value.apiKey,
      baseUrl: editForm.value.baseUrl,
      model: editForm.value.model,
      extra: {
        temperature: editForm.value.temperature,
        maxTokens: editForm.value.maxTokens,
      },
    }
    llmStore.updateProviderConfig(selectedProvider.value, updates)
    await llmStore.saveConfig()
    message.success('配置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

async function testConnection() {
  if (!selectedProvider.value) return

  testing.value = true
  try {
    const success = await llmStore.testConnection(selectedProvider.value)
    if (success) {
      message.success(`${getProviderLabel(selectedProvider.value)} 连接成功`)
    } else {
      message.error(`${getProviderLabel(selectedProvider.value)} 连接失败`)
    }
  } catch (e) {
    message.error(`连接测试出错: ${e}`)
  } finally {
    testing.value = false
  }
}

function resetProvider() {
  dialog.warning({
    title: '确认重置',
    content: `确定要重置 ${getProviderLabel(selectedProvider.value)} 的配置为默认值吗？`,
    positiveText: '重置',
    negativeText: '取消',
    onPositiveClick: () => {
      llmStore.resetConfig()
      selectProvider(selectedProvider.value)
      message.success('已重置为默认配置')
    },
  })
}

function toggleTaskTaskAssignment(taskType, enabled) {
  taskAssignments.value[taskType].enabled = enabled
  if (!enabled) {
    taskAssignments.value[taskType].provider = null
  }
}

function updateTaskAssignment(taskType, providerType) {
  taskAssignments.value[taskType].provider = providerType
}

async function saveTaskAssignments() {
  savingAssignments.value = true
  try {
    const taskLabels = {
      commitSummary: 'Commit 摘要',
      taskDescription: '任务描述',
      branchSuggestion: '分支建议',
      trendAnalysis: '趋势分析',
    }

    // 更新每个任务分配
    for (const [taskType, config] of Object.entries(taskAssignments.value)) {
      if (config.enabled && config.provider) {
        llmStore.setTaskAssignment({
          taskType,
          providerType: config.provider,
          description: taskLabels[taskType] || taskType,
          enabled: config.enabled,
        })
      } else {
        llmStore.setTaskAssignment({
          taskType,
          providerType: null,
          description: taskLabels[taskType] || taskType,
          enabled: false,
        })
      }
    }

    await llmStore.saveConfig()
    message.success('任务分配配置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  } finally {
    savingAssignments.value = false
  }
}
</script>

<style scoped>
.selected-provider {
  background-color: #f0f0f0;
}
</style>
