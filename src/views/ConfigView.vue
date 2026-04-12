<template>
  <div style="padding: 24px;">
    <n-card title="禅道账号管理">
      <template #header-extra>
        <n-space>
          <n-button size="small" @click="exportConfig" :loading="exporting">
            <template #icon><n-icon><DownloadOutline /></n-icon></template>
            导出配置
          </n-button>
          <n-button size="small" @click="triggerImport" :loading="importing">
            <template #icon><n-icon><CloudUploadOutline /></n-icon></template>
            导入配置
          </n-button>
          <input
            ref="importInputRef"
            type="file"
            accept=".json"
            style="display: none"
            @change="handleImport"
          />
        </n-space>
      </template>
      <n-space>
        <!-- 左侧：账号列表 -->
        <n-card size="small" style="width: 250px;">
          <template #header>
            <n-space justify="space-between">
              <n-text strong>已保存账号</n-text>
              <n-button size="small" @click="createNewAccount">+ 新增</n-button>
            </n-space>
          </template>
          <n-list hoverable clickable>
            <n-list-item
              v-for="account in configStore.zentaoAccounts"
              :key="account.id"
              @click="selectAccount(account)"
              :class="{ 'selected-account': selectedAccount && selectedAccount.id === account.id }"
            >
              <n-space vertical>
                <n-text strong>{{ account.name }}</n-text>
                <n-text depth="3" style="font-size: 12px;">{{ account.account }}@{{ account.url }}</n-text>
              </n-space>
            </n-list-item>
          </n-list>
          <n-empty v-if="configStore.zentaoAccounts.length === 0" description="暂无账号，请点击新增添加" />
        </n-card>

        <!-- 右侧：账号编辑表单 -->
        <n-card size="small" style="flex: 1;">
          <template #header>
            <n-text strong>{{ isEditing ? '编辑账号' : (isNew ? '新增账号' : '选择账号以编辑') }}</n-text>
          </template>
          <n-form
            v-if="selectedAccount || isNew"
            :model="editForm"
            label-placement="left"
            label-width="100"
          >
            <n-form-item label="显示名称">
              <n-input v-model:value="editForm.name" placeholder="例如：公司禅道" />
            </n-form-item>
            <n-form-item label="禅道地址">
              <n-input v-model:value="editForm.url" placeholder="http://192.168.1.23/zentao" />
            </n-form-item>
            <n-form-item label="账号">
              <n-input v-model:value="editForm.account" />
            </n-form-item>
            <n-form-item label="密码">
              <n-input v-model:value="editForm.password" type="password" show-password-on="click" />
            </n-form-item>
            <n-form-item label="默认指派给">
              <n-input v-model:value="editForm.assigned_to" />
            </n-form-item>
            <n-form-item label="任务类型">
              <n-select
                v-model:value="editForm.task_type"
                :options="[
                  { label: '开发', value: 'dev' },
                  { label: '设计', value: 'design' },
                  { label: '测试', value: 'qa' },
                  { label: '研究', value: 'research' },
                ]"
              />
            </n-form-item>
          </n-form>
          <n-empty v-else description="请从左侧选择一个账号进行编辑" />

          <n-space v-if="selectedAccount || isNew" style="margin-top: 16px;">
            <n-button type="primary" @click="saveAccount" :loading="saving">
              {{ isNew ? '添加账号' : '保存修改' }}
            </n-button>
            <n-button @click="testConnection" :loading="testing">
              测试连接
            </n-button>
            <n-button v-if="selectedAccount" type="error" @click="deleteAccount" :loading="deleting">
              删除账号
            </n-button>
            <n-button v-if="isNew" @click="cancelNew">
              取消
            </n-button>
          </n-space>
        </n-card>
      </n-space>
    </n-card>

    <!-- Git 配置和输出配置 -->
    <n-card title="其他配置" style="margin-top: 24px;">
      <n-space vertical>
        <n-form :model="otherConfig" label-placement="left" label-width="120">
          <n-divider title-placement="left">Git 配置</n-divider>
          <n-form-item label="最大提交数">
            <n-input-number v-model:value="otherConfig.git.maxCommits" :min="10" :max="500" />
          </n-form-item>
          <n-form-item label="包含合并提交">
            <n-switch v-model:value="otherConfig.git.includeMerged" />
          </n-form-item>
          <n-form-item label="分支过滤">
            <n-input v-model:value="otherConfig.git.branchPattern" placeholder=".*" />
          </n-form-item>

          <n-divider title-placement="left">输出配置</n-divider>
          <n-form-item label="报告目录">
            <n-input v-model:value="otherConfig.output.reportDir" />
          </n-form-item>
          <n-form-item label="详细输出">
            <n-switch v-model:value="otherConfig.output.verbose" />
          </n-form-item>
        </n-form>

        <n-space>
          <n-button type="primary" @click="saveOtherConfig" :loading="savingOther">保存其他配置</n-button>
        </n-space>
      </n-space>
    </n-card>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { DownloadOutline, CloudUploadOutline } from '@vicons/ionicons5'
