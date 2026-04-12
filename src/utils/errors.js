export const ERROR_MESSAGES = {
  'REPOSITORY_NOT_FOUND': '未找到 Git 仓库，请检查路径是否正确',
  'GIT_ERROR': 'Git 操作失败，请检查仓库状态',
  'DATE_FILTER_ERROR': '日期范围无效，请检查选择的日期',
  'ZENTAO_API_ERROR': '禅道 API 请求失败，请检查网络连接和配置',
  'ZENTAO_CONNECTION_ERROR': '无法连接到禅道服务器，请检查 URL 和凭据',
  'LLM_ERROR': 'AI 服务调用失败，请检查 API Key 和网络连接',
  'CONFIG_ERROR': '配置错误，请检查并重新配置',
  'FILE_ERROR': '文件操作失败，请检查文件权限',
  'NETWORK_ERROR': '网络连接失败，请检查网络设置',
}

export function getUserFriendlyError(error) {
  if (typeof error === 'string') {
    // 尝试从错误消息中提取错误码
    for (const [code, message] of Object.entries(ERROR_MESSAGES)) {
      if (error.includes(code)) {
        return message
      }
    }
    return error
  }

  if (error && error.code) {
    return ERROR_MESSAGES[error.code] || error.message || '未知错误'
  }

  return String(error)
}

export function showError(message, messageApi) {
  const userMessage = getUserFriendlyError(message)
  messageApi.error(userMessage)
}
