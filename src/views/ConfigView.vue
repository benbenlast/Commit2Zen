<template>
  <div style="padding: 24px;">
    <n-card title="配置管理">
      <n-space vertical>
        <n-form :model="config" label-placement="left" label-width="120">
          <n-divider title-placement="left">禅道配置</n-divider>
          <n-form-item label="地址">
            <n-input v-model:value="config.zentao.url" placeholder="http://192.168.1.23/zentao" />
          </n-form-item>
          <n-form-item label="账号">
            <n-input v-model:value="config.zentao.account" />
          </n-form-item>
          <n-form-item label="密码">
            <n-input v-model:value="config.zentao.password" type="password" show-password-on="click" />
          </n-form-item>
          <n-form-item label="默认项目ID">
            <n-input-number v-model:value="config.zentao.project_id" :min="1" />
          </n-form-item>
          <n-form-item label="默认指派给">
            <n-input v-model:value="config.zentao.assigned_to" />
          </n-form-item>
          <n-form-item label="任务类型">
            <n-select
              v-model:value="config.zentao.task_type"
              :options="[
                { label: '开发', value: 'dev' },
                { label: '设计', value: 'design' },
                { label: '测试', value: 'qa' },
                { label: '研究', value: 'research' },
              ]"
            />
          </n-form-item>

          <n-divider title-placement="left">Git 配置</n-divider>
          <n-form-item label="最大提交数">
            <n-input-number v-model:value="config.git.max_commits" :min="10" :max="500" />
          </n-form-item>
          <n-form-item label="分支过滤">
            <n-input v-model:value="config.git.branch_pattern" placeholder=".*" />
          </n-form-item>

          <n-divider title-placement="left">输出配置</n-divider>
          <n-form-item label="报告目录">
            <n-input v-model:value="config.output.report_dir" />
          </n-form-item>
        </n-form>

        <n-space>
          <n-button type="primary" @click="saveConfig">保存配置</n-button>
          <n-button @click="loadConfig">重新加载</n-button>
          <n-button @click="testConnection" :loading="testing">测试连接</n-button>
        </n-space>
      </n-space>
    </n-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()
const testing = ref(false)

const config = ref({
  zentao: {
    url: '',
    account: '',
    password: '',
    project_id: 1,
    assigned_to: '',
    task_type: 'dev',
  },
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

onMounted(async () => {
  await loadConfig()
})

const loadConfig = async () => {
  try {
    config.value = await invoke('load_config')
    message.success('配置已加载')
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

const saveConfig = async () => {
  try {
    await invoke('save_config', { config: config.value })
    message.success('配置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  }
}

const testConnection = async () => {
  testing.value = true
  try {
    await invoke('zentao_login', {
      url: config.value.zentao.url,
      account: config.value.zentao.account,
      password: config.value.zentao.password,
    })
    message.success('禅道连接成功')
  } catch (e) {
    message.error(`连接失败: ${e}`)
  } finally {
    testing.value = false
  }
}
</script>
