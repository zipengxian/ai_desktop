import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── TypeScript interfaces matching Rust structs ──────────────

export interface CpuInfo {
  usage: number
}

export interface MemoryInfo {
  total: number
  used: number
  available: number
  usage_percent: number
}

export interface SwapInfo {
  total: number
  used: number
  usage_percent: number
}

export interface SystemInfo {
  cpu: CpuInfo
  memory: MemoryInfo
  swap: SwapInfo
}

export interface DiskInfo {
  name: string
  mount_point: string
  total_space: number
  available_space: number
  used_space: number
  usage_percent: number
}

export interface NetworkInfo {
  name: string
  received_bytes: number
  transmitted_bytes: number
}

// ── Formatting utility ──────────────────────────────────────

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const idx = Math.min(i, units.length - 1)
  return parseFloat((bytes / Math.pow(k, idx)).toFixed(2)) + ' ' + units[idx]
}

// ── Store ───────────────────────────────────────────────────

export const useSystemStore = defineStore('system', () => {
  // State
  const systemInfo = ref<SystemInfo | null>(null)
  const disks = ref<DiskInfo[]>([])
  const networks = ref<NetworkInfo[]>([])
  const loading = ref(false)

  // Previous network data for rate calculation
  const prevNetworks = ref<{ name: string; received_bytes: number; transmitted_bytes: number; timestamp: number }[]>([])

  // ── Computed ──────────────────────────────────────────────

  const cpuUsagePercent = computed(() => {
    return systemInfo.value?.cpu.usage ?? 0
  })

  const memoryUsagePercent = computed(() => {
    return systemInfo.value?.memory.usage_percent ?? 0
  })

  const memoryUsedGB = computed(() => {
    if (!systemInfo.value) return 0
    return parseFloat((systemInfo.value.memory.used / (1024 * 1024 * 1024)).toFixed(2))
  })

  const memoryTotalGB = computed(() => {
    if (!systemInfo.value) return 0
    return parseFloat((systemInfo.value.memory.total / (1024 * 1024 * 1024)).toFixed(2))
  })

  const memoryUsedFormatted = computed(() => {
    if (!systemInfo.value) return '0 B'
    return formatBytes(systemInfo.value.memory.used)
  })

  const memoryTotalFormatted = computed(() => {
    if (!systemInfo.value) return '0 B'
    return formatBytes(systemInfo.value.memory.total)
  })

  const swapUsagePercent = computed(() => {
    return systemInfo.value?.swap.usage_percent ?? 0
  })

  const swapTotalGB = computed(() => {
    if (!systemInfo.value) return 0
    return parseFloat((systemInfo.value.swap.total / (1024 * 1024 * 1024)).toFixed(2))
  })

  const diskUsagePercent = computed(() => {
    if (disks.value.length === 0) return 0
    const totalUsed = disks.value.reduce((sum, d) => sum + d.used_space, 0)
    const totalSpace = disks.value.reduce((sum, d) => sum + d.total_space, 0)
    return totalSpace > 0 ? (totalUsed / totalSpace) * 100 : 0
  })

  // Network rates (bytes per second)
  const networkRates = computed(() => {
    return networks.value.map((net) => {
      const prev = prevNetworks.value.find((p) => p.name === net.name)
      let receivedRate = 0
      let transmittedRate = 0
      if (prev) {
        const elapsed = (Date.now() - prev.timestamp) / 1000
        if (elapsed > 0) {
          receivedRate = (net.received_bytes - prev.received_bytes) / elapsed
          transmittedRate = (net.transmitted_bytes - prev.transmitted_bytes) / elapsed
        }
      }
      return {
        name: net.name,
        receivedRate,
        transmittedRate,
        receivedTotal: net.received_bytes,
        transmittedTotal: net.transmitted_bytes,
      }
    })
  })

  // ── Actions ───────────────────────────────────────────────

  async function fetchSystemInfo() {
    try {
      const result = await invoke<SystemInfo>('get_system_info')
      systemInfo.value = result
    } catch (e) {
      console.error('Failed to fetch system info:', e)
    }
  }

  async function fetchDisks() {
    try {
      const result = await invoke<DiskInfo[]>('get_disks')
      disks.value = result
    } catch (e) {
      console.error('Failed to fetch disks:', e)
    }
  }

  async function fetchNetworks() {
    try {
      const result = await invoke<NetworkInfo[]>('get_networks')
      // Store previous for rate calculation
      prevNetworks.value = networks.value.map((n) => ({
        name: n.name,
        received_bytes: n.received_bytes,
        transmitted_bytes: n.transmitted_bytes,
        timestamp: Date.now(),
      }))
      networks.value = result
    } catch (e) {
      console.error('Failed to fetch networks:', e)
    }
  }

  async function refreshAll() {
    loading.value = true
    try {
      await Promise.all([fetchSystemInfo(), fetchDisks(), fetchNetworks()])
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    systemInfo,
    disks,
    networks,
    loading,
    // Computed
    cpuUsagePercent,
    memoryUsagePercent,
    memoryUsedGB,
    memoryTotalGB,
    memoryUsedFormatted,
    memoryTotalFormatted,
    swapUsagePercent,
    swapTotalGB,
    diskUsagePercent,
    networkRates,
    // Actions
    fetchSystemInfo,
    fetchDisks,
    fetchNetworks,
    refreshAll,
    // Utils
    formatBytes,
  }
})