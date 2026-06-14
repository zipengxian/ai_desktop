<script setup lang="ts">
import { ref, h } from 'vue'
import { useRouter } from 'vue-router'
import { DashboardOutlined, SettingOutlined } from '@vicons/antd'

const router = useRouter()

const menuOptions = [
  {
    label: '仪表盘',
    key: 'dashboard',
    icon: () => h(DashboardOutlined)
  },
  {
    label: '进程管理',
    key: 'process',
    icon: () => h(SettingOutlined)
  }
]

const activeKey = ref('dashboard')

function handleMenuUpdate(key: string) {
  activeKey.value = key
  if (key === 'dashboard') {
    router.push('/')
  } else if (key === 'process') {
    router.push('/process')
  }
}
</script>

<template>
  <n-layout style="height: 100vh">
    <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="200">
      <n-menu
        v-model:value="activeKey"
        :options="menuOptions"
        @update:value="handleMenuUpdate"
      />
    </n-layout-sider>
    <n-layout-content style="padding: 24px">
      <router-view />
    </n-layout-content>
  </n-layout>
</template>