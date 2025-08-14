// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::{initialize::init_contex, router::build_app, serve::AppBuilder};
use axum::{http::StatusCode, response::IntoResponse, Router};
use mf_state::init_logging;
use tauri::{AppHandle, Manager, Emitter};

// 自定义事件处理函数
fn handle_tauri_error(error: tauri::Error) {
    eprintln!("Tauri错误: {:?}", error);
}

// 显示主窗口的命令
#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    println!("显示主窗口命令");

    // 列出所有窗口进行调试
    let all_windows = app.webview_windows();
    println!("当前所有窗口: {:?}", all_windows.keys().collect::<Vec<_>>());

    if let Some(main_window) = app.get_webview_window("main") {
        println!("找到主窗口，开始显示");
        main_window.show().map_err(|e| format!("显示主窗口失败: {}", e))?;
        main_window
            .set_focus()
            .map_err(|e| format!("设置主窗口焦点失败: {}", e))?;
        main_window
            .unminimize()
            .map_err(|e| format!("取消最小化失败: {}", e))?;
        println!("主窗口显示成功");
    } else {
        println!("主窗口不存在，重新创建");

        // 重新创建主窗口
        let main_window = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("/".into()),
        )
        .title("ModuForge Demo")
        .inner_size(1200.0, 1000.0)
        .resizable(true)
        .decorations(false)
        .visible(true)
        .center()
        .build()
        .map_err(|e| {
            println!("创建主窗口失败: {}", e);
            format!("创建主窗口失败: {}", e)
        })?;

        println!("主窗口重新创建成功");
        let _ = main_window.set_focus();

        #[cfg(debug_assertions)]
        main_window.open_devtools();
    }

    Ok(())
}

// 退出应用的命令
#[tauri::command]
async fn quit_app(app: AppHandle) -> Result<(), String> {
    println!("退出应用命令");
    app.exit(0);
    Ok(())
}

