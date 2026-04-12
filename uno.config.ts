import { defineConfig, presetUno, presetAttributify, presetIcons } from 'unocss'

export default defineConfig({
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
      warn: true,
    }),
  ],
  shortcuts: {
    // 布局快捷类
    'flex-center': 'flex items-center justify-center',
    'flex-between': 'flex items-center justify-between',
    'flex-col-center': 'flex flex-col items-center justify-center',
    
    // 卡片基础样式
    'card-base': 'rounded-lg bg-white dark:bg-gray-800 p-4 shadow-sm hover:shadow-md transition-shadow',
    'card-hover': 'hover:shadow-lg transition-shadow cursor-pointer',
    
    // 文本样式
    'text-primary': 'text-[#1890ff] dark:text-[#40a9ff]',
    'text-success': 'text-[#52c41a]',
    'text-warning': 'text-[#faad14]',
    'text-error': 'text-[#ff4d4f]',
    'text-muted': 'text-gray-500 dark:text-gray-400',
    
    // 按钮样式
    'btn-primary': 'px-4 py-2 bg-[#1890ff] text-white rounded hover:bg-[#40a9ff] transition-colors',
    'btn-secondary': 'px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors',
    
    // 输入框样式
    'input-base': 'w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-[#1890ff] transition-all',
  },
  theme: {
    colors: {
      primary: '#1890ff',
      success: '#52c41a',
      warning: '#faad14',
      error: '#ff4d4f',
    },
    borderRadius: {
      'card': '8px',
      'btn': '6px',
    },
  },
})
