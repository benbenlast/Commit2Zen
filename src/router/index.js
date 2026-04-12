import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const routes = [
  { path: '/', name: 'home', component: HomeView },
  { path: '/dashboard', name: 'dashboard', component: () => import('../views/DashboardView.vue') },
  { path: '/config', name: 'config', component: () => import('../views/ConfigView.vue') },
  { path: '/llm', name: 'llm-config', component: () => import('../views/LLMConfigView.vue') },
  { path: '/execute', name: 'execute', component: () => import('../views/ExecuteView.vue') },
  { path: '/history', name: 'history', component: () => import('../views/HistoryView.vue') },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
