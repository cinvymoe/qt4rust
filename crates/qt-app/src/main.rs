// Qt Rust Demo - 应用程序入口点
// 使用 cxx-qt 0.8 API

// Qt 相关模块
mod application;
mod calibration_viewmodel;
mod data_collection_controller;
mod history_viewmodel;
mod monitoring_viewmodel;
mod settings_viewmodel;
mod viewmodel_manager;

// Settings ViewModels (在 src/ 根目录以满足 cxx-qt 构建限制)
mod alarm_threshold_viewmodel;
mod angle_calibration_viewmodel;
mod load_calibration_viewmodel;
mod moment_curve_viewmodel;
mod radius_calibration_viewmodel;

// MVI 架构模块
mod states {
    pub mod calibration_state;
}
mod intents {
    pub mod calibration_intent;
}
mod reducers {
    pub mod calibration_reducer;
}

use application::Application;

/// 初始化 Qt 虚拟键盘所需的环境变量，必须在 QGuiApplication 之前调用
fn setup_virtual_keyboard() {
    std::env::set_var("QT_IM_MODULE", "qtvirtualkeyboard");
    std::env::set_var("QT_VIRTUALKEYBOARD_DESKTOP_DISABLE", "0");

    // 将用户数据目录重定向到可写路径，避免 "Cannot create directory for user data /root/.config/qtvirtualkeyboard"
    if std::env::var("XDG_CONFIG_HOME").is_err() {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/qt-app-config");
    }
    let _ = std::fs::create_dir_all("/tmp/qt-app-config/qtvirtualkeyboard");

    tracing::info!("Virtual Keyboard enabled: QT_IM_MODULE=qtvirtualkeyboard");
}

/// 初始化 fontconfig 配置路径，避免 "Cannot load default config file" 警告
/// 优先使用应用目录下的 fonts/fonts.conf
fn setup_fontconfig() {
    if std::env::var("FONTCONFIG_FILE").is_ok() {
        return;
    }
    let app_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(dir) = app_dir {
        let fonts_conf = dir.join("fonts/fonts.conf");
        if fonts_conf.exists() {
            std::env::set_var("FONTCONFIG_FILE", &fonts_conf);
            tracing::info!("Fontconfig: using {}", fonts_conf.display());
        }
    }
}

fn main() {
    // 初始化日志系统（在其他初始化之前）
    match qt_rust_demo::logging::init_logging_from_file("config/logging.toml") {
        Ok(_) => {
            tracing::info!("日志系统初始化成功");
        }
        Err(e) => {
            // 日志系统未初始化，使用 eprintln 是合理的
            eprintln!("[WARN] 无法加载日志配置文件: {}", e);
            eprintln!("[INFO] 使用默认日志配置");
            qt_rust_demo::logging::init_default_logging();
        }
    }

    setup_virtual_keyboard();
    setup_fontconfig();

    // 创建应用程序实例
    let mut app = match Application::new() {
        Ok(app) => {
            tracing::info!("Application initialized successfully");
            app
        }
        Err(e) => {
            tracing::error!("Failed to initialize application: {}", e);
            std::process::exit(1); // Qt 应用初始化失败
        }
    };

    // 启动应用程序并获取退出码
    let exit_code = app.run();

    // 返回退出码
    std::process::exit(exit_code);
}
