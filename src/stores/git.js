import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useGitStore = defineStore('git', {
  state: () => ({
    selectedProject: null,
    commits: [],
    branchGroups: [],
    scanning: false,
    collecting: false,
  }),

  getters: {
    totalCommits: (state) => state.commits.length,
    branchCount: (state) => state.branchGroups.length,
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
