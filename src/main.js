import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import naive from 'naive-ui'
import 'virtual:uno.css'
import App from './App.vue'
import router from './router'
import { useConfigStore } from './stores/config.js'
import { useLLMStore } from './stores/llm.js'

const app = createApp(App)
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(pinia)
app.use(router)
app.use(naive)

// 预加载配置
const configStore = useConfigStore()
configStore.load().then(() => {
  console.log('[main.js] 配置加载完成:', JSON.stringify(configStore.$state))
})

// 预加载 LLM 配置
const llmStore = useLLMStore()
llmStore.loadConfig().then((success) => {
  if (success) {
    console.log('[main.js] LLM 配置加载完成')
  } else {
    console.warn('[main.js] LLM 配置加载失败，将使用默认配置')
  }
})

app.mount('#app')
