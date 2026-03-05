// Qt Rust Demo - 应用程序入口点
// 使用 cxx-qt 0.8 API

mod application;
mod counter;

use application::Application;

fn main() {
    // 设置虚拟键盘环境变量
    std::env::set_var("QT_IM_MODULE", "qtvirtualkeyboard");
    std::env::set_var("QT_VIRTUALKEYBOARD_DESKTOP_DISABLE", "0");
    
    eprintln!("[INFO] Virtual Keyboard enabled: QT_IM_MODULE=qtvirtualkeyboard");
    
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