// 显示托盘菜单的命令
#[tauri::command]
async fn show_tray_menu(
    app: AppHandle,
    x: f64,
    y: f64,
) -> Result<(), String> {
    println!("显示托盘菜单 at ({}, {})", x, y);

    // 关闭现有菜单窗口
    if let Some(existing_menu) = app.get_webview_window("tray-menu") {
        println!("关闭现有托盘菜单窗口");
        let _ = existing_menu.close();
    }

    // 创建菜单窗口
    println!("创建托盘菜单窗口");
    let menu_window = tauri::WebviewWindowBuilder::new(
        &app,
        "tray-menu",
        tauri::WebviewUrl::App("/?window=tray-menu".into()),
    )
    .title("Tray Menu")
    .inner_size(150.0, 80.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .position(x - 75.0, y - 80.0) // 菜单显示在托盘图标上方
    .visible(true) // 直接显示
    .build()
    .map_err(|e| {
        println!("创建菜单窗口失败: {}", e);
        format!("创建菜单窗口失败: {}", e)
    })?;

    println!("托盘菜单窗口创建成功");
    let _ = menu_window.set_focus();

    Ok(())
}

// 隐藏托盘菜单的命令
#[tauri::command]
async fn hide_tray_menu(app: AppHandle) -> Result<(), String> {
    println!("隐藏托盘菜单");
    if let Some(menu_window) = app.get_webview_window("tray-menu") {
        menu_window.close().map_err(|e| format!("关闭菜单窗口失败: {}", e))?;
    }
    Ok(())
}

// 处理登录成功的命令
#[tauri::command]
fn handle_login_success(_window: tauri::Window) -> Result<(), String> {
    println!("用户登录成功，准备显示主界面");

    // 无需窗口切换，登录成功后由前端路由处理
    Ok(())
}

// 处理退出登录的命令
#[tauri::command]
fn handle_logout(_window: tauri::Window) -> Result<(), String> {
    println!("用户退出登录，准备返回登录界面");

    // 无需窗口切换，退出登录后由前端路由处理
    Ok(())
}

// 创建模块窗口的命令
#[tauri::command]
async fn create_module_window(
    app: AppHandle,
    module_key: String,
    title: String,
    url: String,
) -> Result<(), String> {
    println!("创建模块窗口: {} - {} - URL: {}", module_key, title, url);

    // 检查窗口是否已存在
    let window_label = format!("module-{}", module_key);
    if let Some(existing_window) = app.get_webview_window(&window_label) {
        println!("模块窗口已存在，显示并聚焦");
        existing_window.show().map_err(|e| format!("显示窗口失败: {}", e))?;
        existing_window.set_focus().map_err(|e| format!("设置焦点失败: {}", e))?;
        existing_window.unminimize().map_err(|e| format!("取消最小化失败: {}", e))?;
        return Ok(());
    }

    // 处理 URL：区分开发环境和生产环境
    let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
        // 开发环境：外部 URL
        let parsed_url = url.parse().map_err(|e| {
            let error_msg = format!("URL解析失败: {} - URL: {}", e, url);
            println!("{}", error_msg);
            error_msg
        })?;
        println!("使用外部URL: {:?}", parsed_url);
        tauri::WebviewUrl::External(parsed_url)
    } else {
        // 生产环境：应用内路径
        println!("使用应用内路径: {}", url);
        tauri::WebviewUrl::App(url.into())
    };

    // 创建新的模块窗口
    println!("开始创建窗口，标签: {}", window_label);
    let module_window = tauri::WebviewWindowBuilder::new(
        &app,
        &window_label,
        webview_url,
    )
    .title(&title)
    .inner_size(1200.0, 800.0)
    .min_inner_size(900.0, 600.0)
    .resizable(true)
    .decorations(true)  // 使用系统标题栏
    .visible(true)
    .focused(true)
    .center()
    .build()
    .map_err(|e| {
        println!("创建模块窗口失败: {}", e);
        format!("创建模块窗口失败: {}", e)
    })?;

    println!("模块窗口创建成功: {}", window_label);

    // 强制显示和聚焦窗口
    module_window.show().map_err(|e| format!("显示窗口失败: {}", e))?;
    module_window.set_focus().map_err(|e| format!("设置焦点失败: {}", e))?;

    println!("窗口显示和聚焦完成");

    #[cfg(debug_assertions)]
    module_window.open_devtools();

    Ok(())
}

// 创建子窗口的命令（支持模态和非模态）
#[tauri::command]
async fn create_child_window(
    app: AppHandle,
    window_id: String,
    title: String,
    url: String,
    modal: Option<bool>,
    width: Option<f64>,
    height: Option<f64>,
    parent_window: Option<String>,
) -> Result<(), String> {
    println!("创建子窗口: {} - {} - URL: {} - Modal: {:?}", window_id, title, url, modal);

    let is_modal = modal.unwrap_or(false);
    let window_width = width.unwrap_or(800.0);
    let window_height = height.unwrap_or(600.0);

    // 检查窗口是否已存在
    if let Some(existing_window) = app.get_webview_window(&window_id) {
        println!("子窗口已存在，显示并聚焦");
        existing_window.show().map_err(|e| format!("显示窗口失败: {}", e))?;
        existing_window.set_focus().map_err(|e| format!("设置焦点失败: {}", e))?;
        existing_window.unminimize().map_err(|e| format!("取消最小化失败: {}", e))?;
        return Ok(());
    }

    // 如果是模态窗口，先禁用父窗口
    if is_modal {
        if let Some(parent_label) = &parent_window {
            if let Some(parent_win) = app.get_webview_window(parent_label) {
                // 通过发送事件来禁用父窗口
                let _ = parent_win.emit("window-disabled", true);
                println!("已禁用父窗口: {}", parent_label);
            }
        } else if let Some(main_window) = app.get_webview_window("main") {
            // 默认禁用主窗口
            let _ = main_window.emit("window-disabled", true);
            println!("已禁用主窗口");
        }
    }

    // 处理 URL
    let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
        let parsed_url = url.parse().map_err(|e| {
            let error_msg = format!("URL解析失败: {} - URL: {}", e, url);
            println!("{}", error_msg);
            error_msg
        })?;
        tauri::WebviewUrl::External(parsed_url)
    } else {
        tauri::WebviewUrl::App(url.into())
    };

    // 创建子窗口
    let mut window_builder = tauri::WebviewWindowBuilder::new(
        &app,
        &window_id,
        webview_url,
    )
    .title(&title)
    .inner_size(window_width, window_height)
    .resizable(true)
    .decorations(true)
    .visible(true)
    .focused(true)
    .center();

    // 如果是模态窗口，设置为总是在顶部
    if is_modal {
        window_builder = window_builder.always_on_top(true);
    }

    let child_window = window_builder
        .build()
        .map_err(|e| {
            println!("创建子窗口失败: {}", e);
            format!("创建子窗口失败: {}", e)
        })?;

    println!("子窗口创建成功: {}", window_id);

    // 监听窗口关闭事件，如果是模态窗口，关闭时重新启用父窗口
    if is_modal {
        let app_handle = app.clone();
        let parent_label = parent_window.clone().unwrap_or_else(|| "main".to_string());
        let window_id_clone = window_id.clone();

        child_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                println!("模态窗口 {} 即将关闭，重新启用父窗口", window_id_clone);
                if let Some(parent_win) = app_handle.get_webview_window(&parent_label) {
                    let _ = parent_win.emit("window-enabled", true);
                    println!("已重新启用父窗口: {}", parent_label);
                }
            }
        });
    }

    child_window.show().map_err(|e| format!("显示窗口失败: {}", e))?;
    child_window.set_focus().map_err(|e| format!("设置焦点失败: {}", e))?;

    #[cfg(debug_assertions)]
    child_window.open_devtools();

    Ok(())
}

