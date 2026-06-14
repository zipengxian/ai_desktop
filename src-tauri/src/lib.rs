use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{Cpu, Disks, Networks, Pid, ProcessesToUpdate, Signal, System};

use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::Manager;

// ── Data structures ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwapInfo {
    pub total: u64,
    pub used: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub swap: SwapInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
}

// ── Tauri commands ───────────────────────────────────────────

#[tauri::command]
fn get_system_info(state: tauri::State<'_, Mutex<System>>) -> SystemInfo {
    let mut sys = state.lock().unwrap();
    sys.refresh_all();

    // CPU usage: average across all logical cores
    let cpu_usage = if !sys.cpus().is_empty() {
        sys.cpus().iter().map(Cpu::cpu_usage).sum::<f32>() / sys.cpus().len() as f32
    } else {
        0.0
    };

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();
    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    let swap_usage_percent = if total_swap > 0 {
        (used_swap as f32 / total_swap as f32) * 100.0
    } else {
        0.0
    };

    SystemInfo {
        cpu: CpuInfo { usage: cpu_usage },
        memory: MemoryInfo {
            total: total_memory,
            used: used_memory,
            available: available_memory,
            usage_percent: memory_usage_percent,
        },
        swap: SwapInfo {
            total: total_swap,
            used: used_swap,
            usage_percent: swap_usage_percent,
        },
    }
}

#[tauri::command]
fn get_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent,
            }
        })
        .collect()
}

#[tauri::command]
fn get_networks() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();

    networks
        .list()
        .iter()
        .map(|(name, data)| NetworkInfo {
            name: name.clone(),
            received_bytes: data.received(),
            transmitted_bytes: data.transmitted(),
        })
        .collect()
}

#[tauri::command]
fn get_processes(state: tauri::State<'_, Mutex<System>>) -> Vec<ProcessInfo> {
    let mut sys = state.lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let total_memory = sys.total_memory();

    sys.processes()
        .iter()
        .map(|(pid, process)| {
            let mem = process.memory();
            let memory_percent = if total_memory > 0 {
                (mem as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            };

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: mem,
                memory_percent,
            }
        })
        .collect()
}

#[tauri::command]
fn kill_process(pid: u32, state: tauri::State<'_, Mutex<System>>) -> Result<bool, String> {
    let target = Pid::from_u32(pid);

    // Try finding and killing via sysinfo first (cross-platform)
    {
        let mut sys = state.lock().unwrap();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(process) = sys.process(target) {
            // SIGKILL on Unix, TerminateProcess on Windows
            return process
                .kill_with(Signal::Kill)
                .map_or_else(
                    || Err(format!("Failed to send kill signal to PID {}", pid)),
                    |success| {
                        if success {
                            Ok(true)
                        } else {
                            Err(format!("kill signal sent but process {} may not have terminated", pid))
                        }
                    },
                );
        }
    }

    // Fallback: use system command
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
        Ok(output.status.success())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
        Ok(output.status.success())
    }
}

// ── Entry point ──────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize sysinfo::System and do a first refresh to establish a
            // CPU-usage baseline so the first command call returns real values.
            let mut sys = System::new_all();
            sys.refresh_all();

            app.manage(Mutex::new(sys));

            // ── System tray ──────────────────────────────────

            // Get main window
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            // Intercept close event: hide to tray instead of exiting
            let window_close = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_close.hide();
                }
            });

            // Load tray icon from default window icon
            let icon = app
                .default_window_icon()
                .cloned()
                .expect("Failed to load default window icon");

            // Create tray menu
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&quit_item)
                .build()?;

            // Build system tray
            let _tray = TrayIconBuilder::with_id("system-tray")
                .icon(icon)
                .menu(&menu)
                .tooltip("桌面系统管理器")
                .on_menu_event(|app_handle: &tauri::AppHandle, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray_icon: &tauri::tray::TrayIcon, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app_handle = tray_icon.app_handle();
                        if let Some(w) = app_handle.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            get_disks,
            get_networks,
            get_processes,
            kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}