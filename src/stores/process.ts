import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ProcessInfo {
  pid: number
  name: string
  cpu_usage: number
  memory_usage: number
  memory_percent: number
}

export const useProcessStore = defineStore('process', () => {
  // ── State ────────────────────────────────────────────────
  const processes = ref<ProcessInfo[]>([])
  const searchQuery = ref('')
  const loading = ref(false)
  const sortKey = ref<string>('cpu_usage')
  const sortOrder = ref<'ascend' | 'descend' | false>('descend')

  // ── Getters ──────────────────────────────────────────────
  const filteredProcesses = computed(() => {
    const q = searchQuery.value.trim().toLowerCase()
    if (!q) return processes.value
    return processes.value.filter((p) =>
      p.name.toLowerCase().includes(q)
    )
  })

  const sortedProcesses = computed(() => {
    const list = [...filteredProcesses.value]
    const key = sortKey.value
    const order = sortOrder.value

    if (!order) return list

    list.sort((a, b) => {
      const aVal = a[key as keyof ProcessInfo] as number
      const bVal = b[key as keyof ProcessInfo] as number
      return order === 'ascend' ? aVal - bVal : bVal - aVal
    })

    return list
  })

  // ── Actions ──────────────────────────────────────────────
  async function fetchProcesses() {
    loading.value = true
    try {
      processes.value = await invoke<ProcessInfo[]>('get_processes')
    } catch (e) {
      console.error('Failed to fetch processes:', e)
    } finally {
      loading.value = false
    }
  }

  async function killProcess(pid: number) {
    try {
      await invoke<boolean>('kill_process', { pid })
    } catch (e) {
      console.error('Failed to kill process:', e)
      throw e
    }
    await fetchProcesses()
  }

  function setSearchQuery(q: string) {
    searchQuery.value = q
  }

  function setSort(key: string) {
    if (sortKey.value === key) {
      // Toggle order for same column
      if (sortOrder.value === 'descend') {
        sortOrder.value = 'ascend'
      } else if (sortOrder.value === 'ascend') {
        sortOrder.value = 'descend'
      }
    } else {
      // Different column, default to descend
      sortKey.value = key
      sortOrder.value = 'descend'
    }
  }

  return {
    // state
    processes,
    searchQuery,
    loading,
    sortKey,
    sortOrder,
    // getters
    filteredProcesses,
    sortedProcesses,
    // actions
    fetchProcesses,
    killProcess,
    setSearchQuery,
    setSort,
  }
})