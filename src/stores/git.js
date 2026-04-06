import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const useGitStore = defineStore('git', {
  state: () => ({
    selectedProject: null,
    commits: [],
    branchGroups: [],
    scanning: false,
    collecting: false,
    // 全盘扫描状态
    scanProgress: null,
    activeScanId: null,
    scannedRepos: [],
  }),

  getters: {
    totalCommits: (state) => state.commits.length,
    branchCount: (state) => state.branchGroups.length,
    isScanning: (state) => state.activeScanId !== null,
  },

  actions: {
    async scanRepositories() {
      this.scanning = true
      try {
        const repos = await invoke('scan_git_repositories')
        return repos
      } catch (e) {
        console.error('扫描失败:', e)
        throw e
      } finally {
        this.scanning = false
      }
    },

    async startFolderScan(folderPath) {
      console.log('[调试] startFolderScan 调用:', folderPath)
      try {
        this.scanProgress = {
          status: 'started',
          currentDirectory: '',
          reposFound: 0,
          directoriesScanned: 0,
          percentage: 0,
          repo: null,
        }
        this.scannedRepos = []
        
        // 先注册事件监听器，再调用 invoke，避免错过事件
        console.log('[调试] 注册事件监听器')
        const unlisten = await listen('scan-progress', (event) => {
          console.log('[调试] 收到 scan-progress 事件:', event.payload)
          this.scanProgress = event.payload
          
          // 当发现新仓库时，添加到 scannedRepos
          if (event.payload.status === 'found' && event.payload.repo) {
            // 检查是否已存在
            const exists = this.scannedRepos.some(r => r.path === event.payload.repo.path)
            if (!exists) {
              this.scannedRepos.push(event.payload.repo)
            }
          }
          
          // 如果扫描完成或取消，清理状态
          if (event.payload.status === 'completed' || event.payload.status === 'cancelled') {
            console.log('[调试] 扫描结束:', event.payload.status)
            console.log('[调试] 共发现', this.scannedRepos.length, '个仓库')
            this.activeScanId = null
          }
        })
        
        // 存储 unlisten 以便后续清理
        this._scanProgressUnlisten = unlisten
        
        // 现在调用扫描
        console.log('[调试] 调用 start_folder_scan')
        const scanId = await invoke('start_folder_scan', { folderPath })
        console.log('[调试] 收到 scanId:', scanId)
        this.activeScanId = scanId
        
        return scanId
      } catch (e) {
        console.error('[调试] 扫描异常:', e)
        console.error('文件夹扫描失败:', e)
        this.activeScanId = null
        throw e
      }
    },

    async cancelScan() {
      if (this.activeScanId) {
        try {
          await invoke('cancel_scan', { scanId: this.activeScanId })
        } catch (e) {
          console.error('取消扫描失败:', e)
        }
      }
    },

    resetScan() {
      this.scanProgress = null
      this.activeScanId = null
      this.scannedRepos = []
      if (this._scanProgressUnlisten) {
        this._scanProgressUnlisten()
        this._scanProgressUnlisten = null
      }
    },

    async collectCommits(projectPath, maxCommits = 100) {
      this.collecting = true
      try {
        this.commits = await invoke('collect_git_log', {
          projectPath,
          maxCommits,
        })
        return this.commits
      } catch (e) {
        console.error('收集提交失败:', e)
        throw e
      } finally {
        this.collecting = false
      }
    },

    async groupBranches(branchPattern = null) {
      try {
        this.branchGroups = await invoke('group_commits_by_branch', {
          commits: this.commits,
          branchPattern,
        })
        return this.branchGroups
      } catch (e) {
        console.error('分支分组失败:', e)
        throw e
      }
    },

    selectProject(repo) {
      this.selectedProject = repo
    },
  },
})