// 关闭子窗口的命令
#[tauri::command]
async fn close_child_window(
    app: AppHandle,
    window_id: String,
    parent_window: Option<String>,
) -> Result<(), String> {
    println!("关闭子窗口: {}", window_id);

    if let Some(window) = app.get_webview_window(&window_id) {
        // 如果有父窗口，重新启用它
        if let Some(parent_label) = parent_window {
            if let Some(parent_win) = app.get_webview_window(&parent_label) {
                let _ = parent_win.emit("window-enabled", true);
                println!("已重新启用父窗口: {}", parent_label);
            }
        }

        window.close().map_err(|e| format!("关闭窗口失败: {}", e))?;
        println!("子窗口已关闭: {}", window_id);
    } else {
        return Err(format!("窗口不存在: {}", window_id));
    }

    Ok(())
}

// 暂时注释掉数据交互功能，先让窗口创建正常工作
// TODO: 稍后重新实现数据交互功能

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统，降低tao警告级别
    init_logging("info", None).unwrap();

    // 初始化上下文
    init_contex().await;

    // 启动API服务器
    let app: Router = build_app();
    let builder = AppBuilder::new().next("/api", app.fallback(handler_404));
    builder.build().run();
    // 配置Tauri应用
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 设置启动屏幕的自动关闭机制
            let app_handle_timeout = app_handle.clone();
            tokio::spawn(async move {
                // 等待3秒显示启动屏幕
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                println!("启动屏幕显示完成，切换到主窗口");

                // 关闭启动屏幕
                if let Some(splash_window) =
                    app_handle_timeout.get_webview_window("splashscreen")
                {
                    let _ = splash_window.close();
                }

                // 显示主窗口
                if let Some(main_window) =
                    app_handle_timeout.get_webview_window("main")
                {
                    let _ = main_window.show();
                    let _ = main_window.center();
                    let _ = main_window.set_focus();

                    #[cfg(debug_assertions)]
                    main_window.open_devtools();
                }
            });

            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            handle_login_success,
            handle_logout,
            show_main_window,
            quit_app,
            show_tray_menu,
            hide_tray_menu,
            create_module_window,
            create_child_window,
            close_child_window
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            handle_tauri_error(e);
            anyhow::anyhow!("Tauri应用启动失败")
        })?;

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
