import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: () => import('../views/Dashboard.vue')
    },
    {
      path: '/process',
      name: 'ProcessManager',
      component: () => import('../views/ProcessManager.vue')
    }
  ]
})

export default router