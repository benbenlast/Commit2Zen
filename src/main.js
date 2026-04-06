import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui'
import App from './App.vue'
import router from './router'
import { useConfigStore } from './stores/config.js'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.use(router)
app.use(naive)

// 预加载配置
const configStore = useConfigStore()
configStore.load().then(() => {
  console.log('[main.js] 配置加载完成:', JSON.stringify(configStore.$state))
})

app.mount('#app')