import { useConfigStore } from '../stores/config.js'

const message = useMessage()
const dialog = useDialog()
const configStore = useConfigStore()

const selectedAccount = ref(null)
const isNew = ref(false)
const testing = ref(false)
const saving = ref(false)
const deleting = ref(false)
const savingOther = ref(false)
const exporting = ref(false)
const importing = ref(false)
const importInputRef = ref(null)

const editForm = ref({
  id: '',
  name: '',
  url: '',
  account: '',
  password: '',
  assigned_to: '',
  task_type: 'dev',
})

const otherConfig = ref({
  git: {
    maxCommits: 100,
    includeMerged: false,
    branchPattern: '.*',
  },
  output: {
    reportDir: 'reports',
    verbose: true,
  },
})

const isEditing = ref(false)

// Helper: safely get a value from an object, supporting both camelCase and snake_case
function safeGet(obj, camelKey, snakeKey, fallback) {
  if (obj && obj[camelKey] !== undefined) return obj[camelKey]
  if (obj && obj[snakeKey] !== undefined) return obj[snakeKey]
  return fallback
}

// 当 store 加载完成后，自动同步到表单
watch(
  () => configStore.loaded,
  (loaded) => {
    console.log('[ConfigView] watch loaded:', loaded, 'accounts:', configStore.zentaoAccounts)
    if (loaded) {
      otherConfig.value.git = {
        maxCommits: safeGet(configStore.git, 'maxCommits', 'max_commits', 100),
        includeMerged: safeGet(configStore.git, 'includeMerged', 'include_merged', false),
        branchPattern: safeGet(configStore.git, 'branchPattern', 'branch_pattern', '.*'),
      }
      otherConfig.value.output = {
        reportDir: safeGet(configStore.output, 'reportDir', 'report_dir', 'reports'),
        verbose: safeGet(configStore.output, 'verbose', 'verbose', true),
      }
    }
  },
  { immediate: true }
)

onMounted(async () => {
  console.log('[ConfigView] mounted, loaded:', configStore.loaded, 'accounts:', configStore.zentaoAccounts)
  // 如果 main.js 中已经加载了配置，这里再同步一次
  if (configStore.loaded) {
    otherConfig.value.git = {
      maxCommits: safeGet(configStore.git, 'maxCommits', 'max_commits', 100),
      includeMerged: safeGet(configStore.git, 'includeMerged', 'include_merged', false),
      branchPattern: safeGet(configStore.git, 'branchPattern', 'branch_pattern', '.*'),
    }
    otherConfig.value.output = {
      reportDir: safeGet(configStore.output, 'reportDir', 'report_dir', 'reports'),
      verbose: safeGet(configStore.output, 'verbose', 'verbose', true),
    }
  }
})

const selectAccount = (account) => {
  selectedAccount.value = account
  isNew.value = false
  editForm.value = { ...account }
  isEditing.value = true
}

const createNewAccount = () => {
  isNew.value = true
  selectedAccount.value = null
  editForm.value = {
    id: '',
    name: '',
    url: '',
    account: '',
    password: '',
    assigned_to: '',
    task_type: 'dev',
  }
  isEditing.value = false
}

const cancelNew = () => {
  isNew.value = false
  selectedAccount.value = null
  editForm.value = {
    id: '',
    name: '',
    url: '',
    account: '',
    password: '',
    assigned_to: '',
    task_type: 'dev',
  }
}

const saveAccount = async () => {
  saving.value = true
  try {
    if (!editForm.value.name || !editForm.value.url || !editForm.value.account || !editForm.value.password) {
      message.error('请填写完整的账号信息')
      return
    }

    if (isNew.value) {
      // 新增账号，生成 UUID
      const account = {
        ...editForm.value,
        id: crypto.randomUUID(),
      }
      await configStore.addAccount(account)
      message.success('账号已添加')
      isNew.value = false
      selectedAccount.value = account
    } else {
      // 更新账号
      await configStore.updateAccount(editForm.value)
      message.success('账号已更新')
      selectedAccount.value = { ...editForm.value }
    }
  } catch (e) {
    message.error(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

const deleteAccount = async () => {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除账号 "${selectedAccount.value.name}" 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      deleting.value = true
      try {
        await configStore.deleteAccount(selectedAccount.value.id)
        message.success('账号已删除')
        selectedAccount.value = null
        isNew.value = false
      } catch (e) {
        message.error(`删除失败: ${e}`)
      } finally {
        deleting.value = false
      }
    },
  })
}

