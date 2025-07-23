// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::{initialize::init_contex, router::build_app, serve::AppBuilder};
use axum::{http::StatusCode, response::IntoResponse, Router};
use mf_state::init_logging;
use tauri::{
    tray::TrayIconBuilder, tray::TrayIconEvent, AppHandle, Listener, Manager,
};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统，降低tao警告级别
    init_logging("info", None).unwrap();

    // 初始化上下文
    init_contex().await;

    // 启动API服务器
    tokio::spawn(async move {
        let app: Router = build_app();
        AppBuilder::new()
            .next("/api", app.fallback(handler_404))
            .build()
            .run()
            .await;
    });

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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            handle_login_success,
            handle_logout,
            show_main_window,
            quit_app,
            show_tray_menu,
            hide_tray_menu
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
