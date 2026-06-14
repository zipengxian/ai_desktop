<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useSystemStore, formatBytes } from '../stores/system'

const store = useSystemStore()

let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  store.refreshAll()
  timer = setInterval(() => {
    store.refreshAll()
  }, 1000)
})

onUnmounted(() => {
  if (timer !== null) {
    clearInterval(timer)
    timer = null
  }
})

// ── Progress color helpers ─────────────────────────────────

function progressColor(percent: number): string {
  if (percent > 80) return '#d03050'
  if (percent > 50) return '#f0a020'
  return '#18a058'
}

const cpuColor = computed(() => progressColor(store.cpuUsagePercent))
const memoryColor = computed(() => progressColor(store.memoryUsagePercent))

// ── Disk color cycle ───────────────────────────────────────

const diskColors = ['#18a058', '#2080f0', '#f0a020', '#d03050', '#7c3aed', '#0e7a0d']

function diskColor(index: number): string {
  return diskColors[index % diskColors.length]
}

// ── Network table columns ──────────────────────────────────

const networkColumns = [
  { title: '接口名称', key: 'name', width: 120 },
  { title: '接收速率', key: 'receivedRate', width: 120 },
  { title: '发送速率', key: 'transmittedRate', width: 120 },
  { title: '累计接收', key: 'receivedTotal', width: 120 },
  { title: '累计发送', key: 'transmittedTotal', width: 120 },
]

const networkData = computed(() =>
  store.networkRates.map((r) => ({
    name: r.name,
    receivedRate: formatBytes(r.receivedRate) + '/s',
    transmittedRate: formatBytes(r.transmittedRate) + '/s',
    receivedTotal: formatBytes(r.receivedTotal),
    transmittedTotal: formatBytes(r.transmittedTotal),
  }))
)
</script>

<template>
  <div class="dashboard">
    <n-h2 style="margin-top: 0">系统仪表盘</n-h2>

    <n-grid :cols="2" :x-gap="16" :y-gap="16" responsive="screen" item-responsive>
      <!-- ── Column 1 ──────────────────────────────────────── -->
      <n-grid-item>
        <n-space vertical :size="16">
          <!-- CPU 卡片 -->
          <n-spin :show="store.loading">
            <n-card title="CPU">
              <div class="progress-center">
                <n-progress
                  type="circle"
                  :percentage="store.cpuUsagePercent"
                  :color="cpuColor"
                  :indicator-text-color="cpuColor"
                  :height="180"
                >
                  <span class="progress-label">
                    {{ store.cpuUsagePercent.toFixed(1) }}%
                  </span>
                </n-progress>
              </div>
            </n-card>
          </n-spin>

          <!-- 内存卡片 -->
          <n-spin :show="store.loading">
            <n-card title="内存">
              <n-progress
                type="line"
                :percentage="store.memoryUsagePercent"
                :color="memoryColor"
                :indicator-text-color="memoryColor"
                :height="24"
                :border-radius="4"
                processing
              />
              <div class="memory-detail">
                已用 {{ store.memoryUsedGB }} GB / 总量 {{ store.memoryTotalGB }} GB
              </div>
              <div v-if="store.systemInfo && store.systemInfo.swap.total > 0" class="swap-info">
                交换空间: {{ formatBytes(store.systemInfo.swap.used) }} / {{ formatBytes(store.systemInfo.swap.total) }}
                ({{ store.swapUsagePercent.toFixed(1) }}%)
              </div>
            </n-card>
          </n-spin>
        </n-space>
      </n-grid-item>

      <!-- ── Column 2 ──────────────────────────────────────── -->
      <n-grid-item>
        <n-space vertical :size="16">
          <!-- 磁盘卡片 -->
          <n-spin :show="store.loading">
            <n-card title="磁盘">
              <div v-if="store.disks.length === 0" class="empty-hint">
                暂无磁盘数据
              </div>
              <n-space v-else vertical :size="12">
                <div v-for="(disk, idx) in store.disks" :key="disk.mount_point" class="disk-item">
                  <div class="disk-label">
                    {{ disk.name }} ({{ disk.mount_point }})
                  </div>
                  <n-progress
                    type="line"
                    :percentage="disk.usage_percent"
                    :color="diskColor(idx)"
                    :height="18"
                    :border-radius="4"
                    :show-indicator="false"
                  />
                  <div class="disk-detail">
                    {{ formatBytes(disk.used_space) }} / {{ formatBytes(disk.total_space) }}
                    <span class="disk-percent">{{ disk.usage_percent.toFixed(1) }}%</span>
                  </div>
                </div>
              </n-space>
            </n-card>
          </n-spin>

          <!-- 网络卡片 -->
          <n-spin :show="store.loading">
            <n-card title="网络">
              <div v-if="store.networks.length === 0" class="empty-hint">
                暂无网络数据
              </div>
              <n-table
                v-else
                :columns="networkColumns"
                :data="networkData"
                :single-line="false"
                size="small"
                :bordered="false"
              />
            </n-card>
          </n-spin>
        </n-space>
      </n-grid-item>
    </n-grid>
  </div>
</template>

<style scoped>
.dashboard {
  padding: 8px 0;
}

.progress-center {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 12px 0;
}

.progress-label {
  font-size: 24px;
  font-weight: 600;
}

.memory-detail {
  margin-top: 12px;
  font-size: 14px;
  color: var(--n-text-color-2);
}

.swap-info {
  margin-top: 6px;
  font-size: 13px;
  color: var(--n-text-color-3);
}

.disk-item {
  padding: 4px 0;
}

.disk-label {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
  color: var(--n-text-color);
}

.disk-detail {
  font-size: 12px;
  color: var(--n-text-color-2);
  margin-top: 2px;
}

.disk-percent {
  margin-left: 8px;
  font-weight: 500;
  color: var(--n-text-color);
}

.empty-hint {
  text-align: center;
  padding: 24px;
  color: var(--n-text-color-3);
  font-size: 14px;
}
</style>