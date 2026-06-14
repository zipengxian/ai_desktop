# Tasks

- [x] Task 1: 创建 Tauri v2 + Vue 3 项目脚手架
  - [x] 使用 `npm create tauri-app@latest` 初始化项目，选择 Vue + TypeScript 模板
  - [x] 安装 Naive UI 组件库并配置全局引入
  - [x] 安装 Pinia 状态管理库并创建基础 store
  - [x] 安装 Vue Router 并配置路由结构（仪表盘 / 进程管理两个页面）
  - [x] 配置项目的基本目录结构

- [x] Task 2: 实现 Rust 后端系统信息模块
  - [x] 添加 `sysinfo` crate 依赖
  - [x] 实现获取 CPU 使用率的 Tauri command
  - [x] 实现获取内存信息的 Tauri command
  - [x] 实现获取磁盘信息的 Tauri command
  - [x] 实现获取网络速率信息的 Tauri command
  - [x] 实现获取进程列表的 Tauri command
  - [x] 实现结束进程的 Tauri command

- [x] Task 3: 实现系统仪表盘页面
  - [x] 创建仪表盘 Pinia store，封装数据获取与刷新逻辑
  - [x] 使用 Naive UI 的 Card、Progress、Statistic 等组件构建仪表盘布局
  - [x] 实现 CPU 使用率环形进度条
  - [x] 实现内存使用进度条与详情
  - [x] 实现磁盘使用情况展示
  - [x] 实现网络上下行速率展示
  - [x] 实现每秒自动刷新数据的定时轮询

- [x] Task 4: 实现进程管理页面
  - [x] 创建进程管理 Pinia store，封装进程列表获取与操作
  - [x] 使用 Naive UI 的 DataTable 组件展示进程列表（名称、PID、CPU、内存）
  - [x] 实现进程名称搜索过滤
  - [x] 实现按列排序功能
  - [x] 实现"结束进程"操作（含确认对话框）
  - [x] 实现进程列表定时刷新

- [x] Task 5: 实现系统托盘功能
  - [x] 在 Rust 端配置 Tauri 系统托盘图标与菜单
  - [x] 实现窗口显示/隐藏切换
  - [x] 实现托盘菜单"退出"功能
  - [x] 配置窗口关闭时最小化到托盘而非退出

- [x] Task 6: 构建打包与跨平台验证
  - [x] 配置 Tauri 打包签名设置
  - [x] 执行生产构建验证 (`cargo check` + `npm run build` 均通过)
  - [x] 验证打包产物可正常运行

# Task Dependencies
- Task 2 依赖 Task 1（需要项目脚手架才能编写 Rust 代码）
- Task 3 依赖 Task 2（前端页面需要 Rust 命令接口）
- Task 4 依赖 Task 2（需要进程列表和结束进程的 API）
- Task 5 依赖 Task 1（需要 Tauri 项目结构）
- Task 6 依赖所有以上 Task 完成