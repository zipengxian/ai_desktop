<script setup lang="ts">
import { onMounted, onUnmounted, h } from 'vue'
import { NTag, NButton, NPopconfirm } from 'naive-ui'
import type { DataTableColumn } from 'naive-ui'
import { useMessage } from 'naive-ui'
import { useProcessStore, type ProcessInfo } from '@/stores/process'

const store = useProcessStore()
const message = useMessage()

// ── Formatting utilities ────────────────────────────────────

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatPercent(pct: number): string {
  return `${pct.toFixed(1)}%`
}

// ── Columns definition ──────────────────────────────────────

const columns: DataTableColumn<ProcessInfo>[] = [
  {
    title: '进程名称',
    key: 'name',
    sorter: true,
  },
  {
    title: 'PID',
    key: 'pid',
    sorter: true,
    defaultSortOrder: false,
  },
  {
    title: 'CPU 占用',
    key: 'cpu_usage',
    sorter: 'default',
    defaultSortOrder: 'descend',
    render(row) {
      const pct = formatPercent(row.cpu_usage)
      const type =
        row.cpu_usage > 70 ? 'error' : row.cpu_usage > 30 ? 'warning' : 'success'
      return h(NTag, { type, size: 'small' }, { default: () => pct })
    },
  },
  {
    title: '内存占用',
    key: 'memory_usage',
    sorter: true,
    render(row) {
      const mem = formatBytes(row.memory_usage)
      const pct = formatPercent(row.memory_percent)
      return h('span', `${mem} (${pct})`)
    },
  },
  {
    title: '操作',
    key: 'actions',
    render(row) {
      return h(
        NPopconfirm,
        {
          onPositiveClick: () => handleKill(row.pid),
        },
        {
          trigger: () =>
            h(NButton, { type: 'error', size: 'small' }, { default: () => '结束进程' }),
          default: () => '确定要结束该进程吗？',
        }
      )
    },
  },
]

// ── Kill handler ────────────────────────────────────────────

async function handleKill(pid: number) {
  try {
    await store.killProcess(pid)
    message.success(`进程 ${pid} 已结束`)
  } catch {
    message.error(`结束进程 ${pid} 失败`)
  }
}

// ── Sort handler ────────────────────────────────────────────

interface SortState {
  columnKey: string
  order: 'ascend' | 'descend' | false
}

function handleSorter(sortState: SortState | SortState[] | null) {
  if (!sortState) return
  const state = Array.isArray(sortState) ? sortState[0] : sortState
  store.setSort(state.columnKey)
}

// ── Lifecycle: auto-refresh ─────────────────────────────────

let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  store.fetchProcesses()
  timer = setInterval(() => {
    store.fetchProcesses()
  }, 3000)
})

onUnmounted(() => {
  if (timer !== null) {
    clearInterval(timer)
    timer = null
  }
})
</script>

<template>
  <n-card title="进程管理">
    <!-- Toolbar -->
    <n-space align="center" style="margin-bottom: 16px">
      <n-input
        v-model:value="store.searchQuery"
        placeholder="搜索进程名称..."
        clearable
        style="width: 280px"
      >
        <template #prefix>
          <span style="font-size: 16px">🔍</span>
        </template>
      </n-input>
      <n-button @click="store.fetchProcesses()">刷新</n-button>
    </n-space>

    <!-- Table -->
    <n-spin :show="store.loading">
      <n-data-table
        :columns="columns"
        :data="store.sortedProcesses"
        :row-key="(row: ProcessInfo) => row.pid"
        :bordered="true"
        :pagination="{ pageSize: 20 }"
        @update:sorter="handleSorter"
      />
    </n-spin>
  </n-card>
</template>