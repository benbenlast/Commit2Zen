<template>
  <div style="padding: 24px;">
    <n-card title="禅道账号管理">
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
            <n-input-number v-model:value="otherConfig.git.max_commits" :min="10" :max="500" />
          </n-form-item>
          <n-form-item label="分支过滤">
            <n-input v-model:value="otherConfig.git.branch_pattern" placeholder=".*" />
          </n-form-item>

          <n-divider title-placement="left">输出配置</n-divider>
          <n-form-item label="报告目录">
            <n-input v-model:value="otherConfig.output.report_dir" />
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
    max_commits: 100,
    include_merged: false,
    branch_pattern: '.*',
  },
  output: {
    report_dir: 'reports',
    verbose: true,
  },
})

const isEditing = ref(false)

// 当 store 加载完成后，自动同步到表单
watch(
  () => configStore.loaded,
  (loaded) => {
    console.log('[ConfigView] watch loaded:', loaded, 'accounts:', configStore.zentaoAccounts)
    if (loaded) {
      otherConfig.value.git = { ...configStore.git }
      otherConfig.value.output = { ...configStore.output }
    }
  },
  { immediate: true }
)

onMounted(async () => {
  console.log('[ConfigView] mounted, loaded:', configStore.loaded, 'accounts:', configStore.zentaoAccounts)
  // 如果 main.js 中已经加载了配置，这里再同步一次
  if (configStore.loaded) {
    otherConfig.value.git = { ...configStore.git }
    otherConfig.value.output = { ...configStore.output }
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
    configStore.git = { ...otherConfig.value.git }
    configStore.output = { ...otherConfig.value.output }
    await configStore.save()
    message.success('其他配置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  } finally {
    savingOther.value = false
  }
}
</script>

<style scoped>
.selected-account {
  background-color: #f0f0f0;
}
</style>