const testConnection = async () => {
  if (!editForm.value.url || !editForm.value.account || !editForm.value.password) {
    message.error('请填写完整的连接信息')
    return
  }

  testing.value = true
  try {
    await configStore.testConnection(
      editForm.value.url,
      editForm.value.account,
      editForm.value.password
    )
    message.success('禅道连接成功')
  } catch (e) {
    message.error(`连接失败: ${e}`)
  } finally {
    testing.value = false
  }
}

const saveOtherConfig = async () => {
  savingOther.value = true
  try {
    // 更新 store 中的配置（使用 camelCase）
    configStore.git = {
      maxCommits: otherConfig.value.git.maxCommits,
      includeMerged: otherConfig.value.git.includeMerged,
      branchPattern: otherConfig.value.git.branchPattern,
    }
    configStore.output = {
      reportDir: otherConfig.value.output.reportDir,
      verbose: otherConfig.value.output.verbose,
    }
    await configStore.save()
    message.success('其他配置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  } finally {
    savingOther.value = false
  }
}

// ==================== 导出/导入配置 ====================

/**
 * 导出当前配置为 JSON 文件下载
 */
const exportConfig = async () => {
  exporting.value = true
  try {
    // 构建导出对象，使用 camelCase 命名以便导入时兼容
    const exportData = {
      version: '2.0',
      exportedAt: new Date().toISOString(),
      zentaoAccounts: configStore.zentaoAccounts || [],
      git: {
        maxCommits: safeGet(configStore.git, 'maxCommits', 'max_commits', 100),
        includeMerged: safeGet(configStore.git, 'includeMerged', 'include_merged', false),
        branchPattern: safeGet(configStore.git, 'branchPattern', 'branch_pattern', '.*'),
      },
      output: {
        reportDir: safeGet(configStore.output, 'reportDir', 'report_dir', 'reports'),
        verbose: safeGet(configStore.output, 'verbose', 'verbose', true),
      },
    }

    const json = JSON.stringify(exportData, null, 2)
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)

    const a = document.createElement('a')
    a.href = url
    a.download = `commit2zen-config-${new Date().toISOString().slice(0, 10)}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    message.success('配置已导出')
  } catch (e) {
    message.error(`导出失败: ${e}`)
  } finally {
    exporting.value = false
  }
}

/**
 * 触发文件选择对话框
 */
const triggerImport = () => {
  importInputRef.value?.click()
}

/**
 * 处理导入的 JSON 文件
 */
const handleImport = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  importing.value = true
  try {
    const text = await file.text()
    const imported = JSON.parse(text)

    // 验证基本结构
    if (!imported.zentaoAccounts && !imported.git && !imported.output) {
      throw new Error('无效的配置文件格式')
    }

    // 确认对话框
    dialog.warning({
      title: '确认导入',
      content: '导入配置将覆盖当前配置，确定要继续吗？',
      positiveText: '导入',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          // 应用导入的配置
          if (imported.zentaoAccounts) {
            configStore.zentaoAccounts = imported.zentaoAccounts
          }
          if (imported.git) {
            configStore.git = {
              maxCommits: safeGet(imported.git, 'maxCommits', 'max_commits', 100),
              includeMerged: safeGet(imported.git, 'includeMerged', 'include_merged', false),
              branchPattern: safeGet(imported.git, 'branchPattern', 'branch_pattern', '.*'),
            }
          }
          if (imported.output) {
            configStore.output = {
              reportDir: safeGet(imported.output, 'reportDir', 'report_dir', 'reports'),
              verbose: safeGet(imported.output, 'verbose', 'verbose', true),
            }
          }

          // 同步到本地表单
          otherConfig.value.git = { ...configStore.git }
          otherConfig.value.output = { ...configStore.output }

          // 保存到后端
          await configStore.save()

          message.success('配置已导入并保存')
        } catch (e) {
          message.error(`导入保存失败: ${e}`)
        }
      },
      onNegativeClick: () => {
        // 重置 input 以便可以重新选择同一文件
        event.target.value = ''
      },
    })
  } catch (e) {
    message.error(`导入失败: 文件格式错误 - ${e.message}`)
  } finally {
    importing.value = false
    // 重置 input
    event.target.value = ''
  }
}
</script>

<style scoped>
.selected-account {
  background-color: #f0f0f0;
}
</style>
