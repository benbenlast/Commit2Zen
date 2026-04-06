<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <n-layout has-sider>
        <n-layout-sider bordered :width="200">
          <n-menu
            :value="route.name"
            :options="menuOptions"
            @update:value="handleMenuSelect"
          />
        </n-layout-sider>
        <n-layout-content>
          <router-view />
        </n-layout-content>
      </n-layout>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { ref, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NConfigProvider, NMessageProvider, NLayout, NLayoutSider, NLayoutContent, NMenu } from 'naive-ui'
import { HomeOutline, SettingsOutline, PlayOutline, TimeOutline } from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const theme = ref(null)

const menuOptions = [
  {
    label: '首页',
    key: 'home',
    icon: () => h(HomeOutline),
  },
  {
    label: '配置',
    key: 'config',
    icon: () => h(SettingsOutline),
  },
  {
    label: '执行',
    key: 'execute',
    icon: () => h(PlayOutline),
  },
  {
    label: '历史',
    key: 'history',
    icon: () => h(TimeOutline),
  },
]

const handleMenuSelect = (key) => {
  router.push({ name: key })
}
</script>

<style>
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>
