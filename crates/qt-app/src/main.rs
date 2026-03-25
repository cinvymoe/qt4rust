// Qt Rust Demo - 应用程序入口点
// 使用 cxx-qt 0.8 API

// Qt 相关模块
mod application;
mod monitoring_viewmodel;
mod data_collection_controller;
mod viewmodel_manager;

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

    eprintln!("[INFO] Virtual Keyboard enabled: QT_IM_MODULE=qtvirtualkeyboard");
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
            eprintln!("[INFO] Fontconfig: using {}", fonts_conf.display());
        }
    }
}

fn main() {
    setup_virtual_keyboard();
    setup_fontconfig();
    
    // 创建应用程序实例
    let mut app = match Application::new() {
        Ok(app) => {
            eprintln!("[INFO] Application initialized successfully");
            app
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize application: {}", e);
            std::process::exit(1); // Qt 应用初始化失败
        }
    };
    
    // 启动应用程序并获取退出码
    let exit_code = app.run();
    
    // 返回退出码
    std::process::exit(exit_code);
}
